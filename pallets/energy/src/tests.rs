use frame_support::{assert_noop, assert_ok, pallet};
use frame_support::dispatch::DispatchInfo;
use frame_support::pallet_prelude::{DispatchClass, Pays};
use frame_support::weights::PostDispatchInfo;
use pallet_transaction_payment::{ChargeTransactionPayment, OnChargeTransaction};
use sp_runtime::DispatchError;
use sp_runtime::traits::SignedExtension;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};
use crate::Error;
use crate::mock::*;

use pallet_energy::EnergyBalance;
use pallet_energy::Event as EnergyEvent;

#[test]
fn test_generate_energy_will_fail_when_unsigned() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Energy::generate_energy(
                Origin::none(), 
                1,
                10,
            ),
            DispatchError::BadOrigin
        );
    });
}


#[test]
fn test_generate_energy_will_fail_when_caller_have_not_enough_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account_with_balance(1, 0);
        assert_noop!(
            Energy::generate_energy(
                Origin::signed(caller),
                1,
                10,
            ),
            Error::<Test>::NotEnoughBalance,
        );
    });
}

#[test]
fn test_generate_energy_will_work_when_caller_have_enough_balance() {
    ExtBuilder::default()
        .conversion_ratio(10f64)
        .build().execute_with(|| {
        let caller = account_with_balance(1, 100);
        let receiver = account(10);

        assert_balance!(caller, 100);
        assert_total_issuance!(100);
        assert_energy_balance!(receiver, 0);
        assert_total_energy!(0);

        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver,
                100,
            ),
        );
        assert_balance!(caller, 0);
        assert_total_issuance!(0);
        assert_energy_balance!(receiver, 1000);
        assert_total_energy!(1000);

        System::assert_last_event(EnergyEvent::EnergyGenerated {
            generator: caller,
            receiver,
            burnt_balance: 100,
            generated_energy: 1000,
        }.into());
    });
}


#[test]
fn test_generate_energy_will_increment_total_energy() {
    ExtBuilder::default()
        .conversion_ratio(1.25)
        .build().execute_with(|| {
        let caller = account_with_balance(
            1, 1000,
        );
        let receiver1 = account(2);
        let receiver2 = account(3);

        assert_balance!(caller, 1000);
        assert_total_issuance!(1000);
        assert_energy_balance!(receiver1, 0);
        assert_energy_balance!(receiver2, 0);
        assert_total_energy!(0);

        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver1,
                35,
            ),
        );
        assert_total_issuance!(965);
        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver1,
                55,
            ),
        );
        assert_total_issuance!(910);
        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver2,
                200,
            ),
        );

        assert_total_issuance!(710);
        assert_balance!(caller, 710);
        // 35 * 1.25 = 43.75, 55 * 1.25 = 68.75
        // 43 + 68 = 111
        assert_energy_balance!(receiver1, 111);
        assert_energy_balance!(receiver2, 250);
        // 200 * 1.25 = 250
        assert_total_energy!(361);
        dbg!()
    });
}


///// tests for Energy::OnChargeTransaction

#[derive(Clone, PartialEq, Eq, Debug)]
enum ChargeTransactionError {
    PreDispatch_Payment,
    PostDispatch_Payment,
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
    let info = DispatchInfo {
        weight: fee,
        class: DispatchClass::Normal,
        pays_fee: Pays::Yes,
    };
    let post_info = PostDispatchInfo {
        actual_weight: Some(actual_fee),
        pays_fee: Pays::Yes,
    };

    let pre = ChargeTransactionPayment::<Test>::from(tip)
        .pre_dispatch(
            caller,
            &call,
            &info,
            0,
        ).map_err(|err| {
        if let TransactionValidityError::Invalid(InvalidTransaction::Payment) = err {
            ChargeTransactionError::PreDispatch_Payment
        } else {
            ChargeTransactionError::Other
        }
    })?;

    pre_validator();

    ChargeTransactionPayment::<Test>::post_dispatch(
        Some(pre),
        &info,
        &post_info,
        0,
        &Ok(()),
    ).map_err(|err| {
        if let TransactionValidityError::Invalid(InvalidTransaction::Payment) = err {
            ChargeTransactionError::PostDispatch_Payment
        } else {
            ChargeTransactionError::Other
        }
    })?;

    Ok(())
}

#[test]
fn test_charge_transaction_should_fail_when_no_energy_and_no_sub() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account(1);
        set_sub_balance(caller, 0);
        set_energy_balance(caller, 0);

        assert_eq!(
            charge_transaction(
                &caller,
                100,
                100,
                0,
                || {
                    assert_eq!(get_captured_withdraw_fee_args().unwrap(), WithdrawFeeArgs {
                        who: caller.clone(),
                        fee_with_tip: 100,
                        tip: 0,
                    });
                },
            ).unwrap_err(),
            ChargeTransactionError::PreDispatch_Payment,
        );

        assert!(get_corrected_and_deposit_fee_args().is_none());
    });
}

#[test]
fn test_charge_transaction_should_pay_with_energy_if_enough() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account(1);
        set_sub_balance(caller, 1000);
        set_energy_balance(caller, 1000);

        assert_ok!(
            charge_transaction(
                &caller,
                150,
                100,
                20,
                ||  {
                    assert_energy_balance!(caller, 1000 - 150 - 20); // subtract the expected fees + tip
                    assert_balance!(caller, 1000); // no change
                    assert!(get_captured_withdraw_fee_args().is_none(), "Shouldn't go through the fallback OnChargeTransaction");
                },
            ),
        );
        assert_energy_balance!(caller, 1000 - 100 - 20); // subtract the actual fees + tip
        assert_balance!(caller, 1000); // no change
        assert!(get_corrected_and_deposit_fee_args().is_none(), "Shouldn't go through the fallback OnChargeTransaction");
    });
}


#[test]
fn test_charge_transaction_should_pay_with_sub_if_energy_no_enough() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account(1);
        set_sub_balance(caller, 1000);
        set_energy_balance(caller, 100);

        assert_ok!(
            charge_transaction(
                &caller,
                200,
                50,
                13,
                ||  {
                    assert_energy_balance!(caller, 100); // no change
                    assert_balance!(caller, 1000 - 200 - 13); // subtract the expected fees + tip
                    assert_eq!(get_captured_withdraw_fee_args().unwrap(), WithdrawFeeArgs {
                        who: caller.clone(),
                        fee_with_tip: 200 + 13,
                        tip: 13,
                    });
                },
            ),
        );
        assert_energy_balance!(caller, 100); // no change
        assert_balance!(caller, 1000 - 50 - 13); // subtract the actual fees + tip
        assert!(matches!(get_corrected_and_deposit_fee_args().unwrap(), CorrectAndDepositFeeArgs {
            who: caller,
            corrected_fee_with_tip: 63, // 50 + 13
            already_withdrawn: _, // ignored
        }));
    });
}