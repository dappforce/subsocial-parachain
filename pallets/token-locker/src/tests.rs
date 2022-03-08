use crate::{mock::*, Error, Config, Pallet, LockDetails, Event, PALLET_ID, UnlockAt};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::{Currency, Len, LockableCurrency, WithdrawReasons};
use pallet_balances::BalanceLock;

fn account_with_balance(id: u64, balance: Balance) -> AccountId {
    let account = account(id);
    let _ = <Test as Config>::Currency::make_free_balance_be(&account, balance);
    account
}

fn account(id: u64) -> AccountId {
    id
}

#[ignore]
#[test]
fn dummy_test() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            assert_eq!(1 + 1, 2);
        });
}

#[test]
fn cannot_lock_under_min_lock_amount() {
    ExtBuilder::default()
        .min_lock_amount(10_000)
        .max_lock_amount(100_000)
        .build()
        .execute_with(|| {
            let caller = account_with_balance(0, 5000);

            assert_noop!(
                <Pallet<Test>>::lock_sub(
                    Origin::signed(caller),
                    1_000,
                    caller,
                ),
                Error::<Test>::LockAmountLowerThanMinLock
            );
        });
}

#[test]
fn cannot_lock_above_max_lock_amount() {
    ExtBuilder::default()
        .min_lock_amount(10)
        .max_lock_amount(100)
        .build()
        .execute_with(|| {
            let caller = account_with_balance(0, 5000);


            assert_noop!(
                <Pallet<Test>>::lock_sub(
                    Origin::signed(caller),
                    500,
                    caller,
                ),
                Error::<Test>::LockAmountGreaterThanMaxLock,
            );
        });
}


#[test]
fn cannot_lock_if_already_locked() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let caller = account_with_balance(0, 5000);

            // simulate that account have already locked
            LockDetails::<Test>::insert(caller, 1);

            assert_noop!(
                <Pallet<Test>>::lock_sub(
                    Origin::signed(caller),
                    500,
                    caller,
                ),
                Error::<Test>::AlreadyLocked,
            );
        });
}


#[test]
fn cannot_lock_if_not_enough_balance() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let caller = account_with_balance(0, 10);

            assert_noop!(
                <Pallet<Test>>::lock_sub(
                    Origin::signed(caller),
                    500,
                    caller,
                ),
                Error::<Test>::BalanceIsTooLowToLock,
            );
        });
}


#[test]
fn lock_details_is_added_and_event_is_emitted_and_tokens_locked_when_everything_is_okay() {
    ExtBuilder::default()
        .min_lock_amount(50)
        .max_lock_amount(200)
        .build()
        .execute_with(|| {
            let caller = account_with_balance(0, 150);
            let target = 15;
            let amount_to_lock = 123;

            let res = <Pallet<Test>>::lock_sub(
                Origin::signed(caller),
                amount_to_lock,
                target,
            );

            // the request passes.
            assert_ok!(res);

            // storage is mutated
            assert!(matches!(
                LockDetails::<Test>::get(caller),
                Some(amount_to_lock),
            ));

            // event is emitted
            <frame_system::Pallet<Test>>::assert_last_event(Event::<Test>::SubLocked(
                caller,
                amount_to_lock,
                target,
            ).into());

            // check that there is a lock for the caller
            let mut locks = pallet_balances::Locks::<Test>::get(caller).to_vec();
            assert_eq!(locks.len(), 1);
            let lock = locks.pop().unwrap();
            assert_eq!(
                lock,
                pallet_balances::BalanceLock::<Balance> {
                    id: PALLET_ID,
                    amount: amount_to_lock,
                    reasons: WithdrawReasons::empty().into(),
                }
            );
        });
}

#[test]
fn request_unlock_will_fail_when_no_there_is_no_locks() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            // no lock details
            assert_eq!(LockDetails::<Test>::iter().count(), 0);

            let caller = account(0);

            assert_noop!(
                Pallet::<Test>::request_unlock(Origin::signed(caller)),
                Error::<Test>::NotLocked,
            );
        });
}


#[test]
fn request_unlock_will_fail_when_already_requested() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let caller = account_with_balance(0, 2000);

            // lock subs
            assert_ok!(Pallet::<Test>::lock_sub(Origin::signed(caller), 1000, caller));

            // place an unlock request
            assert_ok!(Pallet::<Test>::request_unlock(Origin::signed(caller)));

            // requesting again should fail
            assert_noop!(
                Pallet::<Test>::request_unlock(Origin::signed(caller)),
                Error::<Test>::UnlockAlreadyRequested,
            );
        });
}


