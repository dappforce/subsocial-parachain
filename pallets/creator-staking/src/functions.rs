use crate::pallet::*;
use frame_support::{pallet_prelude::*, traits::{Currency, ReservableCurrency, LockableCurrency, WithdrawReasons}};
use sp_runtime::{traits::{AccountIdConversion, Zero}, Perbill, Saturating};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use subsocial_support::{SpaceId, traits::SpacesInterface};

impl<T: Config> Pallet<T> {
    /// Calculate the creator reward for the specified era.
    /// If successfull, returns reward amount.
    /// In case reward cannot be claimed or was already claimed, an error is raised.
    pub(super) fn calculate_creator_reward(
        creator_stake_info: &CreatorStakeInfo<BalanceOf<T>>,
        creator_info: &CreatorInfo<T::AccountId>,
        era: EraIndex,
    ) -> Result<BalanceOf<T>, Error<T>> {
        let current_era = Self::current_era();
        if let CreatorStatus::Inactive(unregister_era) = creator_info.status {
            ensure!(era < unregister_era, Error::<T>::InactiveCreator);
        }
        ensure!(era < current_era, Error::<T>::EraOutOfBounds);

        ensure!(
                !creator_stake_info.rewards_claimed,
                Error::<T>::AlreadyClaimedInThisEra,
            );
        ensure!(
                creator_stake_info.total > Zero::zero(),
                Error::<T>::NotStakedCreator,
            );

        let rewards_and_stakes =
            Self::general_era_info(era).ok_or(Error::<T>::EraNotFound)?;

        // Calculate the creator reward for this era.
        let (creator_reward, _) = Self::distributed_rewards_between_creator_and_stakers(creator_stake_info, &rewards_and_stakes);

        Ok(creator_reward)
    }

    /// `Err` if pallet disabled for maintenance, `Ok` otherwise
    pub fn ensure_pallet_enabled() -> Result<(), Error<T>> {
        if PalletDisabled::<T>::get() {
            Err(Error::<T>::PalletIsDisabled)
        } else {
            Ok(())
        }
    }

    /// Returns available staking balance for the potential staker
    pub(super) fn balance_available_for_staking(
        staker: &T::AccountId,
        ledger: &StakerLedgerOf<T>,
    ) -> BalanceOf<T> {
        // Ensure that staker has enough balance to bond & stake.
        let free_balance =
            T::Currency::free_balance(staker).saturating_sub(T::MinimumRemainingFreeBalance::get());

        // Remove already locked funds from the free balance
        free_balance.saturating_sub(ledger.locked)
    }

    /// `true` if creator is active, `false` if it has been unregistered
    pub(super) fn is_creator_active(space_id: SpaceId) -> bool {
        RegisteredCreators::<T>::get(space_id)
            .map_or(false, |info| info.status == CreatorStatus::Active)
    }

    /// `true` if all the conditions for restaking the reward have been met, `false` otherwise
    pub(crate) fn should_restake_reward(
        restake: bool,
        creator_state: CreatorStatus,
        current_stake: BalanceOf<T>,
    ) -> bool {
        restake
            && creator_state == CreatorStatus::Active
            && current_stake > Zero::zero()
    }

    pub(super) fn do_unregister_creator(
        space_id: SpaceId,
        unregister_origin: UnregistrationAuthority<T::AccountId>,
    ) -> DispatchResultWithPostInfo {
        let mut creator_info =
            RegisteredCreators::<T>::get(space_id).ok_or(Error::<T>::CreatorNotFound)?;

        ensure!(creator_info.status == CreatorStatus::Active, Error::<T>::InactiveCreator);
        let stakeholder = creator_info.stakeholder.clone();

        // TODO: make flexible register deposit
        if let UnregistrationAuthority::Root = unregister_origin {
            T::Currency::slash_reserved(&stakeholder, T::CreatorRegistrationDeposit::get());
        } else if let UnregistrationAuthority::Creator(who) = unregister_origin {
            let space_owner = T::SpacesInterface::get_space_owner(space_id)?;
            // FIXME: this error should be NotASpaceOwner
            ensure!(who == space_owner, Error::<T>::NotACreator);
            T::Currency::unreserve(&stakeholder, T::CreatorRegistrationDeposit::get());
        }

        let current_era = Self::current_era();
        creator_info.status = CreatorStatus::Inactive(current_era);
        RegisteredCreators::<T>::insert(space_id, creator_info);

        Ok(().into())
    }

