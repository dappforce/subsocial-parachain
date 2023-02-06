#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_std::vec::Vec;

use crate::{Content, SpaceId};

pub trait SpacePermissionsProvider<AccountId, SpacePermissionsInfo> {
    fn space_permissions_info(id: SpaceId) -> Result<SpacePermissionsInfo, DispatchError>;

    fn ensure_space_owner(id: SpaceId, account: &AccountId) -> DispatchResult;
}

pub trait SpaceFollowsProvider {
    type AccountId;

    fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool;
}

pub trait ProfileManager<AccountId> {
    fn profile_space_id(account: &AccountId) -> Option<SpaceId>;

    fn try_set_profile(account: &AccountId, space_id: SpaceId) -> DispatchResult;

    fn unlink_space_from_profile(account: &AccountId, space_id: SpaceId) -> DispatchResult;
}

pub trait RolesInterface<RoleId, SpaceId, AccountId, SpacePermission, BlockNumber> {
    fn get_role_space(role_id: RoleId) -> Result<SpaceId, DispatchError>;

    fn grant_role(account_id: AccountId, role_id: RoleId) -> DispatchResult;

    fn create_role(
        space_owner: &AccountId,
        space_id: SpaceId,
        time_to_live: Option<BlockNumber>,
        content: Content,
        permissions: Vec<SpacePermission>,
    ) -> Result<RoleId, DispatchError>;
}

pub trait SpacesInterface<AccountId, SpaceId> {
    fn get_space_owner(space_id: SpaceId) -> Result<AccountId, DispatchError>;

    fn create_space(owner: &AccountId, content: Content) -> Result<SpaceId, DispatchError>;
}
