use frame_support::{assert_noop, assert_ok};
use sp_runtime::{DispatchError, DispatchError::BadOrigin, traits::Zero};
use sp_std::convert::TryInto;

use pallet_parachain_utils::mock_functions::{another_valid_content_ipfs, invalid_content_ipfs, valid_content_ipfs};
use pallet_parachain_utils::new_who_and_when;

use crate::{DomainByInnerValue, Event, mock::*};
use crate::Error;
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
                content: valid_content_ipfs(),
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
                    valid_content_ipfs(),
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
                valid_content_ipfs(),
                ExtBuilder::default().reservation_period_limit,
            ),
            Error::<Test>::DomainIsReserved,
        );
    });
}

// `set_inner_value` tests

#[test]
fn set_inner_value_should_work() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        let domain_lc = default_domain_lc();
        let old_value = get_inner_value(&domain_lc);

        assert_ok!(_set_default_inner_value());

        let result_value = get_inner_value(&domain_lc);
        assert!(old_value != result_value);

        let expected_value = Some(inner_value_account_domain_owner());
        assert_eq!(expected_value, result_value);

        assert_eq!(
            DomainByInnerValue::<Test>::get(DOMAIN_OWNER, &expected_value.unwrap()),
            Some(default_domain_lc()),
        );

        System::assert_last_event(Event::<Test>::DomainMetaUpdated {
            who: DOMAIN_OWNER,
            domain: domain_lc,
        }.into());
    });
}

#[test]
fn set_inner_value_should_work_when_same_for_different_domains() {
    let domain_one = domain_from(b"domain-one".to_vec());
    let domain_two = domain_from(b"domain-two".to_vec());

    ExtBuilder::default()
        .base_domain_deposit(0)
        .build()
        .execute_with(|| {
            assert_ok!(Domains::register_domain(
                origin_a(), domain_one.clone(), valid_content_ipfs(), 1
            ));
            assert_ok!(Domains::register_domain(
                origin_b(), domain_two.clone(), valid_content_ipfs(), 1
            ));

            assert_ok!(Domains::set_inner_value(
                origin_a(), domain_one.clone(), Some(inner_value_space_id())
            ));
            assert_ok!(Domains::set_inner_value(
                origin_b(), domain_two.clone(), Some(inner_value_space_id())
            ));

            assert_eq!(
                DomainByInnerValue::<Test>::get(ACCOUNT_A, inner_value_space_id()),
                Some(domain_one.clone()),
            );
            assert_eq!(
                DomainByInnerValue::<Test>::get(ACCOUNT_B, inner_value_space_id()),
                Some(domain_two.clone()),
            );

            System::assert_has_event(Event::<Test>::DomainMetaUpdated {
                who: ACCOUNT_A,
                domain: domain_one,
            }.into());

            System::assert_has_event(Event::<Test>::DomainMetaUpdated {
                who: ACCOUNT_B,
                domain: domain_two,
            }.into());
        });
}

#[test]
fn set_inner_value_should_work_when_value_changes() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        let domain_lc = default_domain_lc();
        let initial_value = inner_value_account_domain_owner();
        let new_value = inner_value_space_id();

        assert_ok!(_set_default_inner_value());

        assert_ok!(Domains::set_inner_value(
            Origin::signed(DOMAIN_OWNER),
            domain_lc.clone(),
            Some(new_value.clone()),
        ));

        assert_eq!(DomainByInnerValue::<Test>::get(DOMAIN_OWNER, &initial_value), None);
        assert_eq!(DomainByInnerValue::<Test>::get(DOMAIN_OWNER, &new_value), Some(default_domain_lc()));

        assert_eq!(Some(new_value), get_inner_value(&domain_lc));
    });
}

#[test]
fn set_inner_value_should_fail_when_domain_has_expired() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        System::set_block_number(ExtBuilder::default().reservation_period_limit + 1);

        assert_noop!(
            _set_default_inner_value(),
            Error::<Test>::DomainHasExpired,
        );
    });
}

#[test]
fn set_inner_value_should_fail_when_not_domain_owner() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_noop!(
            _set_inner_value_with_origin(Origin::signed(DUMMY_ACCOUNT)),
            Error::<Test>::NotDomainOwner,
        );
    });
}

#[test]
fn set_inner_value_should_fail_when_inner_value_not_differ() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_ok!(_set_default_inner_value());

        assert_noop!(
            _set_default_inner_value(),
            Error::<Test>::InnerValueNotChanged,
        );
    });
}

#[test]
fn force_set_inner_value_should_work() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_ok!(
            Domains::force_set_inner_value(
                Origin::root(),
                default_domain_lc(),
                Some(inner_value_account_domain_owner()),
            )
        );
    });
}

#[test]
fn force_set_inner_value_should_fail_when_origin_not_root() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_noop!(
            Domains::force_set_inner_value(
                Origin::signed(DOMAIN_OWNER),
                default_domain_lc(),
                Some(inner_value_account_domain_owner()),
            ),
            BadOrigin,
        );
    });
}

// `set_outer_value` tests

