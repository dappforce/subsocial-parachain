use crate as pallet_token_locker;
use frame_support::{parameter_types, traits::Everything};
use frame_system as system;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
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
		Balances: pallet_balances exclude_parts { Config },
		Locker: pallet_token_locker,
	}
);

pub type AccountId = u64;
pub type BlockNumber = u64;
pub type Balance = u64;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
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

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ();
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

parameter_types! {
	pub static UnlockPeriod: BlockNumber = 0;
	pub static MinLockAmount: Balance = 0;
	pub static MaxLockAmount: Balance = 0;
}

impl pallet_token_locker::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type UnlockPeriod = UnlockPeriod;
    type MinLockAmount = MinLockAmount;
    type MaxLockAmount = MaxLockAmount;
    type WeightInfo = ();
}

pub struct ExtBuilder {
    unlock_period: BlockNumber,
    min_lock_amount: Balance,
    max_lock_amount: Balance,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            unlock_period: 10,
            min_lock_amount: 1,
            max_lock_amount: 10_000,
        }
    }
}

impl ExtBuilder {
    pub fn unlock_period(mut self, unlock_period: BlockNumber) -> Self {
        self.unlock_period = unlock_period;
        self
    }

    pub fn min_lock_amount(mut self, min_lock_amount: Balance) -> Self {
        self.min_lock_amount = min_lock_amount;
        self
    }

    pub fn max_lock_amount(mut self, max_lock_amount: Balance) -> Self {
        self.max_lock_amount = max_lock_amount;
        self
    }

    pub fn set_configs(&self) {
        UNLOCK_PERIOD.with(|x| *x.borrow_mut() = self.unlock_period);
        MIN_LOCK_AMOUNT.with(|x| *x.borrow_mut() = self.min_lock_amount);
        MAX_LOCK_AMOUNT.with(|x| *x.borrow_mut() = self.max_lock_amount);
    }

    pub fn build(self) -> TestExternalities {
        self.set_configs();

        let storage = &mut system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| <frame_system::Pallet<Test>>::set_block_number(1));

        ext
    }
}