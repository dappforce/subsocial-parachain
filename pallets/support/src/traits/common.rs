#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};

use crate::{Content, PostId, SpaceId};

pub trait SpacePermissionsProvider<AccountId, SpacePermissionsInfo> {
    fn space_permissions_info(id: SpaceId) -> Result<SpacePermissionsInfo, DispatchError>;

    fn ensure_space_owner(id: SpaceId, account: &AccountId) -> DispatchResult;
}

pub trait SpaceFollowsProvider {
    type AccountId;

    fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool;
}

pub trait PostFollowsProvider {
    type AccountId;

    fn is_post_follower(account: Self::AccountId, post_id: PostId) -> bool;
}

pub trait ProfileManager<AccountId> {
    fn unlink_space_from_profile(account: &AccountId, space_id: SpaceId);
}

pub trait SpacesInterface<AccountId, SpaceId> {
    fn get_space_owner(space_id: SpaceId) -> Result<AccountId, DispatchError>;

    fn create_space(owner: &AccountId, content: Content) -> Result<SpaceId, DispatchError>;
}

pub trait CreatorStakingProvider<AccountId> {
    fn is_creator_active(
        creator_id: SpaceId,
    ) -> bool;
}

impl<AccountId> CreatorStakingProvider<AccountId> for () {
    fn is_creator_active(
        _creator_id: SpaceId,
    ) -> bool {
        true
    }
}
