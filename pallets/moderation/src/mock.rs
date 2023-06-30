use super::*;

use crate as moderation;

use frame_support::{assert_ok, dispatch::DispatchResult, parameter_types, StorageMap, traits::Everything};
use frame_system as system;

use sp_core::{ConstU32, H256};
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::TryInto;
use sp_std::convert::TryFrom;
use pallet_permissions::{
    SpacePermission as SP,
    default_permissions::DefaultSpacePermissions,
};
use pallet_posts::PostExtension;
use pallet_roles::RoleId;
use pallet_spaces::{types::RESERVED_SPACE_COUNT, SpaceById};
use subsocial_support::User;
use subsocial_support::mock_functions::valid_content_ipfs;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Moderation: moderation::{Pallet, Call, Storage, Event<T>},
		Posts: pallet_posts::{Pallet, Call, Storage, Event<T>},
		Roles: pallet_roles::{Pallet, Call, Storage, Event<T>},
		SpaceFollows: pallet_space_follows::{Pallet, Call, Storage, Event<T>},
		Spaces: pallet_spaces::{Pallet, Call, Storage, Event<T>, Config<T>},
        Timestamp: pallet_timestamp,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

impl pallet_permissions::Config for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl pallet_spaces::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Roles = Roles;
    type SpaceFollows = SpaceFollows;
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
    type MaxSpacesPerAccount = ConstU32<200>;
    type WeightInfo = ();
}

impl pallet_space_follows::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxCommentDepth: u32 = 10;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_posts::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxCommentDepth = MaxCommentDepth;
    type IsPostBlocked = Moderation;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
}

impl pallet_roles::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type SpaceFollows = SpaceFollows;
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
    type SpacePermissionsProvider = Spaces;
    type WeightInfo = ();
}

parameter_types! {
    pub const DefaultAutoblockThreshold: u16 = 3;
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type DefaultAutoblockThreshold = DefaultAutoblockThreshold;
}

pub(crate) type AccountId = u64;

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    pub fn build_with_space_and_post() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
            create_space_and_post();
        });

        ext
    }

    pub fn build_with_space_and_post_then_report() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);

            create_space_and_post();
            assert_ok!(_report_default_post());
        });

        ext
    }

    pub fn build_with_report_then_remove_scope() -> TestExternalities {
        let storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);

            create_space_and_post();
            assert_ok!(_report_default_post());

            SpaceById::<Test>::remove(SPACE1);
        });

        ext
    }

    pub fn build_with_report_then_grant_role_to_suggest_entity_status() -> TestExternalities {
        let mut ext = Self::build_with_space_and_post_then_report();

        ext.execute_with(|| {
            // Create a new role for moderators:
            assert_ok!(Roles::create_role(
                RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER),
                SPACE1,
                None,
                default_role_content_ipfs(),
                vec![SP::SuggestEntityStatus],
            ));

            // Allow the moderator accounts to suggest entity statuses:
            let mods = moderators().into_iter().map(User::Account).collect();
            assert_ok!(Roles::grant_role(
                RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER),
                MODERATOR_ROLE_ID,
                mods
            ));
        });

        ext
    }
}

pub(crate) const ACCOUNT_SCOPE_OWNER: AccountId = 1;
pub(crate) const ACCOUNT_NOT_MODERATOR: AccountId = 2;
pub(crate) const FIRST_MODERATOR_ID: AccountId = 100;

pub(crate) const SPACE1: SpaceId = RESERVED_SPACE_COUNT + 1;
pub(crate) const SPACE2: SpaceId = SPACE1 + 1;

pub(crate) const POST1: PostId = 1;

pub(crate) const REPORT1: ReportId = 1;
pub(crate) const REPORT2: ReportId = 2;

pub(crate) const MODERATOR_ROLE_ID: RoleId = 1;

pub(crate) const AUTOBLOCK_THRESHOLD: u16 = 5;

pub(crate) const fn new_autoblock_threshold() -> SpaceModerationSettingsUpdate {
    SpaceModerationSettingsUpdate {
        autoblock_threshold: Some(Some(AUTOBLOCK_THRESHOLD))
    }
}

pub(crate) const fn empty_moderation_settings_update() -> SpaceModerationSettingsUpdate {
    SpaceModerationSettingsUpdate {
        autoblock_threshold: None
    }
}

pub(crate) fn moderators() -> Vec<AccountId> {
    let first_mod_id = FIRST_MODERATOR_ID;
    let last_mod_id = first_mod_id + DefaultAutoblockThreshold::get() as u64 + 2;
    (first_mod_id..last_mod_id).collect()
}

// TODO: replace with common function when benchmarks PR is merged
// TODO: replace with common function when benchmarks PR is merged
pub(crate) fn default_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

pub(crate) fn create_space_and_post() {
    assert_ok!(Spaces::create_space(
        RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER),
        Content::None,
        None
    ));

    assert_ok!(Posts::create_post(
        RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER),
        Some(SPACE1),
        PostExtension::RegularPost,
        valid_content_ipfs(),
    ));
}

pub(crate) fn _report_default_post() -> DispatchResult {
    _report_entity(None, None, None, None)
}

pub(crate) fn _report_entity(
    origin: Option<RuntimeOrigin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    reason: Option<Content>,
) -> DispatchResult {
    Moderation::report_entity(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        reason.unwrap_or_else(valid_content_ipfs),
    )
}

pub(crate) fn _suggest_blocked_status_for_post() -> DispatchResult {
    _suggest_entity_status(None, None, None, None, None)
}

/// By default (when all options are `None`) makes ACCOUNT1 to suggest 'Blocked' status to the POST1
pub(crate) fn _suggest_entity_status(
    origin: Option<RuntimeOrigin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    status: Option<Option<EntityStatus>>,
    report_id_opt: Option<Option<ReportId>>,
) -> DispatchResult {
    Moderation::suggest_entity_status(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        status.unwrap_or(Some(EntityStatus::Blocked)),
        report_id_opt.unwrap_or(Some(REPORT1)),
    )
}

pub(crate) fn _update_post_status_to_allowed() -> DispatchResult {
    _update_entity_status(None, None, None, None)
}

pub(crate) fn _update_entity_status(
    origin: Option<RuntimeOrigin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
    status_opt: Option<Option<EntityStatus>>,
) -> DispatchResult {
    Moderation::update_entity_status(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
        status_opt.unwrap_or(Some(EntityStatus::Allowed)),
    )
}

pub(crate) fn _delete_post_status() -> DispatchResult {
    _delete_entity_status(None, None, None)
}

pub(crate) fn _delete_entity_status(
    origin: Option<RuntimeOrigin>,
    entity: Option<EntityId<AccountId>>,
    scope: Option<SpaceId>,
) -> DispatchResult {
    Moderation::delete_entity_status(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER)),
        entity.unwrap_or(EntityId::Post(POST1)),
        scope.unwrap_or(SPACE1),
    )
}

pub(crate) fn _update_autoblock_threshold_in_moderation_settings() -> DispatchResult {
    _update_moderation_settings(None, None, None)
}

pub(crate) fn _update_moderation_settings(
    origin: Option<RuntimeOrigin>,
    space_id: Option<SpaceId>,
    settings_update: Option<SpaceModerationSettingsUpdate>,
) -> DispatchResult {
    Moderation::update_moderation_settings(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT_SCOPE_OWNER)),
        space_id.unwrap_or(SPACE1),
        settings_update.unwrap_or_else(new_autoblock_threshold),
    )
}