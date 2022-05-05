#![cfg_attr(not(feature = "std"), no_std)]

///! # Influencer staking module.
///! This module contains the functionality for the influencer staking.
///!
///! ## Overview
///! The influencer staking module is a simple staking system that allows users to stake tokens for
///! a given user (influencer). The staking system is designed to be used by influencers to incentivize
///! their social contributes. Rewards from staking inflation is splitterd between the stakers and the
///! influencer.


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
    use crate::types::{InfluencerInfo, RewardSplitConfig, Round, RoundIndex, StakerInfo};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type MaxUnlockingChunks: Get<u32>;

        #[pallet::constant]
        type InfluenceRegistrationDeposit: Get<BalanceOf<Self>>;

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
    pub type Influencers<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        InfluencerInfo<T>,
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
    pub type StakedPerInfluencerPerStaker<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId, // influencer
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
        InfluencerRegistered {
            influencer: T::AccountId,
        },
        InfluencerUnregistered {
            influencer: T::AccountId,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        StakerDNE,
        InfluencerDNE,
        InfluencerAlreadyRegistered,
        NotStakedForInfluencer,
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
        pub fn register_influencer(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let influencer = ensure_signed(origin)?;
            ensure!(!Self::is_influencer(&influencer), Error::<T>::InfluencerAlreadyRegistered);

            let deposit = T::InfluenceRegistrationDeposit::get();
            T::Currency::reserve(&influencer, deposit)?;

            <Influencers<T>>::insert(
                &influencer,
                InfluencerInfo::from_account(influencer.clone(), deposit),
            );

            Self::deposit_event(Event::InfluencerRegistered { influencer });

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn unregister_influencer(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let influencer = ensure_signed(origin)?;

            let influencer_info = <Influencers<T>>::get(&influencer)
                .ok_or(Error::<T>::InfluencerDNE)?;

            T::Currency::unreserve(&influencer, influencer_info.deposit);

            <Influencers<T>>::remove(&influencer);

            Self::deposit_event(Event::InfluencerUnregistered { influencer });

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn stake(
            origin: OriginFor<T>,
            influencer: T::AccountId,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;
            ensure!(value >= T::MinStake::get(), Error::<T>::StakeTooLow);
            ensure!(
				T::Currency::can_reserve(&staker, value),
				Error::<T>::InsufficientBalance
			);

            let mut influencer_info = <Influencers<T>>::get(&influencer)
                .ok_or(Error::<T>::InfluencerDNE)?;

            let mut staker_info = <Stakers<T>>::get(&staker)
                .unwrap_or_else(|| StakerInfo::<T>::from_account(staker.clone(), Zero::zero()));

            Self::stake_for_influencer(&mut staker_info, &mut influencer_info, value)?;

            <Total<T>>::mutate(|total_stake| *total_stake += value);
            <Influencers<T>>::insert(&influencer, influencer_info);
            <Stakers<T>>::insert(&staker, staker_info);

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn unstake(
            origin: OriginFor<T>,
            influencer: T::AccountId,
            value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;
            ensure!(value > Zero::zero(), Error::<T>::UnstakingWithNoValue);

            let mut staker_info = <Stakers<T>>::get(&staker)
                .ok_or(Error::<T>::StakerDNE)?;

            let mut influencer_info = <Influencers<T>>::get(&influencer)
                .ok_or(Error::<T>::InfluencerDNE)?;

            Self::unstake_form_influencer(&mut staker_info, &mut influencer_info, value)?;

            <Total<T>>::mutate(|total_stake| *total_stake -= value);
            <Influencers<T>>::insert(&influencer, influencer_info);
            <Stakers<T>>::insert(&staker, staker_info);

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn unstake_all(
            origin: OriginFor<T>,
            influencer: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let staker = ensure_signed(origin)?;

            let mut staker_info = <Stakers<T>>::get(&staker)
                .ok_or(Error::<T>::StakerDNE)?;

            let mut influencer_info = <Influencers<T>>::get(&influencer)
                .ok_or(Error::<T>::InfluencerDNE)?;

            let prev_stake = Self::unstake_all_form_influencer(&mut staker_info, &mut influencer_info)?;

            <Total<T>>::mutate(|total_stake| *total_stake -= prev_stake);
            <Influencers<T>>::insert(&influencer, influencer_info);
            <Stakers<T>>::insert(&staker, staker_info);

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_influencer(who: &T::AccountId) -> bool {
            <Influencers<T>>::contains_key(who)
        }

        fn stake_for_influencer(
            staker_info: &mut StakerInfo<T>,
            influencer_info: &mut InfluencerInfo<T>,
            stake: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            staker_info.total = staker_info.total.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;
            staker_info.active = staker_info.active.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;

            let influencer = &influencer_info.id;

            let total_stake = match staker_info.staked_per_influencer.get(influencer) {
                Some(stake) => stake.checked_add(&stake).ok_or(ArithmeticError::Overflow)?,
                None => {
                    influencer_info.stakers_count.checked_add(One::one()).ok_or(ArithmeticError::Overflow)?;
                    stake
                },
            };

            influencer_info.staked_amount = influencer_info.staked_amount.checked_add(&stake).ok_or(ArithmeticError::Overflow)?;
            staker_info.staked_per_influencer.insert(influencer.clone(), total_stake);

            Ok(())
        }

        fn unstake_form_influencer(
            staker_info: &mut StakerInfo<T>,
            influencer_info: &mut InfluencerInfo<T>,
            stake: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            staker_info.total = staker_info.total.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.active = staker_info.active.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;

            let influencer = &influencer_info.id;

            let current_stake = staker_info.staked_per_influencer.get(influencer).ok_or(Error::<T>::NotStakedForInfluencer)?;
            if stake > *current_stake {
                return Err(Error::<T>::RemainingStakeTooLow.into());
            }

            let remaining_stake = current_stake.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            if remaining_stake < T::MinStake::get() {
                return Err(Error::<T>::RemainingStakeTooLow.into());
            }

            influencer_info.staked_amount = influencer_info.staked_amount.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.staked_per_influencer.insert(influencer.clone(), remaining_stake);

            Ok(().into())
        }

        fn unstake_all_form_influencer(
            staker_info: &mut StakerInfo<T>,
            influencer_info: &mut InfluencerInfo<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let influencer = &influencer_info.id;

            let stake = staker_info.staked_per_influencer.get(influencer)
                .ok_or(Error::<T>::NotStakedForInfluencer)?.clone();

            staker_info.total = staker_info.total.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            staker_info.active = staker_info.active.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;

            influencer_info.staked_amount = influencer_info.staked_amount.checked_sub(&stake).ok_or(ArithmeticError::Underflow)?;
            influencer_info.stakers_count.checked_sub(One::one()).ok_or(ArithmeticError::Underflow)?;
            staker_info.staked_per_influencer.remove(influencer);

            Ok(stake)
        }
    }
}
