// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

//! # Energy Pallet

#![cfg_attr(not(feature = "std"), no_std)]

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
    use frame_support::{
        pallet_prelude::*,
        traits::{tokens::Balance, Currency, ExistenceRequirement, WithdrawReasons},
    };
    use frame_system::pallet_prelude::*;
    use pallet_transaction_payment::OnChargeTransaction;
    use sp_runtime::{
        traits::{
            CheckedAdd, CheckedSub, DispatchInfoOf, PostDispatchInfoOf, Saturating, StaticLookup,
            Zero,
        },
        ArithmeticError, FixedI64, FixedPointNumber, FixedPointOperand,
    };
    use sp_std::{convert::TryInto, fmt::Debug};

    use crate::*;

    pub(crate) type BalanceOf<T> = <T as Config>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency type.
        type Currency: Currency<Self::AccountId, Balance = Self::Balance>;

        /// The balance type.
        type Balance: Balance
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + FixedPointOperand;

        /// How much 1 energy is worth in native tokens.
        /// TODO: change to FixedU64 when this is merged https://github.com/paritytech/substrate/pull/11555
        type DefaultValueCoefficient: Get<FixedI64>;

        /// The origin which may update the value coefficient ratio.
        type UpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The fallback [OnChargeTransaction] that should be used if there is not enough energy to
        /// pay the transaction fees.
        type NativeOnChargeTransaction: OnChargeTransaction<Self, Balance = BalanceOf<Self>>;

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
            /// The amount of balance that was burned.
            balance_burned: BalanceOf<T>,
        },
        /// Energy value coefficient has been updated.
        ValueCoefficientUpdated {
            /// The new value coefficient.
            new_coefficient: FixedI64,
        },
        /// An account was removed whose balance was non-zero but below
        /// ExistentialDeposit, resulting in an outright loss.
        DustLost { account: T::AccountId, amount: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Not enough native balance to burn and generate energy.
        NotEnoughBalance,
        /// Value coefficient is not a positive number.
        ValueCoefficientIsNotPositive,
        /// Value too low to create account due to existential deposit
        BalanceBelowExistentialDeposit,
    }

    /// Supplies the [ValueCoefficient] with [T::DefaultValueCoefficient] if empty.
    #[pallet::type_value]
    pub(crate) fn ValueCoefficientOnEmpty<T: Config>() -> FixedI64 {
        T::DefaultValueCoefficient::get()
    }

    /// The current value coefficient.
    #[pallet::storage]
    #[pallet::getter(fn value_coefficient)]
    pub(crate) type ValueCoefficient<T: Config> =
        StorageValue<_, FixedI64, ValueQuery, ValueCoefficientOnEmpty<T>>;

    /// Total energy generated.
    #[pallet::storage]
    #[pallet::getter(fn total_energy)]
    pub(crate) type TotalEnergy<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Energy credited to each account.
    #[pallet::storage]
    #[pallet::getter(fn energy_balance)]
    pub(crate) type EnergyBalance<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Updates the value coefficient. Only callable by the `UpdateOrigin`.
        #[pallet::call_index(0)]
        #[pallet::weight(< T as Config >::WeightInfo::update_value_coefficient())]
        pub fn update_value_coefficient(
            origin: OriginFor<T>,
            new_coefficient: FixedI64,
        ) -> DispatchResult {
            let _ = T::UpdateOrigin::ensure_origin(origin)?;

            ensure!(new_coefficient > Zero::zero(), Error::<T>::ValueCoefficientIsNotPositive);

            ValueCoefficient::<T>::put(new_coefficient);

            Self::deposit_event(Event::ValueCoefficientUpdated { new_coefficient });

            Ok(())
        }

        /// Generate energy for a target account by burning balance from the caller.
        #[pallet::call_index(1)]
        #[pallet::weight(< T as Config >::WeightInfo::generate_energy())]
        pub fn generate_energy(
            origin: OriginFor<T>,
            target: <T::Lookup as StaticLookup>::Source,
            burn_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;

            let caller_balance = T::Currency::free_balance(&caller);
            let caller_balance_after_burn =
                caller_balance.checked_sub(&burn_amount).ok_or(Error::<T>::NotEnoughBalance)?;

            let withdraw_reason = WithdrawReasons::all();

            T::Currency::ensure_can_withdraw(
                &caller,
                burn_amount,
                withdraw_reason,
                caller_balance_after_burn,
            )?;

            let captured_energy_amount = burn_amount;
            let current_energy_balance = Self::energy_balance(&target);
            let new_energy_balance = current_energy_balance
                .checked_add(&captured_energy_amount)
                .ok_or(ArithmeticError::Overflow)?;

            ensure!(
                new_energy_balance >= T::ExistentialDeposit::get(),
                Error::<T>::BalanceBelowExistentialDeposit
            );

            Self::ensure_can_capture_energy(&target, captured_energy_amount)?;

            let _ = T::Currency::withdraw(
                &caller,
                burn_amount,
                withdraw_reason,
                ExistenceRequirement::KeepAlive,
            )?;

            Self::capture_energy(&target, captured_energy_amount);

            Self::deposit_event(Event::EnergyGenerated {
                generator: caller,
                receiver: target,
                balance_burned: burn_amount,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Ensure that [account] can capture the given [amount] of energy, and returns current
        /// energy balance.
        fn ensure_can_capture_energy(
            target: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(Self::total_energy().checked_add(&amount).is_some(), ArithmeticError::Overflow);
            let energy_balance = Self::energy_balance(target);
            ensure!(energy_balance.checked_add(&amount).is_some(), ArithmeticError::Overflow);
            Ok(())
        }

        /// Capture energy for [account]. Increases energy balance by [amount] and also increases
        /// account providers if current energy balance is above [T::ExistentialDeposit].
        fn capture_energy(target: &T::AccountId, amount: BalanceOf<T>) {
            TotalEnergy::<T>::mutate(|total| {
                *total = total.saturating_add(amount);
            });
            let _ = Self::try_mutate_energy_balance(
                target,
                |current_energy_balance| -> Result<BalanceOf<T>, ()> {
                    Ok(current_energy_balance.saturating_add(amount))
                },
            );
        }

        /// Ensure that [account] can consume the given [amount] of energy, and returns current
        /// energy balance.
        fn ensure_can_consume_energy(
            target: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(
                Self::total_energy().checked_sub(&amount).is_some(),
                ArithmeticError::Underflow,
            );
            let energy_balance = Self::energy_balance(target);
            ensure!(energy_balance.checked_sub(&amount).is_some(), ArithmeticError::Underflow);
            Ok(())
        }

        /// Consume energy for [account]. Decreases energy balance by [amount] and also decrease
        /// account providers if current energy balance is below [T::ExistentialDeposit].
        fn consume_energy(target: &T::AccountId, amount: BalanceOf<T>) {
            TotalEnergy::<T>::mutate(|total| {
                *total = total.saturating_sub(amount);
            });
            let _ = Self::try_mutate_energy_balance(
                target,
                |current_energy_balance| -> Result<BalanceOf<T>, ()> {
                    Ok(current_energy_balance.saturating_sub(amount))
                },
            );
        }

        pub(crate) fn try_mutate_energy_balance<E>(
            who: &T::AccountId,
            mutator: impl FnOnce(BalanceOf<T>) -> sp_std::result::Result<BalanceOf<T>, E>,
        ) -> sp_std::result::Result<(), E> {
            EnergyBalance::<T>::try_mutate_exists(who, |maybe_energy_balance| {
                let existed = maybe_energy_balance.is_some();
                let current_energy_balance = maybe_energy_balance.unwrap_or_default();
                let new_energy_balance = mutator(current_energy_balance)?;

                let mut maybe_dust: Option<T::Balance> = None;

                *maybe_energy_balance = if new_energy_balance < T::ExistentialDeposit::get() {
                    // if ED is not zero, but account total is zero, account will be reaped
                    if new_energy_balance.is_zero() {
                        None
                    } else {
                        maybe_dust = Some(new_energy_balance);
                        Some(new_energy_balance)
                    }
                } else {
                    // Note: if ED is zero, account will never be reaped
                    Some(new_energy_balance)
                };

                Ok((existed, maybe_energy_balance.is_some(), maybe_dust))
            })
            .map(|(existed, exists, maybe_dust)| {
                if existed && !exists {
                    // If the account existed before, decrease the number of account providers.
                    // Ignore the result, because if it has failed then there are remaining
                    // consumers, and the account storage in frame_system shouldn't be reaped.
                    let _ = frame_system::Pallet::<T>::dec_providers(who);
                } else if !existed && exists {
                    // If the account is new, increase the number of account providers.
                    frame_system::Pallet::<T>::inc_providers(who);
                }

                if let Some(dust_amount) = maybe_dust {
                    // consume dust amount.
                    // TODO: maybe do something with dust amount?
                    Self::consume_energy(who, dust_amount);

                    Self::deposit_event(Event::DustLost {
                        account: who.clone(),
                        amount: dust_amount,
                    });
                }
            })
        }

        /// Calculate the value of energy that is equivalent to [amount] of native token.
        ///
        /// Example: If we need to pay 10 SUB, and coefficient is 1.25, then the amount of
        /// energy spent on fees will be: 10 / 1.25 = 8
        pub(crate) fn native_token_to_energy(amount: BalanceOf<T>) -> BalanceOf<T> {
            Self::value_coefficient()
                .reciprocal()
                .unwrap() // SAFETY: value_coefficient is always positive. we check for it.
                .saturating_mul_int(amount)
        }
    }

    /// Keeps track of how the user paid for the transaction.
    pub enum LiquidityInfo<T: Config> {
        /// Nothing have been paid.
        Nothing,
        /// Transaction have been paid using energy.
        Energy(BalanceOf<T>),
        /// Transaction have been paid using the native method.
        Native(<T::NativeOnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo),
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
            call: &T::RuntimeCall,
            dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
            fee: Self::Balance,
            tip: Self::Balance,
        ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
            if fee.is_zero() {
                return Ok(LiquidityInfo::Nothing)
            }

            let fee_without_tip = fee.saturating_sub(tip);
            let energy_fee = Self::native_token_to_energy(fee_without_tip);

            // if we don't have enough energy then fallback to paying with native token.
            if Self::energy_balance(&who) < energy_fee {
                return T::NativeOnChargeTransaction::withdraw_fee(
                    who,
                    call,
                    dispatch_info,
                    fee,
                    tip,
                )
                .map(|fallback_info| LiquidityInfo::Native(fallback_info))
            }

            if !tip.is_zero() {
                // TODO: maybe do something with tip?
                let _ = T::Currency::withdraw(
                    who,
                    tip,
                    WithdrawReasons::TIP,
                    ExistenceRequirement::KeepAlive,
                )
                .map_err(|_| -> InvalidTransaction { InvalidTransaction::Payment })?;
            }

            match Self::ensure_can_consume_energy(who, energy_fee) {
                Ok(()) => {
                    Self::consume_energy(who, energy_fee);
                    Ok(LiquidityInfo::Energy(energy_fee))
                },
                Err(_) => Err(InvalidTransaction::Payment.into()),
            }
        }

        fn correct_and_deposit_fee(
            who: &T::AccountId,
            dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
            post_info: &PostDispatchInfoOf<T::RuntimeCall>,
            corrected_fee: Self::Balance,
            tip: Self::Balance,
            already_withdrawn: Self::LiquidityInfo,
        ) -> Result<(), TransactionValidityError> {
            match already_withdrawn {
                LiquidityInfo::Nothing => Ok(()),
                LiquidityInfo::Native(fallback_info) =>
                    T::NativeOnChargeTransaction::correct_and_deposit_fee(
                        who,
                        dispatch_info,
                        post_info,
                        corrected_fee,
                        tip,
                        fallback_info,
                    ),
                LiquidityInfo::Energy(paid) => {
                    let corrected_fee_without_tip = corrected_fee.saturating_sub(tip);
                    let corrected_energy_fee =
                        Self::native_token_to_energy(corrected_fee_without_tip);

                    let refund_amount = paid.saturating_sub(corrected_energy_fee);

                    Self::capture_energy(who, refund_amount);

                    Ok(())
                },
            }
        }
    }
}
