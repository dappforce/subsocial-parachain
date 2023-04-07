use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, Get},
};

use crate::{mock::*, Error};

fn account_with_balance(id: AccountId, balance: Balance) -> AccountId {
    let account = account(id);
    let _ = Balances::make_free_balance_be(&account, balance);
    account
}

fn account(id: AccountId) -> AccountId {
    id
}

fn proxy_deposit(num: Balance) -> Balance {
    <Test as pallet_proxy::Config>::ProxyDepositBase::get() +
        <Test as pallet_proxy::Config>::ProxyDepositFactor::get() * num
}

#[test]
fn add_free_proxy_should_fail_if_not_first_proxy() {
    ExtBuilder::default()
        .deposit_factor(1)
        .deposit_base(10)
        .build()
        .execute_with(|| {
            let delegator = account_with_balance(1, 100);
            let proxy1 = account(2);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            assert_ok!(Proxy::add_proxy(RuntimeOrigin::signed(delegator), proxy1, (), 0,));

            assert_eq!(Balances::reserved_balance(delegator), proxy_deposit(1));

            let proxy2 = account(3);
            assert_noop!(
                FreeProxy::add_free_proxy(RuntimeOrigin::signed(delegator), proxy2, (), 0),
                Error::<Test>::OnlyFirstProxyCanBeFree
            );
        });
}

#[test]
fn add_free_proxy_reserve_nothing() {
    ExtBuilder::default()
        .deposit_factor(1)
        .deposit_base(10)
        .build()
        .execute_with(|| {
            let delegator = account_with_balance(1, 100);
            let proxy1 = account(2);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            assert_ok!(FreeProxy::add_free_proxy(
                RuntimeOrigin::signed(delegator),
                proxy1,
                (),
                0
            ),);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            let proxy2 = account(3);
            assert_ok!(Proxy::add_proxy(RuntimeOrigin::signed(delegator), proxy2, (), 0,));

            assert_eq!(Balances::reserved_balance(delegator), proxy_deposit(2));
        });
}

#[test]
fn remove_free_proxy_should_unreserve_nothing_if_there_are_no_other_proxies() {
    ExtBuilder::default()
        .deposit_factor(1)
        .deposit_base(10)
        .build()
        .execute_with(|| {
            let delegator = account_with_balance(1, 100);
            let proxy1 = account(2);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            assert_ok!(FreeProxy::add_free_proxy(
                RuntimeOrigin::signed(delegator),
                proxy1,
                (),
                0
            ),);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            assert_ok!(Proxy::remove_proxy(RuntimeOrigin::signed(delegator), proxy1, (), 0));

            assert_eq!(Balances::reserved_balance(delegator), 0);

            ///////////
            let proxy2 = account(3);
            assert_ok!(Proxy::add_proxy(RuntimeOrigin::signed(delegator), proxy2, (), 0,));
            assert_eq!(Balances::reserved_balance(delegator), proxy_deposit(1));

            assert_ok!(Proxy::remove_proxy(RuntimeOrigin::signed(delegator), proxy2, (), 0));
            assert_eq!(Balances::reserved_balance(delegator), 0);
        });
}

#[test]
fn remove_free_proxy_should_unreserve_one_proxy_deposit_when_user_have_two_proxies() {
    ExtBuilder::default()
        .deposit_factor(1)
        .deposit_base(10)
        .build()
        .execute_with(|| {
            let delegator = account_with_balance(1, 100);
            let proxy1 = account(2);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            assert_ok!(FreeProxy::add_free_proxy(
                RuntimeOrigin::signed(delegator),
                proxy1,
                (),
                0
            ),);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            let proxy2 = account(3);
            assert_ok!(Proxy::add_proxy(RuntimeOrigin::signed(delegator), proxy2, (), 0,));

            assert_eq!(Balances::reserved_balance(delegator), proxy_deposit(2));

            assert_ok!(Proxy::remove_proxy(RuntimeOrigin::signed(delegator), proxy1, (), 0));

            assert_eq!(Balances::reserved_balance(delegator), proxy_deposit(1));

            assert_ok!(Proxy::remove_proxy(RuntimeOrigin::signed(delegator), proxy2, (), 0));

            assert_eq!(Balances::reserved_balance(delegator), 0);
        });
}

#[test]
fn remove_paid_proxy_should_unreserve_one_proxy_deposit() {
    ExtBuilder::default()
        .deposit_factor(1)
        .deposit_base(10)
        .build()
        .execute_with(|| {
            let delegator = account_with_balance(1, 100);
            let proxy1 = account(2);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            assert_ok!(FreeProxy::add_free_proxy(
                RuntimeOrigin::signed(delegator),
                proxy1,
                (),
                0
            ),);

            assert_eq!(Balances::reserved_balance(delegator), 0);

            let proxy2 = account(3);
            assert_ok!(Proxy::add_proxy(RuntimeOrigin::signed(delegator), proxy2, (), 0,));

            assert_eq!(Balances::reserved_balance(delegator), proxy_deposit(2));

            assert_ok!(Proxy::remove_proxy(RuntimeOrigin::signed(delegator), proxy2, (), 0));

            assert_eq!(Balances::reserved_balance(delegator), proxy_deposit(1));

            assert_ok!(Proxy::remove_proxy(RuntimeOrigin::signed(delegator), proxy1, (), 0));

            assert_eq!(Balances::reserved_balance(delegator), 0);
        });
}
