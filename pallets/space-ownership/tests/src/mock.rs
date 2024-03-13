// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::{pallet_prelude::ConstU32, parameter_types, traits::Everything};
use lazy_static::lazy_static;
use mockall::mock;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::{
    convert::{TryFrom, TryInto},
    sync::{Mutex, MutexGuard},
};

use subsocial_support::{traits::CreatorStakingProvider, SpaceId};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        Permissions: pallet_permissions,
        Roles: pallet_roles,
        Profiles: pallet_profiles,
        SpaceFollows: pallet_space_follows,
        Ownership: pallet_ownership,
        Spaces: pallet_spaces,
        Domains: pallet_domains,
        Posts: pallet_posts,
    }
);

pub(super) type AccountId = u64;
pub(super) type Balance = u64;
pub(super) type BlockNumber = u64;

pub(crate) const DOMAIN_DEPOSIT: Balance = 10;

// Mocks

// CreatorStakingProvider
mock! {
    // This will generate MockSpaces
    pub CreatorStaking {}
    impl CreatorStakingProvider<AccountId> for CreatorStaking {
        fn is_creator_active(creator_id: SpaceId) -> bool;
    }
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
    type DefaultSpacePermissions = pallet_permissions::default_permissions::DefaultSpacePermissions;
}

parameter_types! {
    pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
}

impl pallet_roles::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type SpacePermissionsProvider = Spaces;
    type SpaceFollows = SpaceFollows;
    type IsAccountBlocked = ();
    type IsContentBlocked = ();
    type WeightInfo = ();
}

impl pallet_profiles::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type SpacePermissionsProvider = Spaces;
    type SpacesInterface = Spaces;
    type WeightInfo = ();
}

impl pallet_spaces::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Roles = Roles;
    type SpaceFollows = SpaceFollows;
    type IsAccountBlocked = ();
    type IsContentBlocked = ();
    type MaxSpacesPerAccount = ConstU32<100>;
    type WeightInfo = ();
}

impl pallet_space_follows::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl pallet_posts::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxCommentDepth = ConstU32<10>;
    type IsPostBlocked = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxDomainLength: u32 = 64;
    pub const BaseDomainDeposit: Balance = DOMAIN_DEPOSIT;
    pub const OuterValueByteDeposit: Balance = 5;
    pub const RegistrationPeriodLimit: BlockNumber = 5;
    pub const InitialPaymentBeneficiary: AccountId = 1;
    pub InitialPricesConfig: pallet_domains::types::PricesConfigVec<Test> = vec![(1, 100)];
}

impl pallet_domains::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MinDomainLength = ConstU32<1>;
    type MaxDomainLength = MaxDomainLength;
    type MaxDomainsPerAccount = ConstU32<5>;
    type DomainsInsertLimit = ConstU32<5>;
    type RegistrationPeriodLimit = RegistrationPeriodLimit;
    type MaxOuterValueLength = ConstU32<64>;
    type BaseDomainDeposit = BaseDomainDeposit;
    type OuterValueByteDeposit = OuterValueByteDeposit;
    type InitialPaymentBeneficiary = InitialPaymentBeneficiary;
    type InitialPricesConfig = InitialPricesConfig;
    type WeightInfo = ();
}

impl pallet_ownership::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type ProfileManager = Profiles;
    type SpacesInterface = Spaces;
    type SpacePermissionsProvider = Spaces;
    type CreatorStakingProvider = MockCreatorStaking;
    type DomainsProvider = Domains;
    type PostsProvider = Posts;
    #[cfg(feature = "runtime-benchmarks")]
    type Currency = Balances;
    type WeightInfo = ();
}
