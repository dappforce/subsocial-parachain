#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

///! # Creator staking module.
///! This module contains the functionality for the creator staking.
///!
///! ## Overview
///! The creator staking module is a simple staking system that allows users to stake tokens for
///! a given user (creator). The staking system is designed to be used by creators to incentivize
///! their social contributes. Rewards from staking inflation is splitted between the stakers and the
///! creator.


pub mod types;

#[cfg(test)]
mod mock;
mod inflation;

use frame_support::traits::Currency;
pub use pallet::*;

pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;


#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_support::traits::ReservableCurrency;
    use frame_system::pallet_prelude::*;
    use sp_runtime::ArithmeticError;
    use sp_runtime::traits::{CheckedAdd, CheckedSub, One, Zero};
    use sp_std::prelude::*;
    use crate::BalanceOf;
    use crate::inflation::InflationInfo;
    use crate::types::{CreatorInfo, RewardInfo, RewardConfigInfo, Round, RoundIndex, RoundInfo, StakerInfo, StakeState};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type MaxUnlockingChunks: Get<u32>;

        #[pallet::constant]
        type CreatorRegistrationDeposit: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MinStake: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type RewardConfig<T: Config> = StorageValue<
        _,
        RewardConfigInfo,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn inflation_config)]
    pub type InflationConfig<T: Config> = StorageValue<
        _,
        InflationInfo<BalanceOf<T>>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type CurrentRound<T: Config> = StorageValue<
        _,
        Round<T::BlockNumber>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type Creators<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        CreatorInfo<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub(crate) type Stakers<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        StakerInfo<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type StakedPerCreatorPerStaker<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId, // creator
        Twox64Concat,
        T::AccountId, // staker
        BalanceOf<T>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub(crate) type GeneralRoundInfo<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        RoundIndex,
        RoundInfo<BalanceOf<T>>,
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub inflation_config: InflationInfo<BalanceOf<T>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                inflation_config: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <InflationConfig<T>>::put(self.inflation_config.clone());
        }
    }
    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewRound {
            starting_block: T::BlockNumber,
            round: RoundIndex,
        },
        CreatorRegistered {
            creator: T::AccountId,
        },
        CreatorUnregistered {
            creator: T::AccountId,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        StakerDNE,
        CreatorDNE,
        CreatorAlreadyRegistered,
        NotStakedForCreator,
        StakeTooLow,
        RemainingStakeTooLow,
        NotEnoughStake,
        InsufficientBalance,
        UnstakingWithNoValue,
        RoundNumberOutOfBounds,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // TODO: fix this
            let mut weight = Zero::zero();

            let mut round = <CurrentRound<T>>::get();
            if round.should_update(n) {
                // mutate round
                round.update(n);
                <CurrentRound<T>>::put(round);

                if let Some(prev_index) = round.index.checked_sub(One::one()) {
                    let rewards = Self::compute_round_reward(prev_index);
                    GeneralRoundInfoOps::<T>::set_rewards(prev_index, rewards);
                }
                GeneralRoundInfoOps::<T>::create_from_previous(round.index);

                Self::deposit_event(Event::NewRound {
                    starting_block: round.first,
                    round: round.index,
                });
            }


            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn register_creator(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let creator = ensure_signed(origin)?;
            ensure!(!Self::is_creator(&creator), Error::<T>::CreatorAlreadyRegistered);

            let deposit = T::CreatorRegistrationDeposit::get();
            T::Currency::reserve(&creator, deposit)?;

            <Creators<T>>::insert(
                &creator,
                CreatorInfo::from_account(creator.clone(), deposit),
            );

            Self::deposit_event(Event::CreatorRegistered { creator });

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn unregister_creator(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let creator = ensure_signed(origin)?;

            let creator_info = <Creators<T>>::get(&creator)
                .ok_or(Error::<T>::CreatorDNE)?;

            T::Currency::unreserve(&creator, creator_info.deposit);

            <Creators<T>>::remove(&creator);

            Self::deposit_event(Event::CreatorUnregistered { creator });

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn stake(
            origin: OriginFor<T>,
            creator: T::AccountId,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;
            ensure!(value >= T::MinStake::get(), Error::<T>::StakeTooLow);
            ensure!(
				T::Currency::can_reserve(&staker, value),
				Error::<T>::InsufficientBalance
			);

            let mut creator_info = <Creators<T>>::get(&creator)
                .ok_or(Error::<T>::CreatorDNE)?;

            let mut staker_info = <Stakers<T>>::get(&staker)
                .unwrap_or_else(|| StakerInfo::<T>::from_account(staker.clone(), Zero::zero()));

            let current_round_index = Self::current_round_index();
            Self::stake_for_creator(
                current_round_index,
                &mut staker_info,
                &mut creator_info,
                value,
            )?;

            GeneralRoundInfoOps::<T>::add_stake(current_round_index, value)?;
            <Creators<T>>::insert(&creator, creator_info);
            <Stakers<T>>::insert(&staker, staker_info);

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn unstake(
            origin: OriginFor<T>,
            creator: T::AccountId,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;
            ensure!(value > Zero::zero(), Error::<T>::UnstakingWithNoValue);

            let mut staker_info = <Stakers<T>>::get(&staker)
                .ok_or(Error::<T>::StakerDNE)?;

            let mut creator_info = <Creators<T>>::get(&creator)
                .ok_or(Error::<T>::CreatorDNE)?;

            let current_round_index = Self::current_round_index();
            Self::unstake_form_creator(
                current_round_index,
                &mut staker_info,
                &mut creator_info,
                value,
            )?;

            GeneralRoundInfoOps::<T>::unstake_and_unlock(current_round_index, value)?;
            <Creators<T>>::insert(&creator, creator_info);
            <Stakers<T>>::insert(&staker, staker_info);

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn unstake_all(
            origin: OriginFor<T>,
            creator: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;

            let mut staker_info = <Stakers<T>>::get(&staker)
                .ok_or(Error::<T>::StakerDNE)?;

            let mut creator_info = <Creators<T>>::get(&creator)
                .ok_or(Error::<T>::CreatorDNE)?;

            let current_round_index = Self::current_round_index();
            let prev_stake = Self::unstake_all_form_creator(
                current_round_index,
                &mut staker_info,
                &mut creator_info,
            )?;

            GeneralRoundInfoOps::<T>::unstake_and_unlock(current_round_index, prev_stake)?;
            <Creators<T>>::insert(&creator, creator_info);
            <Stakers<T>>::insert(&staker, staker_info);

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_creator(who: &T::AccountId) -> bool {
            <Creators<T>>::contains_key(who)
        }

        fn current_round_index() -> RoundIndex {
            CurrentRound::<T>::get().index
        }

        fn stake_for_creator(
            round_index: RoundIndex,
            staker_info: &mut StakerInfo<T>,
            creator_info: &mut CreatorInfo<T>,
            stake: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            staker_info.total = staker_info.total.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;
            staker_info.active = staker_info.active.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;

            let creator = &creator_info.id;

            let mut stake_state = match staker_info.stake_per_creator.get(creator) {
                Some(stake_state) => stake_state.clone(),
                None => {
                    creator_info.stakers_count.checked_add(One::one()).ok_or(ArithmeticError::Overflow)?;
                    StakeState::<T>::default()
                }
            };

            stake_state.stake(round_index, stake)?;
            staker_info.stake_per_creator.insert(creator.clone(), stake_state);

            creator_info.staked_amount = creator_info.staked_amount.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;

            Ok(())
        }

        fn unstake_form_creator(
            round_index: RoundIndex,
            staker_info: &mut StakerInfo<T>,
            creator_info: &mut CreatorInfo<T>,
            stake: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            staker_info.total = staker_info.total.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.active = staker_info.active.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;

            let creator = &creator_info.id;

            let mut staking_state = staker_info.stake_per_creator.get(creator)
                .ok_or(Error::<T>::NotStakedForCreator)?
                .clone();
            let current_stake = staking_state.latest_staked_value();

            ensure!(!current_stake.is_zero(), Error::<T>::NotStakedForCreator);

            if stake > current_stake {
                return Err(ArithmeticError::Underflow.into());
            }

            let remaining_stake = current_stake.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            if remaining_stake < T::MinStake::get() {
                return Err(Error::<T>::RemainingStakeTooLow.into());
            }

            staking_state.unstake(round_index, stake)?;

            creator_info.staked_amount = creator_info.staked_amount.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.stake_per_creator.insert(creator.clone(), staking_state);

            Ok(().into())
        }

        fn unstake_all_form_creator(
            round_index: RoundIndex,
            staker_info: &mut StakerInfo<T>,
            creator_info: &mut CreatorInfo<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let creator = &creator_info.id;

            let mut stake_state = staker_info.stake_per_creator.get(creator)
                .ok_or(Error::<T>::NotStakedForCreator)?
                .clone();

            let stake = stake_state.latest_staked_value();

            ensure!(!stake.is_zero(), Error::<T>::NotStakedForCreator);

            staker_info.total = staker_info.total.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.active = staker_info.active.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;

            creator_info.staked_amount = creator_info.staked_amount.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            creator_info.stakers_count.checked_sub(One::one()).ok_or(ArithmeticError::Underflow)?;

            stake_state.unstake(round_index, stake)?;
            staker_info.stake_per_creator.insert(creator.clone(), stake_state);

            Ok(stake)
        }

        fn rotate_round_info(round_index: RoundIndex) {

        }

        /// Compute round issuance based on total locked.
        fn compute_round_issuance(round_index: RoundIndex) -> BalanceOf<T> {
            let total_locked = GeneralRoundInfo::<T>::get(round_index).locked;

            let config = <InflationConfig<T>>::get();

            let round_issuance = crate::inflation::round_issuance_range::<T>(config.round);
            // TODO: consider interpolation instead of bounded range
            if total_locked < config.expect.min {
                round_issuance.min
            } else if total_locked > config.expect.max {
                round_issuance.max
            } else {
                round_issuance.ideal
            }
        }

        /// Split the current round reward between stakers and creators.
        fn compute_round_reward(round_index: RoundIndex) -> RewardInfo<BalanceOf<T>> {
            let split = RewardConfig::<T>::get();
            let issuance = Self::compute_round_issuance(round_index);

            RewardInfo::<BalanceOf<T>> {
                stakers: split.stakers_percentage * issuance,
                creators: split.creators_percentage * issuance,
            }
        }
    }

    pub(crate) struct GeneralRoundInfoOps<T: Config>(PhantomData<T>);
    impl<T: Config> GeneralRoundInfoOps<T> {

        /// Adds a new stake to the round information, increasing both the locked and staked values.
        pub fn add_stake(round_index: RoundIndex, value: BalanceOf<T>) -> DispatchResult {
            GeneralRoundInfo::<T>::try_mutate(round_index, |info| {
                info.staked.checked_add(&value).ok_or(ArithmeticError::Overflow)?;
                info.locked.checked_add(&value).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })
        }

        /// Unstaked funds that will undergo some period to be fully unlocked.
        pub fn unstake_locked(round_index: RoundIndex, value: BalanceOf<T>) -> DispatchResult {
            GeneralRoundInfo::<T>::try_mutate(round_index, |info| {
                info.staked.checked_sub(&value).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })
        }

        /// Unlocks funds that was previously unstaked.
        pub fn unlock_unstaked(round_index: RoundIndex, value: BalanceOf<T>) -> DispatchResult {
            GeneralRoundInfo::<T>::try_mutate(round_index, |info| {
                info.locked.checked_sub(&value).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })
        }

        /// Unstake and unlock funds. Should be called when user will withdraw his funds immediately.
        pub fn unstake_and_unlock(round_index: RoundIndex, value: BalanceOf<T>) -> DispatchResult {
            GeneralRoundInfo::<T>::try_mutate(round_index, |info| {
                info.staked.checked_sub(&value).ok_or(ArithmeticError::Underflow)?;
                info.locked.checked_sub(&value).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })
        }

        /// Sets the rewards information for the current era.
        pub fn set_rewards(round_index: RoundIndex, rewards: RewardInfo<BalanceOf<T>>) {
            GeneralRoundInfo::<T>::mutate(round_index, |info| {
                info.rewards = rewards;
            });
        }

        /// Copies locked and staked information from the previous round.
        pub fn create_from_previous(round_index: RoundIndex) {
            let previous_round_info = round_index.checked_sub(One::one())
                .and_then(|index| GeneralRoundInfo::<T>::get(index).into())
                .unwrap_or_default();

            GeneralRoundInfo::<T>::insert(round_index, RoundInfo::<BalanceOf<T>> {
                rewards: Default::default(),
                locked: previous_round_info.locked,
                staked: previous_round_info.staked,
            });
        }
    }
}
