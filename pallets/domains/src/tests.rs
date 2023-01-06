#![allow(non_snake_case)]
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{DispatchError::BadOrigin, traits::Zero};
use sp_std::convert::TryInto;
use subsocial_support::{Content, new_who_and_when};

use crate::{Event, mock::*};
use crate::Error;
use crate::pallet::{DomainRecords, RegisteredDomains};
use crate::types::*;

// `register_domain` tests

#[test]
fn register_domain_should_work() {
    const LOCAL_DOMAIN_DEPOSIT: Balance = 10;

    ExtBuilder::default()
        .base_domain_deposit(LOCAL_DOMAIN_DEPOSIT)
        .build()
        .execute_with(|| {
            let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

            let expected_domain = default_domain();
            let expected_domain_lc = default_domain_lc();

            assert!(get_reserved_balance(&owner).is_zero());

            assert_ok!(_register_default_domain());

            assert_eq!(Domains::domains_by_owner(&owner), vec![expected_domain_lc.clone()]);

            let domain_meta = Domains::registered_domain(&expected_domain_lc).unwrap();
            assert_eq!(domain_meta, DomainMeta {
                created: new_who_and_when::<Test>(DOMAIN_OWNER),
                updated: None,
                expires_at: ExtBuilder::default().reservation_period_limit + 1,
                owner: DOMAIN_OWNER,
                content: Content::None,
                inner_value: None,
                outer_value: None,
                domain_deposit: LOCAL_DOMAIN_DEPOSIT,
                outer_value_deposit: Zero::zero()
            });

            assert_eq!(get_reserved_balance(&owner), LOCAL_DOMAIN_DEPOSIT);

            System::assert_last_event(Event::<Test>::DomainRegistered {
                who: owner,
                domain: expected_domain,
            }.into());
        });
}

#[test]
fn register_domain_should_fail_when_domain_already_owned() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_noop!(
            _register_default_domain(),
            Error::<Test>::DomainAlreadyOwned,
        );
    });
}

#[test]
fn register_domain_should_fail_when_too_many_domains_registered() {
    ExtBuilder::default()
        .max_domains_per_account(1)
        .build()
        .execute_with(|| {
            let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

            let domain_one = domain_from(b"domain-one".to_vec());
            let domain_two = domain_from(b"domain-two".to_vec());

            assert_ok!(_force_register_domain_with_name(domain_one));
            assert_noop!(
                _force_register_domain_with_name(domain_two),
                Error::<Test>::TooManyDomainsPerAccount,
            );
        });
}

#[test]
fn register_domain_should_fail_when_balance_is_insufficient() {
    ExtBuilder::default()
        .base_domain_deposit(10)
        .build()
        .execute_with(|| {
            let _ = account_with_balance(DOMAIN_OWNER, 9);

            assert_noop!(
                _register_default_domain(),
                pallet_balances::Error::<Test>::InsufficientBalance,
            );
        });
}

#[test]
fn register_domain_should_fail_when_promo_domains_limit_reached() {
    ExtBuilder::default()
        .max_promo_domains_per_account(1)
        .build()
        .execute_with(|| {
            let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

            assert_ok!(_register_default_domain());

            assert_noop!(
                Domains::register_domain(
                    Origin::signed(DOMAIN_OWNER),
                    domain_from(b"second-domain".to_vec()),
                    ExtBuilder::default().reservation_period_limit,
                ),
                Error::<Test>::TooManyDomainsPerAccount,
            );
        });
}

#[test]
fn force_register_domain_should_fail_with_bad_origin() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            _force_register_domain_with_origin(Origin::signed(DOMAIN_OWNER)),
            BadOrigin
        );
    });
}

#[test]
fn force_register_domain_should_fail_when_reservation_period_zero() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            _force_register_domain_with_expires_in(0),
            Error::<Test>::ZeroReservationPeriod,
        );
    });
}

#[test]
fn force_register_domain_should_fail_when_reservation_above_limit() {
    ExtBuilder::default()
        .reservation_period_limit(1000)
        .build()
        .execute_with(|| {
            assert_noop!(
                _force_register_domain_with_expires_in(1001),
                Error::<Test>::TooBigRegistrationPeriod,
            );
        });
}

