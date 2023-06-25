// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use crate as pallet_profiles;
use frame_support::{dispatch::DispatchResult, parameter_types, traits::Everything};
use frame_system as system;
use lazy_static::lazy_static;
use mockall::mock;
use pallet_permissions::{default_permissions::DefaultSpacePermissions, SpacePermissionsInfoOf};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError,
};
use sp_std::sync::{Mutex, MutexGuard};
use subsocial_support::{
    traits::{SpacePermissionsProvider, SpacesInterface},
    Content, SpaceId,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Permissions: pallet_permissions,
        Profiles: pallet_profiles,
    }
);

pub(crate) type AccountId = u64;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
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
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_permissions::Config for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

mock! {
    // This will generate MockSpaces
    pub Spaces {}
    impl SpacePermissionsProvider<AccountId, SpacePermissionsInfoOf<Test>> for Spaces {
        fn space_permissions_info(id: SpaceId) -> Result<SpacePermissionsInfoOf<Test>, DispatchError>;

        fn ensure_space_owner(id: SpaceId, account: &AccountId) -> DispatchResult;
    }

    impl SpacesInterface<AccountId, SpaceId> for Spaces {
        fn get_space_owner(_space_id: SpaceId) -> Result<AccountId, DispatchError>;

        fn create_space(_owner: &AccountId, _content: Content) -> Result<SpaceId, DispatchError>;
    }
}

impl pallet_profiles::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type SpacePermissionsProvider = MockSpaces;
    type SpacesInterface = MockSpaces;
    type WeightInfo = ();
}

lazy_static! {
    static ref MTX: Mutex<()> = Mutex::new(());
}

// mockall crate requires synchronized access for the mocking of static methods.
pub(super) fn use_static_mock() -> MutexGuard<'static, ()> {
    match MTX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub(super) struct ExtBuilder;
impl ExtBuilder {
    /// Default ext configuration with BlockNumber 1
    pub fn build() -> TestExternalities {
        let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }
}
