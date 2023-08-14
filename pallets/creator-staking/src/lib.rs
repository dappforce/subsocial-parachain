#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

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
    use sp_runtime::{traits::{AccountIdConversion, Zero}, Perbill, Saturating};

    use subsocial_support::{traits::SpacesInterface, SpaceId};

    use crate::types::*;

    pub(crate) const STAKING_ID: LockIdentifier = *b"crestake";

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Creators staking pallet Id
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Number of blocks per era.
        #[pallet::constant]
        type BlockPerEra: Get<BlockNumberFor<Self>>;

        /// The staking balance.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
            + ReservableCurrency<Self::AccountId>;

        type SpacesInterface: SpacesInterface<Self::AccountId, SpaceId>;

        #[pallet::constant]
        type RegistrationDeposit: Get<BalanceOf<Self>>;

        /// Minimum amount user must have staked on creator.
        /// User can stake less if they already have the minimum staking amount staked on that
        /// particular creator.
        #[pallet::constant]
        type MinimumStakingAmount: Get<BalanceOf<Self>>;

        // TODO: make it MinimumRemainingRatio
        //  (e.g. 0.1 = 10%, so that account can lock only 90% of its balance)
        /// Minimum amount that should be left on staker account after staking.
        /// Serves as a safeguard to prevent users from locking their entire free balance.
        #[pallet::constant]
        type MinimumRemainingAmount: Get<BalanceOf<Self>>;

        /// Maximum number of unique stakers per creator.
        #[pallet::constant]
        type MaxNumberOfStakersPerCreator: Get<u32>;

        /// Max number of unique `EraStake` values that can exist for a `(staker, creator)`
        /// pairing. When stakers claims rewards, they will either keep the number of
        /// `EraStake` values the same or they will reduce them by one. Stakers cannot add
        /// an additional `EraStake` value by calling `bond&stake` or `unbond&unstake` if they've
        /// reached the max number of values.
        ///
        /// This ensures that history doesn't grow indefinitely - if there are too many chunks,
        /// stakers should first claim their former rewards before adding additional
        /// `EraStake` values.
        #[pallet::constant]
        type MaxEraStakeValues: Get<u32>;

        /// Number of eras that need to pass until unstaked value can be withdrawn.
        /// Current era is always counted as full era (regardless how much blocks are remaining).
        /// When set to `0`, it's equal to having no unbonding period.
        #[pallet::constant]
        type UnbondingPeriod: Get<u32>;

        /// Max number of unlocking chunks per account Id <-> creator Id pairing.
        /// If value is zero, unlocking becomes impossible.
        #[pallet::constant]
        type MaxUnlockingChunks: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn pallet_disabled)]
    pub type PalletDisabled<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// The current era index.
    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, EraIndex, ValueQuery>;

    /// Stores the block number of when the next era starts
    #[pallet::storage]
    #[pallet::whitelist_storage]
    #[pallet::getter(fn next_era_starting_block)]
    pub type NextEraStartingBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// Accumulator for block rewards during an era. It is reset at every new era
    #[pallet::storage]
    #[pallet::getter(fn block_reward_accumulator)]
    pub type BlockRewardAccumulator<T> = StorageValue<_, RewardInfo<BalanceOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn registered_creator)]
    pub(crate) type RegisteredCreators<T: Config> =
        StorageMap<_, Twox64Concat, SpaceId, CreatorInfo<T::AccountId>>;

    /// Staking information about creator in a particular era.
    #[pallet::storage]
    #[pallet::getter(fn creator_stake_info)]
    pub type CreatorEraStake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SpaceId,
        Twox64Concat,
        EraIndex,
        CreatorStakeInfo<BalanceOf<T>>,
    >;

    /// Info about stakers stakes on particular creators.
    #[pallet::storage]
    #[pallet::getter(fn staker_info)]
    pub type GeneralStakerInfo<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        SpaceId,
        StakerInfo<BalanceOf<T>>,
        ValueQuery,
    >;

    /// General information about an era like TVL, total staked value, rewards.
    #[pallet::storage]
    #[pallet::getter(fn general_era_info)]
    pub type GeneralEraInfo<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, EraInfo<BalanceOf<T>>>;

    /// General information about the staker.
    #[pallet::storage]
    #[pallet::getter(fn ledger)]
    pub type Ledger<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AccountLedger<BalanceOf<T>>, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        Staked { who: T::AccountId, creator: SpaceId, amount: BalanceOf<T> },
        Unstaked { who: T::AccountId, creator: SpaceId, amount: BalanceOf<T> },
        RewardsClaimed { who: T::AccountId, amount: BalanceOf<T> },
        WithdrawnFromClaimed { who: T::AccountId, amount: BalanceOf<T> },
        WithdrawnFromUnregistered { who: T::AccountId, amount: BalanceOf<T> },
        AnnualInflationSet { value: Perbill },
        RewardsCalculated { total_rewards_amount: BalanceOf<T> },
        NewStakingEra { index: EraIndex },
        CreatorRegistered { who: T::AccountId, space_id: SpaceId },
        CreatorUnregistered { who: T::AccountId, space_id: SpaceId },
        CreatorUnregisteredWithSlash { who: T::AccountId, space_id: SpaceId, amount: BalanceOf<T> },
        NewDappStakingEra { number: EraIndex },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pallet is disabled.
        Disabled,
        AlreadyUsedCreatorSpace,
        NotOperatedCreator,
        NotACreator,
        CannotStakeZero,
        CannotUnstakeZero,
        MaxNumberOfStakersExceeded,
        UnexpectedStakeInfoEra,
        TooManyEraStakeValues,
        InsufficientValue,
        NotStakedCreator,
        TooManyUnlockingChunks,
        NothingToWithdraw,
        NotUnregisteredCreator,
        UnclaimedRewardsRemaining,
        EraOutOfBounds,
        UnknownEraReward,
        AlreadyClaimedInThisEra,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            // As long as pallet is disabled, we shouldn't allow any storage modifications.
            // This means we might prolong an era but it's acceptable.
            // Runtime upgrade should be timed so we ensure that we complete it before
            // a new era is triggered. This code is just a safety net to ensure nothing is broken
            // if we fail to do that.
            if PalletDisabled::<T>::get() {
                return T::DbWeight::get().reads(1);
            }

            let previous_era = Self::current_era();
            let next_era_starting_block = Self::next_era_starting_block();

            // Value is compared to 1 since genesis block is ignored
            if now >= next_era_starting_block || previous_era.is_zero() {
                let blocks_per_era = T::BlockPerEra::get();
                let next_era = previous_era + 1;
                CurrentEra::<T>::put(next_era);

                NextEraStartingBlock::<T>::put(now + blocks_per_era);

                // TODO: Replace BlockRewardAccumulator with AnnualInflation
                let reward = BlockRewardAccumulator::<T>::take();
                Self::reward_balance_snapshot(previous_era, reward);
                let consumed_weight = Self::rotate_staking_info(previous_era);

                Self::deposit_event(Event::<T>::NewDappStakingEra { number: next_era });

                consumed_weight + T::DbWeight::get().reads_writes(5, 3)
            } else {
                T::DbWeight::get().reads(4)
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_ref_time(100_000) + T::DbWeight::get().reads_writes(3, 1))]
        pub fn force_register_creator(
            origin: OriginFor<T>,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            ensure_root(origin)?;

            ensure!(
                !RegisteredCreators::<T>::contains_key(&space_id),
                Error::<T>::AlreadyUsedCreatorSpace,
            );

            let space_owner = T::SpacesInterface::get_space_owner(space_id)?;
            T::Currency::reserve(&space_owner, T::RegistrationDeposit::get())?;

            RegisteredCreators::<T>::insert(space_id, CreatorInfo::new(space_owner.clone()));

            Self::deposit_event(Event::<T>::CreatorRegistered { who: space_owner, space_id });

            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_ref_time(100_000) + T::DbWeight::get().reads_writes(2, 1))]
        pub fn unregister_creator(
            origin: OriginFor<T>,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_unregister_creator(space_id, UnregisterOrigin::Creator(who.clone()))?;

            Self::deposit_event(Event::<T>::CreatorUnregistered { who, space_id });

            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_ref_time(100_000) + T::DbWeight::get().reads_writes(2, 1))]
        pub fn force_unregister_creator(
            origin: OriginFor<T>,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            Self::do_unregister_creator(space_id, UnregisterOrigin::Root)?;
            let creator_info =
                Self::registered_creator(space_id).ok_or(Error::<T>::NotOperatedCreator)?;

            Self::deposit_event(Event::<T>::CreatorUnregisteredWithSlash {
                who: creator_info.stakeholder,
                space_id,
                amount: T::RegistrationDeposit::get(),
            });

            Ok(Pays::No.into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn stake(
            origin: OriginFor<T>,
            space_id: SpaceId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let staker = ensure_signed(origin)?;

            // Check that creator is ready for staking.
            ensure!(Self::is_active(space_id), Error::<T>::NotOperatedCreator,);

            // Get the staking ledger or create an entry if it doesn't exist.
            let mut ledger = Self::ledger(&staker);
            let available_balance = Self::available_staking_balance(&staker, &ledger);
            let value_to_stake = amount.min(available_balance);
            ensure!(value_to_stake > Zero::zero(), Error::<T>::CannotStakeZero);

            let current_era = Self::current_era();
            let mut staking_info =
                Self::creator_stake_info(space_id, current_era).unwrap_or_default();
            let mut staker_info = Self::staker_info(&staker, space_id);

            Self::stake_to_creator(
                &mut staker_info,
                &mut staking_info,
                value_to_stake,
                current_era,
            )?;

            ledger.locked = ledger.locked.saturating_add(value_to_stake);

            // Update storage
            GeneralEraInfo::<T>::mutate(&current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_add(value_to_stake);
                    x.locked = x.locked.saturating_add(value_to_stake);
                }
            });

            Self::update_ledger(&staker, ledger);
            Self::update_staker_info(&staker, space_id, staker_info);
            CreatorEraStake::<T>::insert(space_id, current_era, staking_info);

            Self::deposit_event(Event::<T>::Staked {
                who: staker,
                creator: space_id,
                amount: value_to_stake,
            });
            Ok(().into())
        }

        // #[weight = 10_000]
        // fn increase_stake(origin, space_id, additional_amount) {
        //     todo!()
        // }
        //
        // #[weight = 10_000]
        // fn move_stake(origin, from_space_id, to_space_id, amount) {
        //     todo!()
        // }

        /// Start unbonding process and unstake balance from the creator.
        ///
        /// The unstaked amount will no longer be eligible for rewards but still won't be unlocked.
        /// User needs to wait for the unbonding period to finish before being able to withdraw
        /// the funds via `withdraw_unbonded` call.
        ///
        /// In case remaining staked balance on creator is below minimum staking amount,
        /// entire stake for that creator will be unstaked.
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn unstake(
            origin: OriginFor<T>,
            space_id: SpaceId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let staker = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::CannotUnstakeZero);
            ensure!(Self::is_active(space_id), Error::<T>::NotOperatedCreator,);

            let current_era = Self::current_era();
            let mut staker_info = Self::staker_info(&staker, space_id);
            let mut stake_info =
                Self::creator_stake_info(space_id, current_era).unwrap_or_default();

            let value_to_unstake =
                Self::unstake_from_creator(&mut staker_info, &mut stake_info, amount, current_era)?;

            // Update the chunks and write them to storage
            let mut ledger = Self::ledger(&staker);
            ledger.unbonding_info.add(UnlockingChunk {
                amount: value_to_unstake,
                unlock_era: current_era + T::UnbondingPeriod::get(),
            });
            // This should be done AFTER insertion since it's possible for chunks to merge
            ensure!(
                ledger.unbonding_info.len() <= T::MaxUnlockingChunks::get(),
                Error::<T>::TooManyUnlockingChunks
            );

            Self::update_ledger(&staker, ledger);

            // Update total staked value in era.
            GeneralEraInfo::<T>::mutate(&current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(value_to_unstake)
                }
            });
            Self::update_staker_info(&staker, space_id, staker_info);
            CreatorEraStake::<T>::insert(space_id, current_era, stake_info);

            Self::deposit_event(Event::<T>::Unstaked {
                who: staker,
                creator: space_id,
                amount: value_to_unstake,
            });

            Ok(().into())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn withdraw_unstaked(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let staker = ensure_signed(origin)?;

            let mut ledger = Self::ledger(&staker);
            let current_era = Self::current_era();

            let (valid_chunks, future_chunks) = ledger.unbonding_info.partition(current_era);
            let withdraw_amount = valid_chunks.sum();

            ensure!(!withdraw_amount.is_zero(), Error::<T>::NothingToWithdraw);

            // Get the staking ledger and update it
            ledger.locked = ledger.locked.saturating_sub(withdraw_amount);
            ledger.unbonding_info = future_chunks;

            Self::update_ledger(&staker, ledger);
            GeneralEraInfo::<T>::mutate(&current_era, |value| {
                if let Some(x) = value {
                    x.locked = x.locked.saturating_sub(withdraw_amount)
                }
            });

            Self::deposit_event(Event::<T>::WithdrawnFromClaimed {
                who: staker,
                amount: withdraw_amount,
            });

            Ok(().into())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn withdraw_from_unregistered(
            origin: OriginFor<T>,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let staker = ensure_signed(origin)?;

            // Creator must exist and it has to be unregistered
            let creator_info =
                RegisteredCreators::<T>::get(space_id).ok_or(Error::<T>::NotOperatedCreator)?;

            let unregistered_era = if let CreatorState::Unregistered(x) = creator_info.state {
                x
            } else {
                return Err(Error::<T>::NotUnregisteredCreator.into());
            };

            // There should be some leftover staked amount
            let mut staker_info = Self::staker_info(&staker, space_id);
            let staked_value = staker_info.latest_staked_value();
            ensure!(staked_value > Zero::zero(), Error::<T>::NotStakedCreator);

            // Don't allow withdrawal until all rewards have been claimed.
            let (claimable_era, _) = staker_info.claim();
            ensure!(
                claimable_era >= unregistered_era || claimable_era.is_zero(),
                Error::<T>::UnclaimedRewardsRemaining
            );

            // Unlock the staked amount immediately. No unbonding period for this scenario.
            let mut ledger = Self::ledger(&staker);
            ledger.locked = ledger.locked.saturating_sub(staked_value);
            Self::update_ledger(&staker, ledger);

            Self::update_staker_info(&staker, space_id, Default::default());

            let current_era = Self::current_era();
            GeneralEraInfo::<T>::mutate(&current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(staked_value);
                    x.locked = x.locked.saturating_sub(staked_value);
                }
            });

            Self::deposit_event(Event::<T>::WithdrawnFromUnregistered {
                who: staker,
                amount: staked_value,
            });

            Ok(().into())
        }

        // Not sure here whether to calculate total rewards for all creators
        //  or to withdraw per-creator rewards (preferably)
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn claim_staker_reward(origin: OriginFor<T>, space_id: SpaceId, restake: bool) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let staker = ensure_signed(origin)?;

            // Ensure we have something to claim
            let mut staker_info = Self::staker_info(&staker, space_id);
            let (era, staked) = staker_info.claim();
            ensure!(staked > Zero::zero(), Error::<T>::NotStakedCreator);

            let creator_info =
                RegisteredCreators::<T>::get(space_id).ok_or(Error::<T>::NotOperatedCreator)?;

            if let CreatorState::Unregistered(unregister_era) = creator_info.state {
                ensure!(era < unregister_era, Error::<T>::NotOperatedCreator);
            }

            let current_era = Self::current_era();
            ensure!(era < current_era, Error::<T>::EraOutOfBounds);

            let staking_info = Self::creator_stake_info(space_id, era).unwrap_or_default();
            let reward_and_stake =
                Self::general_era_info(era).ok_or(Error::<T>::UnknownEraReward)?;

            let (_, stakers_joint_reward) =
                Self::creator_stakers_split(&staking_info, &reward_and_stake);
            let staker_reward =
                Perbill::from_rational(staked, staking_info.total) * stakers_joint_reward;

            let mut ledger = Self::ledger(&staker);

            let should_restake_reward = Self::should_restake_reward(
                restake,
                creator_info.state,
                staker_info.latest_staked_value(),
            );

            if should_restake_reward {
                staker_info
                    .stake(current_era, staker_reward)
                    .map_err(|_| Error::<T>::UnexpectedStakeInfoEra)?;

                // Restaking will, in the worst case, remove one, and add one record,
                // so it's fine if the vector is full
                ensure!(
                    staker_info.len() <= T::MaxEraStakeValues::get(),
                    Error::<T>::TooManyEraStakeValues
                );
            }

            // Withdraw reward funds from the creators staking pot
            let reward_imbalance = T::Currency::withdraw(
                &Self::account_id(),
                staker_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            if should_restake_reward {
                ledger.locked = ledger.locked.saturating_add(staker_reward);
                Self::update_ledger(&staker, ledger);

                // Update storage
                GeneralEraInfo::<T>::mutate(&current_era, |value| {
                    if let Some(x) = value {
                        x.staked = x.staked.saturating_add(staker_reward);
                        x.locked = x.locked.saturating_add(staker_reward);
                    }
                });

                CreatorEraStake::<T>::mutate(space_id, current_era, |staking_info| {
                    if let Some(x) = staking_info {
                        x.total = x.total.saturating_add(staker_reward);
                    }
                });

                Self::deposit_event(Event::<T>::Staked {
                    who: staker.clone(),
                    creator: space_id,
                    amount: staker_reward,
                });
            }

            T::Currency::resolve_creating(&staker, reward_imbalance);
            Self::update_staker_info(&staker, space_id, staker_info);

            // TODO: change event
            Self::deposit_event(Event::<T>::RewardsClaimed {
                who: staker,
                amount: staker_reward,
            });

            /*Ok(Some(if should_restake_reward {
                T::WeightInfo::claim_staker_with_restake()
            } else {
                T::WeightInfo::claim_staker_without_restake()
            })
                .into())*/
            Ok(().into())
        }

        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn claim_creator_reward(origin: OriginFor<T>, space_id: SpaceId, era: EraIndex) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let _ = ensure_signed(origin)?;

            let creator_info =
                RegisteredCreators::<T>::get(space_id).ok_or(Error::<T>::NotOperatedCreator)?;

            let mut creator_stake_info =
                Self::creator_stake_info(space_id, era).unwrap_or_default();

            let creator_reward = Self::calculate_creator_reward(&creator_stake_info, &creator_info, era)?;

            // Withdraw reward funds from the creators staking
            let reward_imbalance = T::Currency::withdraw(
                &Self::account_id(),
                creator_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            T::Currency::resolve_creating(&creator_info.stakeholder, reward_imbalance);

            // TODO: change event
            Self::deposit_event(Event::<T>::RewardsClaimed {
                who: creator_info.stakeholder,
                amount: creator_reward,
            });

            // updated counter for total rewards paid to the creator
            creator_stake_info.creator_reward_claimed = true;
            CreatorEraStake::<T>::insert(space_id, era, creator_stake_info);

            Ok(().into())
        }

        // #[weight = 10_000]
        // fn set_annual_inflation(origin, inflation: Perbill) {
        //     ensure_root(origin)?;
        //     todo!()
        // }
    }

    impl<T: Config> Pallet<T> {
        /// Calculate the dApp reward for the specified era.
        /// If successfull, returns reward amount.
        /// In case reward cannot be claimed or was already claimed, an error is raised.
        fn calculate_creator_reward(
            creator_stake_info: &CreatorStakeInfo<BalanceOf<T>>,
            creator_info: &CreatorInfo<T::AccountId>,
            era: EraIndex,
        ) -> Result<BalanceOf<T>, Error<T>> {
            let current_era = Self::current_era();
            if let CreatorState::Unregistered(unregister_era) = creator_info.state {
                ensure!(era < unregister_era, Error::<T>::NotOperatedCreator);
            }
            ensure!(era < current_era, Error::<T>::EraOutOfBounds);

            ensure!(
                !creator_stake_info.creator_reward_claimed,
                Error::<T>::AlreadyClaimedInThisEra,
            );
            ensure!(
                creator_stake_info.total > Zero::zero(),
                Error::<T>::NotStakedCreator,
            );

            let reward_and_stake =
                Self::general_era_info(era).ok_or(Error::<T>::UnknownEraReward)?;

            // Calculate the creator reward for this era.
            let (creator_reward, _) = Self::creator_stakers_split(&creator_stake_info, &reward_and_stake);

            Ok(creator_reward)
        }

        /// `Err` if pallet disabled for maintenance, `Ok` otherwise
        pub fn ensure_pallet_enabled() -> Result<(), Error<T>> {
            if PalletDisabled::<T>::get() {
                Err(Error::<T>::Disabled)
            } else {
                Ok(())
            }
        }

        /// Returns available staking balance for the potential staker
        fn available_staking_balance(
            staker: &T::AccountId,
            ledger: &AccountLedger<BalanceOf<T>>,
        ) -> BalanceOf<T> {
            // Ensure that staker has enough balance to bond & stake.
            let free_balance =
                T::Currency::free_balance(staker).saturating_sub(T::MinimumRemainingAmount::get());

            // Remove already locked funds from the free balance
            free_balance.saturating_sub(ledger.locked)
        }

        /// `true` if creator is active, `false` if it has been unregistered
        fn is_active(space_id: SpaceId) -> bool {
            RegisteredCreators::<T>::get(space_id)
                .map_or(false, |info| info.state == CreatorState::Registered)
        }

        /// `true` if all the conditions for restaking the reward have been met, `false` otherwise
        pub(crate) fn should_restake_reward(
            restake: bool,
            creator_state: CreatorState,
            latest_staked_value: BalanceOf<T>,
        ) -> bool {
            restake
                && creator_state == CreatorState::Registered
                && latest_staked_value > Zero::zero()
        }

        pub(super) fn do_unregister_creator(
            space_id: SpaceId,
            unregister_origin: UnregisterOrigin<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let mut creator_info =
                RegisteredCreators::<T>::get(space_id).ok_or(Error::<T>::NotOperatedCreator)?;

            ensure!(creator_info.state == CreatorState::Registered, Error::<T>::NotOperatedCreator);
            let stakeholder = creator_info.stakeholder.clone();

            // TODO: make flexible register deposit
            if let UnregisterOrigin::Root = unregister_origin {
                T::Currency::slash_reserved(&stakeholder, T::RegistrationDeposit::get());
            } else if let UnregisterOrigin::Creator(who) = unregister_origin {
                ensure!(who == stakeholder, Error::<T>::NotACreator);
                T::Currency::unreserve(&stakeholder, T::RegistrationDeposit::get());
            }

            let current_era = Self::current_era();
            creator_info.state = CreatorState::Unregistered(current_era);
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
        fn stake_to_creator(
            staker_info: &mut StakerInfo<BalanceOf<T>>,
            staking_info: &mut CreatorStakeInfo<BalanceOf<T>>,
            value: BalanceOf<T>,
            current_era: EraIndex,
        ) -> Result<(), Error<T>> {
            ensure!(
                !staker_info.latest_staked_value().is_zero() ||
                    staking_info.number_of_stakers < T::MaxNumberOfStakersPerCreator::get(),
                Error::<T>::MaxNumberOfStakersExceeded
            );
            if staker_info.latest_staked_value().is_zero() {
                staking_info.number_of_stakers = staking_info.number_of_stakers.saturating_add(1);
            }

            staker_info
                .stake(current_era, value)
                .map_err(|_| Error::<T>::UnexpectedStakeInfoEra)?;
            ensure!(
                // One spot should remain for compounding reward claim call
                staker_info.len() < T::MaxEraStakeValues::get(),
                Error::<T>::TooManyEraStakeValues
            );
            ensure!(
                staker_info.latest_staked_value() >= T::MinimumStakingAmount::get(),
                Error::<T>::InsufficientValue,
            );

            // Increment ledger and total staker value for creator.
            staking_info.total = staking_info.total.saturating_add(value);

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
        fn unstake_from_creator(
            staker_info: &mut StakerInfo<BalanceOf<T>>,
            stake_info: &mut CreatorStakeInfo<BalanceOf<T>>,
            value: BalanceOf<T>,
            current_era: EraIndex,
        ) -> Result<BalanceOf<T>, Error<T>> {
            let staked_value = staker_info.latest_staked_value();
            ensure!(staked_value > Zero::zero(), Error::<T>::NotStakedCreator);

            // Calculate the value which will be unstaked.
            let remaining = staked_value.saturating_sub(value);
            let value_to_unstake = if remaining < T::MinimumStakingAmount::get() {
                stake_info.number_of_stakers = stake_info.number_of_stakers.saturating_sub(1);
                staked_value
            } else {
                value
            };
            stake_info.total = stake_info.total.saturating_sub(value_to_unstake);

            // Sanity check
            ensure!(value_to_unstake > Zero::zero(), Error::<T>::CannotUnstakeZero);

            staker_info
                .unstake(current_era, value_to_unstake)
                .map_err(|_| Error::<T>::UnexpectedStakeInfoEra)?;
            ensure!(
                // One spot should remain for compounding reward claim call
                staker_info.len() < T::MaxEraStakeValues::get(),
                Error::<T>::TooManyEraStakeValues
            );

            Ok(value_to_unstake)
        }

        /// Update the ledger for a staker. This will also update the stash lock.
        /// This lock will lock the entire funds except paying for further transactions.
        fn update_ledger(staker: &T::AccountId, ledger: AccountLedger<BalanceOf<T>>) {
            if ledger.is_empty() {
                Ledger::<T>::remove(&staker);
                T::Currency::remove_lock(STAKING_ID, staker);
            } else {
                T::Currency::set_lock(STAKING_ID, staker, ledger.locked, WithdrawReasons::all());
                Ledger::<T>::insert(staker, ledger);
            }
        }

        /// Update the staker info for the `(staker, creator_id)` pairing.
        /// If staker_info is empty, remove it from the DB. Otherwise, store it.
        fn update_staker_info(
            staker: &T::AccountId,
            creator_id: SpaceId,
            staker_info: StakerInfo<BalanceOf<T>>,
        ) {
            if staker_info.is_empty() {
                GeneralStakerInfo::<T>::remove(staker, creator_id)
            } else {
                GeneralStakerInfo::<T>::insert(staker, creator_id, staker_info)
            }
        }

        /// Calculate reward split between a creator and its stakers.
        ///
        /// Returns (creator reward, joint stakers reward)
        pub(crate) fn creator_stakers_split(
            creator_info: &CreatorStakeInfo<BalanceOf<T>>,
            era_info: &EraInfo<BalanceOf<T>>,
        ) -> (BalanceOf<T>, BalanceOf<T>) {
            let creator_stake_portion =
                Perbill::from_rational(creator_info.total, era_info.staked);

            let creator_reward_part = creator_stake_portion * era_info.rewards.creators;
            let stakers_joint_reward = creator_stake_portion * era_info.rewards.stakers;

            (creator_reward_part, stakers_joint_reward)
        }

        pub(crate) fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// The block rewards are accumulated on the pallets's account during an era.
        /// This function takes a snapshot of the pallet's balance accrued during current era
        /// and stores it for future distribution
        ///
        /// This is called just at the beginning of an era.
        fn reward_balance_snapshot(era: EraIndex, rewards: RewardInfo<BalanceOf<T>>) {
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
        /// This is the most primitive solution since it scales with number of dApps.
        /// It is possible to provide a hybrid solution which allows laziness but also prevents
        /// a situation where we don't have access to the required data.
        fn rotate_staking_info(current_era: EraIndex) -> Weight {
            let next_era = current_era + 1;

            let mut consumed_weight = Weight::zero();

            for (space_id, creator_info) in RegisteredCreators::<T>::iter() {
                // Ignore dapp if it was unregistered
                consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
                if let CreatorState::Unregistered(_) = creator_info.state {
                    continue;
                }

                // Copy data from era `X` to era `X + 1`
                if let Some(mut staking_info) = Self::creator_stake_info(space_id, current_era)
                {
                    staking_info.creator_reward_claimed = false;
                    CreatorEraStake::<T>::insert(space_id, next_era, staking_info);

                    consumed_weight =
                        consumed_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                } else {
                    consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));
                }
            }

            consumed_weight
        }
    }
}
