#![cfg(feature = "runtime-benchmarks")]
// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


use frame_benchmarking::{account, benchmarks};
use frame_support::ensure;
use frame_system::RawOrigin;
use sp_core_hashing::keccak_256;

use crate::{
    evm::{evm_address, evm_secret_key, evm_sign},
    Pallet,
};

use super::*;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    link_evm_address {
        let linker: T::AccountId = account("linker", 24, 0);
        let linker_nonce = frame_system::pallet::Pallet::<T>::account_nonce(&linker);

        let linked_evm_sec = evm_secret_key(b"linked_account");
        let linked_evm_address = evm_address(&linked_evm_sec);

        let message = keccak_256(&Pallet::<T>::eth_signable_message(&linker, linker_nonce));
        let signature = evm_sign(&linked_evm_sec, message.as_slice());

    }: _(RawOrigin::Signed(linker.clone()), linked_evm_address.clone(), signature)
    verify {
       assert_eq!(
            EvmAddressByAccount::<T>::get(linker.clone()).unwrap(),
            linked_evm_address.clone(),
        );
        assert_last_event::<T>(Event::<T>::EvmAddressLinkedToAccount {
            substrate: linker.clone(),
            ethereum: linked_evm_address.clone(),
        }.into());
    }

    unlink_evm_address {
        let account: T::AccountId = account("account", 23, 0);

        let linked_evm_sec = evm_secret_key(b"linked_account");
        let linked_evm_address = evm_address(&linked_evm_sec);

        AccountsByEvmAddress::<T>::insert(linked_evm_address.clone(), BTreeSet::from([account.clone()]));
        EvmAddressByAccount::<T>::insert(account.clone(), linked_evm_address.clone());

    }: _(RawOrigin::Signed(account.clone()), linked_evm_address.clone())
    verify {
       ensure!(
            AccountsByEvmAddress::<T>::get(linked_evm_address.clone()).is_empty(),
            "Substrate account is still linked",
        );
        ensure!(
            EvmAddressByAccount::<T>::get(account.clone()).is_none(),
            "Evm account is still linked",
        );
        assert_last_event::<T>(Event::<T>::EvmAddressUnlinkedFromAccount {
            substrate: account.clone(),
            ethereum: linked_evm_address.clone(),
        }.into());
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}
