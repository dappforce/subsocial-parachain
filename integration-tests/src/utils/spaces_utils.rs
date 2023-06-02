use frame_support::pallet_prelude::*;
use pallet_permissions::SpacePermissions;

use pallet_spaces::types::SpaceUpdate;
use subsocial_support::{Content, SpaceId};

use crate::mock::*;
use crate::utils::{ACCOUNT1, SPACE1};

pub(crate) fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

pub(crate) fn another_space_content_ipfs() -> Content {
    Content::IPFS(b"bafyrelt3cif35x4ribisxgq7unhpun525l54eib3mgbou4xln42qqcgj6q".to_vec())
}

pub(crate) fn updated_space_content() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec())
}

pub(crate) fn update_for_space_content(
    new_content: Content,
) -> SpaceUpdate {
    space_update(Some(new_content))
}

pub(crate) fn space_update(
    content: Option<Content>,
) -> SpaceUpdate {
    SpaceUpdate {
        content,
        permissions: None,
    }
}

pub(crate) fn _create_default_space() -> DispatchResult {
    _create_space(None, None, None)
}

pub(crate) fn _create_space(
    origin: Option<RuntimeOrigin>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>
) -> DispatchResult {
    Spaces::create_space(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        content.unwrap_or_else(space_content_ipfs),
        permissions.unwrap_or_default()
    )
}

pub(crate) fn _create_space_with_content(content: Content) -> DispatchResult {
    _create_space(None, Some(content), None)
}

pub(crate) fn _update_space(
    origin: Option<RuntimeOrigin>,
    space_id: Option<SpaceId>,
    update: Option<SpaceUpdate>,
) -> DispatchResult {
    Spaces::update_space(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        update.unwrap_or_else(|| space_update(None)),
    )
}
