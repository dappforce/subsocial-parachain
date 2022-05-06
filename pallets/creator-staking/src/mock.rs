use frame_support::{
    assert_ok, dispatch::DispatchResult, parameter_types,
    traits::{Currency, Everything},
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header, traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::TryInto;
pub(crate) use crate as pallet_creator_staking;

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
        CreatorStaking: pallet_creator_staking,
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
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

parameter_types! {
    pub static MinDomainLength: u32 = 0;
    pub const MaxDomainLength: u32 = 63;

    pub static MaxDomainsPerAccount: u32 = 0;
    pub static MaxPromoDomainsPerAccount: u32 = 0;

    pub const DomainsInsertLimit: u32 = 2860;
    pub static ReservationPeriodLimit: BlockNumber = 0;
    pub const MaxOuterValueLength: u16 = 256;

    pub static BaseDomainDeposit: Balance = 0;
    pub static OuterValueByteDeposit: Balance = 0;
}

impl pallet_creator_staking::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MaxUnlockingChunks = ();
    type CreatorRegistrationDeposit = ();
    type MinStake = ();
}