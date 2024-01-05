use std::collections::BTreeSet;

use frame_support::{assert_noop, assert_ok};
use sp_core_hashing::keccak_256;
use sp_runtime::DispatchError::BadOrigin;

use crate::{evm::{evm_address, evm_secret_key, evm_sign, EcdsaSignature}, mock::*, AccountsByEvmAddress, Error, EvmAddressByAccount, Pallet, Event};

type MessageHash = [u8; 32];

fn get_nonce(account: &AccountId) -> u64 {
    frame_system::pallet::Pallet::<Test>::account_nonce(&account)
}

fn eth_signable_message(sub_address: &AccountId, sub_nonce: u64) -> MessageHash {
    keccak_256(&Pallet::<Test>::eth_signable_message(sub_address, sub_nonce))
}

#[test]
fn link_substrate_account_should_fail_if_unsigned() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);
        let nonce = get_nonce(&account);

        let evm_sec = evm_secret_key(b"evm_sec");
        let evm_pub = evm_address(&evm_sec);

        let message = eth_signable_message(&account, nonce);

        let sig = evm_sign(&evm_sec, &message);

        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(RuntimeOrigin::none(), evm_pub, sig),
            BadOrigin
        );
    });
}

#[test]
fn link_substrate_account_should_fail_if_bad_signature() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let evm_sec = evm_secret_key(b"evm_sec");
        let evm_pub = evm_address(&evm_sec);

        let bad_sig: EcdsaSignature = [0; 65]; // all zeros

        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(
                RuntimeOrigin::signed(account),
                evm_pub,
                bad_sig
            ),
            Error::<Test>::BadEvmSignature,
        );
    });
}

#[test]
fn link_substrate_account_should_fail_if_signed_with_another_address() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);

        let message = eth_signable_message(&account, get_nonce(&account));

        let evm_sec2 = evm_secret_key(b"evm_sec2");

        let sig = evm_sign(&evm_sec2, &message);
        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(
                RuntimeOrigin::signed(account),
                evm_pub1,
                sig
            ),
            Error::<Test>::EitherBadAddressOrPayload,
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
            &eth_signable_message(&another_account, get_nonce(&another_account)),
        );
        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(
                RuntimeOrigin::signed(account1),
                evm_pub1,
                sig
            ),
            Error::<Test>::EitherBadAddressOrPayload,
        );

        // Using invalid nonce
        let sig = evm_sign(
            &evm_sec1,
            &eth_signable_message(&another_account, get_nonce(&another_account) + 100),
        );
        assert_noop!(
            crate::mock::EvmAccounts::link_evm_address(
                RuntimeOrigin::signed(account1),
                evm_pub1,
                sig
            ),
            Error::<Test>::EitherBadAddressOrPayload,
        );
    });
}

#[test]
fn link_substrate_account_should_work_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let account1 = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);

        let sig = evm_sign(&evm_sec1, &eth_signable_message(&account1, get_nonce(&account1)));

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(
            RuntimeOrigin::signed(account1),
            evm_pub1,
            sig
        ));

        assert_eq!(
            AccountsByEvmAddress::<Test>::get(evm_pub1.clone()),
            BTreeSet::from([account1.clone()])
        );
        assert_eq!(EvmAddressByAccount::<Test>::get(account1.clone()), Some(evm_pub1.clone()));
    });
}

#[test]
fn link_substrate_account_should_work_correctly_with_multiple_accounts() {
    ExtBuilder::default().build().execute_with(|| {
        let account1 = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);

        let sig = evm_sign(&evm_sec1, &eth_signable_message(&account1, get_nonce(&account1)));

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(
            RuntimeOrigin::signed(account1),
            evm_pub1,
            sig
        ));

        assert_eq!(AccountsByEvmAddress::<Test>::get(evm_pub1.clone()), BTreeSet::from([account1.clone()]));
        assert_eq!(EvmAddressByAccount::<Test>::get(account1.clone()), Some(evm_pub1.clone()));

        let account2 = account(2);

        let sig = evm_sign(&evm_sec1, &eth_signable_message(&account2, get_nonce(&account2)));

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(
            RuntimeOrigin::signed(account2),
            evm_pub1,
            sig
        ));
        assert_eq!(
            AccountsByEvmAddress::<Test>::get(evm_pub1.clone()),
            BTreeSet::from([account1.clone(), account2.clone()]),
        );
        assert_eq!(EvmAddressByAccount::<Test>::get(account2.clone()), Some(evm_pub1.clone()));
    });
}

#[test]
fn unlink_evm_address_should_fail_if_unsigned() {
    ExtBuilder::default().build().execute_with(|| {
        let evm_pub = evm_address(&evm_secret_key(b"evm_sec"));

        assert_noop!(
            crate::mock::EvmAccounts::unlink_evm_address(RuntimeOrigin::none(), evm_pub),
            BadOrigin
        );
    });
}

#[test]
fn unlink_evm_address_should_unlink_linked_account() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let evm_sec1 = evm_secret_key(b"evm_sec1");
        let evm_pub1 = evm_address(&evm_sec1);
        let sig = evm_sign(&evm_sec1, &eth_signable_message(&account, get_nonce(&account)));

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(
            RuntimeOrigin::signed(account),
            evm_pub1,
            sig
        ));

        let evm_sec2 = evm_secret_key(b"evm_sec2");
        let evm_pub2 = evm_address(&evm_sec2);
        let sig = evm_sign(&evm_sec2, &eth_signable_message(&account, get_nonce(&account)));

        assert_ok!(crate::mock::EvmAccounts::link_evm_address(
            RuntimeOrigin::signed(account),
            evm_pub2,
            sig
        ));

        assert_ok!(
            crate::mock::EvmAccounts::unlink_evm_address(RuntimeOrigin::signed(account), evm_pub1),
        );

        assert_eq!(
            AccountsByEvmAddress::<Test>::get(evm_pub1.clone()),
            BTreeSet::from([])
        );
        assert_eq!(EvmAddressByAccount::<Test>::get(account.clone()), None);
        System::assert_last_event(Event::<Test>::EvmAddressUnlinkedFromAccount {
            substrate: account,
            ethereum: evm_pub1.clone(),
        }.into());
    });
}