    /// An utility method used to stake specified amount on an arbitrary creator.
    ///
    /// `StakerInfo` and `CreatorStakeInfo` are provided and all checks are made to ensure that
    /// it's possible to complete staking operation.
    ///
    /// # Arguments
    ///
    /// * `staker_info` - info about staker's stakes on the creator up to current moment
    /// * `staking_info` - general info about creator stakes up to current moment
    /// * `value` - value which is being bonded & staked
    /// * `current_era` - current creators-staking era
    ///
    /// # Returns
    ///
    /// If stake operation was successful, given structs are properly modified.
    /// If not, an error is returned and structs are left in an undefined state.
    pub(super) fn stake_to_creator(
        staker_info: &mut StakesInfoOf<T>,
        staking_info: &mut CreatorStakeInfo<BalanceOf<T>>,
        desired_amount: BalanceOf<T>,
        current_era: EraIndex,
    ) -> Result<(), Error<T>> {
        let current_stake = staker_info.current_stake();

        ensure!(
            !current_stake.is_zero() ||
                staking_info.stakers_count < T::MaxNumberOfStakersPerCreator::get(),
            Error::<T>::MaxNumberOfStakersExceeded
        );
        if current_stake.is_zero() {
            staking_info.stakers_count = staking_info.stakers_count.saturating_add(1);
        }

        staker_info
            .increase_stake(current_era, desired_amount)
            .map_err(|_| Error::<T>::UnexpectedStakeInfoEra)?;
        ensure!(
            // One extra item should remain for vector manipulation in a claim call.
            staker_info.len() < T::MaxEraStakeItems::get(),
            Error::<T>::TooManyEraStakeValues
        );
        ensure!(
            staker_info.current_stake() >= T::MinimumStake::get(),
            Error::<T>::InsufficientStakingAmount,
        );

        // Increment ledger and total staker value for creator.
        staking_info.total = staking_info.total.saturating_add(desired_amount);

        Ok(())
    }

    /// An utility method used to unstake specified amount from an arbitrary creator.
    ///
    /// The amount unstaked can be different in case staked amount would fall bellow
    /// `MinimumStakingAmount`. In that case, entire staked amount will be unstaked.
    ///
    /// `StakerInfo` and `CreatorStakeInfo` are provided and all checks are made to ensure that
    /// it's possible to complete unstake operation.
    ///
    /// # Arguments
    ///
    /// * `staker_info` - info about staker's stakes on the creator up to current moment
    /// * `staking_info` - general info about creator stakes up to current moment
    /// * `value` - value which should be unstaked
    /// * `current_era` - current creators-staking era
    ///
    /// # Returns
    ///
    /// If unstake operation was successful, given structs are properly modified and total
    /// unstaked value is returned. If not, an error is returned and structs are left in
    /// an undefined state.
    pub(super) fn calculate_final_unstaking_amount(
        staker_info: &mut StakesInfoOf<T>,
        stake_info: &mut CreatorStakeInfo<BalanceOf<T>>,
        desired_amount: BalanceOf<T>,
        current_era: EraIndex,
    ) -> Result<BalanceOf<T>, Error<T>> {
        let staked_value = staker_info.current_stake();
        ensure!(staked_value > Zero::zero(), Error::<T>::NotStakedCreator);

        // Calculate the value which will be unstaked.
        let remaining = staked_value.saturating_sub(desired_amount);

        // If remaining amount is less than minimum staking amount, unstake the entire amount.
        let amount_to_unstake = if remaining < T::MinimumStake::get() {
            stake_info.stakers_count = stake_info.stakers_count.saturating_sub(1);
            staked_value
        } else {
            desired_amount
        };
        stake_info.total = stake_info.total.saturating_sub(amount_to_unstake);

        // Sanity check
        ensure!(amount_to_unstake > Zero::zero(), Error::<T>::CannotUnstakeZero);

        staker_info
            .unstake(current_era, amount_to_unstake)
            .map_err(|_| Error::<T>::UnexpectedStakeInfoEra)?;
        ensure!(
                // One extra item should remain for vector manipulation in a claim call.
                staker_info.len() < T::MaxEraStakeItems::get(),
                Error::<T>::TooManyEraStakeValues
            );

        Ok(amount_to_unstake)
    }

