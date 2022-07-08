#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchError;

use crate::SpaceId;

pub trait SpacePermissionsProvider<SpacePermissionsInfo> {
    fn space_permissions_info(id: SpaceId) -> Result<SpacePermissionsInfo, DispatchError>;
}

pub trait SpaceFollowsProvider {
    type AccountId;

    fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool;
}
