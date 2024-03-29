// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use crate::pallet::*;
use frame_support::{pallet_prelude::*, traits::{Currency, ReservableCurrency, LockableCurrency, WithdrawReasons}};
use sp_runtime::{traits::{AccountIdConversion, Zero}, Perbill, Saturating};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use subsocial_support::traits::{CreatorStakingProvider, SpacePermissionsProvider};

impl<T: Config> Pallet<T> {
    /// `Err` if pallet disabled for maintenance, `Ok` otherwise
    pub fn ensure_pallet_enabled() -> Result<(), Error<T>> {
        if PalletDisabled::<T>::get() {
            Err(Error::<T>::PalletIsDisabled)
        } else {
            Ok(())
        }
    }

    /// Returns available staking balance for the potential backer
    pub(crate) fn balance_available_for_staking(
        backer: &T::AccountId,
        backer_locks: &BackerLocksOf<T>,
    ) -> BalanceOf<T> {
        // Ensure that backer has enough balance to bond & stake.
        let free_balance = T::Currency::free_balance(backer)
            .saturating_sub(T::MinimumRemainingFreeBalance::get());

        // Remove already locked funds from the free balance
        free_balance.saturating_sub(backer_locks.total_locked)
    }

    /// `true` if creator is active, `false` if it has been unregistered (i.e. inactive)
    pub(crate) fn is_creator_active(creator_id: CreatorId) -> bool {
        Self::require_creator(creator_id)
            .map_or(false, |info| info.status == CreatorStatus::Active)
    }

    pub(crate) fn ensure_creator_is_active(creator_id: CreatorId) -> DispatchResult {
        ensure!(Self::is_creator_active(creator_id), Error::<T>::InactiveCreator);
        Ok(())
    }

    pub(crate) fn ensure_creator_active_in_era(
        creator_info: &CreatorInfo<T::AccountId>,
        era: EraIndex,
    ) -> DispatchResult {
        match creator_info.status {
            CreatorStatus::Active => Ok(()),
            CreatorStatus::Inactive(unregistration_era) if era < unregistration_era => Ok(()),
            _ => Err(Error::<T>::InactiveCreator.into()),
        }
    }

    pub(crate) fn do_unregister_creator(
        creator_id: CreatorId,
        unregister_origin: UnregistrationAuthority<T::AccountId>,
    ) -> DispatchResultWithPostInfo {
        let mut creator_info = Self::require_creator(creator_id)?;

        ensure!(creator_info.status == CreatorStatus::Active, Error::<T>::InactiveCreator);
        let stakeholder = creator_info.stakeholder.clone();

        // TODO: make the registration deposit flexible
        if let UnregistrationAuthority::Root = unregister_origin {
            T::Currency::slash_reserved(&stakeholder, T::CreatorRegistrationDeposit::get());
        } else if let UnregistrationAuthority::Creator(who) = unregister_origin {
            T::SpacePermissionsProvider::ensure_space_owner(creator_id, &who)?;
            T::Currency::unreserve(&stakeholder, T::CreatorRegistrationDeposit::get());
        }

        let current_era = Self::current_era();
        creator_info.status = CreatorStatus::Inactive(current_era);
        RegisteredCreators::<T>::insert(creator_id, creator_info);

        Ok(().into())
    }

    /// A utility method used to stake a specified amount on an arbitrary creator.
    ///
    /// `StakesInfoOf` and `CreatorStakeInfo` are provided and all checks are made to ensure that
    /// it's possible to complete the staking operation.
    ///
    /// # Arguments
    ///
    /// * `backer_stakes` - info about backer's stakes on the creator up to current moment
    /// * `staking_info` - general info about a particular creator's stake up to the current moment
    /// * `value` - value which is being bonded & staked
    /// * `current_era` - the current era of the creator staking system
    ///
    /// # Returns
    ///
    /// If the stake operation was successful, the given structs are properly modified.
    /// If not, an error is returned and the structs are left in an undefined state.
    pub(crate) fn stake_to_creator(
        backer: &T::AccountId,
        backer_stakes: &mut StakesInfoOf<T>,
        creator_stake_info: &mut CreatorStakeInfo<BalanceOf<T>>,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        // let current_era = Self::current_era();
        let staked_before = backer_stakes.staked;

        if staked_before.is_zero() {
            creator_stake_info.backers_count.saturating_inc();
        }

        // backer_stakes
        //     .increase_stake(current_era, amount)
        //     .map_err(|_| Error::<T>::CannotChangeStakeInPastEra)?;

        backer_stakes.staked.saturating_accrue(amount);

        let total_stake = Self::total_staked_amount(&backer).saturating_add(amount);
        ensure!(total_stake >= T::MinimumTotalStake::get(), Error::<T>::InsufficientStakingAmount);

        // Increment the backer's total deposit for a particular creator.
        creator_stake_info.total_staked = creator_stake_info.total_staked.saturating_add(amount);

        Ok(())
    }

