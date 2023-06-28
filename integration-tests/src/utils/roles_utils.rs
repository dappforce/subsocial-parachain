// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2023 DAPPFORCE PTE. Ltd., dappforce@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use frame_support::pallet_prelude::*;

use pallet_permissions::SpacePermission;
use pallet_roles::UsersByRoleId;
use subsocial_support::{Content, SpaceId, User};

use crate::mock::*;
use crate::utils::{ACCOUNT1, ACCOUNT2, SPACE1};

// TODO: fix copy-paste from pallet_roles
/* Roles pallet mocks */

type RoleId = u64;

pub(crate) const ROLE1: RoleId = 1;
pub(crate) const ROLE2: RoleId = 2;

pub(crate) fn default_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}



pub fn _create_default_role() -> DispatchResult {
    _create_role(None, None, None, None, None)
}

pub fn _create_role(
    origin: Option<RuntimeOrigin>,
    space_id: Option<SpaceId>,
    time_to_live: Option<Option<BlockNumber>>,
    content: Option<Content>,
    permissions: Option<Vec<SpacePermission>>,
) -> DispatchResult {
    // TODO: remove
    use crate::utils::permissions_utils::permission_set_default;
    Roles::create_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        time_to_live.unwrap_or_default(), // Should return 'None'
        content.unwrap_or_else(default_role_content_ipfs),
        permissions.unwrap_or_else(permission_set_default),
    )
}

pub fn _grant_default_role() -> DispatchResult {
    _grant_role(None, None, None)
}

pub fn _grant_role(
    origin: Option<RuntimeOrigin>,
    role_id: Option<RoleId>,
    users: Option<Vec<User<AccountId>>>,
) -> DispatchResult {
    Roles::grant_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1),
        users.unwrap_or_else(|| vec![User::Account(ACCOUNT2)]),
    )
}

pub fn _delete_default_role() -> DispatchResult {
    _delete_role(None, None)
}

pub fn _delete_role(
    origin: Option<RuntimeOrigin>,
    role_id: Option<RoleId>,
) -> DispatchResult {
    let role_id = role_id.unwrap_or(ROLE1);
    Roles::delete_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        role_id,
        UsersByRoleId::<TestRuntime>::get(role_id).len() as u32,
    )
}
