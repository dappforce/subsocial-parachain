#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage, decl_event,
    dispatch::{DispatchError, DispatchResult}, ensure,
    traits::{
        Currency, Get,
        Imbalance, OnUnbalanced,
    },
};
use sp_runtime::RuntimeDebug;
use sp_std::{
    collections::btree_set::BTreeSet,
    prelude::*,
};
use frame_system::{self as system};

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;

pub type SpaceId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct WhoAndWhen<T: Config> {
    pub account: T::AccountId,
    pub block: T::BlockNumber,
    pub time: T::Moment,
}

impl<T: Config> WhoAndWhen<T> {
    pub fn new(account: T::AccountId) -> Self {
        WhoAndWhen {
            account,
            block: <system::Pallet<T>>::block_number(),
            time: <pallet_timestamp::Pallet<T>>::now(),
        }
    }
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum User<AccountId> {
    Account(AccountId),
    Space(SpaceId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum Content {
    None,
    Raw(Vec<u8>),
    IPFS(Vec<u8>),
    Hyper(Vec<u8>),
}

impl Content {
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

pub trait Config: system::Config + pallet_timestamp::Config
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;

    /// Minimal length of space/profile handle
    type MinHandleLen: Get<u32>;

    /// Maximal length of space/profile handle
    type MaxHandleLen: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Config> as UtilsModule {
        pub TreasuryAccount get(fn treasury_account) build(|config| config.treasury_account.clone()): T::AccountId;
    }
    add_extra_genesis {
        config(treasury_account): T::AccountId;
        build(|config| {
			// Create Treasury account
			let _ = T::Currency::make_free_balance_be(
				&config.treasury_account,
				T::Currency::minimum_balance(),
			);
		});
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        /// Minimal length of space/profile handle
        const MinHandleLen: u32 = T::MinHandleLen::get();

        /// Maximal length of space/profile handle
        const MaxHandleLen: u32 = T::MaxHandleLen::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// IPFS CID is invalid.
        InvalidIpfsCid,
        /// Unsupported yet type of content 'Raw' is used
        RawContentTypeNotSupported,
        /// Unsupported yet type of content 'Hyper' is used
        HypercoreContentTypeNotSupported,
        /// Space handle is too short.
        HandleIsTooShort,
        /// Space handle is too long.
        HandleIsTooLong,
        /// Space handle contains invalid characters.
        HandleContainsInvalidChars,
        /// Content type is `None`
        ContentIsEmpty,
    }
}

decl_event!(
    pub enum Event<T> where Balance = BalanceOf<T>
    {
		Deposit(Balance),
    }
);

fn num_bits<P>() -> usize {
    sp_std::mem::size_of::<P>() * 8
}

/// Returns `None` for `x == 0`.
pub fn log_2(x: u32) -> Option<u32> {
    if x > 0 {
        Some(
            num_bits::<u32>() as u32
            - x.leading_zeros()
            - 1
        )
    } else { None }
}

pub fn vec_remove_on<F: PartialEq>(vector: &mut Vec<F>, element: F) {
    if let Some(index) = vector.iter().position(|x| *x == element) {
        // TODO fix: swap_remove doesn't remove tha last element.
        vector.swap_remove(index);
    }
}

impl<T: Config> Module<T> {

    pub fn is_valid_content(content: Content) -> DispatchResult {
        match content {
            Content::None => Ok(()),
            Content::Raw(_) => Err(Error::<T>::RawContentTypeNotSupported.into()),
            Content::IPFS(ipfs_cid) => {
                let len = ipfs_cid.len();
                // IPFS CID v0 is 46 bytes.
                // IPFS CID v1 is 59 bytes.df-integration-tests/src/lib.rs:272:5
                ensure!(len == 46 || len == 59, Error::<T>::InvalidIpfsCid);
                Ok(())
            },
            Content::Hyper(_) => Err(Error::<T>::HypercoreContentTypeNotSupported.into())
        }
    }

    pub fn convert_users_vec_to_btree_set(
        users_vec: Vec<User<T::AccountId>>
    ) -> Result<BTreeSet<User<T::AccountId>>, DispatchError> {
        let mut users_set: BTreeSet<User<T::AccountId>> = BTreeSet::new();

        for user in users_vec.iter() {
            users_set.insert(user.clone());
        }

        Ok(users_set)
    }

    /// An example of a valid handle: `good_handle`.
    fn is_valid_handle_char(c: u8) -> bool {
        match c {
            b'0'..=b'9' | b'a'..=b'z' | b'_' => true,
            _ => false,
        }
    }

    /// Check if a handle length fits into min/max values.
    /// Lowercase a provided handle.
    /// Check if a handle consists of valid chars: 0-9, a-z, _.
    /// Check if a handle is unique across all spaces' handles (required one a storage read).
    pub fn lowercase_and_validate_a_handle(handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {
        // Check min and max lengths of a handle:
        ensure!(handle.len() >= T::MinHandleLen::get() as usize, Error::<T>::HandleIsTooShort);
        ensure!(handle.len() <= T::MaxHandleLen::get() as usize, Error::<T>::HandleIsTooLong);

        let handle_in_lowercase = handle.to_ascii_lowercase();

        // Check if a handle consists of valid chars: 0-9, a-z, _.
        ensure!(handle_in_lowercase.iter().all(|&x| Self::is_valid_handle_char(x)), Error::<T>::HandleContainsInvalidChars);

        Ok(handle_in_lowercase)
    }

    pub fn ensure_content_is_some(content: &Content) -> DispatchResult {
        ensure!(!content.is_none(), Error::<T>::ContentIsEmpty);
        Ok(())
    }
}

impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for Module<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
        let numeric_amount = amount.peek();
        let treasury_account = TreasuryAccount::<T>::get();

        // Must resolve into existing but better to be safe.
        let _ = T::Currency::resolve_creating(&treasury_account, amount);

        Self::deposit_event(RawEvent::Deposit(numeric_amount));
    }
}
