use crate::{self as pallet_creator_staking};

use frame_support::{
    construct_runtime, parameter_types,
    traits::{Currency, OnFinalize, OnInitialize},
    dispatch::DispatchResult,
    weights::Weight,
    PalletId,
};

use lazy_static::lazy_static;
use mockall::mock;

use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{DispatchError, Perbill, testing::Header, traits::{BlakeTwo256, ConstU32, IdentityLookup}};
use sp_std::sync::{Mutex, MutexGuard};

pub(crate) type AccountId = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;
pub(crate) type EraIndex = u32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

/// Value shouldn't be less than 2 for testing purposes, otherwise we cannot test certain corner cases.
pub(crate) const EXISTENTIAL_DEPOSIT: Balance = 2;
pub(crate) const MAX_NUMBER_OF_BACKERS: u32 = 4;
/// Value shouldn't be less than 2 for testing purposes, otherwise we cannot test certain corner cases.
pub(crate) const MINIMUM_STAKING_AMOUNT: Balance = 10;
pub(crate) const MINIMUM_REMAINING_AMOUNT: Balance = 1;
pub(crate) const MAX_UNLOCKING_CHUNKS: u32 = 5;
pub(crate) const UNBONDING_PERIOD: EraIndex = 3;
pub(crate) const MAX_ERA_STAKE_VALUES: u32 = 8;

// Do note that this needs to at least be 3 for tests to be valid. It can be greater but not smaller.
pub(crate) const BLOCKS_PER_ERA: BlockNumber = 3;

pub(crate) const REGISTER_DEPOSIT: Balance = 10;

pub(crate) const BACKER_BLOCK_REWARD: Balance = 531911;
pub(crate) const CREATOR_BLOCK_REWARD: Balance = 773333;
pub(crate) const BLOCKS_PER_YEAR: BlockNumber = 2628000;
pub(crate) const TREASURY_ACCOUNT: BlockNumber = 42;
// A fairly high block reward so we can detect slight changes in reward distribution
// due to TVL changes.
pub(crate) const BLOCK_REWARD: Balance = 1_000_000;

