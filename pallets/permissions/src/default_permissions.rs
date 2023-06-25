// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use crate::{SpacePermission as SP, SpacePermissions};

use frame_support::parameter_types;
use sp_std::vec;

parameter_types! {
  pub DefaultSpacePermissions: SpacePermissions = SpacePermissions {

    // No permissions disabled by default
    none: None,

    everyone: Some(vec![
      SP::UpdateOwnSubspaces,
      SP::DeleteOwnSubspaces,
      SP::HideOwnSubspaces,

      SP::UpdateOwnPosts,
      SP::DeleteOwnPosts,
      SP::HideOwnPosts,

      SP::CreateComments,
      SP::UpdateOwnComments,
      SP::DeleteOwnComments,
      SP::HideOwnComments,

      SP::Upvote,
      SP::Downvote,
      SP::Share,
    ].into_iter().collect()),

    // Followers can do everything that everyone else can.
    follower: None,

    space_owner: Some(vec![
      SP::ManageRoles,
      SP::RepresentSpaceInternally,
      SP::RepresentSpaceExternally,
      SP::OverrideSubspacePermissions,
      SP::OverridePostPermissions,

      SP::CreateSubspaces,
      SP::CreatePosts,

      SP::UpdateSpace,
      SP::UpdateAnySubspace,
      SP::UpdateAnyPost,

      SP::DeleteAnySubspace,
      SP::DeleteAnyPost,

      SP::HideAnySubspace,
      SP::HideAnyPost,
      SP::HideAnyComment,

      SP::SuggestEntityStatus,
      SP::UpdateEntityStatus,

      SP::UpdateSpaceSettings,
    ].into_iter().collect()),
  };
}