#[test]
fn register_domain_should_fail_when_domain_reserved() {
    ExtBuilder::default().build().execute_with(|| {
        let word = Domains::bound_domain(b"splitword".to_vec());
        let domain = domain_from(b"split-wo-rd".to_vec());

        assert_ok!(Domains::reserve_words(
            Origin::root(),
            vec![word].try_into().expect("qed; domains vector exceeds the limit"),
        ));

        assert_noop!(
            Domains::register_domain(
                Origin::signed(DOMAIN_OWNER),
                domain,
                ExtBuilder::default().reservation_period_limit,
            ),
            Error::<Test>::DomainIsReserved,
        );
    });
}

// `reserve_domains` tests

#[test]
fn reserve_words_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let domains_list: BoundedDomainsVec<Test> = vec![
            Domains::bound_domain(b"word-one".to_vec()),
            Domains::bound_domain(b"word-two".to_vec()),
            Domains::bound_domain(b"word-three".to_vec()),
        ].try_into().expect("qed; domains vector exceeds the limit");

        assert_ok!(Domains::reserve_words(Origin::root(), domains_list.clone()));

        assert!(Domains::is_word_reserved(&domains_list[0]));
        assert!(Domains::is_word_reserved(&domains_list[1]));
        assert!(Domains::is_word_reserved(&domains_list[2]));

        System::assert_last_event(Event::<Test>::NewWordsReserved { count: 3 }.into());
    });
}

#[test]
fn reserve_words_should_fail_when_word_is_invalid() {
    ExtBuilder::default().build().execute_with(|| {
            let domains_list = vec![
                domain_from(b"domain--one".to_vec())
            ].try_into().expect("qed; domains vector exceeds the limit");

            assert_noop!(
                Domains::reserve_words(Origin::root(), domains_list),
                Error::<Test>::DomainContainsInvalidChar,
            );
        });
}

// `support_tlds` tests

#[test]
fn support_tlds_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(
            Domains::support_tlds(
                Origin::root(),
                vec![default_tld()].try_into().expect("qed; domains vector exceeds the limit"),
            )
        );

        assert!(Domains::is_tld_supported(default_tld()));
        System::assert_last_event(Event::<Test>::NewTldsSupported { count: 1 }.into());
    });
}

#[test]
fn support_tlds_should_fail_when_tld_is_invalid() {
    ExtBuilder::default().build().execute_with(|| {
        let tlds_list = vec![
            domain_from(b"domain--one".to_vec())
        ].try_into().expect("qed; domains vector exceeds the limit");

        assert_noop!(
                Domains::support_tlds(Origin::root(), tlds_list),
                Error::<Test>::DomainContainsInvalidChar,
            );
    });
}

// Test domain name validation function

#[test]
fn ensure_valid_domain_should_work() {
    ExtBuilder::default()
        .min_domain_length(3)
        .build()
        .execute_with(|| {
            assert_ok!(Domains::ensure_valid_domain(&split_domain_from(b"abcde.sub")));
            assert_ok!(Domains::ensure_valid_domain(&split_domain_from(b"a-b-c.sub")));
            assert_ok!(Domains::ensure_valid_domain(&split_domain_from(b"12345.sub")));

            assert_noop!(
                Domains::ensure_valid_domain(&split_domain_from(b"a.sub")),
                Error::<Test>::DomainIsTooShort,
            );
            assert_noop!(
                Domains::ensure_valid_domain(&split_domain_from(b"-ab.sub")),
                Error::<Test>::DomainContainsInvalidChar,
            );
            assert_noop!(
                Domains::ensure_valid_domain(&split_domain_from(b"ab-.sub")),
                Error::<Test>::DomainContainsInvalidChar,
            );
            assert_noop!(
                Domains::ensure_valid_domain(&split_domain_from(b"a--b.sub")),
                Error::<Test>::DomainContainsInvalidChar,
            );
        });
}


// Tests for set_record

// helper for records
fn record_key(k: &[u8]) -> RecordKey<Test> { k.to_vec().try_into().unwrap() }
fn record_value(v: &[u8]) -> RecordValue<Test> { v.to_vec().try_into().unwrap() }