    /// Update the ledger for a staker. This will also update the stash lock.
    /// This lock will lock the entire funds except paying for further transactions.
    pub(super) fn update_ledger(staker: &T::AccountId, ledger: StakerLedgerOf<T>) {
        if ledger.is_empty() {
            Ledger::<T>::remove(staker);
            T::Currency::remove_lock(STAKING_ID, staker);
        } else {
            T::Currency::set_lock(STAKING_ID, staker, ledger.locked, WithdrawReasons::all());
            Ledger::<T>::insert(staker, ledger);
        }
    }

    /// Update the staker info for the `(staker, creator_id)` pairing.
    /// If staker_info is empty, remove it from the DB. Otherwise, store it.
    pub(super) fn update_staker_info(
        staker: &T::AccountId,
        creator_id: SpaceId,
        staker_info: StakesInfoOf<T>,
    ) {
        if staker_info.is_empty() {
            GeneralStakerInfo::<T>::remove(staker, creator_id)
        } else {
            GeneralStakerInfo::<T>::insert(staker, creator_id, staker_info)
        }
    }

    /// Calculate the reward distribution between a creator and all their staking participants.
    ///
    /// Returns (creator's reward, stakers' combined reward)
    pub(crate) fn distributed_rewards_between_creator_and_stakers(
        creator_info: &CreatorStakeInfo<BalanceOf<T>>,
        era_info: &EraInfo<BalanceOf<T>>,
    ) -> (BalanceOf<T>, BalanceOf<T>) {
        let creator_proportional_stake =
            Perbill::from_rational(creator_info.total, era_info.staked);

        let creator_reward_share = creator_proportional_stake * era_info.rewards.creators;
        let combined_stakers_reward_share = creator_proportional_stake * era_info.rewards.stakers;

        (creator_reward_share, combined_stakers_reward_share)
    }

