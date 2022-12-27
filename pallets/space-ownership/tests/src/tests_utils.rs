use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
};

use frame_support::{assert_ok, pallet_prelude::*};
use sp_core::storage::Storage;
use sp_io::TestExternalities;

use pallet_permissions::SpacePermissions;
use pallet_spaces::*;
use subsocial_support::{
    traits::{IsAccountBlocked, IsContentBlocked, IsPostBlocked, IsSpaceBlocked},
    Content, PostId, SpaceId,
};

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

    /// Custom ext configuration with SpaceId 1 and BlockNumber 1
    pub fn build_with_space() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(Self::add_default_space);
        ext
    }

    /// Custom ext configuration with pending ownership transfer without Space
    pub fn build_with_pending_ownership_transfer_no_space() -> TestExternalities {
        let mut ext = Self::build_with_space();
        ext.execute_with(|| {
            assert_ok!(_transfer_default_space_ownership());
            <SpaceById<Test>>::remove(SPACE1);
        });
        ext
    }
}

////// Consts

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const ACCOUNT3: AccountId = 3;

pub(crate) const SPACE1: SpaceId = 1001;

////// Moderation Utils

// Moderation pallet mocks

/* ------------------------------------------------------------------------------------------------ */
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
    fn get_entity_status(id: EntityId, scope: SpaceId) -> Option<EntityStatus> {
        MOCK_MODERATION_STATE.with(|mock_moderation_state| {
            let mock_moderation_state = mock_moderation_state.borrow();
            let status = mock_moderation_state.get(&(id, scope)).cloned();
            status
        })
    }

    fn is_allowed_entity(id: EntityId, scope: SpaceId) -> bool {
        Self::get_entity_status(id, scope).unwrap_or(EntityStatus::Allowed) == EntityStatus::Allowed
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

///////////// Space Utils

pub(crate) fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

pub(crate) fn _create_default_space() -> DispatchResultWithPostInfo {
    _create_space(None, None, None)
}

pub(crate) fn _create_space(
    origin: Option<Origin>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>,
) -> DispatchResultWithPostInfo {
    _create_space_with_parent_id(origin, content, permissions)
}

pub(crate) fn _create_subspace(
    origin: Option<Origin>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>,
) -> DispatchResultWithPostInfo {
    _create_space_with_parent_id(origin, content, permissions)
}

pub(crate) fn _create_space_with_parent_id(
    origin: Option<Origin>,
    content: Option<Content>,
    permissions: Option<Option<SpacePermissions>>,
) -> DispatchResultWithPostInfo {
    Spaces::create_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        content.unwrap_or_else(space_content_ipfs),
        permissions.unwrap_or_default(),
    )
}

//// Space follows utils

pub(crate) fn _default_follow_space() -> DispatchResult {
    _follow_space(None, None)
}

pub(crate) fn _follow_space(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
    SpaceFollows::follow_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}

pub(crate) fn _default_unfollow_space() -> DispatchResult {
    _unfollow_space(None, None)
}

pub(crate) fn _unfollow_space(origin: Option<Origin>, space_id: Option<SpaceId>) -> DispatchResult {
    SpaceFollows::unfollow_space(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}

//////// Space ownership utils

pub(crate) fn _transfer_default_space_ownership() -> DispatchResult {
    _transfer_space_ownership(None, None, None)
}

pub(crate) fn _transfer_space_ownership(
    origin: Option<Origin>,
    space_id: Option<SpaceId>,
    transfer_to: Option<AccountId>,
) -> DispatchResult {
    SpaceOwnership::transfer_space_ownership(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        transfer_to.unwrap_or(ACCOUNT2),
    )
}

pub(crate) fn _accept_default_pending_ownership() -> DispatchResult {
    _accept_pending_ownership(None, None)
}

pub(crate) fn _accept_pending_ownership(
    origin: Option<Origin>,
    space_id: Option<SpaceId>,
) -> DispatchResult {
    SpaceOwnership::accept_pending_ownership(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}

pub(crate) fn _reject_default_pending_ownership() -> DispatchResult {
    _reject_pending_ownership(None, None)
}

pub(crate) fn _reject_default_pending_ownership_by_current_owner() -> DispatchResult {
    _reject_pending_ownership(Some(Origin::signed(ACCOUNT1)), None)
}

pub(crate) fn _reject_pending_ownership(
    origin: Option<Origin>,
    space_id: Option<SpaceId>,
) -> DispatchResult {
    SpaceOwnership::reject_pending_ownership(
        origin.unwrap_or_else(|| Origin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}
