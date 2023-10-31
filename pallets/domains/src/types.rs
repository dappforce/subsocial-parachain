// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::pallet_prelude::*;
use frame_support::traits::Currency;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;

use subsocial_support::{WhoAndWhenOf, new_who_and_when};

use super::*;

pub(crate) type DomainName<T> = BoundedVec<u8, <T as Config>::MaxDomainLength>;
pub(crate) type InnerValueOf<T> = InnerValue<<T as frame_system::pallet::Config>::AccountId>;
pub(crate) type OuterValue<T> = BoundedVec<u8, <T as Config>::MaxOuterValueLength>;

/// This type represents a vector of prices for domain names.
/// Each element is a tuple (domain_length, price).
/// The vector must be sorted by domain length in ascending order to function correctly.
/// Usually, the shorter the domain length, the more expensive it is.
/// A correct example:
///  - [(2, 300), (3, 200), (4, 100)]
///
/// Meaning: A domain with 2 characters costs 300, a domain with 3 characters costs 200, and a domain with 4 characters costs 100.
pub type PricesConfigVec<T> = Vec<(DomainLength, BalanceOf<T>)>;

pub type DomainLength = u32;

pub(crate) type BoundedDomainsVec<T> = BoundedVec<DomainName<T>, <T as Config>::DomainsInsertLimit>;

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;

/// A domain deposit information wrapped into Option to use in this pallet.
pub(crate) type DomainDepositOf<T> =
    DomainDeposit<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

/// Domains inner value variants
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum InnerValue<AccountId> {
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}

/// A domain deposit info.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct DomainDeposit<AccountId, Balance> {
    pub(super) depositor: AccountId,
    pub(super) deposit: Balance,
}

/// A domain metadata.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct DomainMeta<T: Config> {
    /// When the domain was created.
    pub(super) created: WhoAndWhenOf<T>,
    /// When the domain was updated.
    pub(super) updated: Option<WhoAndWhenOf<T>>,

    /// Specific block, when the domain will become unavailable.
    pub(super) expires_at: T::BlockNumber,

    /// The domain owner.
    pub(super) owner: T::AccountId,

    /// Some additional domain metadata. For example avatar and description for this domain.
    pub(super) content: Content,

    /// The inner domain link to Subsocial entity such as Account, Space, or Post.
    pub(super) inner_value: Option<InnerValueOf<T>>,

    /// The outer domain link (any string).
    pub(super) outer_value: Option<OuterValue<T>>,

    /// The amount was held as a deposit for storing this structure.
    pub(super) domain_deposit: DomainDepositOf<T>,
    /// The amount was held as a deposit for storing outer value.
    pub(super) outer_value_deposit: BalanceOf<T>,
}

impl<T: Config> DomainMeta<T> {
    pub fn new(
        caller: T::AccountId,
        owner: T::AccountId,
        expires_at: T::BlockNumber,
        content: Content,
        domain_deposit: DomainDepositOf<T>,
    ) -> Self {
        Self {
            created: new_who_and_when::<T>(caller),
            updated: None,
            expires_at,
            owner,
            content,
            inner_value: None,
            outer_value: None,
            domain_deposit,
            outer_value_deposit: Zero::zero(),
        }
    }
}

impl<AccountId, Balance> From<(AccountId, Balance)> for DomainDeposit<AccountId, Balance> {
    fn from((depositor, deposit): (AccountId, Balance)) -> Self {
        Self { depositor, deposit }
    }
}
