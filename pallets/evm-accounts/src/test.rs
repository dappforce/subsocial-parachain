use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError::BadOrigin;

use crate::{
    evm::{evm_address, evm_secret_key, evm_sign, Eip712Signature, SingableMessage},
    mock::*,
    pallet::{*, EvmAccounts as EvmAccountsStorage},
    Error,
};

#[test]
fn link_substrate_account_should_fail_if_unsigned() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let evm_sec = evm_secret_key(b"evm_sec");
        let evm_pub = evm_address(&evm_sec);

        let message = SingableMessage::<Test>::LinkEvmAddress {
            evm_address: evm_pub.clone(),
            substrate_address: account.clone(),
        };

        let sig = evm_sign(&evm_sec, &message.message_hash());

        assert_noop!(crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::none(), evm_pub, sig), BadOrigin);
    });
}

#[test]
fn link_substrate_account_should_fail_if_bad_signature() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let evm_sec = evm_secret_key(b"evm_sec");
        let evm_pub = evm_address(&evm_sec);

        let bad_sig: Eip712Signature = [0; 65]; // all zeros

        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account), evm_pub, bad_sig),
            Error::<Test>::BadSignature,
        );
    });
}

#[test]
fn link_substrate_account_should_fail_if_signed_with_another_address() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);

        let message = SingableMessage::<Test>::LinkEvmAddress {
            evm_address: evm_pub1.clone(),
            substrate_address: account.clone(),
        };

        let evm_sec2 = evm_secret_key(b"evm_sec2");

        let sig = evm_sign(&evm_sec2, &message.message_hash());
        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account), evm_pub1, sig),
            Error::<Test>::BadSignature,
        );
    });
}

#[test]
fn link_substrate_account_should_fail_if_message_is_incorrect() {
    ExtBuilder::default().build().execute_with(|| {
        let account1 = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);

        //// Using wrong account

        let another_account = account(123);
        let sig = evm_sign(
            &evm_sec1,
            &SingableMessage::<Test>::LinkEvmAddress {
                evm_address: evm_pub1.clone(),
                substrate_address: another_account.clone(),
            }
            .message_hash(),
        );
        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account1), evm_pub1, sig),
            Error::<Test>::BadSignature,
        );

        //// Using wrong evm address

        let another_evm = evm_address(&evm_secret_key(b"another_evm"));
        let sig = evm_sign(
            &evm_sec1,
            &SingableMessage::<Test>::LinkEvmAddress {
                evm_address: another_evm.clone(),
                substrate_address: account1.clone(),
            }
            .message_hash(),
        );
        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account1), evm_pub1, sig),
            Error::<Test>::BadSignature,
        );
    });
}

#[test]
fn link_substrate_account_should_work_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let account1 = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);

        let sig = evm_sign(
            &evm_sec1,
            &SingableMessage::<Test>::LinkEvmAddress {
                evm_address: evm_pub1.clone(),
                substrate_address: account1.clone(),
            }
                .message_hash(),
        );

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account1), evm_pub1, sig));

        assert_eq!(SubstrateAccounts::<Test>::get(evm_pub1.clone()), vec![account1.clone()]);
        assert_eq!(EvmAccountsStorage::<Test>::get(account1.clone()), Some(evm_pub1.clone()));
    });
}


#[test]
fn link_substrate_account_should_work_correctly_with_multiple_accounts() {
    ExtBuilder::default()
        .max_linked_accounts(2)
        .build().execute_with(|| {
        let account1 = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);

        let sig = evm_sign(
            &evm_sec1,
            &SingableMessage::<Test>::LinkEvmAddress {
                evm_address: evm_pub1.clone(),
                substrate_address: account1.clone(),
            }
                .message_hash(),
        );

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account1), evm_pub1, sig));

        assert_eq!(SubstrateAccounts::<Test>::get(evm_pub1.clone()), vec![account1.clone()]);
        assert_eq!(EvmAccountsStorage::<Test>::get(account1.clone()), Some(evm_pub1.clone()));

        let account2 = account(2);

        let sig = evm_sign(
            &evm_sec1,
            &SingableMessage::<Test>::LinkEvmAddress {
                evm_address: evm_pub1.clone(),
                substrate_address: account2.clone(),
            }
                .message_hash(),
        );

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account2), evm_pub1, sig));
        assert_eq!(SubstrateAccounts::<Test>::get(evm_pub1.clone()), vec![account1.clone(), account2.clone()]);
        assert_eq!(EvmAccountsStorage::<Test>::get(account2.clone()), Some(evm_pub1.clone()));
    });
}

#[test]
fn link_substrate_account_should_fail_when_linking_more_than_max_linked_accounts() {
    ExtBuilder::default()
        .max_linked_accounts(1)
        .build().execute_with(|| {
        let account1 = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);

        let sig = evm_sign(
            &evm_sec1,
            &SingableMessage::<Test>::LinkEvmAddress {
                evm_address: evm_pub1.clone(),
                substrate_address: account1.clone(),
            }
                .message_hash(),
        );

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account1), evm_pub1, sig));

        assert_eq!(SubstrateAccounts::<Test>::get(evm_pub1.clone()), vec![account1.clone()]);
        assert_eq!(EvmAccountsStorage::<Test>::get(account1.clone()), Some(evm_pub1.clone()));

        let account2 = account(2);

        let sig = evm_sign(
            &evm_sec1,
            &SingableMessage::<Test>::LinkEvmAddress {
                evm_address: evm_pub1.clone(),
                substrate_address: account2.clone(),
            }
                .message_hash(),
        );

        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::signed(account2), evm_pub1, sig),
            Error::<Test>::CannotLinkMoreAccounts,

        );
    });
}