// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use frame_support::{assert_ok, pallet_prelude::*};
use sp_core::storage::Storage;
use sp_io::TestExternalities;

use pallet_permissions::SpacePermissions;
use pallet_posts::{PostExtension, PostUpdate};
use pallet_reactions::{ReactionId, ReactionKind};
use pallet_spaces::types::SpaceUpdate;
use subsocial_support::{Content, PostId, SpaceId};

use crate::mock::*;

////// Ext Builder

pub struct ExtBuilder;

impl ExtBuilder {
    fn configure_storages(storage: &mut Storage) {
        let mut accounts = Vec::new();
        for account in ACCOUNT1..=ACCOUNT3 {
            accounts.push(account);
        }

        let _ = pallet_balances::GenesisConfig::<Test> {
            balances: accounts.iter().cloned().map(|k| (k, 100)).collect(),
        }
        .assimilate_storage(storage);
    }

    /// Default ext configuration with BlockNumber 1
    pub fn build() -> TestExternalities {
        let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        Self::configure_storages(&mut storage);

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    fn add_default_space() {
        assert_ok!(_create_default_space());
    }

    fn add_another_space() {
        assert_ok!(_create_space_with_content(another_space_content_ipfs()));
    }

    fn add_post() {
        Self::add_default_space();
        assert_ok!(_create_default_post());
    }

    /// Custom ext configuration with SpaceId 1, PostId 1 and BlockNumber 1
    pub fn build_with_post() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(Self::add_post);
        ext
    }

    /// Custom ext configuration with SpaceId 1-2, PostId 1 where BlockNumber 1
    pub fn build_with_post_and_two_spaces() -> TestExternalities {
        let mut ext = Self::build_with_post();
        ext.execute_with(Self::add_another_space);
        ext
    }

    /// Custom ext configuration with SpaceId 1, PostId 1 and ReactionId 1 (on post) where
    /// BlockNumber is 1
    pub fn build_with_reacted_post_and_two_spaces() -> TestExternalities {
        let mut ext = Self::build_with_post_and_two_spaces();
        ext.execute_with(|| {
            assert_ok!(_create_default_post_reaction());
        });
        ext
    }
}

////// Consts

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const ACCOUNT3: AccountId = 3;

pub(crate) const SPACE1: SpaceId = 1001;

pub(crate) const POST1: PostId = 1;

pub(crate) const REACTION1: ReactionId = 1;
pub(crate) const REACTION2: ReactionId = 2;

///////////// Space Utils

pub(crate) fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

pub(crate) fn another_space_content_ipfs() -> Content {
    Content::IPFS(b"bafyrelt3cif35x4ribisxgq7unhpun525l54eib3mgbou4xln42qqcgj6q".to_vec())
}

pub(crate) fn space_update(content: Option<Content>, hidden: Option<bool>) -> SpaceUpdate {
    SpaceUpdate { content, hidden, permissions: None }
}

pub(crate) fn _create_default_space() -> DispatchResult {
    _create_space(None, None, None)
}

pub(crate) fn _create_space(
    origin: Option<RuntimeOrigin>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>,
) -> DispatchResult {
    Spaces::create_space(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        content.unwrap_or_else(space_content_ipfs),
        permissions.unwrap_or_default(),
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
        update.unwrap_or_else(|| space_update(None, None)),
    )
}

///////////// Post Utils

pub(crate) fn post_content_ipfs() -> Content {
    Content::IPFS(b"bafyreidzue2dtxpj6n4x5mktrt7las5wz5diqma47zr25uau743dhe76we".to_vec())
}

pub(crate) fn post_update(
    space_id: Option<SpaceId>,
    content: Option<Content>,
    hidden: Option<bool>,
) -> PostUpdate {
    PostUpdate { space_id, content, hidden }
}

pub(crate) fn extension_regular_post() -> PostExtension {
    PostExtension::RegularPost
}

pub(crate) fn _create_default_post() -> DispatchResult {
    _create_post(None, None, None, None)
}

pub(crate) fn _create_post(
    origin: Option<RuntimeOrigin>,
    space_id_opt: Option<Option<SpaceId>>,
    extension: Option<PostExtension>,
    content: Option<Content>,
) -> DispatchResult {
    Posts::create_post(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        space_id_opt.unwrap_or(Some(SPACE1)),
        extension.unwrap_or_else(extension_regular_post),
        content.unwrap_or_else(post_content_ipfs),
    )
}

pub(crate) fn _update_post(
    origin: Option<RuntimeOrigin>,
    post_id: Option<PostId>,
    update: Option<PostUpdate>,
) -> DispatchResult {
    Posts::update_post(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        post_id.unwrap_or(POST1),
        update.unwrap_or_else(|| post_update(None, None, None)),
    )
}

//////// Reaction utils

pub(crate) fn reaction_upvote() -> ReactionKind {
    ReactionKind::Upvote
}

pub(crate) fn reaction_downvote() -> ReactionKind {
    ReactionKind::Downvote
}

pub(crate) fn _create_default_post_reaction() -> DispatchResult {
    _create_post_reaction(None, None, None)
}

pub(crate) fn _create_post_reaction(
    origin: Option<RuntimeOrigin>,
    post_id: Option<PostId>,
    kind: Option<ReactionKind>,
) -> DispatchResult {
    Reactions::create_post_reaction(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        post_id.unwrap_or(POST1),
        kind.unwrap_or_else(reaction_upvote),
    )
}

pub(crate) fn _update_post_reaction(
    origin: Option<RuntimeOrigin>,
    post_id: Option<PostId>,
    reaction_id: ReactionId,
    kind: Option<ReactionKind>,
) -> DispatchResult {
    Reactions::update_post_reaction(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        post_id.unwrap_or(POST1),
        reaction_id,
        kind.unwrap_or_else(reaction_upvote),
    )
}

pub(crate) fn _delete_post_reaction(
    origin: Option<RuntimeOrigin>,
    post_id: Option<PostId>,
    reaction_id: ReactionId,
) -> DispatchResult {
    Reactions::delete_post_reaction(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        post_id.unwrap_or(POST1),
        reaction_id,
    )
}
