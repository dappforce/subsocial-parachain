//! # Evm Integration Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::{Dispatchable, GetDispatchInfo},
    pallet_prelude::*,
    traits::IsSubType,
};
use frame_system::pallet_prelude::*;
use sp_std::{collections::btree_set::BTreeSet, convert::TryInto};

pub use pallet::*;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod test;

mod evm;

#[frame_support::pallet]
pub mod pallet {
    use crate::evm::*;
    use frame_system::Pallet as System;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// The max number of substrate accounts that are linked to a given evm address.
        type MaxLinkedAccounts: Get<u32>;
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Account have been linked to evm address
        EvmAddressLinkedToAccount { ethereum: EvmAddress, substrate: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The substrate address have an existing linkage already.
        EvmAddressAlreadyLinkedToThisAccount,
        /// The provided signature is invalid
        BadSignature,
        /// Either provided payload (message or nonce) or evm address is invalid.
        EitherBadAddressOrPayload,
    }

    /// Map of one evm address to many substrate accounts
    #[pallet::storage]
    pub type AccountsByEvmAddress<T: Config> =
        StorageMap<_, Blake2_128Concat, EvmAddress, BTreeSet<T::AccountId>, ValueQuery>;

    /// Map of substrate account to EVM address
    #[pallet::storage]
    pub type EvmAddressByAccount<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, EvmAddress, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Link substrate address to EVM address.
        #[pallet::call_index(0)]
        // FIXME: put here at least something near real weights
        #[pallet::weight(Weight::from_ref_time(3_000_000_000))]
        // .saturating_add(T::DbWeight::get().reads(3 as u64))
        // .saturating_add(T::DbWeight::get().writes(2 as u64)))]
        pub fn link_evm_address(
            origin: OriginFor<T>,
            evm_address: EvmAddress,
            evm_signature: EcdsaSignature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let sub_nonce = System::<T>::account_nonce(&who);

            // recover evm address from signature
            let address = Self::verify_signature(
                &evm_signature,
                &who,
                sub_nonce,
            )
            .ok_or(Error::<T>::BadSignature)?;

            ensure!(evm_address == address, Error::<T>::EitherBadAddressOrPayload);

            AccountsByEvmAddress::<T>::try_mutate(evm_address, |accounts| {
                ensure!(!accounts.contains(&who), Error::<T>::EvmAddressAlreadyLinkedToThisAccount);
                accounts.insert(who.clone());
                Ok::<(), DispatchError>(())
            })?;
            EvmAddressByAccount::<T>::insert(&who, evm_address);

            Self::deposit_event(Event::EvmAddressLinkedToAccount {
                substrate: who,
                ethereum: evm_address,
            });

            Ok(())
        }
    }
}