construct_runtime!(
    pub struct TestRuntime
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        CreatorStaking: pallet_creator_staking,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(Weight::from_ref_time(1024));
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type RuntimeCall = RuntimeCall;
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
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MaxLocks: u32 = 4;
    pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for TestRuntime {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 3;
}

impl pallet_timestamp::Config for TestRuntime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

use pallet_permissions::default_permissions::DefaultSpacePermissions;
use pallet_permissions::SpacePermissionsInfoOf;
use subsocial_support::{Content, SpaceId};
use subsocial_support::traits::{SpacePermissionsProvider, SpacesInterface};

impl pallet_permissions::Config for TestRuntime {
    type DefaultSpacePermissions = DefaultSpacePermissions;
}

parameter_types! {
    pub const CreatorRegistrationDeposit: Balance = REGISTER_DEPOSIT;
    pub const BlockPerEra: BlockNumber = BLOCKS_PER_ERA;
    pub const MaxNumberOfBackersPerCreator: u32 = MAX_NUMBER_OF_BACKERS;
    pub const MinimumStake: Balance = MINIMUM_STAKING_AMOUNT;
    pub const CreatorStakingPalletId: PalletId = PalletId(*b"mokcrstk");
    pub const MinimumRemainingFreeBalance: Balance = MINIMUM_REMAINING_AMOUNT;
    #[derive(PartialEq)]
    pub const MaxUnlockingChunks: u32 = MAX_UNLOCKING_CHUNKS;
    pub const UnbondingPeriodInEras: EraIndex = UNBONDING_PERIOD;
    pub const MaxEraStakeItems: u32 = MAX_ERA_STAKE_VALUES;
    pub const AnnualInflation: Perbill = Perbill::from_percent(10);
    pub const BlocksPerYear: BlockNumber = BLOCKS_PER_YEAR;
    pub const TreasuryAccount: AccountId = TREASURY_ACCOUNT;
}

mock! {
    // This will generate MockSpaces
    pub Spaces {}
    impl SpacePermissionsProvider<AccountId, SpacePermissionsInfoOf<TestRuntime>> for Spaces {
        fn space_permissions_info(id: SpaceId) -> Result<SpacePermissionsInfoOf<TestRuntime>, DispatchError>;

        fn ensure_space_owner(id: SpaceId, account: &AccountId) -> DispatchResult;
    }

    impl SpacesInterface<AccountId, SpaceId> for Spaces {
        fn get_space_owner(_space_id: SpaceId) -> Result<AccountId, DispatchError>;

        fn create_space(_owner: &AccountId, _content: Content) -> Result<SpaceId, DispatchError>;
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

impl pallet_creator_staking::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = CreatorStakingPalletId;
    type BlockPerEra = BlockPerEra;
    type Currency = Balances;
    type SpacesInterface = MockSpaces;
    type SpacePermissionsProvider = MockSpaces;
    type CreatorRegistrationDeposit = CreatorRegistrationDeposit;
    type MinimumStake = MinimumStake;
    type MinimumRemainingFreeBalance = MinimumRemainingFreeBalance;
    type MaxNumberOfBackersPerCreator = MaxNumberOfBackersPerCreator;
    type MaxEraStakeItems = MaxEraStakeItems;
    type StakeExpirationInEras = ConstU32<10>;
    type UnbondingPeriodInEras = UnbondingPeriodInEras;
    type MaxUnlockingChunks = MaxUnlockingChunks;
    // Inflation config:
    type AnnualInflation = AnnualInflation;
    type BlocksPerYear = BlocksPerYear;
    type TreasuryAccount = TreasuryAccount;
    // type WeightInfo = weights::SubstrateWeight<TestRuntime>;
}

pub struct ExternalityBuilder;

impl ExternalityBuilder {
    pub fn build() -> TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        pallet_balances::GenesisConfig::<TestRuntime> {
            balances: vec![
                (1, 9000),
                (2, 800),
                (3, 10000),
                (4, 4900),
                (5, 3800),
                (6, 10),
                (7, 1000),
                (8, 2000),
                (9, 10000),
                (10, 300),
                (11, 400),
                (20, 10),
                (540, EXISTENTIAL_DEPOSIT),
                (1337, 1_000_000_000_000),
            ],
        }
        .assimilate_storage(&mut storage)
        .ok();

        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

/// Used to run to the specified block number
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        CreatorStaking::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        // This is performed outside of creators staking but we expect it before on_initialize
        payout_block_rewards();
        CreatorStaking::on_initialize(System::block_number());
    }
}

/// Used to run the specified number of blocks
pub fn run_for_blocks(n: u64) {
    run_to_block(System::block_number() + n);
}

/// Advance blocks to the beginning of an era.
///
/// Function has no effect if era is already passed.
pub fn advance_to_era(n: EraIndex) {
    while CreatorStaking::current_era() < n {
        run_for_blocks(1);
    }
}

/// Initialize first block.
/// This method should only be called once in a UT otherwise the first block will get initialized multiple times.
pub fn initialize_first_block() {
    // This assert prevents method misuse
    assert_eq!(System::block_number(), 1 as BlockNumber);

    // This is performed outside of creators staking but we expect it before on_initialize
    payout_block_rewards();
    CreatorStaking::on_initialize(System::block_number());
    run_to_block(2);
}

/// Returns total block rewards that goes to creator-staking.
/// Contains both `creators` reward and `backers` reward.
pub fn joint_block_reward() -> Balance {
    BACKER_BLOCK_REWARD + CREATOR_BLOCK_REWARD
}

/// Payout block rewards to backers & creators
fn payout_block_rewards() {
    CreatorStaking::add_to_reward_pool(
        Balances::issue(BACKER_BLOCK_REWARD.into()),
        Balances::issue(CREATOR_BLOCK_REWARD.into()),
    );
}

// Used to get a vec of all creators staking events
pub fn creator_staking_events() -> Vec<crate::Event<TestRuntime>> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let RuntimeEvent::CreatorStaking(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect()
}
