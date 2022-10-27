use codec::Decode;
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    parameter_types,
    traits::{Currency, Everything},
};
use mockall::mock;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::{
    any::Any,
    convert::{TryFrom, TryInto},
};

use pallet_permissions::SpacePermission;
use subsocial_support::{
    traits::{RolesInterface, SpacesInterface},
    Content,
};

pub(crate) use crate as pallet_subscriptions;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Subscriptions: pallet_subscriptions,
    }
);

pub type AccountId = u64;
pub type Balance = u64;
pub type RoleId = u64;
pub type SpaceId = u64;
pub type BlockNumber = u64;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
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
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

mock! {
    pub Spaces {}
    impl SpacesInterface<AccountId, SpaceId> for Spaces {
        fn get_space_owner(space_id: SpaceId) -> Result<AccountId, DispatchError>;

        fn create_space(owner: &AccountId, content: Content) -> Result<SpaceId, DispatchError>;
    }
}

mock! {
    pub Roles {}
    impl RolesInterface<RoleId, SpaceId, AccountId, SpacePermission, BlockNumber> for Roles {
        fn get_role_space(role_id: RoleId) -> Result<SpaceId, DispatchError>;

        fn grant_role(account_id: AccountId, role_id: RoleId) -> DispatchResult;

        fn create_role(
            space_owner: &AccountId,
            space_id: SpaceId,
            time_to_live: Option<BlockNumber>,
            content: Content,
            permissions: Vec<SpacePermission>,
        ) -> Result<RoleId, DispatchError>;
    }
}

pub struct BenchSpaces;

impl SpacesInterface<AccountId, SpaceId> for BenchSpaces {
    fn get_space_owner(_space_id: SpaceId) -> Result<AccountId, DispatchError> {
        Ok(AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap())
    }

    fn create_space(_owner: &AccountId, _content: Content) -> Result<SpaceId, DispatchError> {
        Ok(101)
    }
}

pub struct BenchRoles;

impl RolesInterface<RoleId, SpaceId, AccountId, SpacePermission, BlockNumber> for BenchRoles {
    fn get_role_space(_role_id: RoleId) -> Result<SpaceId, DispatchError> {
        Ok(101)
    }

    fn grant_role(_account_id: AccountId, _role_id: RoleId) -> DispatchResult {
        Ok(())
    }

    fn create_role(
        _space_owner: &AccountId,
        _space_id: SpaceId,
        _time_to_live: Option<BlockNumber>,
        _content: Content,
        _permissions: Vec<SpacePermission>,
    ) -> Result<RoleId, DispatchError> {
        Ok(111)
    }
}

impl pallet_subscriptions::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type SpaceId = SpaceId;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type SpacesInterface = MockSpaces;
    #[cfg(feature = "runtime-benchmarks")]
    type SpacesInterface = BenchSpaces;
    type RoleId = RoleId;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type RolesInterface = MockRoles;
    #[cfg(feature = "runtime-benchmarks")]
    type RolesInterface = BenchRoles;
    type WeightInfo = pallet_subscriptions::weights::SubstrateWeight<Test>;
}

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
    pub(crate) fn build(self) -> TestExternalities {
        let storage = &mut frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| {
            System::set_block_number(1);
        });

        ext
    }
}
