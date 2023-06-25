// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchInfo, GetDispatchInfo, extract_actual_weight, PostDispatchInfo, Weight},
    pallet_prelude::{DispatchClass, Pays},
};
use pallet_transaction_payment::ChargeTransactionPayment;
use sp_runtime::{
    traits::{Dispatchable, SignedExtension},
    transaction_validity::{InvalidTransaction, TransactionValidityError},
    DispatchError, FixedI64, FixedPointNumber,
};

use pallet_energy::{Call as EnergyCall, EnergyBalance, Event as EnergyEvent};

use crate::{mock::*, Error, WeightInfo};

///// tests for Energy::update_value_coefficient()

#[test]
fn update_value_coefficient_should_fail_when_unsigned() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Energy::update_value_coefficient(RuntimeOrigin::none(), FixedI64::from_float(1.5),),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn update_value_coefficient_should_fail_when_caller_is_not_update_origin() {
    ExtBuilder::default().update_origin(1).build().execute_with(|| {
        let not_update_origin = 2;
        assert_noop!(
            Energy::update_value_coefficient(
                RuntimeOrigin::signed(not_update_origin),
                FixedI64::from_float(1.5),
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn update_value_coefficient_should_fail_when_new_ratio_is_negative() {
    let update_origin = account(1);
    ExtBuilder::default().update_origin(update_origin).build().execute_with(|| {
        assert_noop!(
            Energy::update_value_coefficient(
                RuntimeOrigin::signed(update_origin),
                FixedI64::from_float(-4.0),
            ),
            Error::<Test>::ValueCoefficientIsNotPositive,
        );
    });
}

#[test]
fn update_value_coefficient_should_work_as_expected() {
    let update_origin = account(1);
    ExtBuilder::default()
        .value_coefficient(987.654)
        .update_origin(update_origin)
        .build()
        .execute_with(|| {
            assert_eq!(
                Energy::value_coefficient(),
                FixedI64::from_float(987.654),
                "Default value coefficient should be 987.654"
            );

            let coff = 1.12354;
            assert_ok!(Energy::update_value_coefficient(
                RuntimeOrigin::signed(update_origin),
                FixedI64::from_float(coff),
            ),);

            assert_eq!(Energy::value_coefficient(), FixedI64::from_float(coff));

            System::assert_last_event(
                EnergyEvent::ValueCoefficientUpdated {
                    new_coefficient: FixedI64::from_float(coff),
                }
                .into(),
            );
        });
}

#[test]
fn update_value_coefficient_should_have_correct_weight() {
    let update_origin = account(1);
    ExtBuilder::default()
        .value_coefficient(1.25)
        .update_origin(update_origin)
        .build()
        .execute_with(|| {
            let call: RuntimeCall = EnergyCall::<Test>::update_value_coefficient {
                new_coefficient: FixedI64::from_float(1.5),
            }
            .into();

            let info = call.get_dispatch_info();
            let result = call.dispatch(RuntimeOrigin::signed(update_origin));

            assert_ok!(result);

            assert_eq!(
                extract_actual_weight(&result, &info),
                <Test as pallet_energy::Config>::WeightInfo::update_value_coefficient(),
            );
        });
}

///// tests for Energy::generate_energy()

#[test]
fn generate_energy_should_fail_when_unsigned() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(Energy::generate_energy(RuntimeOrigin::none(), 1, 10,), DispatchError::BadOrigin);
    });
}

#[test]
fn generate_energy_should_fail_when_caller_have_not_enough_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account_with_balance(1, 0);
        assert_noop!(
            Energy::generate_energy(RuntimeOrigin::signed(caller), 1, 10,),
            Error::<Test>::NotEnoughBalance,
        );
    });
}

