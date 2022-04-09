use frame_support::pallet_prelude::*;
use frame_support::traits::Currency;
use sp_runtime::traits::Zero;

use pallet_parachain_utils::{WhoAndWhenOf, new_who_and_when};

use super::*;

pub(crate) type DomainName<T> = BoundedVec<u8, <T as Config>::MaxDomainLength>;
pub(crate) type DomainsVec<T> = BoundedVec<DomainName<T>, <T as Config>::MaxDomainsPerAccount>;
pub(crate) type InnerValue<T> = Option<DomainInnerLink<<T as frame_system::pallet::Config>::AccountId>>;
pub(crate) type OuterValue<T> = Option<BoundedVec<u8, <T as Config>::MaxOuterValueLength>>;

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum DomainInnerLink<AccountId> {
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}

// A domain metadata.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct DomainMeta<T: Config> {
    // When the domain was created.
    created: WhoAndWhenOf<T>,
    // When the domain was updated.
    updated: Option<WhoAndWhenOf<T>>,

    // Specific block, when the domain will become unavailable.
    pub(super) expires_at: T::BlockNumber,

    // The domain owner.
    pub(super) owner: T::AccountId,

    // Some additional (custom) domain metadata.
    pub(super) content: Content,

    // The inner domain link (some Subsocial entity).
    pub(super) inner_value: InnerValue<T>,
    // The outer domain link (any string).
    pub(super) outer_value: OuterValue<T>,

    // The amount was held as a deposit for storing this structure.
    pub(super) domain_deposit: BalanceOf<T>,
    // The amount was held as a deposit for storing outer value.
    pub(super) outer_value_deposit: BalanceOf<T>,
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
