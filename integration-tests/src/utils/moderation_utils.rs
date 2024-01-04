// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::pallet_prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use subsocial_support::{
    traits::{IsAccountBlocked, IsContentBlocked, IsPostBlocked, IsSpaceBlocked},
    mock_functions::valid_content_ipfs, Content, PostId, SpaceId,
};

use crate::mock::*;
use crate::utils::{ACCOUNT1, SPACE1};

// Moderation pallet mocks

/*------------------------------------------------------------------------------------------------*/
// Moderation tests

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum EntityId {
    Content(Content),
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}

impl Hash for EntityId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            EntityId::Content(content) => match content {
                Content::None => 0.hash(state),
                Content::Other(content) => content.hash(state),
                Content::IPFS(content) => content.hash(state),
            },
            EntityId::Account(account) => account.hash(state),
            EntityId::Space(space) => space.hash(state),
            EntityId::Post(post) => post.hash(state),
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, Hash)]
pub enum EntityStatus {
    Allowed,
    Blocked,
}

thread_local! {
    pub static MOCK_MODERATION_STATE: RefCell<HashMap<(EntityId, SpaceId), EntityStatus>> = RefCell::new(Default::default());
}
pub struct MockModeration;

impl MockModeration {
    fn set_entity_status(id: EntityId, space: SpaceId, status: EntityStatus) {
        MOCK_MODERATION_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.insert((id, space), status);
        });
    }

    fn get_entity_status(id: EntityId, scope: SpaceId) -> Option<EntityStatus> {
        MOCK_MODERATION_STATE.with(|state| {
            let state = state.borrow();
            let status = state.get(&(id, scope)).cloned();
            status
        })
    }

    fn is_allowed_entity(id: EntityId, scope: SpaceId) -> bool {
        Self::get_entity_status(id, scope)
            .unwrap_or(EntityStatus::Allowed) == EntityStatus::Allowed
    }

    fn is_blocked_entity(id: EntityId, scope: SpaceId) -> bool {
        Self::get_entity_status(id, scope) == Some(EntityStatus::Blocked)
    }
}

impl IsPostBlocked<PostId> for MockModeration {
    fn is_blocked_post(post_id: PostId, scope: SpaceId) -> bool {
        Self::is_blocked_entity(EntityId::Post(post_id), scope)
    }

    fn is_allowed_post(post_id: PostId, scope: SpaceId) -> bool {
        Self::is_allowed_entity(EntityId::Post(post_id), scope)
    }
}

impl IsAccountBlocked<AccountId> for MockModeration {
    fn is_blocked_account(account: AccountId, scope: SpaceId) -> bool {
        Self::is_blocked_entity(EntityId::Account(account), scope)
    }

    fn is_allowed_account(account: AccountId, scope: SpaceId) -> bool {
        Self::is_allowed_entity(EntityId::Account(account), scope)
    }
}

impl IsSpaceBlocked for MockModeration {
    fn is_blocked_space(space_id: SpaceId, scope: SpaceId) -> bool {
        Self::is_blocked_entity(EntityId::Space(space_id), scope)
    }

    fn is_allowed_space(space_id: SpaceId, scope: SpaceId) -> bool {
        Self::is_allowed_entity(EntityId::Space(space_id), scope)
    }
}

impl IsContentBlocked for MockModeration {
    fn is_blocked_content(content: Content, scope: SpaceId) -> bool {
        Self::is_blocked_entity(EntityId::Content(content), scope)
    }

    fn is_allowed_content(content: Content, scope: SpaceId) -> bool {
        Self::is_allowed_entity(EntityId::Content(content), scope)
    }
}

pub(crate) fn block_account_in_space_1() {
    MockModeration::set_entity_status(
        EntityId::Account(ACCOUNT1),
        SPACE1,
        EntityStatus::Blocked,
    );
}

pub(crate) fn block_content_in_space_1() {
    MockModeration::set_entity_status(
        EntityId::Content(valid_content_ipfs()),
        SPACE1,
        EntityStatus::Blocked,
    );
}