#[test]
fn generate_energy_should_fail_when_energy_balance_below_existential_deposit() {
    ExtBuilder::default().energy_existential_deposit(100).build().execute_with(|| {
        let caller = account_with_balance(1, 1000);
        let receiver = account(10);

        assert_noop!(
            Energy::generate_energy(RuntimeOrigin::signed(caller), receiver, 10),
            Error::<Test>::BalanceBelowExistentialDeposit
        );

        assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(caller), receiver, 100));
    });
}

#[test]
fn generate_energy_should_work_when_energy_balance_equal_to_existential_deposit() {
    ExtBuilder::default().energy_existential_deposit(100).build().execute_with(|| {
        let caller = account_with_balance(1, 1000);
        let receiver = account(10);

        assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(caller), receiver, 100));

        assert_total_energy!(100);
        assert_energy_balance!(receiver, 100);
        assert_balance!(caller, 900);
    });
}

#[test]
fn generate_energy_should_work_when_caller_have_enough_balance() {
    ExtBuilder::default()
        .native_existential_deposit(0)
        .value_coefficient(10f64)
        .build()
        .execute_with(|| {
            let caller = account_with_balance(1, 100);
            let receiver = account(10);

            assert_balance!(caller, 100);
            assert_total_issuance!(100);
            assert_energy_balance!(receiver, 0);
            assert_total_energy!(0);

            assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(caller), receiver, 100,),);
            assert_balance!(caller, 0);
            assert_total_issuance!(0);
            assert_energy_balance!(receiver, 100);
            assert_total_energy!(100);

            System::assert_last_event(
                EnergyEvent::EnergyGenerated { generator: caller, receiver, balance_burned: 100 }
                    .into(),
            );
        });
}

#[test]
fn generate_energy_should_increase_total_energy() {
    ExtBuilder::default().value_coefficient(1.25).build().execute_with(|| {
        let caller = account_with_balance(1, 1000);
        let receiver1 = account(2);
        let receiver2 = account(3);

        assert_balance!(caller, 1000);
        assert_total_issuance!(1000);
        assert_energy_balance!(receiver1, 0);
        assert_energy_balance!(receiver2, 0);
        assert_total_energy!(0);

        assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(caller), receiver1, 35,),);
        assert_total_issuance!(965);
        assert_total_energy!(35);
        assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(caller), receiver1, 55,),);
        assert_total_issuance!(910);
        assert_total_energy!(90);
        assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(caller), receiver2, 200,),);

        assert_total_issuance!(710);
        assert_balance!(caller, 710);
        assert_energy_balance!(receiver1, 90);
        assert_energy_balance!(receiver2, 200);
        assert_total_energy!(290);
    });
}

#[test]
fn generate_energy_should_have_correct_weight() {
    ExtBuilder::default().value_coefficient(1.25).build().execute_with(|| {
        let caller = account_with_balance(1, 1000);
        let receiver = account(2);

        let call: RuntimeCall =
            EnergyCall::<Test>::generate_energy { target: receiver, burn_amount: 100 }.into();

        let info = call.get_dispatch_info();
        let result = call.dispatch(RuntimeOrigin::signed(caller));

        assert_ok!(result);

        assert_eq!(
            extract_actual_weight(&result, &info),
            <Test as pallet_energy::Config>::WeightInfo::generate_energy(),
        );
    });
}

///// tests for Energy::OnChargeTransaction