#[test]
fn set_record_should_fail_when_caller_unsigned() {
    ExtBuilder::default()
        .build_with_default_domain_registered()
        .execute_with(|| {
            assert_noop!(
                Domains::set_record(
                    Origin::none(),
                    default_domain(),
                    b"key".to_vec().try_into().unwrap(),
                    Some(b"value".to_vec().try_into().unwrap()),
                ),
                BadOrigin,
            );
        });
}

#[test]
fn set_record_should_fail_when_caller_is_not_domain_owner() {
    ExtBuilder::default()
        .build_with_default_domain_registered()
        .execute_with(|| {
            assert_noop!(
                Domains::set_record(
                    Origin::signed(DUMMY_ACCOUNT),
                    default_domain(),
                    record_key(b"key"),
                    record_value(b"value").into(),
                ),
                Error::<Test>::NotDomainOwner,
            );
        });
}

#[test]
fn set_record_should_fail_when_domain_is_not_found() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            assert_noop!(
                Domains::set_record(
                    Origin::signed(DUMMY_ACCOUNT),
                    default_domain(),
                    record_key(b"key"),
                    record_value(b"value").into(),
                ),
                Error::<Test>::DomainNotFound,
            );
        });
}

#[test]
fn set_record_should_work_correctly() {
    ExtBuilder::default()
        .record_byte_deposit(0)
        .build_with_default_domain_registered()
        .execute_with(|| {
            let key = record_key(b"key");
            let value = record_value(b"value");
            let value_opt: Option<RecordValue<Test>> = Some(value.clone());
            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    value_opt.clone(),
                ),
            );

            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value, DOMAIN_OWNER, 0)));

            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key,
                    value: value_opt,
                    deposit: 0,
                }.into()
            );
        });
}

#[test]
fn set_record_should_fail_when_owner_have_no_record_deposit() {
    ExtBuilder::default()
        .record_byte_deposit(10)
        .build_with_default_domain_registered()
        .execute_with(|| {
            assert_noop!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    record_key(b"key"),
                    record_value(b"value").into(),
                ),
                pallet_balances::Error::<Test>::InsufficientBalance,
            );
        });
}

#[test]
fn set_record_should_reserve_correct_record_deposit() {
    ExtBuilder::default()
        .record_byte_deposit(120)
        .build_with_default_domain_registered()
        .execute_with(|| {

            account_with_balance(DOMAIN_OWNER, 1000);
            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 1000);

            let key = record_key(b"123");
            let value = record_value(b"456");

            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    Some(value.clone()),
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 280);


            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value.clone(), DOMAIN_OWNER, 720)));

            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key,
                    value: Some(value),
                    deposit: 720,
                }.into(),
            );
        });
}


#[test]
fn set_record_should_refund_full_record_deposit_when_record_is_deleted() {
    ExtBuilder::default()
        .record_byte_deposit(120)
        .build_with_default_domain_registered()
        .execute_with(|| {

            account_with_balance(DOMAIN_OWNER, 1000);
            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 1000);

            let key = record_key(b"123");
            let value = record_value(b"456");

            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    Some(value.clone()),
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 280);


            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value.clone(), DOMAIN_OWNER, 720)));

            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key: key.clone(),
                    value: Some(value),
                    deposit: 720,
                }.into(),
            );

            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    None,
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 1000);

            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), None);

            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key,
                    value: None,
                    deposit: 0,
                }.into()
            );
        });
}


#[test]
fn set_record_should_refund_part_of_deposit_when_new_record_is_smaller() {
    ExtBuilder::default()
        .record_byte_deposit(120)
        .build_with_default_domain_registered()
        .execute_with(|| {

            account_with_balance(DOMAIN_OWNER, 1000);
            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 1000);

            let key = record_key(b"123");
            let value = record_value(b"456");

            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    Some(value.clone()),
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 280);


            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value.clone(), DOMAIN_OWNER, 720)));

            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key: key.clone(),
                    value: Some(value),
                    deposit: 720,
                }.into()
            );

            let value = record_value(b"4");

            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    Some(value.clone()),
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 520);

            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value.clone(), DOMAIN_OWNER, 480)));

            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key,
                    value: Some(value),
                    deposit: 480,
                }.into(),
            );
        });
}

