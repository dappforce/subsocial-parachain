use frame_support::{
    log,
    storage::migration,
    traits::{Currency, LockableCurrency, OnRuntimeUpgrade, ReservableCurrency},
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

    use super::*;

    pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> MigrateToV1<T> {
        fn clear_storage(storage_name: &[u8]) -> Weight {
            let res = migration::clear_storage_prefix(
                Pallet::<T>::name().as_bytes(),
                storage_name,
                b"",
                None,
                None,
            );

            log::info!(
                target: LOG_TARGET,
                "Cleared '{}' entries from '{:?}' storage prefix",
                res.unique,
                storage_name,
            );

            if res.maybe_cursor.is_some() {
                log::error!(
                    target: LOG_TARGET,
                    "Storage prefix '{:?}' is not completely cleared",
                    storage_name,
                );
            }

            T::DbWeight::get().writes(res.unique.into())
        }

        fn ensure_storage_clean(records_count: u64) -> Result<(), &'static str> {
            ensure!(records_count == 0, "the records count after the migration should be zero");
            Ok(())
        }
    }

    impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
        fn on_runtime_upgrade() -> Weight {
            let current_version = Pallet::<T>::current_storage_version();
            let onchain_version = Pallet::<T>::on_chain_storage_version();

            let mut weight = T::DbWeight::get().reads(2);

            log::info!(
                target: LOG_TARGET,
                "Running migration with current storage version {:?} / onchain {:?}",
                current_version,
                onchain_version
            );

            if onchain_version == 0 && current_version == 1 {
                PalletDisabled::<T>::put(true);
                CurrentEra::<T>::kill();
                ForceEra::<T>::kill();
                NextEraStartingBlock::<T>::kill();
                BlockRewardAccumulator::<T>::kill();
                ActiveRewardDistributionConfig::<T>::kill();

                let pot_account = Pallet::<T>::account_id();
                let balance_to_burn = T::Currency::free_balance(&pot_account);
                let balance_burnt = T::Currency::burn(balance_to_burn);
                if let Err(_) = T::Currency::settle(
                    &pot_account,
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

                let registered_creators_count = RegisteredCreators::<T>::iter().count() as u64;
                for (_, info) in RegisteredCreators::<T>::iter() {
                    T::Currency::unreserve(&info.stakeholder, T::RegistrationDeposit::get());
                }

                weight.saturating_accrue(Self::clear_storage(b"RegisteredCreators"));
                weight.saturating_accrue(Self::clear_storage(b"CreatorEraStake"));
                weight.saturating_accrue(Self::clear_storage(b"GeneralStakerInfo"));
                weight.saturating_accrue(Self::clear_storage(b"GeneralEraInfo"));

                let ledger_count = Ledger::<T>::iter().count() as u64;
                for (staker, _) in Ledger::<T>::iter() {
                    T::Currency::remove_lock(STAKING_ID, &staker);
                }

                weight.saturating_accrue(Self::clear_storage(b"Ledger"));

                current_version.put::<Pallet<T>>();

                weight.saturating_add(
                    T::DbWeight::get()
                        .reads(registered_creators_count.saturating_add(ledger_count)),
                )
            } else {
                log::info!(
                    target: LOG_TARGET,
                    "Migration did not execute. This probably should be removed"
                );
                weight
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
            Ok(Vec::new())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
            let registered_creators_count = RegisteredCreators::<T>::iter().count() as u64;
            let creator_era_stake_count = CreatorEraStake::<T>::iter().count() as u64;
            let general_staker_info_count = GeneralStakerInfo::<T>::iter().count() as u64;
            let general_era_info_count = GeneralEraInfo::<T>::iter().count() as u64;
            let ledger_count = Ledger::<T>::iter().count() as u64;

            Self::ensure_storage_clean(registered_creators_count)?;
            Self::ensure_storage_clean(creator_era_stake_count)?;
            Self::ensure_storage_clean(general_staker_info_count)?;
            Self::ensure_storage_clean(general_era_info_count)?;
            Self::ensure_storage_clean(ledger_count)?;

            ensure!(Pallet::<T>::on_chain_storage_version() == 1, "wrong storage version");
            ensure!(PalletDisabled::<T>::get() == true, "pallet should be disabled");
            ensure!(CurrentEra::<T>::get() == 0, "current era should be 0");
            ensure!(ForceEra::<T>::get() == Forcing::NotForcing, "force era should be NotForcing");
            ensure!(NextEraStartingBlock::<T>::get().is_zero(), "next era starting block should be 0");
            ensure!(
                BlockRewardAccumulator::<T>::get() == RewardInfo::default(),
                "block reward accumulator should be default",
            );
            ensure!(
                ActiveRewardDistributionConfig::<T>::get() == RewardDistributionConfig::default(),
                "active reward distribution config should be default",
            );

            ensure!(
                T::Currency::free_balance(&Pallet::<T>::account_id()).is_zero(),
                "pot account should be empty"
            );

            Ok(())
        }
    }
}
