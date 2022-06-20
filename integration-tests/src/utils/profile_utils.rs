use frame_support::pallet_prelude::*;

use pallet_profiles::ProfileUpdate;
use pallet_utils::Content;

use crate::mock::*;
use crate::utils::{ACCOUNT1, SPACE1};

pub(crate) fn profile_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiaRtqdyoW2CuDgwxkA5".to_vec())
}


pub(crate) fn _create_default_profile() -> DispatchResult {
    _create_profile(None, None)
}

pub(crate) fn _create_profile(
    origin: Option<Origin>,
    content: Option<Content>
) -> DispatchResult {
    Profiles::create_profile(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        content.unwrap_or_else(profile_content_ipfs),
    )
}

pub(crate) fn _update_profile(
    origin: Option<Origin>,
    content: Option<Content>
) -> DispatchResult {
    Profiles::update_profile(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        ProfileUpdate {
            content,
        },
    )
}