use frame_support::{assert_noop, assert_ok, pallet};
use sp_runtime::DispatchError;
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
        assert_eq!(Energy::available_energy(receiver), 0);
        assert_eq!(Energy::total_energy(), 0);

        assert_ok!(
            Energy::generate_energy(
                Origin::signed(caller),
                receiver,
                100,
            ),
        );

        assert_eq!(Balances::free_balance(caller), 0);
        assert_eq!(Energy::available_energy(receiver), 100);
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
        assert_eq!(Energy::available_energy(receiver1), 0);
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
        assert_eq!(Energy::available_energy(receiver1), 80);
        assert_eq!(Energy::available_energy(receiver2), 20);
        assert_eq!(Energy::total_energy(), 100);
    });
}