    /// A utility method used to unstake a specified amount from an arbitrary creator.
    ///
    /// The amount unstaked can be different in case the staked amount would fall bellow zero.
    /// In that case, only the staked amount will be unstaked.
    ///
    /// `StakesInfoOf` and `CreatorStakeInfo` are provided and all checks are made to ensure that
    /// it's possible to complete the unstake operation.
    ///
    /// # Arguments
    ///
    /// * `backer_stakes` - info about backer's stakes on the creator up to current moment
    /// * `staking_info` - general info about creator stakes up to current moment
    /// * `value` - value which should be unstaked
    /// * `current_era` - current creator-staking era
    ///
    /// # Returns
    ///
    /// If the unstake operation was successful, the given structs are properly modified and the total
    /// unstaked value is returned. If not, an error is returned and the structs are left in
    /// an undefined state.
    pub(crate) fn calculate_and_apply_stake_decrease(
        backer_stakes: &mut StakesInfoOf<T>,
        creator_stake_info: &mut CreatorStakeInfo<BalanceOf<T>>,
        desired_amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        // let current_era = Self::current_era();
        let staked_value = backer_stakes.staked;
        ensure!(staked_value > Zero::zero(), Error::<T>::NotStakedCreator);

        // If the remaining amount equals to zero, unstake the entire amount by this creator.
        let amount_to_decrease = if desired_amount >= staked_value {
            creator_stake_info.backers_count = creator_stake_info.backers_count.saturating_sub(1);
            staked_value
        } else {
            desired_amount
        };

        // Sanity check
        ensure!(amount_to_decrease > Zero::zero(), Error::<T>::CannotUnstakeZero);

        // Modify data
        creator_stake_info.total_staked = creator_stake_info.total_staked.saturating_sub(amount_to_decrease);

        // backer_stakes
        //     .decrease_stake(current_era, amount_to_decrease)
        //     .map_err(|_| Error::<T>::CannotChangeStakeInPastEra)?;

        backer_stakes.staked.saturating_reduce(amount_to_decrease);

        Ok(amount_to_decrease)
    }

    /// Update the locks for a backer. This will also update the stash lock.
    /// This lock will lock the entire funds except paying for further transactions.
    pub(crate) fn update_backer_locks(backer: &T::AccountId, backer_locks: BackerLocksOf<T>) {
        if backer_locks.is_empty() {
            BackerLocksByAccount::<T>::remove(backer);
            T::Currency::remove_lock(STAKING_LOCK_ID, backer);
        } else {
            T::Currency::set_lock(STAKING_LOCK_ID, backer, backer_locks.total_locked, WithdrawReasons::all());
            BackerLocksByAccount::<T>::insert(backer, backer_locks);
        }
    }

    /// Update the backer info for the `(backer, creator_id)` pairing.
    /// If backer_stakes is empty, remove it from the DB. Otherwise, store it.
    pub(crate) fn update_backer_stakes(
        backer: &T::AccountId,
        creator_id: CreatorId,
        backer_stakes: StakesInfoOf<T>,
    ) {
        if backer_stakes.is_empty() && backer_stakes.staked.is_zero() {
            BackerStakesByCreator::<T>::remove(backer, creator_id)
        } else {
            BackerStakesByCreator::<T>::insert(backer, creator_id, backer_stakes)
        }
    }

    /// Calculate the reward for a creator in a specific era.
    pub(crate) fn calculate_creator_reward(
        creator_info: &CreatorStakeInfo<BalanceOf<T>>,
        era_info: &EraInfo<BalanceOf<T>>,
    ) -> BalanceOf<T> {
        let creator_reward_ratio =
            Perbill::from_rational(creator_info.total_staked, era_info.staked);

        creator_reward_ratio * era_info.rewards.creators
    }

    /// This utility function converts the PalletId specified in `Config` into an account ID.
    /// Rewards are deposited into this account before they are distributed to creators and backers.
    pub(crate) fn rewards_pot_account() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    pub(crate) fn total_staked_amount(backer: &T::AccountId) -> BalanceOf<T> {
        let backer_locks = BackerLocksByAccount::<T>::get(&backer);
        backer_locks.total_staked()
    }

