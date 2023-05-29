use frame_support::{assert_ok, dispatch::DispatchResult, parameter_types, traits::Everything};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError,
};
use sp_std::convert::{TryFrom, TryInto};

use pallet_permissions::{
    default_permissions::DefaultSpacePermissions, PermissionChecker, SpacePermission,
    SpacePermissionsContext,
};
use pallet_spaces::NextSpaceId;
use subsocial_support::{traits::SpaceFollowsProvider, Content, SpaceId, User};

pub(crate) use crate as pallet_resource_discussions;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(super) type AccountId = u64;
pub(super) type Balance = u64;
type BlockNumber = u64;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Posts: pallet_posts,
        Spaces: pallet_spaces,
        SpaceFollows: pallet_space_follows,
        Permissions: pallet_permissions,
        ResourceDiscussions: pallet_resource_discussions,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = BlockNumber;
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
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
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

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
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

impl pallet_space_follows::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

parameter_types! {
  pub const MaxCommentDepth: u32 = 10;
}

impl pallet_posts::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxCommentDepth = MaxCommentDepth;
    type IsPostBlocked = ();
    type WeightInfo = ();
}

pub struct FakeImpls;

impl PermissionChecker for FakeImpls {
    type AccountId = AccountId;

    fn ensure_user_has_space_permission(
        user: User<Self::AccountId>,
        ctx: SpacePermissionsContext,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {
        Ok(())
    }
}

impl SpaceFollowsProvider for FakeImpls {
    type AccountId = AccountId;

    fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool {
        false
    }
}

parameter_types! {
    pub const MaxSpacesPerAccount: u32 = 4096;
}

impl pallet_spaces::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Roles = FakeImpls;
    type SpaceFollows = FakeImpls;
    type IsAccountBlocked = ();
    type IsContentBlocked = ();
    type MaxSpacesPerAccount = MaxSpacesPerAccount;
    type WeightInfo = ();
}

parameter_types! {
    pub static ResourcesSpaceId: SpaceId = 0;
    pub static MaxResourcesIdLength: u32 = 100;
}

impl pallet_resource_discussions::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxResourceIdLength = MaxResourcesIdLength;
}

pub struct ExtBuilder {
    max_resource_id_length: u32,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder { max_resource_id_length: 100 }
    }
}

impl ExtBuilder {
    pub(crate) fn max_resource_id_length(mut self, max_resource_id_length: u32) -> Self {
        self.max_resource_id_length = max_resource_id_length;
        self
    }

    fn set_configs(&self) {
        MAX_RESOURCES_ID_LENGTH.with(|x| *x.borrow_mut() = self.max_resource_id_length);
    }

    pub(crate) fn build(self) -> TestExternalities {
        self.set_configs();

        let storage = &mut frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| {
            System::set_block_number(1);
        });

        ext
    }
}
