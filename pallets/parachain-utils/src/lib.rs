// TODO Try to reuse these utility functions via crate in the future,
// when solochain and parachain will use the same substrate version.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use frame_support::pallet_prelude::*;
use sp_std::{vec, vec::Vec};

pub type SpaceId = u64;
pub type PostId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct WhoAndWhen<AccountId, BlockNumber, Moment> {
    pub account: AccountId,
    pub block: BlockNumber,
    pub time: Moment,
}

pub type WhoAndWhenOf<T> =
    WhoAndWhen<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::BlockNumber,
        <T as pallet_timestamp::Config>::Moment,
    >;

pub fn new_who_and_when<T>(
    account: T::AccountId
) -> WhoAndWhen<T::AccountId, T::BlockNumber, T::Moment>
where
    T: frame_system::Config + pallet_timestamp::Config
{
    WhoAndWhen {
        account,
        block: frame_system::Pallet::<T>::block_number(),
        time: pallet_timestamp::Pallet::<T>::now(),
    }
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

#[derive(Encode, Decode, RuntimeDebug)]
pub enum Error {
    /// IPFS CID is invalid.
    InvalidIpfsCid,
    /// `Raw` content type is not yet supported.
    RawContentTypeNotSupported,
    /// `Hyper` content type is not yet supported.
    HypercoreContentTypeNotSupported,
    /// Content type is `None`.
    ContentIsEmpty,
}

impl From<Error> for &'static str {
    fn from(e: Error) -> &'static str {
        match e {
            Error::InvalidIpfsCid => "InvalidIpfsCid",
            Error::RawContentTypeNotSupported => "RawContentTypeNotSupported",
            Error::HypercoreContentTypeNotSupported => "HypercoreContentTypeNotSupported",
            Error::ContentIsEmpty => "ContentIsEmpty",
        }
    }
}

pub fn ensure_content_is_valid(content: Content) -> DispatchResult {
    match content {
        Content::None => Ok(()),
        Content::Raw(_) => Err(
            DispatchError::Other(Error::RawContentTypeNotSupported.into())
        ),
        Content::IPFS(ipfs_cid) => {
            let len = ipfs_cid.len();
            // IPFS CID v0 is 46 bytes.
            // IPFS CID v1 is 59 bytes.
            ensure!(len == 46 || len == 59, DispatchError::Other(Error::InvalidIpfsCid.into()));
            Ok(())
        }
        Content::Hyper(_) => Err(
            DispatchError::Other(Error::HypercoreContentTypeNotSupported.into())
        ),
    }
}

/// Ensure that a given content is not `None`.
pub fn ensure_content_is_some(content: &Content) -> DispatchResult {
    ensure!(content.is_some(), DispatchError::Other(Error::ContentIsEmpty.into()));
    Ok(())
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

pub mod mock_functions {
    use super::Content;

    pub fn valid_content_ipfs() -> Content {
        Content::IPFS(b"QmRAQB6YaCaidP37UdDnjFY5aQuiBrbqdyoW1CaDgwxkD4".to_vec())
    }

    pub fn another_valid_content_ipfs() -> Content {
        // Only the last character is changed, only for testing purposes.
        Content::IPFS(b"QmRAQB6YaCaidP37UdDnjFY5aQuiBrbqdyoW1CaDgwxkD5".to_vec())
    }

    pub fn invalid_content_ipfs() -> Content {
        Content::IPFS(b"QmRAQB6DaazhR8".to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::remove_from_vec;

    #[test]
    fn remove_from_vec_should_work_with_zero_elements() {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![];

        remove_from_vec(vector, element);
        assert!(vector.is_empty());
    }

    #[test]
    fn remove_from_vec_should_work_with_last_element() {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![6, 2];

        vector.remove(0);
        assert_eq!(vector, &mut vec![2]);

        remove_from_vec(vector, element);
        assert!(vector.is_empty());
    }

    #[test]
    fn remove_from_vec_should_work_with_two_elements() {
        let element: u16 = 2;
        let vector: &mut Vec<u16> = &mut vec![6, 2, 7];

        vector.remove(0);
        assert_eq!(vector, &mut vec![2, 7]);

        remove_from_vec(vector, element);
        assert_eq!(vector, &mut vec![7]);
    }
}