macro_rules! div_coeff {
    ($val:expr, $coff:expr) => {
        (($val as f64 / $coff as f64) as Balance)
    };
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum ChargeTransactionError {
    PreDispatchPayment,
    PostDispatchPayment,
    Other,
}

fn charge_transaction<PreValidator: FnOnce()>(
    caller: &AccountId,
    fee: Balance,
    actual_fee: Balance,
    tip: Balance,
    pre_validator: PreValidator,
) -> Result<(), ChargeTransactionError> {
    let call = frame_system::Call::<Test>::remark { remark: vec![] }.into();
    let info = DispatchInfo { weight: Weight::from_ref_time(fee), class: DispatchClass::Normal, pays_fee: Pays::Yes };
    let post_info = PostDispatchInfo { actual_weight: Some(Weight::from_ref_time(actual_fee)), pays_fee: Pays::Yes };

    let pre = ChargeTransactionPayment::<Test>::from(tip)
        .pre_dispatch(caller, &call, &info, 0)
        .map_err(|err| {
            if let TransactionValidityError::Invalid(InvalidTransaction::Payment) = err {
                ChargeTransactionError::PreDispatchPayment
            } else {
                ChargeTransactionError::Other
            }
        })?;

    pre_validator();

    ChargeTransactionPayment::<Test>::post_dispatch(Some(pre), &info, &post_info, 0, &Ok(()))
        .map_err(|err| {
            if let TransactionValidityError::Invalid(InvalidTransaction::Payment) = err {
                ChargeTransactionError::PostDispatchPayment
            } else {
                ChargeTransactionError::Other
            }
        })?;

    Ok(())
}

#[test]
fn charge_transaction_should_fail_when_no_energy_and_no_native_balance() {
    ExtBuilder::default().value_coefficient(1.25).build().execute_with(|| {
        let caller = account(1);
        set_native_balance(caller, 0);
        set_energy_balance(caller, 0);

        assert_eq!(
            charge_transaction(&caller, 100, 100, 0, || panic!(
                "should not be called, because there was a pre_dispatch error"
            ),)
            .unwrap_err(),
            ChargeTransactionError::PreDispatchPayment,
        );

        assert!(get_corrected_and_deposit_fee_args().is_none());
    });
}

#[test]
fn charge_transaction_should_pay_with_energy_if_enough() {
    ExtBuilder::default().value_coefficient(2f64).build().execute_with(|| {
        let caller = account(1);
        set_native_balance(caller, 1000);
        set_energy_balance(caller, 1000);

        assert_ok!(charge_transaction(&caller, 150, 100, 20, || {
            assert_energy_balance!(caller, 1000 - div_coeff!(150, 2)); // subtract the expected fees / coefficient
            assert_balance!(caller, 1000 - 20); // tip subtracted from the native balance
            assert!(
                get_captured_withdraw_fee_args().is_none(),
                "Shouldn't go through the fallback OnChargeTransaction"
            );
        },),);
        assert_energy_balance!(caller, 1000 - div_coeff!(100, 2));
        // subtract the actual (fees + tip) / coefficient
        assert_balance!(caller, 1000 - 20); // tip subtracted from the native balance
        assert!(
            get_corrected_and_deposit_fee_args().is_none(),
            "Shouldn't go through the fallback OnChargeTransaction"
        );
    });
}

#[test]
fn charge_transaction_should_fail_when_no_native_balance_to_pay_tip() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account(1);
        set_native_balance(caller, 10);
        set_energy_balance(caller, 1000);

        assert_eq!(
            charge_transaction(&caller, 150, 100, 20, || panic!(
                "should not be called, because there was a pre_dispatch error"
            ))
            .unwrap_err(),
            ChargeTransactionError::PreDispatchPayment
        );
        assert_energy_balance!(caller, 1000);
        assert_balance!(caller, 10);
        assert!(
            get_corrected_and_deposit_fee_args().is_none(),
            "Shouldn't go through the fallback OnChargeTransaction"
        );
    });
}

#[test]
fn charge_transaction_should_pay_nothing_if_fee_is_zero() {
    ExtBuilder::default().value_coefficient(10f64).build().execute_with(|| {
        let caller = account(1);
        set_native_balance(caller, 1000);
        set_energy_balance(caller, 1000);

        assert_ok!(charge_transaction(&caller, 0, 0, 0, || {
            assert_energy_balance!(caller, 1000); // no change
            assert_balance!(caller, 1000); // no change
            assert!(
                get_captured_withdraw_fee_args().is_none(),
                "Shouldn't go through the fallback OnChargeTransaction"
            );
        },),);
        assert_energy_balance!(caller, 1000);
        // no change
        assert_balance!(caller, 1000); // no change
        assert!(
            get_corrected_and_deposit_fee_args().is_none(),
            "Shouldn't go through the fallback OnChargeTransaction"
        );
    });
}

