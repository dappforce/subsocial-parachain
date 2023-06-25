// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchError, DispatchResult};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use subsocial_support::{SpaceId, SpacePermissionsInfo, User};

use super::*;

pub type SpacePermissionsInfoOf<T> =
    SpacePermissionsInfo<<T as frame_system::Config>::AccountId, SpacePermissions>;

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SpacePermission {
    /// Create, update, delete, grant and revoke roles in this space.
    ManageRoles,

    /// Act on behalf of this space within this space.
    RepresentSpaceInternally,
    /// Act on behalf of this space outside of this space.
    RepresentSpaceExternally,

    /// Update this space.
    UpdateSpace,

    // Related to subspaces in this space:
    CreateSubspaces,
    UpdateOwnSubspaces,
    DeleteOwnSubspaces,
    HideOwnSubspaces,

    UpdateAnySubspace,
    DeleteAnySubspace,
    HideAnySubspace,

    // Related to posts in this space:
    CreatePosts,
    UpdateOwnPosts,
    DeleteOwnPosts,
    HideOwnPosts,

    UpdateAnyPost,
    DeleteAnyPost,
    HideAnyPost,

    // Related to comments in this space:
    CreateComments,
    UpdateOwnComments,
    DeleteOwnComments,
    HideOwnComments,

    // NOTE: It was made on purpose that it's not possible to update or delete not own comments.
    // Instead it's possible to allow to hide and block comments.
    HideAnyComment,

    /// Upvote any post or comment in this space.
    Upvote,
    /// Downvote any post or comment in this space.
    Downvote,
    /// Share any post or comment from this space to another outer space.
    Share,

    /// Override permissions per subspace in this space.
    OverrideSubspacePermissions,
    /// Override permissions per post in this space.
    OverridePostPermissions,

    // Related to the moderation pallet:
    /// Suggest new entity status in space (whether it's blocked or allowed)
    SuggestEntityStatus,
    /// Update entity status in space
    UpdateEntityStatus,

    // Related to space settings:
    /// Allows to update space settings across different pallets.
    UpdateSpaceSettings,
}

pub type SpacePermissionSet = BTreeSet<SpacePermission>;

/// These are a set of built-in roles which can be given different permissions within a given space.
/// For example: everyone can comment (`CreateComments`), but only followers can post
/// (`CreatePosts`).
#[derive(Encode, Decode, Default, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SpacePermissions {
    /// None represents a set of permissions which is not capable of being performed by anyone.
    /// For example, if you want to create a space similar to Twitter, you would set the
    /// permissions for `UpdateOwnPosts`, `UpdateOwnComments`, and `Downvote` to `none`.
    pub none: Option<SpacePermissionSet>,

    /// Everyone represents a set of permissions which are capable of being performed by every
    /// account in a given space.
    pub everyone: Option<SpacePermissionSet>,

    /// Follower represents a set of permissions which are capable of being performed by every
    /// account that follows a given space.
    pub follower: Option<SpacePermissionSet>,

    /// Space owner represents a set of permissions which are capable of being performed by an
    /// account that is a current owner of a given space.
    pub space_owner: Option<SpacePermissionSet>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SpacePermissionsContext {
    pub space_id: SpaceId,
    pub is_space_owner: bool,
    pub is_space_follower: bool,
    pub space_perms: Option<SpacePermissions>,
}

impl SpacePermission {
    pub(super) fn is_present_in_role(&self, perms_opt: Option<SpacePermissionSet>) -> bool {
        if let Some(perms) = perms_opt {
            if perms.contains(self) {
                return true
            }
        }
        false
    }
}

pub trait PermissionChecker {
    type AccountId;

    fn ensure_user_has_space_permission(
        user: User<Self::AccountId>,
        ctx: SpacePermissionsContext,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult;

    fn ensure_account_has_space_permission(
        account: Self::AccountId,
        ctx: SpacePermissionsContext,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {
        Self::ensure_user_has_space_permission(User::Account(account), ctx, permission, error)
    }
}
