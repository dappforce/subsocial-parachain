use super::*;

use sp_core::H256;
use sp_io::TestExternalities;
use sp_std::{collections::btree_set::BTreeSet, prelude::Vec};

use frame_support::{
    assert_ok,
    dispatch::{DispatchError, DispatchResult},
    parameter_types,
    traits::{ConstU32, Everything},
};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use pallet_permissions::{SpacePermission, SpacePermission as SP, SpacePermissions};
use subsocial_support::{
    traits::{SpaceFollowsProvider, SpacePermissionsProvider as SpacePermissionsProviderT},
    Content, SpaceId, SpacePermissionsInfo, User,
};

use crate as roles;

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
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Roles: roles::{Pallet, Call, Storage, Event<T>},
        Spaces: pallet_spaces::{Pallet, Call, Storage, Event<T>},
    }
);

pub(super) type AccountId = u64;
pub(super) type Balance = u64;
type BlockNumber = u64;

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

use pallet_permissions::default_permissions::DefaultSpacePermissions;

impl pallet_permissions::Config for Test {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
  pub const MaxUsersToProcessPerDeleteRole: u16 = 20;
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    #[cfg(feature = "runtime-benchmarks")]
    type SpacePermissionsProvider = Spaces;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type SpacePermissionsProvider = Self;
    type SpaceFollows = Roles;
    type IsAccountBlocked = ();
    type IsContentBlocked = ();
    type WeightInfo = ();
}

impl pallet_spaces::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Roles = Roles;
    type SpaceFollows = Roles;
    type IsAccountBlocked = ();
    type IsContentBlocked = ();
    type MaxSpacesPerAccount = ConstU32<100>;
    type WeightInfo = ();
}

impl SpacePermissionsProviderT<AccountId, SpacePermissionsInfo<AccountId, SpacePermissions>>
    for Test
{
    // This function should return an error every time Space doesn't exist by SpaceId
    // Currently, we have a list of valid space id's to check
    fn space_permissions_info(
        id: SpaceId,
    ) -> Result<SpacePermissionsInfo<AccountId, SpacePermissions>, DispatchError> {
        if valid_space_ids().contains(&id) {
            return Ok(SpacePermissionsInfo { owner: ACCOUNT1, permissions: None })
        }

        Err("mock:SpaceNotFound".into())
    }

    fn ensure_space_owner(id: SpaceId, account: &AccountId) -> DispatchResult {
        if valid_space_ids().contains(&id) {
            if *account == ACCOUNT1 {
                return Ok(())
            }
        }

        Err("mock:NotSpaceOwner".into())
    }
}

impl<T: Config> SpaceFollowsProvider for Pallet<T> {
    type AccountId = AccountId;

    fn is_space_follower(_account: Self::AccountId, _space_id: u64) -> bool {
        true
    }
}

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> TestExternalities {
        let storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    pub fn build_with_a_few_roles_granted_to_account2() -> TestExternalities {
        let storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
            let user = User::Account(ACCOUNT2);

            assert_ok!(_create_role(None, None, None, None, Some(self::permission_set_random()))); // RoleId 1
            assert_ok!(_create_default_role()); // RoleId 2

            assert_ok!(_grant_role(None, Some(ROLE1), Some(vec![user.clone()])));
            assert_ok!(_grant_role(None, Some(ROLE2), Some(vec![user])));
        });

        ext
    }
}

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const ACCOUNT3: AccountId = 3;

pub(crate) const ROLE1: RoleId = 1;
pub(crate) const ROLE2: RoleId = 2;
pub(crate) const ROLE3: RoleId = 3;
pub(crate) const ROLE4: RoleId = 4;

pub(crate) const SPACE1: SpaceId = 1;
pub(crate) const SPACE2: SpaceId = 2;

pub(crate) fn default_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec())
}

pub(crate) fn updated_role_content_ipfs() -> Content {
    Content::IPFS(b"QmZENA8YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDaazhR8".to_vec())
}

pub(crate) fn invalid_role_content_ipfs() -> Content {
    Content::IPFS(b"QmRAQB6DaazhR8".to_vec())
}

/// Permissions Set that includes next permission: ManageRoles
pub(crate) fn permission_set_default() -> Vec<SpacePermission> {
    vec![SP::ManageRoles]
}

/// Permissions Set that includes next permissions: ManageRoles, CreatePosts
pub(crate) fn permission_set_updated() -> Vec<SpacePermission> {
    vec![SP::ManageRoles, SP::CreatePosts]
}

/// Permissions Set that includes random permissions
pub(crate) fn permission_set_random() -> Vec<SpacePermission> {
    vec![SP::CreatePosts, SP::UpdateOwnPosts, SP::UpdateAnyPost, SP::UpdateEntityStatus]
}

pub(crate) fn valid_space_ids() -> Vec<SpaceId> {
    vec![SPACE1]
}

/// Permissions Set that includes nothing
pub(crate) fn permission_set_empty() -> Vec<SpacePermission> {
    vec![]
}

pub(crate) fn role_update(
    disabled: Option<bool>,
    content: Option<Content>,
    permissions: Option<BTreeSet<SpacePermission>>,
) -> RoleUpdate {
    RoleUpdate { disabled, content, permissions }
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
    Roles::create_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        space_id.unwrap_or(SPACE1),
        time_to_live.unwrap_or_default(), // Should return 'None'
        content.unwrap_or_else(self::default_role_content_ipfs),
        permissions.unwrap_or_else(self::permission_set_default),
    )
}

pub(crate) fn _update_default_role() -> DispatchResult {
    _update_role(None, None, None)
}

pub(crate) fn _update_role(
    origin: Option<RuntimeOrigin>,
    role_id: Option<RoleId>,
    update: Option<RoleUpdate>,
) -> DispatchResult {
    Roles::update_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1),
        update.unwrap_or_else(|| {
            self::role_update(
                Some(true),
                Some(self::updated_role_content_ipfs()),
                Some(self::permission_set_updated().into_iter().collect()),
            )
        }),
    )
}

pub(crate) fn _grant_default_role() -> DispatchResult {
    _grant_role(None, None, None)
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

pub(crate) fn _revoke_default_role() -> DispatchResult {
    _revoke_role(None, None, None)
}

pub(crate) fn _revoke_role(
    origin: Option<RuntimeOrigin>,
    role_id: Option<RoleId>,
    users: Option<Vec<User<AccountId>>>,
) -> DispatchResult {
    Roles::revoke_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        role_id.unwrap_or(ROLE1),
        users.unwrap_or_else(|| vec![User::Account(ACCOUNT2)]),
    )
}

pub(crate) fn _delete_default_role() -> DispatchResult {
    _delete_role(None, None)
}

pub(crate) fn _delete_role(origin: Option<RuntimeOrigin>, role_id: Option<RoleId>) -> DispatchResult {
    let role_id = role_id.unwrap_or(ROLE1);
    Roles::delete_role(
        origin.unwrap_or_else(|| RuntimeOrigin::signed(ACCOUNT1)),
        role_id,
        UsersByRoleId::<Test>::get(role_id).len() as u32,
    )
}
