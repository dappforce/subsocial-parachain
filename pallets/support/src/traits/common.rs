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

pub trait OwnershipTransferValidator<AccountId> {
    fn ensure_can_transfer_ownership(
        current_owner: &AccountId,
        new_owner: &AccountId,
        space_id: SpaceId,
    ) -> Result<(), &'static str>;
}

impl<AccountId> OwnershipTransferValidator<AccountId> for () {
    fn ensure_can_transfer_ownership(
        _current_owner: &AccountId,
        _new_owner: &AccountId,
        _space_id: SpaceId,
    ) -> Result<(), &'static str> {
        Ok(())
    }
}
