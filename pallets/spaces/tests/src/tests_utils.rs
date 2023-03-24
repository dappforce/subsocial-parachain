use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
};

use frame_support::{assert_ok, pallet_prelude::*};
use sp_core::storage::Storage;
use sp_io::TestExternalities;

use pallet_permissions::{
    default_permissions::DefaultSpacePermissions, SpacePermission as SP, SpacePermission,
    SpacePermissions,
};
use pallet_posts::PostExtension;
use pallet_spaces::types::SpaceUpdate;
use subsocial_support::{
    mock_functions::valid_content_ipfs,
    traits::{IsAccountBlocked, IsContentBlocked, IsPostBlocked, IsSpaceBlocked},
    Content, PostId, SpaceId, User,
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

    fn add_space_with_custom_permissions(permissions: SpacePermissions) {
        assert_ok!(_create_space(None, None, Some(Some(permissions))));
    }

    /// Custom ext configuration with SpaceId 1 and BlockNumber 1
    pub fn build_with_space() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(Self::add_default_space);
        ext
    }

    /// Custom ext configuration with specified permissions granted (includes SpaceId 1)
    pub fn build_with_a_few_roles_granted_to_account2(perms: Vec<SP>) -> TestExternalities {
        let mut ext = Self::build_with_space();

        ext.execute_with(|| {
            let user = User::Account(ACCOUNT2);
            assert_ok!(_create_default_role_with_permissions(perms)); // RoleId 1
            assert_ok!(_create_default_role()); // RoleId 2

            assert_ok!(_grant_role(None, Some(ROLE1), Some(vec![user.clone()])));
            assert_ok!(_grant_role(None, Some(ROLE2), Some(vec![user])));
        });

        ext
    }

    /// Custom ext configuration with a space and override the space permissions
    pub fn build_with_space_and_custom_permissions(
        permissions: SpacePermissions,
    ) -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(|| Self::add_space_with_custom_permissions(permissions));
        ext
    }
}

////// Consts

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const ACCOUNT3: AccountId = 3;

pub(crate) const SPACE1: SpaceId = 1001;
pub(crate) const SPACE2: SpaceId = 1002;

type RoleId = u64;

pub(crate) const ROLE1: RoleId = 1;
pub(crate) const ROLE2: RoleId = 2;

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
    fn set_entity_status(entity: EntityId, space: SpaceId, status: EntityStatus) {
        MOCK_MODERATION_STATE.with(|mock_moderation_state| {
            let mut mock_moderation_state = mock_moderation_state.borrow_mut();
            mock_moderation_state.insert((entity, space), status);
        });
    }

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

pub(crate) fn block_account_in_space_1() {
    MockModeration::set_entity_status(EntityId::Account(ACCOUNT1), SPACE1, EntityStatus::Blocked);
}

pub(crate) fn block_content_in_space_1() {
    MockModeration::set_entity_status(
        EntityId::Content(valid_content_ipfs()),
        SPACE1,
        EntityStatus::Blocked,
    );
}

///////////// Space Utils

pub(crate) fn space_content_ipfs() -> Content {
    Content::IPFS(b"bafyreib3mgbou4xln42qqcgj6qlt3cif35x4ribisxgq7unhpun525l54e".to_vec())
}

pub(crate) fn updated_space_content() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW2CuDgwxkD4".to_vec())
}

pub(crate) fn update_for_space_content(new_content: Content) -> SpaceUpdate {
    space_update(Some(new_content), None)
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

//// Space follows utils

pub(crate) fn _default_follow_space() -> DispatchResult {
    _follow_space(None, None)
}

pub(crate) fn _follow_space(origin: Option<RuntimeOrigin>, space_id: Option<SpaceId>) -> DispatchResult {
    SpaceFollows::follow_space(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT2)),
        space_id.unwrap_or(SPACE1),
    )
}

/////// Roles utils

pub(crate) fn default_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

pub(crate) fn _create_default_role() -> DispatchResult {
    _create_role(None, None, None, None, None)
}

pub(crate) fn _create_role(
    origin: Option<RuntimeOrigin>,
    space_id: Option<SpaceId>,
    time_to_live: Option<Option<BlockNumber>>,
    content: Option<Content>,
    permissions: Option<Vec<SpacePermission>>,
) -> DispatchResult {
    // TODO: remove
    Roles::create_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        time_to_live.unwrap_or_default(), // Should return 'None'
        content.unwrap_or_else(default_role_content_ipfs),
        permissions.unwrap_or_else(permission_set_default),
    )
}

pub(crate) fn _create_default_role_with_permissions(permissions: Vec<SpacePermission>) -> DispatchResult {
    _create_role(None, None, None, None, Some(permissions))
}

pub(crate) fn _grant_role(
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

pub(crate) fn _delete_default_role() -> DispatchResult {
    _delete_role(None, None)
}

pub(crate) fn _delete_role(origin: Option<RuntimeOrigin>, role_id: Option<RoleId>) -> DispatchResult {
    let users_count = Roles::users_by_role_id(role_id.unwrap_or(ROLE1)).len();
    Roles::delete_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1),
        users_count as u32,
    )
}

/////// Permissions utils

pub(crate) fn permissions_where_everyone_can_create_post() -> SpacePermissions {
    let mut default_permissions = DefaultSpacePermissions::get();
    default_permissions.everyone = default_permissions.everyone.map(|mut permissions| {
        permissions.insert(SP::CreatePosts);
        permissions
    });

    default_permissions
}

pub(crate) fn permissions_where_follower_can_create_post() -> SpacePermissions {
    let mut default_permissions = DefaultSpacePermissions::get();
    default_permissions.follower = Some(vec![SP::CreatePosts].into_iter().collect());

    default_permissions
}

/// Permissions Set that includes next permission: ManageRoles
pub(crate) fn permission_set_default() -> Vec<SpacePermission> {
    vec![SP::ManageRoles]
}
