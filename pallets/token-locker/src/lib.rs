#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult, pallet_prelude::*,
        traits::{Currency, LockableCurrency, LockIdentifier, WithdrawReasons},
    };
    use frame_system::Pallet as System;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Saturating, StaticLookup};
    use crate::weights::WeightInfo;

    const PALLET_ID: LockIdentifier = *b"brdglck ";

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: LockableCurrency<Self::AccountId>;

        #[pallet::constant]
        type UnlockPeriod: Get<Self::BlockNumber>;

        #[pallet::constant]
        type MinLockAmount: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxLockAmount: Get<BalanceOf<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn lock_details)]
    pub type LockDetails<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn unlock_at)]
    pub type UnlockAt<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, T::BlockNumber>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        SubLocked(
            T::AccountId, // locker
            BalanceOf<T>, // amount
            T::AccountId, // target
        ),
        RefundRequested(T::AccountId),
        SubRefunded(T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// User cannot lock more than once.
        AlreadyLocked,
        /// Cannot request unlock since tokens weren't locked.
        NotLocked,
        /// Cannot lock that few tokens.
        LockAmountLowerThanMinLock,
        /// Cannot lock that many tokens.
        LockAmountGreaterThanMaxLock,
        /// User cannot lock more than a free balance he has.
        BalanceIsTooLowToLock,
        /// Unlock was already requested.
        UnlockAlreadyRequested,
        /// Unlock was not requested.
        UnlockNotRequested,
        /// Unlock period isn't over yet.
        TooEarlyToRefund,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::lock_sub())]
        pub fn lock_sub(
            origin: OriginFor<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            target: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;

            ensure!(Self::lock_details(&who).is_none(), Error::<T>::AlreadyLocked);

            ensure!(amount >= T::MinLockAmount::get(), Error::<T>::LockAmountLowerThanMinLock);
            ensure!(amount <= T::MaxLockAmount::get(), Error::<T>::LockAmountGreaterThanMaxLock);

            let free = T::Currency::free_balance(&who);
            ensure!(free > amount, Error::<T>::BalanceIsTooLowToLock);

            T::Currency::set_lock(PALLET_ID, &who, amount, WithdrawReasons::empty());

            LockDetails::<T>::insert(&who, amount);

            Self::deposit_event(Event::SubLocked(who, amount, target));
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::request_unlock())]
        pub fn request_unlock(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Self::lock_details(&who).is_some(), Error::<T>::NotLocked);
            ensure!(Self::unlock_at(&who).is_none(), Error::<T>::UnlockAlreadyRequested);

            let current_block = System::<T>::block_number();
            UnlockAt::<T>::insert(&who, current_block.saturating_add(T::UnlockPeriod::get()));

            Self::deposit_event(Event::RefundRequested(who));
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::try_refund())]
        pub fn try_refund(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let amount = Self::lock_details(&who).ok_or(Error::<T>::NotLocked)?;
            let unlock_at = Self::unlock_at(&who).ok_or(Error::<T>::UnlockNotRequested)?;

            ensure!(System::<T>::block_number() >= unlock_at, Error::<T>::TooEarlyToRefund);

            T::Currency::remove_lock(PALLET_ID, &who);
            LockDetails::<T>::remove(&who);
            UnlockAt::<T>::remove(&who);

            Self::deposit_event(Event::SubRefunded(who, amount));
            Ok(())
        }
    }
}
