use frame_support::{assert_noop, assert_ok, pallet};
use frame_support::dispatch::DispatchInfo;
use frame_support::pallet_prelude::{DispatchClass, Pays};
use frame_support::weights::PostDispatchInfo;
use pallet_transaction_payment::{ChargeTransactionPayment, OnChargeTransaction};
use sp_runtime::DispatchError;
use sp_runtime::traits::SignedExtension;
use crate::Error;
use crate::mock::*;

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
    ExtBuilder::default().build().execute_with(|| {
        let caller = account_with_balance(1, 100);
        let receiver = account(10);

        assert_eq!(Balances::free_balance(caller), 100);
        assert_eq!(Energy::energy_balance(receiver), 0);
        assert_eq!(Energy::total_energy(), 0);

        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver,
                100,
            ),
        );

        assert_eq!(Balances::free_balance(caller), 0);
        assert_eq!(Energy::energy_balance(receiver), 100);
        assert_eq!(Energy::total_energy(), 100);

        System::assert_last_event(EnergyEvent::EnergyGenerated {
            generator: caller,
            receiver,
            burnt_balance: 100,
            generated_energy: 100,
        }.into());
    });
}


#[test]
fn test_generate_energy_will_increment_total_energy() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account_with_balance(
            1, 1000,
        );
        let receiver1 = account(2);
        let receiver2 = account(3);

        assert_eq!(Balances::free_balance(caller), 1000);
        assert_eq!(Energy::energy_balance(receiver1), 0);
        assert_eq!(Energy::total_energy(), 0);

        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver1,
                30,
            ),
        );
        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver1,
                50,
            ),
        );
        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver2,
                20,
            ),
        );

        assert_eq!(Balances::free_balance(caller), 900);
        assert_eq!(Energy::energy_balance(receiver1), 80);
        assert_eq!(Energy::energy_balance(receiver2), 20);
        assert_eq!(Energy::total_energy(), 100);
    });
}


///// tests for Energy::OnChargeTransaction

fn charge_transaction(
    caller: &AccountId,
    fee: Balance,
    actual_fee: Balance,
    tip: Balance,
    pre_validator: fn(),
) {
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
        ).expect("ChargeTransactionPayment pre_dispatch failed");

    pre_validator();

    ChargeTransactionPayment::<Test>::post_dispatch(
        Some(pre),
        &info,
        &post_info,
        0,
        &Ok(()),
    ).expect("ChargeTransactionPayment post_dispatch failed");
}

#[test]
#[should_panic(expected = "ChargeTransactionPayment pre_dispatch failed: TransactionValidityError::Invalid(InvalidTransaction::Payment)")]
fn test_charge_transaction_should_fail_when_no_energy_and_no_sub() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account(1);
        set_sub_balance(caller, 0);
        set_energy_balance(caller, 0);

        charge_transaction(
            &caller,
            100,
            100,
            0,
            || {},
        );

        panic!("should panic before this line");
    });
}