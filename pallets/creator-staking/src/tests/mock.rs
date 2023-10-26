use crate::{self as pallet_creator_staking, PalletDisabled};

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
use sp_runtime::{DispatchError, testing::Header, traits::{BlakeTwo256, ConstU32, IdentityLookup}};
use sp_std::sync::{Mutex, MutexGuard};

pub(super) type AccountId = u64;
pub(super) type BlockNumber = u64;
pub(super) type Balance = u128;
pub(super) type EraIndex = u32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

/// Value shouldn't be less than 2 for testing purposes, otherwise we cannot test certain corner cases.
pub(super) const EXISTENTIAL_DEPOSIT: Balance = 2;
pub(super) const REGISTER_DEPOSIT: Balance = 10;
pub(super) const MAX_NUMBER_OF_BACKERS: u32 = 4;
/// Value shouldn't be less than 2 for testing purposes, otherwise we cannot test certain corner cases.
pub(super) const MINIMUM_STAKING_AMOUNT: Balance = 10;
pub(super) const MINIMUM_REMAINING_AMOUNT: Balance = 1;
pub(super) const MAX_UNBONDING_CHUNKS: u32 = 5;
pub(super) const UNBONDING_PERIOD_IN_ERAS: EraIndex = 3;
pub(super) const MAX_ERA_STAKE_ITEMS: u32 = 8;

// Do note that this needs to at least be 3 for tests to be valid. It can be greater but not smaller.
pub(super) const BLOCKS_PER_ERA: BlockNumber = 3;
pub(super) const BLOCKS_PER_YEAR: BlockNumber = 2628000;
pub(super) const BLOCK_REWARD: Balance = 100;

pub(super) const TREASURY_ACCOUNT: BlockNumber = 42;

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
use crate::tests::tests::Rewards;

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
    pub const MaxUnbondingChunks: u32 = MAX_UNBONDING_CHUNKS;
    pub const UnbondingPeriodInEras: EraIndex = UNBONDING_PERIOD_IN_ERAS;
    pub const MaxEraStakeItems: u32 = MAX_ERA_STAKE_ITEMS;
    pub const BlockReward: Balance = BLOCK_REWARD;
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
    type MaxUnbondingChunks = MaxUnbondingChunks;
    // Inflation config:
    type BlockReward = BlockReward;
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
        ext.execute_with(|| {
            System::set_block_number(1);
            // TODO: revert when default PalletDisabled changed back to false
            PalletDisabled::<TestRuntime>::put(false);
        });
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
    PalletDisabled::<TestRuntime>::put(false);

    // This is performed outside of creators staking but we expect it before on_initialize
    payout_block_rewards();
    CreatorStaking::on_initialize(System::block_number());
    run_to_block(2);
}

/// Payout block rewards to backers & creators
fn payout_block_rewards() {
    let Rewards { backers_reward, creators_reward, .. } =
        Rewards::calculate(&CreatorStaking::reward_config());

    CreatorStaking::add_to_reward_pool(
        Balances::issue(backers_reward),
        Balances::issue(creators_reward),
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