#[test]
fn set_outer_value_should_work() {
    const LOCAL_DOMAIN_DEPOSIT: Balance = 10;
    const LOCAL_BYTE_DEPOSIT: Balance = 1;

    ExtBuilder::default()
        .base_domain_deposit(LOCAL_DOMAIN_DEPOSIT)
        .outer_value_byte_deposit(LOCAL_BYTE_DEPOSIT)
        .build_with_default_domain_registered()
        .execute_with(|| {
            let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

            let domain_lc = default_domain_lc();
            let old_value = get_outer_value(&domain_lc);

            assert_ok!(_set_default_outer_value());

            let expected_value = Some(default_outer_value(None));
            let result_value = get_outer_value(&domain_lc);

            assert!(old_value != result_value);
            assert_eq!(expected_value, result_value);

            let reserved_balance = get_reserved_balance(&owner);
            assert_eq!(
                reserved_balance,
                expected_value.unwrap().len() as u64 * LOCAL_BYTE_DEPOSIT + LOCAL_DOMAIN_DEPOSIT
            );

            System::assert_last_event(Event::<Test>::DomainMetaUpdated {
                who: owner,
                domain: domain_lc,
            }.into());
        });
}

#[test]
fn set_outer_value_should_reserve_correct_deposit_when_outer_value_keep_changing() {
    const LOCAL_BYTE_DEPOSIT_INIT: Balance = 1;
    let domain_deposit = ExtBuilder::default().base_domain_deposit;

    ExtBuilder::default()
        .outer_value_byte_deposit(LOCAL_BYTE_DEPOSIT_INIT)
        .build_with_default_domain_registered()
        .execute_with(|| {
            let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());
            let calc_deposit = |value| value as Balance * LOCAL_BYTE_DEPOSIT_INIT + domain_deposit;

            let initial_value = default_outer_value(Some(10));
            let updated_value = default_outer_value(Some(20));

            // Set outer value with length 10 and ensure that an appropriate deposit reserved
            assert_ok!(_set_outer_value_with_value(Some(initial_value.clone())));
            assert_eq!(get_reserved_balance(&owner), calc_deposit(initial_value.len()));

            // Set outer value with length 20 and ensure that deposit has increased
            assert_ok!(_set_outer_value_with_value(Some(updated_value.clone())));
            assert_eq!(get_reserved_balance(&owner), calc_deposit(updated_value.len()));

            // Set outer value with length 10 and ensure that an appropriate deposit decreased
            assert_ok!(_set_outer_value_with_value(Some(initial_value.clone())));
            assert_eq!(get_reserved_balance(&owner), calc_deposit(initial_value.len()));

            // Remove outer value and ensure that deposit was unreserved
            assert_ok!(_set_outer_value_with_value(None));
            assert_eq!(get_reserved_balance(&owner), calc_deposit(0));
        });
}

#[test]
fn set_outer_value_should_fail_when_domain_has_expired() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        System::set_block_number(ExtBuilder::default().reservation_period_limit + 1);

        assert_noop!(
            _set_default_outer_value(),
            Error::<Test>::DomainHasExpired,
        );
    });
}

#[test]
fn set_outer_value_should_fail_when_not_domain_owner() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_noop!(
            _set_outer_value_with_origin(Origin::signed(DUMMY_ACCOUNT)),
            Error::<Test>::NotDomainOwner,
        );
    });
}

#[test]
fn set_outer_value_should_fail_when_value_not_differ() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        assert_ok!(_set_default_outer_value());

        assert_noop!(
            _set_default_outer_value(),
            Error::<Test>::OuterValueNotChanged,
        );
    });
}

#[test]
fn set_outer_value_should_fail_when_balance_is_insufficient() {
    const LOCAL_BYTE_DEPOSIT: Balance = 1;

    ExtBuilder::default()
        .outer_value_byte_deposit(LOCAL_BYTE_DEPOSIT)
        .build_with_default_domain_registered()
        .execute_with(|| {
            // At this point account has 0 free balance
            assert_noop!(
                _set_default_outer_value(),
                pallet_balances::Error::<Test>::InsufficientBalance,
            );
        });
}

// `set_domain_content` tests

#[test]
fn set_domain_content_should_work() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        let domain_lc = default_domain_lc();
        let old_content = get_domain_content(&domain_lc);

        assert_ok!(_set_default_domain_content());

        let result_content = get_domain_content(&domain_lc);

        assert!(old_content != result_content);
        assert_eq!(another_valid_content_ipfs(), result_content);

        System::assert_last_event(Event::<Test>::DomainMetaUpdated {
            who: owner,
            domain: domain_lc,
        }.into());
    });
}

#[test]
fn set_domain_content_should_fail_when_domain_expired() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        System::set_block_number(ExtBuilder::default().reservation_period_limit + 1);

        assert_noop!(
            _set_default_domain_content(),
            Error::<Test>::DomainHasExpired,
        );
    });
}

#[test]
fn set_domain_content_should_fail_when_not_domain_owner() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_noop!(
            _set_domain_content_with_origin(Origin::signed(DUMMY_ACCOUNT)),
            Error::<Test>::NotDomainOwner,
        );
    });
}

#[test]
fn set_domain_content_should_fail_when_content_not_differ() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_ok!(_set_default_domain_content());

        assert_noop!(
            _set_default_domain_content(),
            Error::<Test>::DomainContentNotChanged,
        );
    });
}

#[test]
fn set_domain_content_should_fail_when_content_is_invalid() {
    ExtBuilder::default().build_with_default_domain_registered().execute_with(|| {
        assert_noop!(
            _set_domain_content_with_content(invalid_content_ipfs()),
            DispatchError::Other(pallet_parachain_utils::Error::InvalidIpfsCid.into()),
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
