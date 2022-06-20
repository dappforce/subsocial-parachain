use sp_core::H256;
use sp_io::TestExternalities;

use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    testing::Header,
    Storage,
};

use frame_support::{
    assert_ok,
    parameter_types,
    storage::StorageMap,
    traits::Everything,
};
use frame_system as system;

use pallet_permissions::{
    SpacePermission as SP,
    SpacePermissions,
};
use pallet_posts::{Post, PostUpdate, PostExtension, Comment, Error as PostsError};
use pallet_profiles::{ProfileUpdate, Error as ProfilesError};
use pallet_profile_follows::Error as ProfileFollowsError;
use pallet_reactions::{ReactionId, ReactionKind, Error as ReactionsError};
use pallet_spaces::{SpaceById, SpaceUpdate, Error as SpacesError, SpacesSettings};
use pallet_space_follows::Error as SpaceFollowsError;
use pallet_space_ownership::Error as SpaceOwnershipError;
use pallet_moderation::{EntityId, EntityStatus, ReportId};
use pallet_utils::{
    mock_functions::*,
    DEFAULT_MIN_HANDLE_LEN, DEFAULT_MAX_HANDLE_LEN,
    Error as UtilsError,
    SpaceId, PostId, User, Content,
};
use crate::utils::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

frame_support::construct_runtime!(
        pub enum TestRuntime where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: system::{Pallet, Call, Config, Storage, Event<T>},
            Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
            Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
            Moderation: pallet_moderation::{Pallet, Call, Storage, Event<T>},
            Permissions: pallet_permissions::{Pallet, Call},
            Posts: pallet_posts::{Pallet, Call, Storage, Event<T>},
            PostHistory: pallet_post_history::{Pallet, Storage},
            ProfileFollows: pallet_profile_follows::{Pallet, Call, Storage, Event<T>},
            Profiles: pallet_profiles::{Pallet, Call, Storage, Event<T>},
            ProfileHistory: pallet_profile_history::{Pallet, Storage},
            Reactions: pallet_reactions::{Pallet, Call, Storage, Event<T>},
            Roles: pallet_roles::{Pallet, Call, Storage, Event<T>},
            SpaceFollows: pallet_space_follows::{Pallet, Call, Storage, Event<T>},
            SpaceHistory: pallet_space_history::{Pallet, Storage},
            SpaceOwnership: pallet_space_ownership::{Pallet, Call, Storage, Event<T>},
            Spaces: pallet_spaces::{Pallet, Call, Storage, Event<T>, Config<T>},
            Utils: pallet_utils::{Pallet, Storage, Event<T>, Config<T>},
        }
    );

parameter_types! {
        pub const BlockHashCount: u64 = 250;
    }

impl system::Config for TestRuntime {
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
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_types! {
        pub const MinimumPeriod: u64 = 5;
    }

impl pallet_timestamp::Config for TestRuntime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
    }

impl pallet_balances::Config for TestRuntime {
    type Balance = u64;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

parameter_types! {
      pub const MinHandleLen: u32 = DEFAULT_MIN_HANDLE_LEN;
      pub const MaxHandleLen: u32 = DEFAULT_MAX_HANDLE_LEN;
    }

impl pallet_utils::Config for TestRuntime {
    type Event = Event;
    type Currency = Balances;
    type MinHandleLen = MinHandleLen;
    type MaxHandleLen = MaxHandleLen;
}

use pallet_permissions::default_permissions::DefaultSpacePermissions;

impl pallet_permissions::Config for TestRuntime {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
        pub const MaxCommentDepth: u32 = 10;
    }

impl pallet_posts::Config for TestRuntime {
    type Event = Event;
    type MaxCommentDepth = MaxCommentDepth;
    type AfterPostUpdated = PostHistory;
    type IsPostBlocked = Moderation;
}

impl pallet_post_history::Config for TestRuntime {}

impl pallet_profile_follows::Config for TestRuntime {
    type Event = Event;
    type BeforeAccountFollowed = ();
    type BeforeAccountUnfollowed = ();
}

impl pallet_profiles::Config for TestRuntime {
    type Event = Event;
    type AfterProfileUpdated = ProfileHistory;
}

impl pallet_profile_history::Config for TestRuntime {}

impl pallet_reactions::Config for TestRuntime {
    type Event = Event;
}

parameter_types! {
        pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
    }

impl pallet_roles::Config for TestRuntime {
    type Event = Event;
    type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
    type Spaces = Spaces;
    type SpaceFollows = SpaceFollows;
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
}

impl pallet_space_follows::Config for TestRuntime {
    type Event = Event;
    type BeforeSpaceFollowed = ();
    type BeforeSpaceUnfollowed = ();
}

impl pallet_space_ownership::Config for TestRuntime {
    type Event = Event;
}

pub(crate) const HANDLE_DEPOSIT: u64 = 15;

parameter_types! {
        pub const HandleDeposit: u64 = HANDLE_DEPOSIT;
    }

impl pallet_spaces::Config for TestRuntime {
    type Event = Event;
    type Currency = Balances;
    type Roles = Roles;
    type SpaceFollows = SpaceFollows;
    type BeforeSpaceCreated = SpaceFollows;
    type AfterSpaceUpdated = SpaceHistory;
    type IsAccountBlocked = Moderation;
    type IsContentBlocked = Moderation;
    type HandleDeposit = HandleDeposit;
}

impl pallet_space_history::Config for TestRuntime {}

parameter_types! {
        pub const DefaultAutoblockThreshold: u16 = 20;
    }

impl pallet_moderation::Config for TestRuntime {
    type Event = Event;
    type DefaultAutoblockThreshold = DefaultAutoblockThreshold;
}

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;


pub struct ExtBuilder;

// TODO: refactor
use crate::utils::posts_utils::*;
use crate::utils::spaces_utils::*;
use crate::utils::roles_utils::*;
use crate::utils::space_ownership_utils::*;
use crate::utils::reactions_utils::*;
use crate::utils::space_follows_utils::*;


// TODO: make created space/post/comment configurable or by default
impl ExtBuilder {
    fn configure_storages(storage: &mut Storage) {
        let mut accounts = Vec::new();
        for account in ACCOUNT1..=ACCOUNT3 {
            accounts.push(account);
        }

        let _ = pallet_balances::GenesisConfig::<TestRuntime> {
            balances: accounts.iter().cloned().map(|k|(k, 100)).collect()
        }.assimilate_storage(storage);
    }

