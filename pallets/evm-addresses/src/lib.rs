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
    use crate::evm::*;

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
        type MaxLinkedAddresses: Get<u32>;
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
        // EvmCallExecuted {
        //     result: DispatchResult,
        // },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The evm address have an existing linkage already.
        EvmAddressAlreadyLinked,
        /// The substrate address have an existing linkage already.
        SubstrateAddressAlreadyLinked,
        /// The provided signature is invalid
        BadSignature,
        /// The substrate address is not linked to the given evm address
        AddressNotLinked,
        /// User have reached the maximum number of linked addresses.
        CannotLinkMoreAddresses,
    }

    /// Map of one EVM addresses to many substrate addresses
    #[pallet::storage]
    pub type SubstrateAddresses<T: Config> = StorageMap<
        _,
        Twox64Concat,
        EvmAddress,
        BoundedVec<T::AccountId, T::MaxLinkedAddresses>,
        ValueQuery,
    >;

    /// Map of substrate address to EVM account
    #[pallet::storage]
    pub type EvmAddresses<T: Config> =
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
            evm_signature: Eip712Signature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // ensure account_id and evm_address has not been linked
            ensure!(!EvmAddresses::<T>::contains_key(&who), Error::<T>::SubstrateAddressAlreadyLinked);
            ensure!(
                !SubstrateAddresses::<T>::get(evm_address).contains(&who),
                Error::<T>::EvmAddressAlreadyLinked
            );

            let message = SingableMessage::LinkEvmAddress {
                evm_address: evm_address.clone(),
                substrate_address: who.clone(),
            };
            // recover evm address from signature
            let address = Self::verify_eip712_signature(&message, &evm_signature)
                .ok_or(Error::<T>::BadSignature)?;
            ensure!(evm_address == address, Error::<T>::BadSignature);

            SubstrateAddresses::<T>::mutate(evm_address, |addresses| {
                addresses.try_push(who.clone()).map_err(|e| Error::<T>::CannotLinkMoreAddresses)
            })?;
            EvmAddresses::<T>::insert(&who, evm_address);

            Self::deposit_event(Event::AccountLinked { substrate: who, ethereum: evm_address });

            Ok(())
        }
    }
}
