#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;
pub mod functions;
pub mod inflation;
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
    use sp_runtime::{traits::Zero, Perbill, Saturating};

    use subsocial_support::{traits::SpacesInterface, SpaceId};

    pub use crate::types::*;

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

        /// Specifies the amount of tokens required as a deposit for creator registration.
        ///
        /// This constant defines the number of tokens that a creator must lock as collateral
        /// in order to register on the platform. The tokens act as a deposit and may be
        /// returned under certain conditions, such as account de-registration.
        #[pallet::constant]
        type CreatorRegistrationDeposit: Get<BalanceOf<Self>>;

        /// Minimum amount can be staked to the creator.
        /// User can stake less if they already have the minimum staking amount staked to that
        /// particular creator.
        #[pallet::constant]
        type MinimumStake: Get<BalanceOf<Self>>;

        // TODO: make it MinimumRemainingRatio
        //  (e.g. 0.1 = 10%, so that account can lock only 90% of its balance)
        /// Minimum amount that should be left on staker account after staking.
        /// Serves as a safeguard to prevent users from locking their entire free balance.
        #[pallet::constant]
        type MinimumRemainingFreeBalance: Get<BalanceOf<Self>>;

        /// Maximum number of unique stakers per creator.
        #[pallet::constant]
        type MaxNumberOfStakersPerCreator: Get<u32>;

        /// Max number of unique `EraStake` items that can exist for a `(staker, creator)`
        /// pairing. When stakers claims rewards, they will either keep the number of
        /// `EraStake` values the same or they will reduce them by one. Stakers cannot add
        /// an additional `EraStake` value by calling `bond() & stake()` or `unbond() & unstake()` if they've
        /// reached the max number of values.
        ///
        /// This ensures that history doesn't grow indefinitely - if there are too many chunks,
        /// stakers should first claim their former rewards before adding additional
        /// `EraStake` values.
        #[pallet::constant]
        type MaxEraStakeItems: Get<u32>;

        #[pallet::constant]
        type StakeExpirationInEras: Get<EraIndex>;

        /// Number of eras that need to pass until unstaked value can be withdrawn.
        /// Current era is always counted as full era (regardless how much blocks are remaining).
        /// When set to `0`, it's equal to having no unbonding period.
        #[pallet::constant]
        type UnbondingPeriodInEras: Get<u32>;

        /// Max number of unlocking chunks per `(staker, creator)` pairing.
        /// If value is zero, unlocking becomes impossible.
        #[pallet::constant]
        type MaxUnlockingChunks: Get<u32>;

        #[pallet::constant]
        type AnnualInflation: Get<Perbill>;

        /// Represents the estimated number of blocks that are generated within the span of one year.
        #[pallet::constant]
        type BlocksPerYear: Get<Self::BlockNumber>;

        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;
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
        StakesInfoOf<T>,
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
        StorageMap<_, Blake2_128Concat, T::AccountId, StakerLedgerOf<T>, ValueQuery>;

    /// Accumulator for block rewards during an era. It is reset at every new era
    #[pallet::storage]
    #[pallet::getter(fn block_reward_accumulator)]
    pub type BlockRewardAccumulator<T> = StorageValue<_, RewardInfo<BalanceOf<T>>, ValueQuery>;

    #[pallet::type_value]
    pub fn RewardConfigOnEmpty() -> RewardsDistributionConfig {
        RewardsDistributionConfig::default()
    }

    /// An active list of configuration parameters used to calculate reward distribution portions.
    #[pallet::storage]
    #[pallet::getter(fn reward_config)]
    pub type ActiveRewardDistributionConfig<T> =
        StorageValue<_, RewardsDistributionConfig, ValueQuery, RewardConfigOnEmpty>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        Staked { who: T::AccountId, creator: SpaceId, era: EraIndex, amount: BalanceOf<T> },
        Unstaked { who: T::AccountId, creator: SpaceId, era: EraIndex, amount: BalanceOf<T> },
        StakerRewardsClaimed { who: T::AccountId, creator: SpaceId, amount: BalanceOf<T> },
        CreatorRewardsClaimed { who: T::AccountId, amount: BalanceOf<T> },
        WithdrawnUnstaked { who: T::AccountId, amount: BalanceOf<T> },
        WithdrawnFromInactiveCreator { who: T::AccountId, amount: BalanceOf<T> },
        AnnualInflationSet { value: Perbill },
        RewardsCalculated { total_rewards_amount: BalanceOf<T> },
        CreatorRegistered { who: T::AccountId, space_id: SpaceId },
        CreatorUnregistered { who: T::AccountId, space_id: SpaceId },
        CreatorUnregisteredWithSlash { space_id: SpaceId, slash_amount: BalanceOf<T> },
        NewCreatorStakingEra { number: EraIndex },
        MaintenanceModeSet { enabled: bool },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Pallet is disabled.
        PalletIsDisabled,
        CreatorAlreadyRegistered,
        CreatorNotFound,
        InactiveCreator,
        NotACreator,
        CannotStakeZero,
        CannotUnstakeZero,
        MaxNumberOfStakersExceeded,
        UnexpectedStakeInfoEra,
        TooManyEraStakeValues,
        InsufficientStakingAmount,
        NotStakedCreator,
        TooManyUnlockingChunks,
        NothingToWithdraw,
        NotUnregisteredCreator,
        UnclaimedRewardsRemaining,
        EraOutOfBounds,
        EraNotFound,
        AlreadyClaimedInThisEra,
        MaintenanceModeNotChanged,
        RewardDistributionConfigInconsistent,
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
                Self::reward_balance_snapshot(previous_era, reward);
                let consumed_weight = Self::rotate_staking_info(previous_era);

                if force_new_era {
                    ForceEra::<T>::put(Forcing::NotForcing);
                }

                Self::deposit_event(Event::<T>::NewCreatorStakingEra { number: next_era });

                let force_new_era_write = if force_new_era { 1 } else { 0 };
                consumed_weight + T::DbWeight::get().reads_writes(6, 3 + force_new_era_write)
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
                !RegisteredCreators::<T>::contains_key(space_id),
                Error::<T>::CreatorAlreadyRegistered,
            );

            let space_owner = T::SpacesInterface::get_space_owner(space_id)?;
            T::Currency::reserve(&space_owner, T::CreatorRegistrationDeposit::get())?;

            RegisteredCreators::<T>::insert(space_id, CreatorInfo::new(space_owner.clone()));

            Self::deposit_event(Event::<T>::CreatorRegistered { who: space_owner, space_id });

            Ok(Pays::No.into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_ref_time(100_000) + T::DbWeight::get().reads_writes(2, 1))]
        pub fn unregister_creator(
            origin: OriginFor<T>,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let who = ensure_signed(origin)?;

            Self::do_unregister_creator(space_id, UnregistrationAuthority::Creator(who.clone()))?;

            Self::deposit_event(Event::<T>::CreatorUnregistered { who, space_id });

            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_ref_time(100_000) + T::DbWeight::get().reads_writes(2, 1))]
        pub fn force_unregister_creator(
            origin: OriginFor<T>,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            ensure_root(origin)?;

            Self::do_unregister_creator(space_id, UnregistrationAuthority::Root)?;

            Self::deposit_event(Event::<T>::CreatorUnregisteredWithSlash {
                space_id,
                slash_amount: T::CreatorRegistrationDeposit::get(),
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
            ensure!(Self::is_creator_active(space_id), Error::<T>::InactiveCreator);

            // Get the staking ledger or create an entry if it doesn't exist.
            let mut ledger = Self::ledger(&staker);
            let available_balance = Self::balance_available_for_staking(&staker, &ledger);
            let amount_to_stake = amount.min(available_balance);
            ensure!(amount_to_stake > Zero::zero(), Error::<T>::CannotStakeZero);

            let current_era = Self::current_era();
            let mut staker_info = Self::staker_info(&staker, space_id);
            let mut staking_info =
                Self::creator_stake_info(space_id, current_era).unwrap_or_default();

            Self::stake_to_creator(
                &mut staker_info,
                &mut staking_info,
                amount_to_stake,
                current_era,
            )?;

            ledger.locked = ledger.locked.saturating_add(amount_to_stake);

            // Update storage
            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_add(amount_to_stake);
                    x.locked = x.locked.saturating_add(amount_to_stake);
                }
            });

            Self::update_ledger(&staker, ledger);
            Self::update_staker_info(&staker, space_id, staker_info);
            CreatorEraStake::<T>::insert(space_id, current_era, staking_info);

            Self::deposit_event(Event::<T>::Staked {
                who: staker,
                creator: space_id,
                era: current_era,
                amount: amount_to_stake,
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
            ensure!(Self::is_creator_active(space_id), Error::<T>::InactiveCreator);

            let current_era = Self::current_era();
            let mut staker_info = Self::staker_info(&staker, space_id);
            let mut stake_info =
                Self::creator_stake_info(space_id, current_era).unwrap_or_default();

            let amount_to_unstake =
                Self::calculate_final_unstaking_amount(&mut staker_info, &mut stake_info, amount, current_era)?;

            // Update the chunks and write them to storage
            let mut ledger = Self::ledger(&staker);
            ledger.unbonding_info.add(UnlockingChunk {
                amount: amount_to_unstake,
                unlock_era: current_era + T::UnbondingPeriodInEras::get(),
            });
            // This should be done AFTER insertion since it's possible for chunks to merge
            ensure!(
                ledger.unbonding_info.len() <= T::MaxUnlockingChunks::get(),
                Error::<T>::TooManyUnlockingChunks
            );

            Self::update_ledger(&staker, ledger);

            // Update total staked value in era.
            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(amount_to_unstake)
                }
            });
            Self::update_staker_info(&staker, space_id, staker_info);
            CreatorEraStake::<T>::insert(space_id, current_era, stake_info);

            Self::deposit_event(Event::<T>::Unstaked {
                who: staker,
                creator: space_id,
                era: current_era,
                amount: amount_to_unstake,
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

            let (withdrawable_chunks, future_chunks) = ledger.unbonding_info.partition(current_era);
            let withdraw_amount = withdrawable_chunks.sum();

            ensure!(!withdraw_amount.is_zero(), Error::<T>::NothingToWithdraw);

            // Get the staking ledger and update it
            ledger.locked = ledger.locked.saturating_sub(withdraw_amount);
            ledger.unbonding_info = future_chunks;
            Self::update_ledger(&staker, ledger);

            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.locked = x.locked.saturating_sub(withdraw_amount)
                }
            });

            Self::deposit_event(Event::<T>::WithdrawnUnstaked {
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
                RegisteredCreators::<T>::get(space_id).ok_or(Error::<T>::CreatorNotFound)?;

            let unregistered_era = if let CreatorStatus::Inactive(x) = creator_info.status {
                x
            } else {
                return Err(Error::<T>::NotUnregisteredCreator.into());
            };

            // There should be some leftover staked amount
            let mut staker_info = Self::staker_info(&staker, space_id);
            let staked_value = staker_info.current_stake();
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
            GeneralEraInfo::<T>::mutate(current_era, |value| {
                if let Some(x) = value {
                    x.staked = x.staked.saturating_sub(staked_value);
                    x.locked = x.locked.saturating_sub(staked_value);
                }
            });

            Self::deposit_event(Event::<T>::WithdrawnFromInactiveCreator {
                who: staker,
                amount: staked_value,
            });

            Ok(().into())
        }

        // Claim rewards for the staker on the oldest unclaimed era where they has a stake 
        // and optionally restake the rewards to the same creator.
        // Not sure here whether to calculate total rewards for all creators
        //  or to withdraw per-creator rewards (preferably)
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn claim_staker_reward(origin: OriginFor<T>, space_id: SpaceId, restake: bool) -> DispatchResultWithPostInfo {
            Self::ensure_pallet_enabled()?;
            let staker = ensure_signed(origin)?;

            // Ensure we have something to claim
            let mut staker_info = Self::staker_info(&staker, space_id);
            let (era_to_claim, backer_staked) = staker_info.claim();
            ensure!(backer_staked > Zero::zero(), Error::<T>::NotStakedCreator);

            let creator_info =
                RegisteredCreators::<T>::get(space_id).ok_or(Error::<T>::CreatorNotFound)?;

            if let CreatorStatus::Inactive(unregister_era) = creator_info.status {
                ensure!(era_to_claim < unregister_era, Error::<T>::InactiveCreator);
            }

            let current_era = Self::current_era();
            ensure!(era_to_claim < current_era, Error::<T>::EraOutOfBounds);

            let staking_info = Self::creator_stake_info(space_id, era_to_claim).unwrap_or_default();
            let reward_and_stake =
                Self::general_era_info(era_to_claim).ok_or(Error::<T>::EraNotFound)?;

            let (_, combined_stakers_reward_share) =
                Self::distributed_rewards_between_creator_and_stakers(&staking_info, &reward_and_stake);
            let staker_reward =
                Perbill::from_rational(backer_staked, staking_info.total) * combined_stakers_reward_share;

            let mut ledger = Self::ledger(&staker);

            // Can restake only if the backer has a stake on the active creator:
            let should_restake_reward = Self::should_restake_reward(
                restake,
                creator_info.status,
                staker_info.current_stake(),
            );

            if should_restake_reward {
                staker_info
                    .increase_stake(current_era, staker_reward)
                    .map_err(|_| Error::<T>::UnexpectedStakeInfoEra)?;

                // Restaking will, in the worst case, remove one, and add one record,
                // so it's fine if the vector is full
                ensure!(
                    staker_info.len() <= T::MaxEraStakeItems::get(),
                    Error::<T>::TooManyEraStakeValues
                );
            }

            // Withdraw reward funds from rewards holding account
            let reward_imbalance = T::Currency::withdraw(
                &Self::rewards_pot_account(),
                staker_reward,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?;

            T::Currency::resolve_creating(&staker, reward_imbalance);

            if should_restake_reward {
                ledger.locked = ledger.locked.saturating_add(staker_reward);
                Self::update_ledger(&staker, ledger);

                // Update storage
                GeneralEraInfo::<T>::mutate(current_era, |value| {
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
                    era: current_era,
                    amount: staker_reward,
                });
            }

            Self::update_staker_info(&staker, space_id, staker_info);

            Self::deposit_event(Event::<T>::StakerRewardsClaimed {
                who: staker,
                creator: space_id,
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
                RegisteredCreators::<T>::get(space_id).ok_or(Error::<T>::CreatorNotFound)?;

            let mut creator_stake_info =
                Self::creator_stake_info(space_id, era).unwrap_or_default();

            let creator_reward = Self::calculate_creator_reward(&creator_stake_info, &creator_info, era)?;

            // Withdraw reward funds from the creators staking
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
            CreatorEraStake::<T>::insert(space_id, era, creator_stake_info);

            Ok(().into())
        }

        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
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

        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn force_new_era(origin: OriginFor<T>) -> DispatchResult {
            Self::ensure_pallet_enabled()?;
            ensure_root(origin)?;
            ForceEra::<T>::put(Forcing::ForceNew);
            Ok(Pays::No.into())
        }

        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        pub fn set_reward_distribution_config(
            origin: OriginFor<T>,
            new_config: RewardsDistributionConfig,
        ) -> DispatchResult {
            Self::ensure_pallet_enabled()?;
            ensure_root(origin)?;

            ensure!(new_config.is_consistent(), Error::<T>::RewardDistributionConfigInconsistent);
            ActiveRewardDistributionConfig::<T>::put(new_config);

            Ok(())
        }

        // #[weight = 10_000]
        // fn set_annual_inflation(origin, inflation: Perbill) {
        //     ensure_root(origin)?;
        //     todo!()
        // }
    }
}
