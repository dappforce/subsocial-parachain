use frame_support::pallet_prelude::*;
use frame_support::traits::Currency;
use sp_runtime::traits::Zero;

use subsocial_support::{WhoAndWhenOf, new_who_and_when};

use super::*;

/// Is a domain name vector.
///
/// It is a vector of characters, which represents either the full domain name (e.g.
/// "polkaverse.sub") or a part of it (e.g. "polkaverse", "sub").
///
/// Can be split to a domain subset with [`Pallet::split_domain_by_dot`] function.
pub(crate) type DomainName<T> = BoundedVec<u8, <T as Config>::MaxDomainLength>;
pub(crate) type InnerValueOf<T> = InnerValue<<T as frame_system::pallet::Config>::AccountId>;
pub(crate) type OuterValue<T> = BoundedVec<u8, <T as Config>::MaxOuterValueLength>;

pub(crate) type BoundedDomainsVec<T> = BoundedVec<DomainName<T>, <T as Config>::DomainsInsertLimit>;

pub type PriceRangeStart = u32;
/// A subset of second level domain.
/// Alias to a tuple: `(subdomain, top-level domain)`.
pub(crate) type DomainSubset<T> =
    (/* subdomain */ DomainName<T>, /* tld */ DomainName<T>);

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;

/// Domains inner value variants
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum InnerValue<AccountId> {
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}

pub(super) enum IsForced {
    Yes,
    No,
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
    pub(super) domain_deposit: BalanceOf<T>,
    /// The amount was held as a deposit for storing outer value.
    pub(super) outer_value_deposit: BalanceOf<T>,
}

pub(crate) struct DomainData<T: Config> {
    pub owner: T::AccountId,
    pub full_domain: DomainName<T>,
    pub content: Content,
    pub expires_in: T::BlockNumber,
}

impl<T: Config> DomainData<T> {
    pub fn new(
        owner: T::AccountId,
        full_domain: DomainName<T>,
        content: Content,
        expires_in: T::BlockNumber,
    ) -> Self {
        Self { owner, full_domain, content, expires_in }
    }
}

impl<T: Config> DomainMeta<T> {
    pub fn new(
        expires_at: T::BlockNumber,
        owner: T::AccountId,
        content: Content,
        domain_deposit: BalanceOf<T>,
    ) -> Self {
        Self {
            created: new_who_and_when::<T>(owner.clone()),
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
