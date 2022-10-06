use frame_support::pallet_prelude::*;

/// Subscription settings for a space
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SpaceSubscriptionSettings<Balance, RoleId> {
    /// The balance required to subscribe to a space.
    pub subscription: Balance,

    /// Determines if subscriptions for a space s disabled.
    pub disabled: bool,

    /// The id of the role that will be granted for space subscriber.
    pub role_id: RoleId,
}

/// Information about space subscriber.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SpaceSubscriberInfo<Balance, RoleId, BlockNumber> {
    /// The block number at which the subscriptions became active.
    pub subscribed_on: BlockNumber,

    /// The balance paid for the subscriptions.
    pub subscription: Balance,

    /// The if of the granted role due to subscriptions.
    pub granted_role_id: RoleId,

    /// Determines if the user has marked themself as unsubscribed.
    pub unsubscribed: bool,
}

pub trait SubscriptionSpacesInterface<AccountId, SpaceId> {
    fn is_space_owner(owner: AccountId, space_id: SpaceId) -> bool;

    fn get_space_owner(space_id: SpaceId) -> Option<AccountId>;
}

pub trait SubscriptionRolesInterface<RoleId, SpaceId, AccountId> {
    fn does_role_exist_in_space(role_id: RoleId, space_id: SpaceId) -> bool;

    fn grant_role(account_id: AccountId, role_id: RoleId);
}
