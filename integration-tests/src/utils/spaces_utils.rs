use frame_support::pallet_prelude::*;
use pallet_permissions::SpacePermissions;

use pallet_spaces::types::{SpacesSettings, SpaceUpdate};
use pallet_parachain_utils::{Content, SpaceId};

use crate::mock::*;
use crate::utils::{ACCOUNT1, SPACE1};

pub(crate) fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

pub(crate) fn updated_space_content() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec())
}


pub(crate) fn update_for_space_handle(
    new_handle: Option<Vec<u8>>,
) -> SpaceUpdate {
    space_update(Some(new_handle), None, None)
}

pub(crate) fn update_for_space_content(
    new_content: Content,
) -> SpaceUpdate {
    space_update(None, Some(new_content), None)
}

pub(crate) fn space_update(
    handle: Option<Option<Vec<u8>>>,
    content: Option<Content>,
    hidden: Option<bool>,
) -> SpaceUpdate {
    SpaceUpdate {
        parent_id: None,
        handle,
        content,
        hidden,
        permissions: None,
    }
}

pub(crate) fn space_settings_with_handles_disabled() -> SpacesSettings {
    SpacesSettings { handles_enabled: false }
}

pub(crate) fn space_settings_with_handles_enabled() -> SpacesSettings {
    SpacesSettings { handles_enabled: true }
}


pub(crate) fn _create_default_space() -> DispatchResultWithPostInfo {
    _create_space(None, None, None, None)
}

pub(crate) fn _create_space(
    origin: Option<Origin>,
    handle: Option<Option<Vec<u8>>>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>
) -> DispatchResultWithPostInfo {
    _create_space_with_parent_id(
        origin,
        None,
        handle,
        content,
        permissions,
    )
}

pub(crate) fn _create_subspace(
    origin: Option<Origin>,
    parent_id_opt: Option<Option<SpaceId>>,
    handle: Option<Option<Vec<u8>>>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>
) -> DispatchResultWithPostInfo {
    _create_space_with_parent_id(
        origin,
        parent_id_opt,
        handle,
        content,
        permissions,
    )
}

pub(crate) fn _create_space_with_parent_id(
    origin: Option<Origin>,
    parent_id_opt: Option<Option<SpaceId>>,
    handle: Option<Option<Vec<u8>>>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>
) -> DispatchResultWithPostInfo {
    Spaces::create_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        parent_id_opt.unwrap_or_default(),
        handle.unwrap_or_default(),
        content.unwrap_or_else(space_content_ipfs),
        permissions.unwrap_or_default()
    )
}

pub(crate) fn _update_space(
    origin: Option<Origin>,
    space_id: Option<SpaceId>,
    update: Option<SpaceUpdate>,
) -> DispatchResult {
    Spaces::update_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        update.unwrap_or_else(|| space_update(None, None, None)),
    )
}

pub(crate) fn _update_space_settings_with_handles_enabled() -> DispatchResultWithPostInfo {
    _update_space_settings(None, Some(space_settings_with_handles_enabled()))
}

pub(crate) fn _update_space_settings_with_handles_disabled() -> DispatchResultWithPostInfo {
    _update_space_settings(None, Some(space_settings_with_handles_disabled()))
}

/// Default origin is a root.
pub(crate) fn _update_space_settings(origin: Option<Origin>, new_settings: Option<SpacesSettings>) -> DispatchResultWithPostInfo {
    Spaces::update_settings(
        origin.unwrap_or_else(Origin::root),
        new_settings.unwrap_or_else(space_settings_with_handles_disabled)
    )
}
