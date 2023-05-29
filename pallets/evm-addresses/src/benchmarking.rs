#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks, Zero};
use frame_support::ensure;
use frame_system::RawOrigin;
use sp_core_hashing::keccak_256;

use crate::{
    evm::{evm_address, evm_secret_key, evm_sign},
    Pallet,
};

use super::*;

benchmarks! {
    link_evm_address {
        let linker: T::AccountId = account("linker", 24, 0);
        let linker_nonce = frame_system::pallet::Pallet::<T>::account_nonce(&linker);

        let linked_evm_sec = evm_secret_key(b"linked_account");
        let linked_evm_address = evm_address(&linked_evm_sec);

        let message = keccak_256(&Pallet::<T>::eth_signable_message(&linker, linker_nonce));
        let sig = evm_sign(&linked_evm_sec, message.as_slice());

    }: _(RawOrigin::Signed(linker.clone()), linked_evm_address.clone(), sig)
    verify {
       ensure!(
            AccountsByEvmAddress::<T>::get(linked_evm_address.clone()) == BTreeSet::from([linker.clone()]),
            "account isn't linked",
        );
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}
