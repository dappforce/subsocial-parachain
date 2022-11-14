use std::borrow::Borrow;

use codec::Decode;
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    parameter_types,
    traits::{Everything, Get},
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::{TryFrom, TryInto};

use pallet_permissions::SpacePermission;
use subsocial_support::{
    traits::{RolesInterface, SpacesInterface},
    Content,
};

pub(crate) use crate as pallet_subscriptions;
use crate::clearable_parameter_type;

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

pub struct MockSpaces;

parameter_types! {
    pub static get_space_owner__return: Result<AccountId, DispatchError> = Ok(AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap());
    pub static create_space__return: Result<SpaceId, DispatchError> = Ok(101);
}

clearable_parameter_type!(pub static get_space_owner__space_id: SpaceId);
clearable_parameter_type!(pub static create_space__owner: AccountId);
clearable_parameter_type!(pub static create_space__content: Content);


impl SpacesInterface<AccountId, SpaceId> for MockSpaces {
    fn get_space_owner(space_id: SpaceId) -> Result<AccountId, DispatchError> {
        get_space_owner__space_id::set(space_id);
        get_space_owner__return::get()
    }

    fn create_space(owner: &AccountId, content: Content) -> Result<SpaceId, DispatchError> {
        create_space__owner::set(owner.clone());
        create_space__content::set(content.clone());
        create_space__return::get()
    }
}

pub struct MockRoles;

parameter_types! {
    pub static get_role_space__return: Result<SpaceId, DispatchError> = Ok(101);
    pub static grant_role__return: DispatchResult = Ok(());
    pub static create_role__return: Result<RoleId, DispatchError> = Ok(111);
}

clearable_parameter_type!(pub static get_role_space__role_id: RoleId);

clearable_parameter_type!(pub static grant_role__account_id: AccountId);
clearable_parameter_type!(pub static grant_role__role_id: RoleId);

clearable_parameter_type!(pub static grant_role_space__role_id: RoleId);
clearable_parameter_type!(pub static grant_role__owner: AccountId);
clearable_parameter_type!(pub static grant_role__content: Content);

clearable_parameter_type!(pub static create_role__space_owner: RoleId);
clearable_parameter_type!(pub static create_role__space_id: SpaceId);
clearable_parameter_type!(pub static create_role__time_to_live: Option<BlockNumber>);
clearable_parameter_type!(pub static create_role__content: Content);
clearable_parameter_type!(pub static create_role__permissions: Vec<SpacePermission>);

impl RolesInterface<RoleId, SpaceId, AccountId, SpacePermission, BlockNumber> for MockRoles {
    fn get_role_space(role_id: RoleId) -> Result<SpaceId, DispatchError> {
        get_space_owner__space_id::set(role_id);
        get_role_space__return::get()
    }

    fn grant_role(account_id: AccountId, role_id: RoleId) -> DispatchResult {
        grant_role__account_id::set(account_id.clone());
        grant_role__role_id::set(role_id);
        grant_role__return::get()
    }

    fn create_role(
        space_owner: &AccountId,
        space_id: SpaceId,
        time_to_live: Option<BlockNumber>,
        content: Content,
        permissions: Vec<SpacePermission>,
    ) -> Result<RoleId, DispatchError> {
        create_role__space_owner::set(space_owner.clone());
        create_role__space_id::set(space_id);
        create_role__time_to_live::set(time_to_live);
        create_role__content::set(content.clone());
        create_role__permissions::set(permissions.clone());
        create_role__return::get()
    }
}

impl pallet_subscriptions::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type SpaceId = SpaceId;
    type SpacesInterface = MockSpaces;
    type RoleId = RoleId;
    type RolesInterface = MockRoles;
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