    /// The block rewards are accumulated in this pallet's account during each era.
    /// This function takes a snapshot of the pallet's balance accrued during the current era
    /// and stores it for future distribution.
    ///
    /// This is only called at the beginning of an era.
    pub(crate) fn reward_balance_snapshot(era: EraIndex, rewards: RewardInfo<BalanceOf<T>>) {
        // Gets the reward and stake information for the previous era
        let mut era_info = Self::general_era_info(era).unwrap_or_default();

        // Prepare info for the next era
        GeneralEraInfo::<T>::insert(
            era + 1,
            EraInfo {
                rewards: Default::default(),
                staked: era_info.staked,
                locked: era_info.locked,
            },
        );

        // Set the reward for the previous era.
        era_info.rewards = rewards;

        GeneralEraInfo::<T>::insert(era, era_info);
    }

    /// Used to copy all `CreatorStakeInfo` from the previous era over to the next era.
    ///
    /// This is the most primitive solution since it scales with the number of creators.
    /// It is possible to provide a hybrid solution which allows laziness, but might also lead to
    /// a situation where we don't have access to the required data.
    pub(crate) fn rotate_staking_info(current_era: EraIndex) -> Weight {
        let next_era = current_era + 1;

        let mut consumed_weight = Weight::zero();

        for (creator_id, creator_info) in RegisteredCreators::<T>::iter() {
            // Ignore creator if it is inactive
            consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
            if let CreatorStatus::Inactive(_) = creator_info.status {
                continue;
            }

            // Copy data from era `X` to era `X + 1`
            if let Some(mut staking_info) = Self::creator_stake_info(creator_id, current_era)
            {
                staking_info.rewards_claimed = false;
                CreatorStakeInfoByEra::<T>::insert(creator_id, next_era, staking_info);

                consumed_weight =
                    consumed_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            } else {
                consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
            }
        }

        consumed_weight
    }

    pub(crate) fn require_creator(creator_id: CreatorId) -> Result<CreatorInfo<T::AccountId>, DispatchError> {
        RegisteredCreators::<T>::get(creator_id).ok_or(Error::<T>::CreatorNotFound.into())
    }

    pub(crate) fn can_restake_reward(
        restake: bool,
        creator_status: CreatorStatus,
        staked: BalanceOf<T>,
    ) -> bool {
        // Can restake only if the backer is already staking on the active creator
        restake && creator_status == CreatorStatus::Active && staked > Zero::zero()
    }

    pub(crate) fn do_restake_reward(
        backer: &T::AccountId,
        backer_reward: BalanceOf<T>,
        creator_id: CreatorId,
        current_era: EraIndex,
    ) {
        let mut backer_locks = Self::backer_locks(&backer);
        backer_locks.total_locked = backer_locks.total_locked.saturating_add(backer_reward);
        Self::update_backer_locks(&backer, backer_locks);

        // Update storage
        GeneralEraInfo::<T>::mutate(current_era, |value| {
            if let Some(x) = value {
                x.staked = x.staked.saturating_add(backer_reward);
                x.locked = x.locked.saturating_add(backer_reward);
            }
        });

        CreatorStakeInfoByEra::<T>::mutate(creator_id, current_era, |staking_info| {
            if let Some(x) = staking_info {
                x.total_staked = x.total_staked.saturating_add(backer_reward);
            }
        });

        Self::deposit_event(Event::<T>::Staked {
            who: backer.clone(),
            creator_id,
            era: current_era,
            amount: backer_reward,
        });
    }

    pub(crate) fn calculate_reward_for_backer_in_era(
        staked: BalanceOf<T>,
        era: EraIndex,
    ) -> BalanceOf<T> {
        if let Some(era_info) = Self::general_era_info(era) {
            Perbill::from_rational(staked, era_info.staked) * era_info.rewards.backers
        } else {
            Zero::zero()
        }
    }

    // For internal use only.
    fn get_unregistration_era_index(creator_id: CreatorId) -> Result<EraIndex, DispatchError> {
        return if let Some(creator_info) = Self::registered_creator(creator_id) {
            if let CreatorStatus::Inactive(unregistration_era) = creator_info.status {
                Ok(unregistration_era)
            } else {
                Err(Error::<T>::CreatorIsActive.into())
            }
        } else {
            Err(Error::<T>::CreatorNotFound.into())
        }
    }

    /// Returns total value locked by creator-staking.
    ///
    /// Note that this can differ from _total staked value_ since some funds might be undergoing the unbonding period.
    pub fn tvl() -> BalanceOf<T> {
        let current_era = Self::current_era();
        if let Some(era_info) = Self::general_era_info(current_era) {
            era_info.locked
        } else {
            // Should never happen since era info for current era must always exist
            Zero::zero()
        }
    }

