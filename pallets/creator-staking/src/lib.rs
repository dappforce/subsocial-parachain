#![cfg_attr(not(feature = "std"), no_std)]

///! # Creator staking module.
///! This module contains the functionality for the creator staking.
///!
///! ## Overview
///! The creator staking module is a simple staking system that allows users to stake tokens for
///! a given user (creator). The staking system is designed to be used by creators to incentivize
///! their social contributes. Rewards from staking inflation is splitted between the stakers and the
///! creator.


pub mod types;

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
    use crate::BalanceOf;
    use crate::types::{CreatorInfo, RewardSplitConfig, Round, RoundIndex, StakerInfo};

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
        RewardSplitConfig,
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
    pub type Stakers<T: Config> = StorageMap<
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

    /// Total capital locked by this pallet.
    #[pallet::storage]
    pub(crate) type Total<T: Config> = StorageValue<
        _,
        BalanceOf<T>,
        ValueQuery,
    >;

    /// Total capital locked by this pallet.
    #[pallet::storage]
    pub(crate) type StakedPerRound<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        RoundIndex,
        BalanceOf<T>,
        ValueQuery,
    >;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
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
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            todo!()
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

            Self::stake_for_creator(&mut staker_info, &mut creator_info, value)?;

            <Total<T>>::mutate(|total_stake| *total_stake += value);
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

            Self::unstake_form_creator(&mut staker_info, &mut creator_info, value)?;

            <Total<T>>::mutate(|total_stake| *total_stake -= value);
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

            let prev_stake = Self::unstake_all_form_creator(&mut staker_info, &mut creator_info)?;

            <Total<T>>::mutate(|total_stake| *total_stake -= prev_stake);
            <Creators<T>>::insert(&creator, creator_info);
            <Stakers<T>>::insert(&staker, staker_info);

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_creator(who: &T::AccountId) -> bool {
            <Creators<T>>::contains_key(who)
        }

        fn stake_for_creator(
            staker_info: &mut StakerInfo<T>,
            creator_info: &mut CreatorInfo<T>,
            stake: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            staker_info.total = staker_info.total.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;
            staker_info.active = staker_info.active.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;

            let creator = &creator_info.id;

            let total_stake = match staker_info.staked_per_creator.get(creator) {
                Some(stake) => stake.checked_add(&stake).ok_or(ArithmeticError::Overflow)?,
                None => {
                    creator_info.stakers_count.checked_add(One::one()).ok_or(ArithmeticError::Overflow)?;
                    stake
                },
            };

            creator_info.staked_amount = creator_info.staked_amount.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;
            staker_info.staked_per_creator.insert(creator.clone(), total_stake);

            Ok(())
        }

        fn unstake_form_creator(
            staker_info: &mut StakerInfo<T>,
            creator_info: &mut CreatorInfo<T>,
            stake: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            staker_info.total = staker_info.total.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.active = staker_info.active.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;

            let creator = &creator_info.id;

            let current_stake = staker_info.staked_per_creator.get(creator).ok_or(Error::<T>::NotStakedForCreator)?;
            if stake > *current_stake {
                return Err(Error::<T>::RemainingStakeTooLow.into());
            }

            let remaining_stake = current_stake.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            if remaining_stake < T::MinStake::get() {
                return Err(Error::<T>::RemainingStakeTooLow.into());
            }

            creator_info.staked_amount = creator_info.staked_amount.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.staked_per_creator.insert(creator.clone(), remaining_stake);

            Ok(().into())
        }

        fn unstake_all_form_creator(
            staker_info: &mut StakerInfo<T>,
            creator_info: &mut CreatorInfo<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let creator = &creator_info.id;

            let stake = staker_info.staked_per_creator.get(creator)
                .ok_or(Error::<T>::NotStakedForCreator)?.clone();

            staker_info.total = staker_info.total.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.active = staker_info.active.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;

            creator_info.staked_amount = creator_info.staked_amount.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            creator_info.stakers_count.checked_sub(One::one()).ok_or(ArithmeticError::Underflow)?;
            staker_info.staked_per_creator.remove(creator);

            Ok(stake)
        }
    }
}
