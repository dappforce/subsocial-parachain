//! # Energy Pallet

#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

pub use pallet::*;

pub use crate::weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::traits::{Currency, ExistenceRequirement, tokens::Balance, WithdrawReasons};
    use frame_system::pallet_prelude::*;
    use pallet_transaction_payment::OnChargeTransaction;
    use sp_runtime::{ArithmeticError, FixedI64, FixedPointNumber, FixedPointOperand};
    use sp_runtime::traits::{CheckedAdd, CheckedSub, DispatchInfoOf, PostDispatchInfoOf, Saturating, StaticLookup, Zero};
    use sp_std::convert::TryInto;
    use sp_std::fmt::Debug;

    use crate::*;

    pub(crate) type BalanceOf<T> = <T as Config>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency type.
        type Currency: Currency<Self::AccountId, Balance=Self::Balance>;

        /// The balance type.
        type Balance: Balance
        + MaybeSerializeDeserialize
        + Debug
        + MaxEncodedLen
        + FixedPointOperand;


        /// How much 1 NRG is worth in SUB.
        type DefaultValueCoefficient: Get<FixedI64>;

        /// The origin which may update the value coefficient ratio.
        type UpdateOrigin: EnsureOrigin<Self::Origin>;

        /// The fallback [OnChargeTransaction] that should be used if there is not enough energy to
        /// pay the transaction fees.
        type FallbackOnChargeTransaction: OnChargeTransaction<Self, Balance=BalanceOf<Self>>;

        /// The minimum amount of energy required to keep an account.
        type ExistentialDeposit: Get<Self::Balance>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
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
        },
        /// Energy value coefficient has been updated.
        ValueCoefficientRatioUpdated {
            /// The new value coefficient.
            new_coefficient: FixedI64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Not enough SUB balance to burn and generate energy.
        NotEnoughBalance,
        /// Value coefficient is not a positive number.
        ValueCoefficientIsNotPositive,
    }

    /// Supplies the [ValueCoefficient] with [T::DefaultValueCoefficient] if empty.
    #[pallet::type_value]
    pub(crate) fn ValueCoefficientOnEmpty<T: Config>() -> FixedI64 { T::DefaultValueCoefficient::get() }

    /// The current value coefficient.
    #[pallet::storage]
    #[pallet::getter(fn value_coefficient)]
    pub(crate) type ValueCoefficient<T: Config> = StorageValue<_, FixedI64, ValueQuery, ValueCoefficientOnEmpty<T>>;

    /// Total energy generated.
    #[pallet::storage]
    #[pallet::getter(fn total_energy)]
    pub(crate) type TotalEnergy<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Energy credited to each account.
    #[pallet::storage]
    #[pallet::getter(fn energy_balance)]
    pub(crate) type EnergyBalance<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Updates the value coefficient. Only callable by the `UpdateOrigin`.
        #[pallet::weight(< T as Config >::WeightInfo::update_value_coefficient())]
        pub fn update_value_coefficient(
            origin: OriginFor<T>,
            new_coefficient: FixedI64,
        ) -> DispatchResult {
            let _ = T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(new_coefficient > Zero::zero(), Error::<T>::ValueCoefficientIsNotPositive);

            ValueCoefficient::<T>::put(new_coefficient);

            Self::deposit_event(Event::ValueCoefficientRatioUpdated { new_coefficient });

            Ok(())
        }

        /// Generate energy for a target account by burning balance from the caller.
        #[pallet::weight(< T as Config >::WeightInfo::generate_energy())]
        pub fn generate_energy(
            origin: OriginFor<T>,
            target: <T::Lookup as StaticLookup>::Source,
            burn_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;

            let caller_balance = T::Currency::free_balance(&caller);
            let caller_balance_after_burn = caller_balance
                .checked_sub(&burn_amount)
                .ok_or(Error::<T>::NotEnoughBalance)?;

            let withdraw_reason = WithdrawReasons::all();

            T::Currency::ensure_can_withdraw(
                &caller,
                burn_amount,
                withdraw_reason,
                caller_balance_after_burn,
            )?;

            let captured_energy_amount = burn_amount;

            let current_balance = Self::ensure_can_capture_energy(&target, captured_energy_amount)?;

            let _ = T::Currency::withdraw(
                &caller,
                burn_amount,
                withdraw_reason,
                ExistenceRequirement::KeepAlive,
            )?;
            Self::capture_energy(current_balance, &target, captured_energy_amount);

            Self::deposit_event(Event::EnergyGenerated {
                generator: caller,
                receiver: target,
                burnt_balance: burn_amount,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Ensure that [account] can capture the given [amount] of energy.
        fn ensure_can_capture_energy(
            target: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            ensure!(
                Self::total_energy().checked_add(&amount).is_some(),
                ArithmeticError::Overflow,
            );
            let energy_balance = Self::energy_balance(target);
            ensure!(
                energy_balance.checked_add(&amount).is_some(),
                ArithmeticError::Overflow,
            );
            Ok(energy_balance)
        }

        /// Capture energy for [account].
        fn capture_energy(current_balance: BalanceOf<T>, target: &T::AccountId, amount: BalanceOf<T>) {
            if current_balance.saturating_add(amount) >= T::ExistentialDeposit::get() {
                frame_system::Pallet::<T>::inc_providers(target);
            }

            TotalEnergy::<T>::mutate(|total| {
                *total = total.saturating_add(amount);
            });
            EnergyBalance::<T>::mutate(target, |energy| {
                *energy = energy.saturating_add(amount);
                energy.clone()
            });
        }

        /// Ensure that [account] can consume the given [amount] of energy.
        fn ensure_can_consume_energy(
            target: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            ensure!(
                Self::total_energy().checked_sub(&amount).is_some(),
                ArithmeticError::Underflow,
            );
            let energy_balance = Self::energy_balance(target);
            ensure!(
                energy_balance.checked_sub(&amount).is_some(),
                ArithmeticError::Underflow,
            );
            Ok(energy_balance)
        }

        /// Consume energy for [account].
        fn consume_energy(current_balance: BalanceOf<T>, target: &T::AccountId, mut amount: BalanceOf<T>) {
            if current_balance.saturating_sub(amount) < T::ExistentialDeposit::get() {
                frame_system::Pallet::<T>::dec_providers(target);

                amount = current_balance;
            }

            TotalEnergy::<T>::mutate(|total| {
                *total = total.saturating_sub(amount);
            });
            if amount == current_balance {
                EnergyBalance::<T>::remove(target);
            } else {
                EnergyBalance::<T>::mutate(target, |energy| {
                    *energy = energy.saturating_sub(amount);
                    energy.clone()
                });
            }
        }
    }


    /// Keeps track of how the user paid for the transaction.
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

            let adjusted_fee = Self::value_coefficient()
                .reciprocal()
                .unwrap() // SAFETY: value_coefficient is always positive. we check for it.
                .saturating_mul_int(fee);

            if Self::energy_balance(&who) < adjusted_fee {
                return T::FallbackOnChargeTransaction::withdraw_fee(who, call, dispatch_info, fee, tip)
                    .map(|fallback_info| LiquidityInfo::Fallback(fallback_info));
            }

            match Self::ensure_can_consume_energy(who, adjusted_fee) {
                Ok(current_balance) => {
                    Self::consume_energy(current_balance, who, adjusted_fee);
                    Ok(LiquidityInfo::Energy(adjusted_fee))
                }
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
                    let adjusted_corrected_fee = Self::value_coefficient()
                        .reciprocal()
                        .unwrap() // SAFETY: value_coefficient is always positive. we check for it.
                        .saturating_mul_int(corrected_fee);

                    let refund_amount = paid.saturating_sub(adjusted_corrected_fee);

                    let current_balance = Self::energy_balance(who);
                    let _ = Self::capture_energy(current_balance, who, refund_amount);

                    // we don't do anything with the fees + tip.
                    // TODO: maybe we tip using SUB?

                    Ok(())
                }
            }
        }
    }
}