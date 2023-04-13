use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError::BadOrigin;

use crate::{
    eth::{eth_address, eth_secret_key, eth_sign, Eip712Signature, SingableMessage},
    mock::*,
    pallet::*,
    Error,
};

#[test]
fn link_substrate_account_should_fail_if_unsigned() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let eth_sec = eth_secret_key(b"eth_sec");
        let eth_pub = eth_address(&eth_sec);

        let message = SingableMessage::<Test>::LinkEthAddress {
            eth_address: eth_pub.clone(),
            substrate_address: account.clone(),
        };

        let sig = eth_sign(&eth_sec, &message.message_hash());

        assert_noop!(EvmAccounts::link_substrate_account(RuntimeOrigin::none(), eth_pub, sig), BadOrigin);
    });
}

#[test]
fn link_substrate_account_should_fail_if_bad_signature() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let eth_sec = eth_secret_key(b"eth_sec");
        let eth_pub = eth_address(&eth_sec);

        let bad_sig: Eip712Signature = [0; 65]; // all zeros

        assert_noop!(
            EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account), eth_pub, bad_sig),
            Error::<Test>::BadSignature,
        );
    });
}

#[test]
fn link_substrate_account_should_fail_if_signed_with_another_address() {
    ExtBuilder::default().build().execute_with(|| {
        let account = account(1);

        let eth_sec1 = eth_secret_key(b"eth_sec1");
        let eth_pub1 = eth_address(&eth_sec1);

        let message = SingableMessage::<Test>::LinkEthAddress {
            eth_address: eth_pub1.clone(),
            substrate_address: account.clone(),
        };

        let eth_sec2 = eth_secret_key(b"eth_sec2");

        let sig = eth_sign(&eth_sec2, &message.message_hash());
        assert_noop!(
            EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account), eth_pub1, sig),
            Error::<Test>::BadSignature,
        );
    });
}

#[test]
fn link_substrate_account_should_fail_if_message_is_incorrect() {
    ExtBuilder::default().build().execute_with(|| {
        let account1 = account(1);

        let eth_sec1 = eth_secret_key(b"eth_sec1");
        let eth_pub1 = eth_address(&eth_sec1);

        //// Using wrong account

        let another_account = account(123);
        let sig = eth_sign(
            &eth_sec1,
            &SingableMessage::<Test>::LinkEthAddress {
                eth_address: eth_pub1.clone(),
                substrate_address: another_account.clone(),
            }
            .message_hash(),
        );
        assert_noop!(
            EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account1), eth_pub1, sig),
            Error::<Test>::BadSignature,
        );

        //// Using wrong eth address

        let another_eth = eth_address(&eth_secret_key(b"another_eth"));
        let sig = eth_sign(
            &eth_sec1,
            &SingableMessage::<Test>::LinkEthAddress {
                eth_address: another_eth.clone(),
                substrate_address: account1.clone(),
            }
            .message_hash(),
        );
        assert_noop!(
            EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account1), eth_pub1, sig),
            Error::<Test>::BadSignature,
        );
    });
}

#[test]
fn link_substrate_account_should_work_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let account1 = account(1);

        let eth_sec1 = eth_secret_key(b"eth_sec1");
        let eth_pub1 = eth_address(&eth_sec1);

        let sig = eth_sign(
            &eth_sec1,
            &SingableMessage::<Test>::LinkEthAddress {
                eth_address: eth_pub1.clone(),
                substrate_address: account1.clone(),
            }
                .message_hash(),
        );

        assert_ok!(EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account1), eth_pub1, sig));

        assert_eq!(Accounts::<Test>::get(eth_pub1.clone()), vec![account1.clone()]);
        assert_eq!(EthAddresses::<Test>::get(account1.clone()), Some(eth_pub1.clone()));
    });
}


#[test]
fn link_substrate_account_should_work_correctly_with_multiple_accounts() {
    ExtBuilder::default()
        .max_linked_accounts(2)
        .build().execute_with(|| {
        let account1 = account(1);

        let eth_sec1 = eth_secret_key(b"eth_sec1");
        let eth_pub1 = eth_address(&eth_sec1);

        let sig = eth_sign(
            &eth_sec1,
            &SingableMessage::<Test>::LinkEthAddress {
                eth_address: eth_pub1.clone(),
                substrate_address: account1.clone(),
            }
                .message_hash(),
        );

        assert_ok!(EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account1), eth_pub1, sig));

        assert_eq!(Accounts::<Test>::get(eth_pub1.clone()), vec![account1.clone()]);
        assert_eq!(EthAddresses::<Test>::get(account1.clone()), Some(eth_pub1.clone()));

        let account2 = account(2);

        let sig = eth_sign(
            &eth_sec1,
            &SingableMessage::<Test>::LinkEthAddress {
                eth_address: eth_pub1.clone(),
                substrate_address: account2.clone(),
            }
                .message_hash(),
        );

        assert_ok!(EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account2), eth_pub1, sig));
        assert_eq!(Accounts::<Test>::get(eth_pub1.clone()), vec![account1.clone(), account2.clone()]);
        assert_eq!(EthAddresses::<Test>::get(account2.clone()), Some(eth_pub1.clone()));
    });
}

#[test]
fn link_substrate_account_should_fail_when_linking_more_than_max_linked_accounts() {
    ExtBuilder::default()
        .max_linked_accounts(1)
        .build().execute_with(|| {
        let account1 = account(1);

        let eth_sec1 = eth_secret_key(b"eth_sec1");
        let eth_pub1 = eth_address(&eth_sec1);

        let sig = eth_sign(
            &eth_sec1,
            &SingableMessage::<Test>::LinkEthAddress {
                eth_address: eth_pub1.clone(),
                substrate_address: account1.clone(),
            }
                .message_hash(),
        );

        assert_ok!(EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account1), eth_pub1, sig));

        assert_eq!(Accounts::<Test>::get(eth_pub1.clone()), vec![account1.clone()]);
        assert_eq!(EthAddresses::<Test>::get(account1.clone()), Some(eth_pub1.clone()));

        let account2 = account(2);

        let sig = eth_sign(
            &eth_sec1,
            &SingableMessage::<Test>::LinkEthAddress {
                eth_address: eth_pub1.clone(),
                substrate_address: account2.clone(),
            }
                .message_hash(),
        );

        assert_noop!(
            EvmAccounts::link_substrate_account(RuntimeOrigin::signed(account2), eth_pub1, sig),
            Error::<Test>::CannotLinkMoreAccounts,

        );
    });
}