#[test]
fn charge_transaction_should_pay_with_native_tokens_if_not_enough_energy() {
    ExtBuilder::default().value_coefficient(3.36f64).build().execute_with(|| {
        let caller = account(1);
        set_native_balance(caller, 1000);
        set_energy_balance(caller, 50);

        assert_ok!(charge_transaction(&caller, 200, 50, 13, || {
            assert_energy_balance!(caller, 50); // no change
            assert_balance!(caller, 1000 - 200 - 13); // subtract the expected fees + tip
            assert_eq!(
                get_captured_withdraw_fee_args().unwrap(),
                WithdrawFeeArgs { who: caller, fee_with_tip: 200 + 13, tip: 13 }
            );
        },),);
        assert_energy_balance!(caller, 50);
        // no change
        assert_balance!(caller, 1000 - 50 - 13); // subtract the actual fees + tip
        assert!(matches!(
            get_corrected_and_deposit_fee_args().unwrap(),
            CorrectAndDepositFeeArgs {
                who: _caller,
                corrected_fee_with_tip: 63, // 50 + 13
                already_withdrawn: _,       // ignored
            }
        ));
    });
}

#[test]
fn update_value_coefficient_should_reflect_on_future_charge_transactions() {
    let update_origin = account(1);

    ExtBuilder::default()
        .value_coefficient(1.25)
        .update_origin(update_origin)
        .energy_existential_deposit(10)
        .build()
        .execute_with(|| {
            let caller = account(1);

            let charge_transaction = |val| {
                assert_ok!(charge_transaction(&caller, val, val, 0, || {},),);
            };

            assert_eq!(
                <Test as pallet_energy::Config>::DefaultValueCoefficient::get().to_float(),
                1.25,
                "Default value coefficient should be 1.25",
            );

            assert_eq!(
                Energy::value_coefficient().to_float(),
                1.25,
                "Stored value coefficient should be 1.25",
            );

            set_energy_balance(caller, 1_000_000);
            charge_transaction(100);

            // 100 fee in native token at coefficient 1.25 equals 80 energy
            assert_energy_balance!(caller, 1_000_000 - 80);

            assert_ok!(Energy::update_value_coefficient(
                RuntimeOrigin::signed(update_origin),
                FixedI64::checked_from_rational(50, 100).unwrap(), // 50%
            ),);

            assert_eq!(
                Energy::value_coefficient().to_float(),
                0.5,
                "Stored value coefficient should be 0.5",
            );

            set_energy_balance(caller, 1_000_000);
            charge_transaction(150);

            assert_energy_balance!(caller, 1_000_000 - 300);

            assert_ok!(Energy::update_value_coefficient(
                RuntimeOrigin::signed(update_origin),
                FixedI64::checked_from_rational(12_345, 10_000).unwrap(), // 123.45%
            ),);

            assert_eq!(
                Energy::value_coefficient().to_float(),
                1.2345,
                "Stored value coefficient should be 1.2345",
            );

            set_energy_balance(caller, 1_000_000);
            charge_transaction(700_000);

            assert_energy_balance!(caller, 1_000_000 - 567_031);

            assert_ok!(Energy::update_value_coefficient(
                RuntimeOrigin::signed(update_origin),
                FixedI64::checked_from_rational(333_333_334, 1_000_000_000).unwrap(), // 33.3333334%
            ),);

            assert_eq!(
                Energy::value_coefficient().to_float(),
                0.333333334,
                "Stored value coefficient should be 0.333333334",
            );

            set_energy_balance(caller, 2_000_000);
            charge_transaction(406_950);

            assert_energy_balance!(caller, 2_000_000 - 1_220_849);
        });
}