#[test]
fn request_unlock_will_pass_if_lock_is_placed_and_not_requested_before() {
    const UNLOCK_PERIOD: BlockNumber = 125;
    const CURRENT_BLOCK_NUMBER: BlockNumber = 65486;

    ExtBuilder::default()
        .unlock_period(UNLOCK_PERIOD)
        .build()
        .execute_with(|| {
            // nothing is stored at first
            assert_eq!(UnlockAt::<Test>::iter().count(), 0);

            let caller = account_with_balance(0, 2000);

            <frame_system::Pallet<Test>>::set_block_number(CURRENT_BLOCK_NUMBER);

            // lock subs
            assert_ok!(Pallet::<Test>::lock_sub(Origin::signed(caller), 1000, caller));

            // place an unlock request
            assert_ok!(Pallet::<Test>::request_unlock(Origin::signed(caller)));

            assert_eq!(UnlockAt::<Test>::get(caller), Some(CURRENT_BLOCK_NUMBER + UNLOCK_PERIOD));

            <frame_system::Pallet<Test>>::assert_last_event(Event::<Test>::RefundRequested(caller).into());
        });
}

#[test]
fn try_refund_will_fail_if_not_locked() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let caller = account(25);

            // nothing is locked
            assert_eq!(LockDetails::<Test>::iter().count(), 0);

            assert_noop!(
                Pallet::<Test>::try_refund(Origin::signed(caller)),
                Error::<Test>::NotLocked,
            );
        });
}

#[test]
fn try_refund_will_fail_if_not_requested() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let caller = account_with_balance(25, 1000);

            // lock sub
            assert_ok!(Pallet::<Test>::lock_sub(Origin::signed(caller), 100, caller));

            assert_noop!(
                Pallet::<Test>::try_refund(Origin::signed(caller)),
                Error::<Test>::UnlockNotRequested,
            );
        });
}


#[test]
fn try_refund_will_fail_if_unlock_period_didnot_pass() {
    ExtBuilder::default()
        .unlock_period(100)
        .build()
        .execute_with(|| {
            let caller = account_with_balance(25, 1000);

            <frame_system::Pallet<Test>>::set_block_number(10);

            // lock sub
            assert_ok!(Pallet::<Test>::lock_sub(Origin::signed(caller), 100, caller));

            // request to unlock funds
            assert_ok!(Pallet::<Test>::request_unlock(Origin::signed(caller)));

            // trying immediately after requesting
            assert_noop!(
                Pallet::<Test>::try_refund(Origin::signed(caller)),
                Error::<Test>::TooEarlyToRefund,
            );

            // after 1 block
            <frame_system::Pallet<Test>>::set_block_number(11);
            assert_noop!(
                Pallet::<Test>::try_refund(Origin::signed(caller)),
                Error::<Test>::TooEarlyToRefund,
            );


            // before unlock time by one block
            <frame_system::Pallet<Test>>::set_block_number(109);
            assert_noop!(
                Pallet::<Test>::try_refund(Origin::signed(caller)),
                Error::<Test>::TooEarlyToRefund,
            );

            // just in time
            <frame_system::Pallet<Test>>::set_block_number(110);
            assert_ok!(
                Pallet::<Test>::try_refund(Origin::signed(caller)),
            );
        });
}


#[test]
fn try_refund_will_unlock_funds_and_emit_event() {
    ExtBuilder::default()
        .unlock_period(50)
        .build()
        .execute_with(|| {
            let caller = account_with_balance(25, 1000);

            let amount_to_lock = 325;

            <frame_system::Pallet<Test>>::set_block_number(1_000);

            // lock sub
            assert_ok!(Pallet::<Test>::lock_sub(Origin::signed(caller), amount_to_lock, caller));

            // request to unlock funds
            assert_ok!(Pallet::<Test>::request_unlock(Origin::signed(caller)));

            // 1000 block later
            <frame_system::Pallet<Test>>::set_block_number(2_000);


            // make sure everything is set correctly
            assert_eq!(
                pallet_balances::Locks::<Test>::get(caller).to_vec().pop().unwrap(),
                BalanceLock::<Balance> {
                    id: PALLET_ID,
                    amount: amount_to_lock,
                    reasons: WithdrawReasons::FEE.into(),
                },
                "Lock is placed",
            );
            assert_eq!(
                LockDetails::<Test>::get(caller).unwrap(),
                amount_to_lock,
            );
            assert_eq!(
                UnlockAt::<Test>::get(caller).unwrap(),
                1_050,
            );

            // do the transaction
            assert_ok!(Pallet::<Test>::try_refund(Origin::signed(caller)));

            // correct event is emitted
            <frame_system::Pallet<Test>>::assert_last_event(Event::<Test>::SubRefunded(caller, amount_to_lock, ).into());

            // make sure everything is cleared
            assert!(pallet_balances::Locks::<Test>::get(caller).is_empty());
            assert!(LockDetails::<Test>::get(caller).is_none());
            assert!(UnlockAt::<Test>::get(caller).is_none());
        });
}