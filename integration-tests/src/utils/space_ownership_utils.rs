// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::pallet_prelude::*;

use subsocial_support::SpaceId;

use crate::mock::*;
use crate::utils::{ACCOUNT1, ACCOUNT2, SPACE1};

pub(crate) fn _transfer_default_space_ownership() -> DispatchResult {
    _transfer_space_ownership(None, None, None)
}

pub(crate) fn _transfer_space_ownership(
    origin: Option<RuntimeOrigin>,
    space_id: Option<SpaceId>,
    transfer_to: Option<AccountId>,
) -> DispatchResult {
    SpaceOwnership::transfer_ownership(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        transfer_to.unwrap_or(ACCOUNT2),
    )
}

pub(crate) fn _accept_default_pending_ownership() -> DispatchResult {
    _accept_pending_ownership(None, None)
}

pub(crate) fn _accept_pending_ownership(origin: Option<RuntimeOrigin>, space_id: Option<SpaceId>) -> DispatchResult {
    SpaceOwnership::accept_pending_ownership(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}

pub(crate) fn _reject_default_pending_ownership() -> DispatchResult {
    _reject_pending_ownership(None, None)
}

pub(crate) fn _reject_default_pending_ownership_by_current_owner() -> DispatchResult {
    _reject_pending_ownership(Some(RuntimeOrigin::signed(ACCOUNT1)), None)
}

pub(crate) fn _reject_pending_ownership(origin: Option<RuntimeOrigin>, space_id: Option<SpaceId>) -> DispatchResult {
    SpaceOwnership::reject_pending_ownership(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}
