#![cfg_attr(not(feature = "std"), no_std)]
// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


pub use pallet::*;

pub mod types;
pub mod functions;
pub mod inflation;
#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
        traits::{Currency, LockIdentifier, LockableCurrency, ReservableCurrency, WithdrawReasons, ExistenceRequirement},
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::Zero, Perbill, Saturating};

    use pallet_permissions::SpacePermissionsInfoOf;
    use subsocial_support::{traits::{SpacesInterface, SpacePermissionsProvider}, SpaceId};

    pub use crate::types::*;

    /// An identifier for the locks made in this pallet.
    /// Used to determine the locks in this pallet so that they can be replaced or removed.
    pub(crate) const STAKING_LOCK_ID: LockIdentifier = *b"crestake";

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_permissions::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Creator staking pallet Id
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Number of blocks per era.
        #[pallet::constant]
        type BlockPerEra: Get<BlockNumberFor<Self>>;

        /// The currency trait.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
            + ReservableCurrency<Self::AccountId>;

        type SpacesInterface: SpacesInterface<Self::AccountId, SpaceId>;

        type SpacePermissionsProvider: SpacePermissionsProvider<
            Self::AccountId,
            SpacePermissionsInfoOf<Self>,
        >;

        /// Specifies the amount of tokens required as a deposit for creator registration.
        ///
        /// This constant defines the number of tokens that a creator must lock as collateral
        /// in order to register on the platform. The tokens act as a deposit and may be
        /// returned under certain conditions, such as account de-registration.
        #[pallet::constant]
        type CreatorRegistrationDeposit: Get<BalanceOf<Self>>;

        /// The minimum amount that should be staked in the creator staking system.
        /// User can stake less if they already have the minimum stake staked in the system.
        #[pallet::constant]
        type MinimumTotalStake: Get<BalanceOf<Self>>;

        // TODO: make it MinimumRemainingRatio
        //  (e.g. 0.1 = 10%, so that account can lock only 90% of its balance)
        /// The minimum amount a backer's balance that should be left on their account after staking.
        /// Serves as a safeguard to prevent users from locking their entire free balance.
        #[pallet::constant]
        type MinimumRemainingFreeBalance: Get<BalanceOf<Self>>;

        /// Maximum number of unique backers per creator.
        #[pallet::constant]
        type MaxNumberOfBackersPerCreator: Get<u32>;

        /// The max number of unique `EraStake` items that can exist for a `(backer, creator)`
        /// pair. When backers claims rewards, they will either keep the number of
        /// `EraStake` items the same or they will reduce them by one. Backers cannot add
        /// an additional `EraStake` value by calling `bond() & stake()` or `unbond() & unstake()` if they've
        /// reached the max number of values.
        ///
        /// This ensures that history doesn't grow indefinitely - if there are too many chunks,
        /// backers should first claim their former rewards before adding additional
        /// `EraStake` values.
        #[pallet::constant]
        type MaxEraStakeItems: Get<u32>;

        #[pallet::constant]
        type StakeExpirationInEras: Get<EraIndex>;

        /// The number of eras that need to pass until an unstaked value can be withdrawn.
        /// Current era is always counted as full era (regardless how much blocks are remaining).
        /// When set to `0`, it's equal to having no unbonding period.
        #[pallet::constant]
        type UnbondingPeriodInEras: Get<u32>;

        /// The max number of unbonding chunks per `(backer, creator)` pair.
        /// If value is zero, unbonding becomes impossible.
        #[pallet::constant]
        type MaxUnbondingChunks: Get<u32>;

        #[pallet::constant]
        type InitialRewardPerBlock: Get<BalanceOf<Self>>;

        /// Represents the estimated number of blocks that are generated within the span of one year.
        #[pallet::constant]
        type BlocksPerYear: Get<Self::BlockNumber>;

        /// The chain's treasury account, where we deposit leftover tokens after distributing rewards,
        /// not to make any extra tokens left on a rewards holding account.
        ///
        /// Furthermore, a part of inflation may be distributed into this account in accordance
        /// with `ActiveRewardDistributionConfig`.
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::type_value]
    pub fn PalletDisabledDefault() -> bool {
        true
    }
    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn pallet_disabled)]
    pub type PalletDisabled<T: Config> = StorageValue<_, bool, ValueQuery, PalletDisabledDefault>;

    /// The current era index.
    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, EraIndex, ValueQuery>;

    #[pallet::type_value]
    pub fn ForceEraOnEmpty() -> Forcing {
        Forcing::NotForcing
    }

    /// Mode of era forcing.
    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn force_era)]
    pub type ForceEra<T> = StorageValue<_, Forcing, ValueQuery, ForceEraOnEmpty>;

    /// Stores the block number of when the next era starts
    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn next_era_starting_block)]
    pub type NextEraStartingBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn registered_creator)]
    pub(crate) type RegisteredCreators<T: Config> =
        StorageMap<_, Twox64Concat, CreatorId, CreatorInfo<T::AccountId>>;

    /// Staking information about a creator in a particular era.
    #[pallet::storage]
    #[pallet::getter(fn creator_stake_info)]
    pub type CreatorStakeInfoByEra<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CreatorId,
        Twox64Concat,
        EraIndex,
        CreatorStakeInfo<BalanceOf<T>>,
    >;

    /// Info about backers stakes on particular creator.
    #[pallet::storage]
    #[pallet::getter(fn backer_stakes)]
    pub type BackerStakesByCreator<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        CreatorId,
        StakesInfoOf<T>,
        ValueQuery,
    >;

    /// General information about an era like TVL, total staked value, rewards.
    #[pallet::storage]
    #[pallet::getter(fn general_era_info)]
    pub type GeneralEraInfo<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, EraInfo<BalanceOf<T>>>;

    /// General information about the backer.
    #[pallet::storage]
    #[pallet::getter(fn backer_locks)]
    pub type BackerLocksByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BackerLocksOf<T>, ValueQuery>;

    /// Accumulator for block rewards during an era. It is reset at every new era
    #[pallet::storage]
    #[pallet::getter(fn block_reward_accumulator)]
    pub type BlockRewardAccumulator<T> = StorageValue<_, RewardInfo<BalanceOf<T>>, ValueQuery>;

    #[pallet::type_value]
    pub fn RewardConfigOnEmpty() -> RewardDistributionConfig {
        RewardDistributionConfig::default()
    }

    /// An active list of the configuration parameters used to calculate the reward distribution.
    #[pallet::storage]
    #[pallet::getter(fn reward_config)]
    pub type ActiveRewardDistributionConfig<T> =
        StorageValue<_, RewardDistributionConfig, ValueQuery, RewardConfigOnEmpty>;

    #[pallet::type_value]
    pub fn RewardPerBlockOnEmpty<T: Config>() -> BalanceOf<T> {
        T::InitialRewardPerBlock::get()
    }

    #[pallet::storage]
    #[pallet::getter(fn per_block_reward)]
    pub type RewardPerBlock<T> = StorageValue<_, BalanceOf<T>, ValueQuery, RewardPerBlockOnEmpty<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        Staked { who: T::AccountId, creator_id: CreatorId, era: EraIndex, amount: BalanceOf<T> },
        Unstaked { who: T::AccountId, creator_id: CreatorId, era: EraIndex, amount: BalanceOf<T> },
        StakeMoved { who: T::AccountId, from_creator_id: CreatorId, to_creator_id: CreatorId, amount: BalanceOf<T> },
        BackerRewardsClaimed { who: T::AccountId, creator_id: CreatorId, amount: BalanceOf<T> },
        CreatorRewardsClaimed { who: T::AccountId, amount: BalanceOf<T> },
        StakeWithdrawn { who: T::AccountId, amount: BalanceOf<T> },
        StakeWithdrawnFromInactiveCreator { who: T::AccountId, creator_id: CreatorId, amount: BalanceOf<T> },
        AnnualInflationSet { value: Perbill },
        RewardsCalculated { total_rewards_amount: BalanceOf<T> },
        CreatorRegistered { who: T::AccountId, creator_id: CreatorId },
        CreatorUnregistered { who: T::AccountId, creator_id: CreatorId },
        CreatorUnregisteredWithSlash { creator_id: CreatorId, slash_amount: BalanceOf<T> },
        NewCreatorStakingEra { era: EraIndex },
        MaintenanceModeSet { enabled: bool },
        RewardDistributionConfigChanged { new_config: RewardDistributionConfig },
        RewardPerBlockChanged { new_reward: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pallet is disabled.
        PalletIsDisabled,
        CreatorAlreadyRegistered,
        CreatorNotFound,
        InactiveCreator,
        CannotStakeZero,
        CannotUnstakeZero,
        MaxNumberOfBackersExceeded,
        CannotChangeStakeInPastEra,
        TooManyEraStakeValues,
        InsufficientStakingAmount,
        NotStakedCreator,
        TooManyUnbondingChunks,
        NothingToWithdraw,
        CreatorIsActive,
        UnclaimedRewardsRemaining,
        CannotClaimInFutureEra,
        EraNotFound,
        AlreadyClaimedInThisEra,
        MaintenanceModeNotChanged,
        InvalidSumOfRewardDistributionConfig,
        StakeHasExpired,
        CannotMoveStakeToSameCreator,
        CannotMoveZeroStake,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            // This code serves as a safety measure to prevent any changes to storage
            // while the pallet is disabled.
            //
            // Even though this might extend the current era, that's considered acceptable.
            // Ideally, any runtime upgrades should be completed before a new era starts.
            // This is just a fallback mechanism to ensure system integrity
            // if the timing isn't perfect.
            if PalletDisabled::<T>::get() {
                return T::DbWeight::get().reads(1);
            }

            let force_new_era = Self::force_era().eq(&Forcing::ForceNew);
            let previous_era = Self::current_era();
            let next_era_starting_block = Self::next_era_starting_block();

            // Value is compared to 1 since genesis block is ignored
            if now >= next_era_starting_block || force_new_era || previous_era.is_zero() {
                let blocks_per_era = T::BlockPerEra::get();
                let next_era = previous_era + 1;
                CurrentEra::<T>::put(next_era);

                NextEraStartingBlock::<T>::put(now + blocks_per_era);

                let reward = BlockRewardAccumulator::<T>::take();
                // 2 reads and 1 write inside the `reward_balance_snapshot` fn
                Self::reward_balance_snapshot(previous_era, reward);
                let consumed_weight = Self::rotate_staking_info(previous_era);

                if force_new_era {
                    ForceEra::<T>::put(Forcing::NotForcing);
                }

                Self::deposit_event(Event::<T>::NewCreatorStakingEra { era: next_era });

                let force_new_era_write = if force_new_era { 1 } else { 0 };
                consumed_weight + T::DbWeight::get().reads_writes(6, 5 + force_new_era_write)
            } else {
                T::DbWeight::get().reads(4)
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(100_000, 0) + T::DbWeight::get().reads_writes(3, 1))]
        pub fn force_register_creator(
            origin: OriginFor<T>,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            ensure_root(origin)?;

            ensure!(
                !RegisteredCreators::<T>::contains_key(space_id),
                Error::<T>::CreatorAlreadyRegistered,
            );

            let space_owner = T::SpacesInterface::get_space_owner(space_id)?;
            T::Currency::reserve(&space_owner, T::CreatorRegistrationDeposit::get())?;

            RegisteredCreators::<T>::insert(space_id, CreatorInfo::new(space_owner.clone()));

            Self::deposit_event(Event::<T>::CreatorRegistered { who: space_owner, creator_id: space_id });

            Ok(Pays::No.into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(100_000, 0) + T::DbWeight::get().reads_writes(2, 1))]
        pub fn unregister_creator(
            origin: OriginFor<T>,
            creator_id: CreatorId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let who = ensure_signed(origin)?;

            Self::do_unregister_creator(creator_id, UnregistrationAuthority::Creator(who.clone()))?;

            Self::deposit_event(Event::<T>::CreatorUnregistered { who, creator_id });

            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(100_000, 0) + T::DbWeight::get().reads_writes(2, 1))]
        pub fn force_unregister_creator(
            origin: OriginFor<T>,
            creator_id: CreatorId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            ensure_root(origin)?;

            Self::do_unregister_creator(creator_id, UnregistrationAuthority::Root)?;

            Self::deposit_event(Event::<T>::CreatorUnregisteredWithSlash {
                creator_id,
                slash_amount: T::CreatorRegistrationDeposit::get(),
            });

            Ok(Pays::No.into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn stake(
            origin: OriginFor<T>,
            creator_id: CreatorId,
            #[pallet::compact] desired_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let backer = ensure_signed(origin)?;

            // Check that a creator is ready for staking.
            Self::ensure_creator_is_active(creator_id)?;

            // Retrieve the backer's locks, or create an entry if it doesn't exist.
            let mut backer_locks = Self::backer_locks(&backer);
            let available_balance = Self::balance_available_for_staking(&backer, &backer_locks);
            let amount_to_stake = desired_amount.min(available_balance);
            ensure!(amount_to_stake > Zero::zero(), Error::<T>::CannotStakeZero);

            let current_era = Self::current_era();
            let mut backer_stakes = Self::backer_stakes(&backer, creator_id);
            let mut staking_info =
                Self::creator_stake_info(creator_id, current_era).unwrap_or_default();

            Self::stake_to_creator(
                &backer,
                &mut backer_stakes,
                &mut staking_info,
                amount_to_stake,
            )?;

            backer_locks.total_locked = backer_locks.total_locked.saturating_add(amount_to_stake);

            // Update storage
            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_add(amount_to_stake);
                    x.locked = x.locked.saturating_add(amount_to_stake);
                }
            });

            Self::update_backer_locks(&backer, backer_locks);
            Self::update_backer_stakes(&backer, creator_id, backer_stakes);
            CreatorStakeInfoByEra::<T>::insert(creator_id, current_era, staking_info);

            Self::deposit_event(Event::<T>::Staked {
                who: backer,
                creator_id,
                era: current_era,
                amount: amount_to_stake,
            });
            Ok(().into())
        }

        /// Start unbonding process and unstake balance from the creator.
        ///
        /// The unstaked amount will no longer be eligible for rewards but still won't be unlocked.
        /// User needs to wait for the unbonding period to finish before being able to withdraw
        /// the funds via the `withdraw_unbonded` call.
        ///
        /// If the remaining balance staked on that creator is below the minimum staking amount,
        /// the entire stake for that creator will be unstaked.
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn unstake(
            origin: OriginFor<T>,
            creator_id: CreatorId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let backer = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::CannotUnstakeZero);
            Self::ensure_creator_is_active(creator_id)?;

            let current_era = Self::current_era();
            let mut backer_stakes = Self::backer_stakes(&backer, creator_id);
            let mut creator_stake_info =
                Self::creator_stake_info(creator_id, current_era).unwrap_or_default();

            let amount_to_unstake =
                Self::calculate_and_apply_stake_decrease(&mut backer_stakes, &mut creator_stake_info, amount)?;

            // Update the chunks and write them to storage
            let mut backer_locks = Self::backer_locks(&backer);

            ensure!(
                backer_locks.total_staked().saturating_sub(amount_to_unstake) >= T::MinimumTotalStake::get(),
                Error::<T>::InsufficientStakingAmount,
            );

            backer_locks.unbonding_info.add(UnbondingChunk {
                amount: amount_to_unstake,
                unlock_era: current_era + T::UnbondingPeriodInEras::get(),
            }).map_err(|_| Error::<T>::TooManyUnbondingChunks)?;

            Self::update_backer_locks(&backer, backer_locks);

            // Update total staked value in era.
            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(amount_to_unstake)
                }
            });
            Self::update_backer_stakes(&backer, creator_id, backer_stakes);
            CreatorStakeInfoByEra::<T>::insert(creator_id, current_era, creator_stake_info);

            Self::deposit_event(Event::<T>::Unstaked {
                who: backer,
                creator_id,
                era: current_era,
                amount: amount_to_unstake,
            });

            Ok(().into())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn withdraw_unstaked(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let backer = ensure_signed(origin)?;

            let mut backer_locks = Self::backer_locks(&backer);
            let current_era = Self::current_era();

            let (withdrawable_chunks, future_chunks) = backer_locks.unbonding_info.partition(current_era);
            let withdraw_amount = withdrawable_chunks.sum();

            ensure!(!withdraw_amount.is_zero(), Error::<T>::NothingToWithdraw);

            // Get the staking backer locks and update them
            backer_locks.total_locked = backer_locks.total_locked.saturating_sub(withdraw_amount);
            backer_locks.unbonding_info = future_chunks;
            Self::update_backer_locks(&backer, backer_locks);

            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.locked = x.locked.saturating_sub(withdraw_amount)
                }
            });

            Self::deposit_event(Event::<T>::StakeWithdrawn {
                who: backer,
                amount: withdraw_amount,
            });

            Ok(().into())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn withdraw_from_inactive_creator(
            origin: OriginFor<T>,
            creator_id: CreatorId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let backer = ensure_signed(origin)?;

            // Creator must exist and be unregistered
            let creator_info = Self::require_creator(creator_id)?;

            let unregistration_era = if let CreatorStatus::Inactive(x) = creator_info.status {
                x
            } else {
                return Err(Error::<T>::CreatorIsActive.into());
            };

            // There should be some leftover staked amount
            let mut backer_stakes = Self::backer_stakes(&backer, creator_id);
            let staked_value = backer_stakes.current_stake();
            ensure!(staked_value > Zero::zero(), Error::<T>::NotStakedCreator);

            // Don't allow withdrawal until all rewards have been claimed.
            let (claimable_era, _) = backer_stakes.claim();
            ensure!(
                claimable_era >= unregistration_era || claimable_era.is_zero(),
                Error::<T>::UnclaimedRewardsRemaining
            );

            // Unlock the staked amount immediately. No unbonding period for this scenario.
            let mut backer_locks = Self::backer_locks(&backer);
            backer_locks.total_locked = backer_locks.total_locked.saturating_sub(staked_value);
            Self::update_backer_locks(&backer, backer_locks);

            Self::update_backer_stakes(&backer, creator_id, Default::default());

            let current_era = Self::current_era();
            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(staked_value);
                    x.locked = x.locked.saturating_sub(staked_value);
                }
            });

            Self::deposit_event(Event::<T>::StakeWithdrawnFromInactiveCreator {
                who: backer,
                creator_id,
                amount: staked_value,
            });

            Ok(().into())
        }

        /// Move stake from one creator to another.
        ///
        /// It follows the same rules as the `stake` and `unstake` functions, with one notable
        /// difference: there is no unbonding period.
        ///
        /// # Parameters
        /// - `from_creator_id`: The ID of the source creator.
        /// - `to_creator_id`: The ID of the target creator.
        /// - `amount`: The amount of stake to be moved.
        ///
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn move_stake(
            origin: OriginFor<T>,
            from_creator_id: CreatorId,
            to_creator_id: CreatorId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let backer = ensure_signed(origin)?;

            // Creators must differ and both must be active
            ensure!(
                from_creator_id != to_creator_id,
                Error::<T>::CannotMoveStakeToSameCreator
            );

            ensure!(amount > Zero::zero(), Error::<T>::CannotMoveZeroStake);

            Self::ensure_creator_is_active(to_creator_id)?;

            // Validate and update previous creator related data
            let current_era = Self::current_era();
            let mut backer_stakes_by_source_creator = Self::backer_stakes(&backer, from_creator_id);
            let mut source_creator_info =
                Self::creator_stake_info(from_creator_id, current_era).unwrap_or_default();

            // Backer stake is decreased in `calculate_and_apply_stake_decrease`
            let stake_amount_to_move = Self::calculate_and_apply_stake_decrease(
                &backer,
                &mut backer_stakes_by_source_creator,
                &mut source_creator_info,
                amount,
            )?;

            // Validate and update target creator related data
            let mut backer_stakes_by_target_creator = Self::backer_stakes(&backer, to_creator_id);
            let mut target_creator_info =
                Self::creator_stake_info(to_creator_id, current_era).unwrap_or_default();

            Self::stake_to_creator(
                &backer,
                &mut backer_stakes_by_target_creator,
                &mut target_creator_info,
                stake_amount_to_move,
            )?;

            CreatorStakeInfoByEra::<T>::insert(from_creator_id, current_era, source_creator_info);
            Self::update_backer_stakes(&backer, from_creator_id, backer_stakes_by_source_creator);

            CreatorStakeInfoByEra::<T>::insert(to_creator_id, current_era, target_creator_info);
            Self::update_backer_stakes(&backer, to_creator_id, backer_stakes_by_target_creator);

            Self::deposit_event(Event::<T>::StakeMoved {
                who: backer,
                from_creator_id,
                to_creator_id,
                amount: stake_amount_to_move,
            });

            Ok(().into())
        }

        // Claim rewards for the backer on the oldest unclaimed era where they have a stake
        // and optionally restake the rewards to the same creator.
        // Not sure here whether to calculate total rewards for all creators
        //  or to withdraw per-creator rewards (preferably)
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn claim_backer_reward(origin: OriginFor<T>, creator_id: CreatorId, restake: bool) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let backer = ensure_signed(origin)?;

            // Ensure we have something to claim
            let mut backer_stakes = Self::backer_stakes(&backer, creator_id);
            let (era_to_claim, backer_staked) = backer_stakes.claim();
            ensure!(backer_staked > Zero::zero(), Error::<T>::NotStakedCreator);

            let creator_info = Self::require_creator(creator_id)?;

            Self::ensure_creator_active_in_era(&creator_info, era_to_claim)?;

            let current_era = Self::current_era();
            ensure!(era_to_claim < current_era, Error::<T>::CannotClaimInFutureEra);

            let reward_and_stake =
                Self::general_era_info(era_to_claim).ok_or(Error::<T>::EraNotFound)?;

            // TODO: move to separate function
            let backer_reward =
                Perbill::from_rational(backer_staked, reward_and_stake.staked) * reward_and_stake.rewards.backers;

            // FIXME: we mustn't modify `backer_stakes` here!
            let can_restake_reward = Self::ensure_can_restake_reward(
                restake, creator_info.status, &mut backer_stakes, current_era, backer_reward
            )?;

            // Withdraw reward funds from the rewards holding account
            let reward_imbalance = T::Currency::withdraw(
                &Self::rewards_pot_account(),
                backer_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            T::Currency::resolve_creating(&backer, reward_imbalance);

            if can_restake_reward {
                Self::do_restake_reward(&backer, backer_reward, creator_id, current_era);
            }

            Self::update_backer_stakes(&backer, creator_id, backer_stakes);

            Self::deposit_event(Event::<T>::BackerRewardsClaimed {
                who: backer,
                creator_id,
                amount: backer_reward,
            });

            /*Ok(Some(if should_restake_reward {
                T::WeightInfo::claim_backer_with_restake()
            } else {
                T::WeightInfo::claim_backer_without_restake()
            })
                .into())*/
            Ok(().into())
        }

        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn claim_creator_reward(origin: OriginFor<T>, creator_id: CreatorId, era: EraIndex) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let _ = ensure_signed(origin)?;

            let creator_info = Self::require_creator(creator_id)?;

            let mut creator_stake_info =
                Self::creator_stake_info(creator_id, era).unwrap_or_default();

            Self::ensure_creator_active_in_era(&creator_info, era)?;

            let current_era = Self::current_era();
            ensure!(era < current_era, Error::<T>::CannotClaimInFutureEra);

            ensure!(
                !creator_stake_info.rewards_claimed,
                Error::<T>::AlreadyClaimedInThisEra,
            );

            ensure!(
                creator_stake_info.total_staked > Zero::zero(),
                Error::<T>::NotStakedCreator,
            );

            let rewards_and_stakes =
                Self::general_era_info(era).ok_or(Error::<T>::EraNotFound)?;

            // Calculate the creator reward for this era.
            let creator_reward = Self::calculate_creator_reward(
                &creator_stake_info,
                &rewards_and_stakes,
            );

            // Withdraw the reward funds from the creator staking pot account
            let reward_imbalance = T::Currency::withdraw(
                &Self::rewards_pot_account(),
                creator_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            T::Currency::resolve_creating(&creator_info.stakeholder, reward_imbalance);

            Self::deposit_event(Event::<T>::CreatorRewardsClaimed {
                who: creator_info.stakeholder,
                amount: creator_reward,
            });

            // updated counter for total rewards paid to the creator
            creator_stake_info.rewards_claimed = true;
            CreatorStakeInfoByEra::<T>::insert(creator_id, era, creator_stake_info);

            Ok(().into())
        }

        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn set_maintenance_mode(
            origin: OriginFor<T>,
            enable_maintenance: bool,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let is_disabled = PalletDisabled::<T>::get();
            let does_switch_state = is_disabled != enable_maintenance;

            ensure!(
                does_switch_state,
                Error::<T>::MaintenanceModeNotChanged
            );
            PalletDisabled::<T>::put(enable_maintenance);

            Self::deposit_event(Event::<T>::MaintenanceModeSet { enabled: enable_maintenance });
            Ok(().into())
        }

        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn force_new_era(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ForceEra::<T>::put(Forcing::ForceNew);
            Ok(Pays::No.into())
        }

        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn set_reward_distribution_config(
            origin: OriginFor<T>,
            new_config: RewardDistributionConfig,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(new_config.is_sum_equal_to_one(), Error::<T>::InvalidSumOfRewardDistributionConfig);
            ActiveRewardDistributionConfig::<T>::put(new_config.clone());

            Self::deposit_event(Event::<T>::RewardDistributionConfigChanged { new_config });

            Ok(())
        }

        #[pallet::call_index(13)]
        #[pallet::weight(T::DbWeight::get().writes(1) + Weight::from_parts(10_000, 0))]
        pub fn set_per_block_reward(origin: OriginFor<T>, new_reward: BalanceOf<T>) -> DispatchResult {
            ensure_root(origin)?;

            RewardPerBlock::<T>::put(new_reward);

            Self::deposit_event(Event::<T>::RewardPerBlockChanged { new_reward });

            Ok(())
        }
    }
}
