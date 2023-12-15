//! # Evm Integration Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{collections::btree_set::BTreeSet, convert::TryInto};

pub use pallet::*;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod test;

mod evm;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use crate::evm::*;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Account have been linked to evm address
        EvmAddressLinkedToAccount { ethereum: EvmAddress, substrate: T::AccountId },
        /// Account have been unlinked from evm address
        EvmAddressUnlinkedFromAccount { ethereum: EvmAddress, substrate: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The substrate address have an existing linkage already.
        EvmAddressAlreadyLinkedToThisAccount,
        /// The evm address has not an existing linkage.
        EvmAddressNotLinkedToThisAccount,
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
        #[pallet::weight(
            Weight::from_parts(300_000_000, 0)
                .saturating_add(T::DbWeight::get().reads_writes(2, 2))
        )]
        pub fn link_evm_address(
            origin: OriginFor<T>,
            evm_address: EvmAddress,
            evm_signature: EcdsaSignature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let sub_nonce = frame_system::Pallet::<T>::account_nonce(&who);

            // recover evm address from signature
            let address = Self::verify_signature(&evm_signature, &who, sub_nonce)
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

        /// Unlink substrate address from EVM address.
        #[pallet::call_index(1)]
        // FIXME: put here at least something near real weights
        #[pallet::weight(
            Weight::from_parts(300_000_000, 0)
                .saturating_add(T::DbWeight::get().reads_writes(1, 2))
        )]
        pub fn unlink_evm_address(origin: OriginFor<T>, evm_address: EvmAddress) -> DispatchResult {
            let who = ensure_signed(origin)?;

            AccountsByEvmAddress::<T>::try_mutate(evm_address, |accounts| {
                ensure!(accounts.contains(&who), Error::<T>::EvmAddressNotLinkedToThisAccount);
                accounts.remove(&who);
                Ok::<(), DispatchError>(())
            })?;
            EvmAddressByAccount::<T>::remove(&who);

            Self::deposit_event(Event::EvmAddressUnlinkedFromAccount {
                substrate: who,
                ethereum: evm_address,
            });

            Ok(())
        }
    }
}