    /// This utility function converts the specified in a `Config` PalletId into an account ID.
    /// This account is deposited rewards before they are distributed to creators and stakers.
    pub(crate) fn rewards_pot_account() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    /// The block rewards are accumulated on the pallets's account during an era.
    /// This function takes a snapshot of the pallet's balance accrued during current era
    /// and stores it for future distribution.
    ///
    /// This is called just at the beginning of an era.
    pub(super) fn reward_balance_snapshot(era: EraIndex, rewards: RewardInfo<BalanceOf<T>>) {
        // Get the reward and stake information for previous era
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

    /// Used to copy all `CreatorStakeInfo` from the ending era over to the next era.
    /// This is the most primitive solution since it scales with number of creators.
    /// It is possible to provide a hybrid solution which allows laziness but also prevents
    /// a situation where we don't have access to the required data.
    pub(super) fn rotate_staking_info(current_era: EraIndex) -> Weight {
        let next_era = current_era + 1;

        let mut consumed_weight = Weight::zero();

        for (space_id, creator_info) in RegisteredCreators::<T>::iter() {
            // Ignore creator if it was unregistered
            consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
            if let CreatorStatus::Inactive(_) = creator_info.status {
                continue;
            }

            // Copy data from era `X` to era `X + 1`
            if let Some(mut staking_info) = Self::creator_stake_info(space_id, current_era)
            {
                staking_info.rewards_claimed = false;
                CreatorEraStake::<T>::insert(space_id, next_era, staking_info);

                consumed_weight =
                    consumed_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            } else {
                consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
            }
        }

        consumed_weight
    }

    fn calculate_reward_for_staker_in_era(
        creator_stake_info: &CreatorStakeInfo<BalanceOf<T>>,
        staked: BalanceOf<T>,
        era: EraIndex,
    ) -> BalanceOf<T> {
        if let Some(reward_and_stake) = Self::general_era_info(era) {
            let (_, combined_stakers_reward_share) =
                Self::distributed_rewards_between_creator_and_stakers(creator_stake_info, &reward_and_stake);
            Perbill::from_rational(staked, creator_stake_info.total) * combined_stakers_reward_share
        } else {
            Zero::zero()
        }
    }

    pub fn estimated_staker_rewards_by_creators(
        staker: T::AccountId,
        mut target_creators: Vec<SpaceId>,
    ) -> Vec<(SpaceId, BalanceOf<T>)> {
        let mut estimated_rewards: Vec<(SpaceId, BalanceOf<T>)> = Vec::new();
        target_creators.dedup();

        let current_era = Self::current_era();

        for creator_id in target_creators {
            let mut staker_info_for_creator = Self::staker_info(&staker, creator_id);

            let mut unregistered_era = current_era;
            if let Some(creator_info) = Self::registered_creator(creator_id) {
                if let CreatorStatus::Inactive(era) = creator_info.status {
                    unregistered_era = era;
                }
            }

            if staker_info_for_creator.stakes.is_empty() {
                estimated_rewards.push((creator_id, Zero::zero()));
                continue;
            }

            let mut total_staker_rewards_for_eras: BalanceOf<T> = Zero::zero();
            loop {
                let (era, staked) = staker_info_for_creator.claim();
                if era >= unregistered_era || era == 0 {
                    break;
                }
                let creator_stake_info = Self::creator_stake_info(creator_id, era).unwrap_or_default();

                total_staker_rewards_for_eras = total_staker_rewards_for_eras.saturating_add(
                    Self::calculate_reward_for_staker_in_era(&creator_stake_info, staked, era)
                );
            }

            estimated_rewards.push((creator_id, total_staker_rewards_for_eras));
        }

        estimated_rewards
    }

    pub fn withdrawable_amounts_from_inactive_creators(
        staker: T::AccountId,
    ) -> Vec<(SpaceId, BalanceOf<T>)> {
        let mut withdrawable_amounts_by_creator = Vec::new();

        for (creator_id, staker_info) in GeneralStakerInfo::<T>::iter_prefix(&staker) {
            if !Self::is_creator_active(creator_id) {
                if let Some(most_recent_stake) = staker_info.stakes.last() {
                    withdrawable_amounts_by_creator.push((creator_id, most_recent_stake.staked));
                }
            }
        }

        withdrawable_amounts_by_creator
    }

    pub fn available_claims_by_staker(
        staker: T::AccountId,
    ) -> Vec<(SpaceId, u32)> {
        let mut available_claims_by_creator = BTreeMap::new();

        let current_era = Self::current_era();

        for (creator, mut stakes_info) in GeneralStakerInfo::<T>::iter_prefix(&staker) {
            let mut unregistered_era = current_era;
            if let Some(creator_info) = Self::registered_creator(creator) {
                if let CreatorStatus::Inactive(era) = creator_info.status {
                    unregistered_era = era;
                }
            } else {
                continue;
            }

            loop {
                let (era, _) = stakes_info.claim();
                if era >= unregistered_era || era == 0 {
                    break;
                }

                available_claims_by_creator.entry(creator).and_modify(|e| *e += 1).or_insert(1);
            }
        }

        available_claims_by_creator.into_iter().collect()
    }
}
