use frame_support::pallet_prelude::*;
use frame_support::traits::Currency;
use sp_runtime::traits::{StaticLookup, Zero};
use sp_std::vec::Vec;

use subsocial_support::{WhoAndWhenOf, new_who_and_when, Content};

use super::*;

/// A vector of characters representing a domain name.
///
/// It is a vector of characters, which represents either the full domain name (e.g.
/// "example.sub") or a part of it (e.g. "example", "sub").
///
/// Can be split into a domain subset with the [`Pallet::split_domain_by_dot`] function.
pub(crate) type DomainName<T> = BoundedVec<u8, <T as Config>::MaxDomainLength>;
#[deprecated]
pub(crate) type InnerValueOf<T> = InnerValue<<T as frame_system::pallet::Config>::AccountId>;
#[deprecated]
pub(crate) type OuterValue<T> = BoundedVec<u8, <T as Config>::MaxOuterValueLength>;

pub(crate) type DomainRecordKey<T> = BoundedVec<u8, <T as Config>::MaxRecordKeyLength>;
pub(crate) type DomainRecordValue<T> = BoundedVec<u8, <T as Config>::MaxRecordValueLength>;

pub(crate) type BoundedDomainsVec<T> = BoundedVec<DomainName<T>, <T as Config>::DomainsInsertLimit>;
pub(crate) type PricesConfigVec<T> = Vec<(DomainLength, BalanceOf<T>)>;

pub type DomainLength = u32;

/// A subset of second level domains.
/// Alias to a tuple: `(subdomain, top-level domain)`.
pub(crate) type DomainParts<T> =
    (/* subdomain */ DomainName<T>, /* tld */ DomainName<T>);

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;


#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct  RecordValueWithDepositInfo<T: Config> {
    pub record_value: DomainRecordValue<T>,
    pub depositor: <T as frame_system::pallet::Config>::AccountId,
    pub deposit: BalanceOf<T>,
}

impl<T: Config> From<(DomainRecordValue<T>, <T as frame_system::pallet::Config>::AccountId, BalanceOf<T>)> for RecordValueWithDepositInfo<T> {
    fn from(value: (DomainRecordValue<T>, <T as frame_system::Config>::AccountId, BalanceOf<T>)) -> Self {
        RecordValueWithDepositInfo {
            record_value: value.0,
            depositor: value.1,
            deposit: value.2,
        }
    }
}

pub(crate) type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

pub(crate) type LookupOf<T> = <T as frame_system::Config>::Lookup;

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

pub(super) enum DomainPayer<T: Config> {
    ForceOrigin,
    Account(T::AccountId),
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

    #[deprecated]
    /// The inner domain link to Subsocial entity such as Account, Space, or Post.
    pub(super) inner_value: Option<InnerValueOf<T>>,

    #[deprecated]
    /// The outer domain link (any string).
    pub(super) outer_value: Option<OuterValue<T>>,

    /// The amount was held as a deposit for storing this structure.
    pub(super) domain_deposit: BalanceOf<T>,
    /// The amount was held as a deposit for storing outer value.
    pub(super) outer_value_deposit: BalanceOf<T>,
}

pub(crate) struct DomainRegisterData<T: Config> {
    pub owner: T::AccountId,
    pub full_domain: DomainName<T>,
    pub expires_in: T::BlockNumber,
}

impl<T: Config> DomainRegisterData<T> {
    pub fn new(
        owner: T::AccountId,
        full_domain: DomainName<T>,
        expires_in: T::BlockNumber,
    ) -> Self {
        Self { owner, full_domain, expires_in }
    }
}

impl<T: Config> DomainMeta<T> {
    pub fn new(
        expires_at: T::BlockNumber,
        owner: T::AccountId,
        domain_deposit: BalanceOf<T>,
    ) -> Self {
        Self {
            created: new_who_and_when::<T>(owner.clone()),
            updated: None,
            expires_at,
            owner,
            content: Content::None,
            inner_value: None,
            outer_value: None,
            domain_deposit,
            outer_value_deposit: Zero::zero(),
        }
    }
}