    pub fn estimated_backer_rewards_by_creators(
        backer: T::AccountId,
        mut target_creators: Vec<CreatorId>,
    ) -> Vec<(CreatorId, BalanceOf<T>)> {
        let mut estimated_rewards: Vec<(CreatorId, BalanceOf<T>)> = Vec::new();
        target_creators.dedup();

        let current_era = Self::current_era();

        for creator_id in target_creators {
            let mut backer_stakes_for_creator = Self::backer_stakes(&backer, creator_id);

            let unregistration_era =
                Self::get_unregistration_era_index(creator_id).unwrap_or(current_era);

            if backer_stakes_for_creator.stakes.is_empty() {
                estimated_rewards.push((creator_id, Zero::zero()));
                continue;
            }

            let mut total_backer_rewards_for_eras: BalanceOf<T> = Zero::zero();
            loop {
                let (era, staked) = backer_stakes_for_creator.claim();
                if era >= unregistration_era || era == 0 {
                    break;
                }

                let backer_era_reward = |_| {
                    Self::calculate_reward_for_backer_in_era(staked, era)
                };
                let creator_stake_info = Self::creator_stake_info(creator_id, era);

                total_backer_rewards_for_eras = total_backer_rewards_for_eras.saturating_add(
                    creator_stake_info.map_or(Zero::zero(), backer_era_reward)
                );
            }

            estimated_rewards.push((creator_id, total_backer_rewards_for_eras));
        }

        estimated_rewards
    }

    pub fn withdrawable_amounts_from_inactive_creators(
        backer: T::AccountId,
    ) -> Vec<(CreatorId, BalanceOf<T>)> {
        let mut withdrawable_amounts_by_creator = Vec::new();

        for (creator_id, backer_stakes) in BackerStakesByCreator::<T>::iter_prefix(&backer) {
            if !Self::is_creator_active(creator_id) {
                if let Some(most_recent_stake) = backer_stakes.stakes.last() {
                    withdrawable_amounts_by_creator.push((creator_id, most_recent_stake.staked));
                }
            }
        }

        withdrawable_amounts_by_creator
    }

    pub fn available_claims_by_backer(backer: T::AccountId) -> Vec<(CreatorId, u32)> {
        let mut available_backer_claims_by_creator = BTreeMap::new();

        let current_era = Self::current_era();

        for (creator, mut stakes_info) in BackerStakesByCreator::<T>::iter_prefix(&backer) {
            let last_claimable_era = match Self::get_unregistration_era_index(creator) {
                Err(error) if error.eq(&Error::<T>::CreatorNotFound.into()) => continue,
                Ok(unregistration_era) => unregistration_era,
                _ => current_era,
            };

            loop {
                let (era, _) = stakes_info.claim();
                if era >= last_claimable_era || era == 0 {
                    break;
                }

                available_backer_claims_by_creator.entry(creator)
                    .and_modify(|count| *count += 1).or_insert(1);
            }
        }

        available_backer_claims_by_creator.into_iter().collect()
    }

    pub fn estimated_creator_rewards(creator_id: CreatorId) -> BalanceOf<T> {
        let mut estimated_rewards: BalanceOf<T> = Zero::zero();

        for (era, stake_info) in CreatorStakeInfoByEra::<T>::iter_prefix(creator_id) {
            if !stake_info.rewards_claimed {
                if let Some(era_info) = Self::general_era_info(era) {
                    let creator_reward =
                        Self::calculate_creator_reward(&stake_info, &era_info);
                    estimated_rewards = estimated_rewards
                        .saturating_add(creator_reward);
                }
            }
        }

        estimated_rewards
    }

    pub fn available_claims_by_creator(creator_id: CreatorId) -> Vec<EraIndex> {
        let mut available_claims: Vec<EraIndex> = Vec::new();

        let current_era = Self::current_era();
        for (era, stake_info) in CreatorStakeInfoByEra::<T>::iter_prefix(creator_id) {
            if !stake_info.rewards_claimed && era < current_era {
                available_claims.push(era);
            }
        }

        available_claims.sort();
        available_claims
    }
}

/// Implementation of `CreatorStakingProvider` for `creator-staking` pallet.
///
/// This is used in ownership pallet to forbid ownership transfer for spaces, which
/// are registered as creators.
impl<T: Config> CreatorStakingProvider<T::AccountId> for Pallet<T> {
    fn is_creator_active(
        creator_id: CreatorId,
    ) -> bool {
        Self::is_creator_active(creator_id)
    }
}
