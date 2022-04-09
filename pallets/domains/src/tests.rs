use crate::mock::*;
use frame_support::assert_ok;
use crate::Domain;

// `register_domain` tests

#[test]
fn register_domain_should_work() {
    ExtBuilder::build_with_tld().execute_with(|| {
        assert_ok!(_register_default_domain());

        let domain_lc = default_domain_lc();
        let Domain { tld, domain } = domain_lc.clone();

        assert_eq!(crate::RegisteredDomainsByOwner::<Test>::get(&DOMAIN_OWNER), vec![domain_lc]);

        let domain_meta = Domains::registered_domain(tld, domain).unwrap();
        assert_eq!(domain_meta.expires_at, System::block_number() + ReservationPeriodLimit::get());
    });
}

// #[test]
// fn set_inner_value_should_work() {
//     ExtBuilder::build_with_balance_for_domain_owner().execute_with(|| {
//         assert_ok!(Balances::set_balance(Origin::root(), ACCOUNT1, 100, 0));
//
//         let full_domain = add_domain(ACCOUNT1);
//         let full_domain_lc = Domains::lower_domain(&full_domain);
//
//         let Domain { tld, domain } = &full_domain_lc;
//
//         assert_eq!(RegisteredDomainsByOwner::<Test>::get(ACCOUNT1), vec![full_domain_lc.clone()]);
//         let old_inner = Domains::registered_domain(tld, domain).unwrap().inner_value;
//
//         let new_inner = DomainInnerLink::Account(ACCOUNT1);
//         assert_ok!(Domains::set_inner_value(Origin::signed(ACCOUNT1), full_domain, Some(new_inner.clone())));
//
//         let actual_inner = Domains::registered_domain(tld, domain).unwrap().inner_value;
//         assert!(old_inner != actual_inner);
//         assert_eq!(Some(new_inner), actual_inner);
//     });
// }
//
// #[test]
// fn set_outer_value_should_reserve_deposit_correctly() {
//     ExtBuilder::build_with_balance_for_domain_owner().execute_with(|| {
//         assert_ok!(Balances::set_balance(Origin::root(), ACCOUNT1, 1000, 0));
//
//         let full_domain = add_domain(ACCOUNT1);
//         let full_domain_lc = Domains::lower_domain(&full_domain);
//
//         let Domain { tld, domain } = &full_domain_lc;
//
//         let outer_value = Some(vec![b'A'; OuterValueLimit::get() as usize]);
//         assert_ok!(Domains::set_outer_value(Origin::signed(ACCOUNT1), full_domain, outer_value.clone()));
//
//         let actual_outer_value = Domains::registered_domain(tld, domain).unwrap().outer_value;
//         assert_eq!(outer_value, actual_outer_value);
//
//         let account_reserved_balance = Balances::reserved_balance(ACCOUNT1);
//         let expected_outer_value_deposit = OuterValueByteDeposit::get() * <BalanceOf<Test>>::from(outer_value.unwrap().len() as u32);
//         assert_eq!(account_reserved_balance, DomainDeposit::get() + expected_outer_value_deposit)
//     });
// }