///// tests for existential deposit and providers

#[test]
fn existential_deposit_and_providers() {
    ExtBuilder::default()
        .native_existential_deposit(10)
        .energy_existential_deposit(100)
        .build()
        .execute_with(|| {
            let treasury = account(0);
            set_native_balance(treasury, 1_000_000_000);
            set_energy_balance(treasury, 1_000_000_000);

            let account1 = account(1);
            assert_eq!(System::providers(&account1), 0);

            assert_ok!(pallet_balances::Pallet::<Test>::transfer(
                RuntimeOrigin::signed(treasury),
                account1,
                10_000
            ));
            assert_eq!(System::providers(&account1), 1);

            assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(account1), account1, 100));
            assert_eq!(System::providers(&account1), 2);

            assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(treasury), account1, 90));
            assert_eq!(System::providers(&account1), 2);

            assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(treasury), account1, 1000));
            assert_eq!(System::providers(&account1), 2);

            // Now Account 1 has 1190 energy

            assert_ok!(charge_transaction(&account1, 90, 90, 0, || {}));
            assert_eq!(System::providers(&account1), 2);

            assert_ok!(charge_transaction(&account1, 550, 550, 0, || {}));
            assert_eq!(System::providers(&account1), 2);

            assert_ok!(charge_transaction(&account1, 500, 500, 0, || {}));

            // Now Account 1 has 50 energy. This is less then the existential deposit (100 energy).
            // That's why we expect that these 50 energy to be burned.
            assert_energy_balance!(account1, 0);

            // And the number of providers should be decremented by 1.
            assert_eq!(System::providers(&account1), 1);

            // Here 10 native tokens will be used to pay tx fees,
            // because we lost dust (50 energy) on the previous steps of this test.
            assert_ok!(charge_transaction(&account1, 10, 10, 0, || {}));
            assert_eq!(System::providers(&account1), 1);

            assert_ok!(Energy::generate_energy(RuntimeOrigin::signed(account1), account1, 900));

            assert_balance!(account1, 8990);
            assert_energy_balance!(account1, 900);
            assert_eq!(System::providers(&account1), 2);

            assert_ok!(pallet_balances::Pallet::<Test>::transfer_all(
                RuntimeOrigin::signed(account1),
                treasury,
                false
            ));

            assert_balance!(account1, 0);
            assert_eq!(System::providers(&account1), 1);

            assert_ok!(charge_transaction(&account1, 850, 850, 0, || {},),);

            System::assert_has_event(
                EnergyEvent::DustLost { account: account1, amount: 50 }.into(),
            );

            assert_energy_balance!(account1, 0); // the remaining 50 energy is burned
            assert_eq!(System::providers(&account1), 0);
        });
}

// TODO Add a test for DustLost event

///// test native_token_to_energy

#[test]
fn native_token_to_energy() {
    ExtBuilder::default().value_coefficient(1.25).build().execute_with(|| {
        assert_eq!(pallet_energy::Pallet::<Test>::native_token_to_energy(100), 80);
    });

    ExtBuilder::default().value_coefficient(1.5).build().execute_with(|| {
        assert_eq!(pallet_energy::Pallet::<Test>::native_token_to_energy(200), 133);
    });

    ExtBuilder::default().value_coefficient(10.0).build().execute_with(|| {
        assert_eq!(pallet_energy::Pallet::<Test>::native_token_to_energy(500), 50);
    });

    ExtBuilder::default().value_coefficient(0.5).build().execute_with(|| {
        assert_eq!(pallet_energy::Pallet::<Test>::native_token_to_energy(33), 66);
    });

    ExtBuilder::default().value_coefficient(0.1).build().execute_with(|| {
        assert_eq!(pallet_energy::Pallet::<Test>::native_token_to_energy(33), 330);
    });
}
