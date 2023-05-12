//! # Evm Integration Pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::{Codec, DispatchInfo, Dispatchable, GetDispatchInfo, PostDispatchInfo},
    pallet_prelude::*,
    traits::{tokens::Balance, Currency, IsSubType},
};
use frame_system::pallet_prelude::*;
use pallet_transaction_payment::OnChargeTransaction;
use sp_runtime::{
    traits::{
        Bounded, CheckedAdd, CheckedSub, Extrinsic, Hash, MaybeSerialize, Saturating,
        SignedExtension, StaticLookup, Zero,
    },
    FixedPointNumber, FixedPointOperand,
};
use sp_std::{
    convert::TryInto,
    fmt::Debug,
    marker::{Send, Sync},
};

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

mod evm;

type BalanceOf<T> = <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use sp_core::crypto::{Ss58Codec};
    use crate::evm::*;
    use frame_system::Pallet as System;
    use sp_core::hexdisplay::AsBytesRef;
    use sp_runtime::SaturatedConversion;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<
                RuntimeOrigin = Self::RuntimeOrigin,
                Info = DispatchInfo,
                PostInfo = PostDispatchInfo,
            > + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// The type of hash used for hashing the call.
        type CallHasher: Hash;

        /// The max number of substrate accounts that are linked to a given evm address.
        type MaxLinkedAccounts: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Account have been linked to evm address
        AccountLinked {
            substrate: T::AccountId,
            ethereum: EvmAddress,
        },
        EvmCallExecuted {
            result: DispatchResult,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The evm address have an existing linkage already.
        EvmAddressAlreadyLinked,
        /// The substrate address have an existing linkage already.
        AccountAlreadyLinked,
        /// The provided signature is invalid
        BadSignature,
        /// The substrate address is not linked to the given evm address
        AccountNotLinked,
        /// User have reached the maximum number of linked accounts.
        CannotLinkMoreAccounts,
    }

    /// Map of one EVM account to many substrate addresses
    #[pallet::storage]
    pub type SubstrateAccounts<T: Config> = StorageMap<
        _,
        Twox64Concat,
        EvmAddress,
        BoundedVec<T::AccountId, T::MaxLinkedAccounts>,
        ValueQuery,
    >;

    /// Map of substrate address to EVM account
    #[pallet::storage]
    pub type EvmAccounts<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, EvmAddress, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Link substrate address to EVM address.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_ref_time(340_000_000)
            .saturating_add(T::DbWeight::get().reads(3 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64)))]
        pub fn link_evm_address(
            origin: OriginFor<T>,
            evm_address: EvmAddress,
            evm_signature: EcdsaSignature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let sub_nonce = System::<T>::account_nonce(&who).encode().as_bytes_ref();
            
            // TODO: Need to convert to ss58 address using to_ss58check from Ss58Codec
            let ss58_address = who.clone();
            // recover evm address from signature
            let address = Self::verify_signature(&evm_signature, ss58_address, sub_nonce)
                .ok_or(Error::<T>::BadSignature)?;
            ensure!(evm_address == address, Error::<T>::BadSignature);

            SubstrateAccounts::<T>::mutate(evm_address, |accounts| {
                accounts.try_push(who.clone()).map_err(|e| Error::<T>::CannotLinkMoreAccounts)
            })?;
            EvmAccounts::<T>::insert(&who, evm_address);

            Self::deposit_event(Event::AccountLinked { substrate: who, ethereum: evm_address });

            Ok(())
        }
}
