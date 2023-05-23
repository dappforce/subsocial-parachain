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
            evm_signature: Eip712Signature,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // ensure account_id and evm_address has not been linked
            ensure!(!EvmAccounts::<T>::contains_key(&who), Error::<T>::AccountAlreadyLinked);
            ensure!(
                !SubstrateAccounts::<T>::get(evm_address).contains(&who),
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

            SubstrateAccounts::<T>::mutate(evm_address, |accounts| {
                accounts.try_push(who.clone()).map_err(|e| Error::<T>::CannotLinkMoreAccounts)
            })?;
            EvmAccounts::<T>::insert(&who, evm_address);

            Self::deposit_event(Event::AccountLinked { substrate: who, ethereum: evm_address });

            Ok(())
        }
        //
        // #[pallet::call_index(1)]
        // #[pallet::weight(Weight::from_ref_time(999_000_000))]
        // pub fn eth_address_call(
        //     origin: OriginFor<T>,
        //     caller: EthAddress,
        //     call: Box<<T as Config>::RuntimeCall>,
        //     call_signature: Eip712Signature,
        // ) -> DispatchResult {
        //     ensure_none(origin)?;
        //     let mapped_account =
        //         Self::validate_eth_call_signature(&caller, call.clone(), &call_signature)
        //             .map_err(|e| -> Error<T> { e.into() })?;
        //
        //     let origin: T::RuntimeOrigin =
        //         frame_system::RawOrigin::Signed(mapped_account.clone()).into();
        //
        //     let call_res = call.clone().dispatch(origin);
        //
        //     Self::deposit_event(Event::EthCallExecuted {
        //         result: call_res.map(|_| ()).map_err(|e| e.error),
        //     });
        //
        //     Ok(())
        // }
    }
    //
    // #[repr(u8)]
    // pub enum ValidityError {
    //     /// The Ethereum signature is invalid.
    //     BadSignature = 0,
    //     /// There is no mapped substrate address to the etherum signer.
    //     NoMappedAccount = 1,
    // }
    //
    // impl From<ValidityError> for u8 {
    //     fn from(err: ValidityError) -> Self {
    //         err as u8
    //     }
    // }
    //
    // impl<T: Config> From<ValidityError> for Error<T> {
    //     fn from(err: ValidityError) -> Self {
    //         match err {
    //             ValidityError::BadSignature => Error::<T>::BadSignature,
    //             ValidityError::NoMappedAccount => Error::<T>::NoMappedAccount,
    //         }
    //     }
    // }
    //
    // impl Into<TransactionValidityError> for ValidityError {
    //     fn into(self) -> TransactionValidityError {
    //         TransactionValidityError::Invalid(InvalidTransaction::Custom(self.into()))
    //     }
    // }
    //
    // impl<T: Config> Pallet<T> {
    //     pub(crate) fn validate_eth_call_signature(
    //         caller: &EthAddress,
    //         call: Box<<T as Config>::RuntimeCall>,
    //         call_signature: &Eip712Signature,
    //     ) -> Result<T::AccountId, ValidityError> {
    //         let substrate_account =
    //             Self::evm_address_to_account(caller).ok_or(ValidityError::NoMappedAccount)?;
    //
    //         let message = SingableMessage::EthAddressCall {
    //             call_hash: T::CallHasher::hash_of(&call),
    //             account_nonce: frame_system::Account::<T>::get(substrate_account.clone()).nonce,
    //         };
    //         let address = Self::verify_eip712_signature(&message, call_signature)
    //             .ok_or(ValidityError::BadSignature)?;
    //
    //         ensure!(caller.clone() == address, ValidityError::BadSignature);
    //
    //         Ok(substrate_account)
    //     }
    // }
    //
    // #[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo, Default)]
    // #[scale_info(skip_type_params(T))]
    // pub struct ChargeTransactionPaymentEvmMapped<
    //     T: Config + sp_std::marker::Send + sp_std::marker::Sync,
    // >(sp_std::marker::PhantomData<T>);
    //
    // impl<T: Config + sp_std::marker::Send + sp_std::marker::Sync> sp_std::fmt::Debug
    //     for ChargeTransactionPaymentEvmMapped<T>
    // {
    //     #[cfg(feature = "std")]
    //     fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
    //         write!(f, "ChargeTransactionPaymentEvmMapped")
    //     }
    //     #[cfg(not(feature = "std"))]
    //     fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
    //         Ok(())
    //     }
    // }
    //
    // impl<T: Config + sp_std::marker::Send + sp_std::marker::Sync> SignedExtension
    //     for ChargeTransactionPaymentEvmMapped<T>
    // where
    //     BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand,
    //     <T as frame_system::Config>::RuntimeCall:
    //         Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    //     <T as frame_system::Config>::RuntimeCall: IsType<<T as Config>::RuntimeCall>,
    // {
    //     const IDENTIFIER: &'static str = "ChargeTransactionPaymentEvmMapped";
    //     type AccountId = T::AccountId;
    //     type Call = <T as Config>::RuntimeCall;
    //     type AdditionalSigned = ();
    //     type Pre = ();
    //
    //     fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
    //         Ok(())
    //     }
    //     fn pre_dispatch(
    //         self,
    //         who: &Self::AccountId,
    //         call: &Self::Call,
    //         info: &DispatchInfoOf<Self::Call>,
    //         len: usize,
    //     ) -> Result<Self::Pre, TransactionValidityError> {
    //         self.validate(who, call, info, len).map(|_| ())
    //     }
    //
    //     fn validate_unsigned(
    //         call: &Self::Call,
    //         info: &DispatchInfoOf<Self::Call>,
    //         len: usize,
    //     ) -> TransactionValidity {
    //         if let Some(Call::eth_address_call { caller, call: inner_call, call_signature }) =
    //             call.is_sub_type()
    //         {
    //             let mapped_account = Pallet::<T>::validate_eth_call_signature(
    //                 &caller,
    //                 inner_call.clone(),
    //                 &call_signature,
    //             )
    //             .map_err(|e| -> InvalidTransaction { InvalidTransaction::Custom(e.into()) })?;
    //
    //             let tip = BalanceOf::<T>::min_value();
    //             let extra: ChargeTransactionPayment<T> =
    // ChargeTransactionPayment::<T>::from(tip);             let call = <T as
    // frame_system::Config>::RuntimeCall::from_ref(call);             return
    // extra.validate(&mapped_account, call, info, len)         }
    //         Ok(ValidTransaction::default())
    //     }
    // }
}
