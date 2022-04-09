#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;
pub mod mock_functions;

#[cfg(test)]
mod tests;

pub type SpaceId = u64;
pub type PostId = u64;

pub const DEFAULT_MIN_HANDLE_LEN: u32 = 5;
pub const DEFAULT_MAX_HANDLE_LEN: u32 = 50;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct WhoAndWhen<AccountId, BlockNumber, Moment> {
    pub account: AccountId,
    pub block: BlockNumber,
    pub time: Moment,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Content {
    /// No content.
    None,
    /// A raw vector of bytes.
    Raw(Vec<u8>),
    /// IPFS CID v0 of content.
    #[allow(clippy::upper_case_acronyms)]
    IPFS(Vec<u8>),
    /// Hypercore protocol (former DAT) id of content.
    Hyper(Vec<u8>),
}

impl From<Content> for Vec<u8> {
    fn from(content: Content) -> Vec<u8> {
        match content {
            Content::None => vec![],
            Content::Raw(vec_u8) => vec_u8,
            Content::IPFS(vec_u8) => vec_u8,
            Content::Hyper(vec_u8) => vec_u8,
        }
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::None
    }
}

impl Content {
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_ipfs(&self) -> bool {
        matches!(self, Self::IPFS(_))
    }
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::error]
    pub enum Error<T> {
        /// IPFS CID is invalid.
        InvalidIpfsCid,
        /// `Raw` content type is not yet supported.
        RawContentTypeNotSupported,
        /// `Hyper` content type is not yet supported.
        HypercoreContentTypeNotSupported,
        /// Content type is `None`.
        ContentIsEmpty,
    }

    impl<T: Config> Pallet<T> {
        pub fn ensure_content_is_valid(content: Content) -> DispatchResult {
            match content {
                Content::None => Ok(()),
                Content::Raw(_) => Err(Error::<T>::RawContentTypeNotSupported.into()),
                Content::IPFS(ipfs_cid) => {
                    let len = ipfs_cid.len();
                    // IPFS CID v0 is 46 bytes.
                    // IPFS CID v1 is 59 bytes.df-integration-tests/src/lib.rs:272:5
                    ensure!(len == 46 || len == 59, Error::<T>::InvalidIpfsCid);
                    Ok(())
                }
                Content::Hyper(_) => Err(Error::<T>::HypercoreContentTypeNotSupported.into()),
            }
        }

        /// Ensure that a given content is not `None`.
        pub fn ensure_content_is_some(content: &Content) -> DispatchResult {
            ensure!(content.is_some(), Error::<T>::ContentIsEmpty);
            Ok(())
        }
    }
}

pub fn remove_from_vec<F: PartialEq>(vector: &mut Vec<F>, element: F) {
    if let Some(index) = vector.iter().position(|x| *x == element) {
        vector.swap_remove(index);
    }
}

pub fn bool_to_option(value: bool) -> Option<bool> {
    if value {
        Some(value)
    } else {
        None
    }
}