#[test]
fn set_record_should_reserve_more_deposit_when_new_record_is_bigger() {
    ExtBuilder::default()
        .record_byte_deposit(120)
        .build_with_default_domain_registered()
        .execute_with(|| {

            account_with_balance(DOMAIN_OWNER, 1000);
            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 1000);

            let key = record_key(b"123");
            let value = record_value(b"456");

            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    Some(value.clone()),
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 280);


            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value.clone(), DOMAIN_OWNER, 720)));

            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key: key.clone(),
                    value: Some(value),
                    deposit: 720,
                }.into(),
            );

            let value = record_value(b"45678");

            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    Some(value.clone()),
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 40);

            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value.clone(), DOMAIN_OWNER, 960)));

            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key,
                    value: Some(value),
                    deposit: 960,
                }.into(),
            );
        });
}

#[test]
fn set_record_should_refund_to_correct_depositor() {
    ExtBuilder::default()
        .record_byte_deposit(10)
        .build_with_default_domain_registered()
        .execute_with(|| {
            let DOMAIN_OWNER_2 = 10;
            let DOMAIN_OWNER_3 = 11;

            account_with_balance(DOMAIN_OWNER, 1000);
            account_with_balance(DOMAIN_OWNER_2, 1000);
            account_with_balance(DOMAIN_OWNER_3, 1000);

            fn change_ownership(new_owner: &AccountId) {
                RegisteredDomains::<Test>::mutate(default_domain_lc(), |maybe_meta| {
                    if let Some(meta) = maybe_meta {
                        meta.owner = *new_owner;
                    }
                })
            }

            let key = record_key(b"12345");
            let value = record_value(b"67890");

            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER),
                    default_domain(),
                    key.clone(),
                    Some(value.clone()),
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 900);

            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value.clone(), DOMAIN_OWNER, 100)));
            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER,
                    domain: default_domain_lc(),
                    key: key.clone(),
                    value: Some(value),
                    deposit: 100,
                }.into(),
            );

            change_ownership(&DOMAIN_OWNER_2);


            let value = record_value(b"6789");
            assert_ok!(
                Domains::set_record(
                    Origin::signed(DOMAIN_OWNER_2),
                    default_domain(),
                    key.clone(),
                    Some(value.clone()),
                ),
            );

            assert_eq!(Balances::free_balance(DOMAIN_OWNER), 1000);
            assert_eq!(Balances::free_balance(DOMAIN_OWNER_2), 910);

            assert_eq!(DomainRecords::<Test>::get(default_domain_lc(), key.clone()), Some((value.clone(), DOMAIN_OWNER_2, 90)));
            System::assert_last_event(
                Event::DomainRecordUpdated {
                    account: DOMAIN_OWNER_2,
                    domain: default_domain_lc(),
                    key,
                    value: Some(value),
                    deposit: 90,
                }.into(),
            );
        });
}

// Test calc_record_deposit
#[test]
fn test_calc_record_deposit() {
    let test = |deposit: Balance, key: &[u8], value_opt: Option<&[u8]>, expected: Balance| {
        let key: RecordKey<Test> = key.to_vec().try_into().unwrap();
        let value_opt: Option<RecordValue<Test>> = value_opt.map(|value| value.to_vec().try_into().unwrap());

        ExtBuilder::default()
            .record_byte_deposit(deposit)
            .build()
            .execute_with(|| {
                let res = pallet_domains::Pallet::<Test>::calc_record_deposit(key.clone(), value_opt.clone());
                assert_eq!(res, expected, "Expected deposit of ({},{:?}) to be {} but the result is {}",
                           String::from_utf8(key.to_vec()).unwrap(),
                           value_opt.map(|v| String::from_utf8(v.to_vec()).unwrap()),
                           expected,
                           res,
                );
            })
    };


    test(1, b"123", Some(b"123"), 6);
    test(10, b"1", Some(b"123456789"), 100);
    test(1, b"123", None, 0);
    test(33, b"1", Some(b"23"), 99);
    test(33, b"112121215458421", None, 0);
}