    /// Default ext configuration with BlockNumber 1
    pub fn build() -> TestExternalities {
        let mut storage = system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        Self::configure_storages(&mut storage);

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    fn add_default_space() {
        assert_ok!(_create_default_space());
    }

    fn add_space_with_custom_permissions(permissions: SpacePermissions) {
        assert_ok!(_create_space(None, None, None, Some(Some(permissions))));
    }

    fn add_space_with_no_handle() {
        assert_ok!(_create_space(None, Some(None), None, None));
    }

    fn add_post() {
        Self::add_default_space();
        assert_ok!(_create_default_post());
    }

    fn add_comment() {
        Self::add_post();
        assert_ok!(_create_default_comment());
    }

    /// Custom ext configuration with SpaceId 1 and BlockNumber 1
    pub fn build_with_space() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(Self::add_default_space);
        ext
    }

    /// Custom ext configuration with SpaceId 1, PostId 1 and BlockNumber 1
    pub fn build_with_post() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(Self::add_post);
        ext
    }

    /// Custom ext configuration with SpaceId 1, PostId 1, PostId 2 (as comment) and BlockNumber 1
    pub fn build_with_comment() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(Self::add_comment);
        ext
    }

    /// Custom ext configuration with SpaceId 1-2, PostId 1 where BlockNumber 1
    pub fn build_with_post_and_two_spaces() -> TestExternalities {
        let mut ext = Self::build_with_post();
        ext.execute_with(Self::add_space_with_no_handle);
        ext
    }

    /// Custom ext configuration with SpaceId 1, PostId 1 and ReactionId 1 (on post) where BlockNumber is 1
    pub fn build_with_reacted_post_and_two_spaces() -> TestExternalities {
        let mut ext = Self::build_with_post_and_two_spaces();
        ext.execute_with(|| { assert_ok!(_create_default_post_reaction()); });
        ext
    }

    /// Custom ext configuration with pending ownership transfer without Space
    pub fn build_with_pending_ownership_transfer_no_space() -> TestExternalities {
        let mut ext = Self::build_with_space();
        ext.execute_with(|| {
            assert_ok!(_transfer_default_space_ownership());
            <SpaceById<TestRuntime>>::remove(SPACE1);
        });
        ext
    }

    /// Custom ext configuration with specified permissions granted (includes SpaceId 1)
    pub fn build_with_a_few_roles_granted_to_account2(perms: Vec<SP>) -> TestExternalities {
        let mut ext = Self::build_with_space();

        ext.execute_with(|| {
            let user = User::Account(ACCOUNT2);
            assert_ok!(_create_role(
                    None,
                    None,
                    None,
                    None,
                    Some(perms)
                ));
            // RoleId 1
            assert_ok!(_create_default_role()); // RoleId 2

            assert_ok!(_grant_role(None, Some(ROLE1), Some(vec![user.clone()])));
            assert_ok!(_grant_role(None, Some(ROLE2), Some(vec![user])));
        });

        ext
    }

    /// Custom ext configuration with space follow without Space
    pub fn build_with_space_follow_no_space() -> TestExternalities {
        let mut ext = Self::build_with_space();

        ext.execute_with(|| {
            assert_ok!(_default_follow_space());
            <SpaceById<TestRuntime>>::remove(SPACE1);
        });

        ext
    }

    /// Custom ext configuration with a space and override the space permissions
    pub fn build_with_space_and_custom_permissions(permissions: SpacePermissions) -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(|| Self::add_space_with_custom_permissions(permissions));
        ext
    }

    /// Custom ext configuration with SpaceId 1, BlockNumber 1, and disable handles
    pub fn build_with_space_then_disable_handles() -> TestExternalities {
        let mut ext = Self::build_with_space();
        ext.execute_with(|| {
            assert_ok!(_update_space_settings_with_handles_disabled());
        });
        ext
    }
}