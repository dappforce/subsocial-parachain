use frame_support::{
    log,
    traits::{Currency, Imbalance, OnRuntimeUpgrade},
};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

use super::*;

pub mod v1 {
    use frame_support::{
        pallet_prelude::*,
        traits::{tokens::WithdrawReasons, ExistenceRequirement},
        weights::Weight,
    };
    use num_traits::Zero;
    use sp_arithmetic::traits::Saturating;
    use crate::inflation::NegativeImbalanceOf;

    use super::*;

    pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);

    #[derive(Encode, Decode)]
    struct V0IssuanceAndRewards<Balance> {
        total_issuance: Balance,
        rewards: Balance,
    }

    impl<T: Config> MigrateToV1<T> {
        fn distribute_rewards(
            config: &RewardDistributionConfig,
            reward: NegativeImbalanceOf<T>,
        ) -> (NegativeImbalanceOf<T>, NegativeImbalanceOf<T>) {
            // Pre-calculate the balance that will be deposited for each beneficiary
            let backers_balance = config.backers_percent * reward.peek();
            let creators_balance = config.creators_percent * reward.peek();

            // Prepare imbalances
            let (creators_imbalance, remainder) = reward.split(creators_balance);
            let (backers_imbalance, _) = remainder.split(backers_balance);

            (creators_imbalance, backers_imbalance)
        }
    }

    impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
        fn on_runtime_upgrade() -> Weight {
            let current_version = Pallet::<T>::current_storage_version();
            let onchain_version = Pallet::<T>::on_chain_storage_version();

            let base_weight = T::DbWeight::get().reads(2);

            log::info!(
                target: LOG_TARGET,
                "Running migration with current storage version {:?} / onchain {:?}",
                current_version,
                onchain_version
            );

            if onchain_version == 0 && current_version == 1 {
                PalletDisabled::<T>::put(true);

                let rewards_pot_account = Pallet::<T>::rewards_pot_account();

                let balance_to_burn = T::Currency::free_balance(&rewards_pot_account);
                let balance_burnt = T::Currency::burn(balance_to_burn);

                if let Err(_) = T::Currency::settle(
                    &rewards_pot_account,
                    balance_burnt,
                    WithdrawReasons::empty(),
                    ExistenceRequirement::AllowDeath,
                ) {
                    log::error!(
                        target: LOG_TARGET,
                        "Failed to burn {:?} tokens from the pot account",
                        balance_to_burn
                    );
                } else {
                    log::info!(
                        target: LOG_TARGET,
                        "Burned {:?} tokens from the pot account",
                        balance_to_burn
                    );
                }

                let current_era = Pallet::<T>::current_era();
                let distribution_config = Pallet::<T>::reward_config();

                let blocks_per_era: u32 = u32::decode(&mut &T::BlockPerEra::get().encode()[..])
                    .expect("Failed to decode blocks_per_era");
                let amount_to_issue_per_era = T::BlockReward::get().saturating_mul(blocks_per_era.into());

                for era in 0..current_era {
                    if let Some(mut era_info) = Pallet::<T>::general_era_info(era) {
                        let new_tokens_per_era = T::Currency::issue(amount_to_issue_per_era);

                        let (
                            creators_imbalance,
                            backers_imbalance,
                        ) = Self::distribute_rewards(&distribution_config, new_tokens_per_era);

                        era_info.rewards = RewardInfo {
                            backers: backers_imbalance.peek(),
                            creators: creators_imbalance.peek(),
                        };

                        GeneralEraInfo::<T>::insert(era, era_info);

                        T::Currency::resolve_creating(&rewards_pot_account, backers_imbalance.merge(creators_imbalance));
                    }
                }

                BlockRewardAccumulator::<T>::put(RewardInfo::default());
                NextEraStartingBlock::<T>::put(frame_system::Pallet::<T>::block_number() + T::BlockPerEra::get());
                ForceEra::<T>::put(Forcing::ForceNew);

                current_version.put::<Pallet<T>>();

                let current_era: u64 = current_era.into();
                base_weight.saturating_add(
                    T::DbWeight::get().reads_writes(
                        3 + current_era,
                        3 + current_era,
                    ),
                )
            } else {
                log::info!(
                    target: LOG_TARGET,
                    "Migration did not execute. This probably should be removed"
                );
                base_weight
            }
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
            let current_version = Pallet::<T>::current_storage_version();
            let onchain_version = Pallet::<T>::on_chain_storage_version();

            ensure!(
                onchain_version == 0 && current_version == 1,
                "Failed migrating from version 0 to 1",
            );

            let rewards_pot_account = Pallet::<T>::rewards_pot_account();
            let rewards_amount = T::Currency::free_balance(&rewards_pot_account);
            let total_issuance = T::Currency::active_issuance();

            let issuance_and_rewards = V0IssuanceAndRewards {
                total_issuance,
                rewards: rewards_amount,
            };

            Ok(issuance_and_rewards.encode())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(issuance_and_rewards_encoded: Vec<u8>) -> Result<(), &'static str> {
            let V0IssuanceAndRewards { total_issuance, rewards }: V0IssuanceAndRewards<BalanceOf<T>> =
                V0IssuanceAndRewards::decode(&mut &issuance_and_rewards_encoded[..])
                    .map_err(|_| "Failed to decode V0IssuanceAndRewards")?;

            let rewards_pot_account = Pallet::<T>::rewards_pot_account();
            let new_total_issuance = T::Currency::active_issuance();
            let new_rewards = T::Currency::free_balance(&rewards_pot_account);

            let expected_issuance = total_issuance.saturating_sub(rewards).saturating_add(new_rewards);

            ensure!(
                new_total_issuance == expected_issuance,
                "issuance was modified incorrectly"
            );

            let current_era = Pallet::<T>::current_era();
            let distribution_config = Pallet::<T>::reward_config();

            let blocks_per_era = u32::decode(&mut &T::BlockPerEra::get().encode()[..])
                .expect("Failed to decode blocks_per_era");
            let amount_to_issue_per_era = T::BlockReward::get()
                .saturating_mul(blocks_per_era.into());

            for era in 0..current_era {
                if let Some(era_info) = Pallet::<T>::general_era_info(era) {
                    let new_tokens_per_era = T::Currency::issue(amount_to_issue_per_era);

                    let (
                        creators_imbalance,
                        backers_imbalance,
                    ) = Self::distribute_rewards(&distribution_config, new_tokens_per_era);

                    ensure!(era_info.rewards.backers == backers_imbalance.peek(), "backers reward were modified incorrectly");
                    ensure!(era_info.rewards.creators == creators_imbalance.peek(), "creators reward were modified incorrectly");
                }
            }

            let next_era_starting_block = Pallet::<T>::next_era_starting_block();
            let block_rewards_accumulator = BlockRewardAccumulator::<T>::get();

            ensure!(
                next_era_starting_block == frame_system::Pallet::<T>::block_number() + T::BlockPerEra::get(),
                "next_era_starting_block was modified incorrectly",
            );
            ensure!(block_rewards_accumulator.backers.is_zero(), "backers accumulated reward were modified incorrectly");
            ensure!(block_rewards_accumulator.creators.is_zero(), "creators accumulated reward were modified incorrectly");

            Ok(())
        }
    }
}
