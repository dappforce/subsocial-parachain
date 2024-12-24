// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::{pallet_prelude::ConstU32, parameter_types, traits::Everything};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::{TryFrom, TryInto};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        Permissions: pallet_permissions,
        Roles: pallet_roles,
        SpaceFollows: pallet_space_follows,
        Posts: pallet_posts,
        Reactions: pallet_reactions,
        Spaces: pallet_spaces,
    }
);

pub(super) type AccountId = u64;
pub(super) type Balance = u64;

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
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
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
    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type MaxHolds = ();
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

impl pallet_reactions::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}
