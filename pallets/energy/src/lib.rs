//! # Energy Module

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::traits::Currency;
    use pallet_transaction_payment::OnChargeTransaction;
    use sp_runtime::ArithmeticError;
    use sp_runtime::traits::{CheckedAdd, CheckedSub, DispatchInfoOf, PostDispatchInfoOf, Saturating, StaticLookup, Zero};
    use sp_std::convert::TryInto;
    use crate::*;

    pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency type.
        type Currency: Currency<Self::AccountId>;

        /// The fallback [OnChargeTransaction] that should be used if there is not enough energy to
        /// pay the transaction fees.
        type FallbackOnChargeTransaction: OnChargeTransaction<Self, Balance=BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Energy have been generated to an account.
        EnergyGenerated {
            /// The account that generated the energy.
            generator: T::AccountId,
            /// The account that received the energy.
            receiver: T::AccountId,
            /// The amount of balance that was burnt.
            burnt_balance: BalanceOf<T>,
            /// The amount of energy that have been generated.
            generated_energy: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Not enough balance to generate energy.
        NotEnoughBalance,
    }

    /// Total energy generated.
    #[pallet::storage]
    #[pallet::getter(fn total_energy)]
    pub(crate) type TotalEnergy<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Energy credited to each account.
    #[pallet::storage]
    #[pallet::getter(fn avilable_energy)]
    pub(crate) type EnergyPerAccount<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Generate energy for a target account by burning balance from the caller.
        #[pallet::weight(10_000)]
        pub fn generate(
            origin: OriginFor<T>,
            target: <T::Lookup as StaticLookup>::Source,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;

            ensure!(T::Currency::can_slash(&caller, amount), Error::<T>::NotEnoughBalance);
            let _ = T::Currency::slash(&caller, amount);

            Self::generate_energy(&target, amount)?;

            Self::deposit_event(Event::EnergyGenerated {
                generator: caller,
                receiver: target,
                burnt_balance: amount,
                generated_energy: amount,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn generate_energy(target: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            TotalEnergy::<T>::mutate(|total| -> Result<(), ArithmeticError> {
                *total = total.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            EnergyPerAccount::<T>::mutate(target, |energy| -> Result<(), ArithmeticError> {
                *energy = energy.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            Ok(())
        }

        fn consume_energy(target: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            EnergyPerAccount::<T>::mutate(target, |energy| -> Result<(), ArithmeticError> {
                *energy = energy.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;
            TotalEnergy::<T>::mutate(|total| -> Result<(), ArithmeticError> {
                *total = total.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;
            Ok(())
        }
    }


    pub enum LiquidityInfo<T: Config> {
        /// Nothing have been paid.
        Nothing,
        /// Transaction have been paid using energy.
        Energy(BalanceOf<T>),
        /// Transaction have been paid using the fallback method.
        Fallback(<T::FallbackOnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo),
    }

    impl<T: Config> Default for LiquidityInfo<T> {
        fn default() -> Self {
            LiquidityInfo::Nothing
        }
    }

    impl<T: Config> OnChargeTransaction<T> for Pallet<T> {
        type Balance = BalanceOf<T>;
        type LiquidityInfo = LiquidityInfo<T>;

        fn withdraw_fee(
            who: &T::AccountId,
            call: &T::Call,
            dispatch_info: &DispatchInfoOf<T::Call>,
            fee: Self::Balance,
            tip: Self::Balance,
        ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
            if fee.is_zero() {
                return Ok(LiquidityInfo::Nothing);
            }

            if Self::avilable_energy(&who) < fee {
                return T::FallbackOnChargeTransaction::withdraw_fee(who, call, dispatch_info, fee, tip)
                    .map(|fallback_info| LiquidityInfo::Fallback(fallback_info));
            }

            match Self::consume_energy(who, fee) {
                Ok(()) => Ok(LiquidityInfo::Energy(fee)),
                Err(_) => Err(InvalidTransaction::Payment.into()),
            }
        }

        fn correct_and_deposit_fee(
            who: &T::AccountId,
            dispatch_info: &DispatchInfoOf<T::Call>,
            post_info: &PostDispatchInfoOf<T::Call>,
            corrected_fee: Self::Balance,
            tip: Self::Balance,
            already_withdrawn: Self::LiquidityInfo,
        ) -> Result<(), TransactionValidityError> {
            match already_withdrawn {
                LiquidityInfo::Nothing => Ok(()),
                LiquidityInfo::Fallback(fallback_info) => T::FallbackOnChargeTransaction::correct_and_deposit_fee(
                    who,
                    dispatch_info,
                    post_info,
                    corrected_fee,
                    tip,
                    fallback_info,
                ),
                LiquidityInfo::Energy(paid) => {
                    let refund_amount = paid.saturating_sub(corrected_fee);
                    let _ = Self::generate_energy(who, refund_amount);

                    // we don't do anything with the fees + tip.

                    Ok(())
                }
            }
        }
    }
}