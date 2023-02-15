use codec::Decode;
use frame_support::{
    dispatch::RawOrigin,
    pallet_prelude::{DispatchClass, Pays, Weight},
    parameter_types,
    traits::{ConstU8, Currency, EnsureOrigin, Everything, Get, Imbalance, IsType},
    weights::{
        DispatchInfo, WeightToFee, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    },
};
use frame_system::limits::BlockWeights;
use pallet_balances::NegativeImbalance;
use smallvec::smallvec;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, DispatchInfoOf, IdentityLookup, One, PostDispatchInfoOf},
    transaction_validity::TransactionValidityError,
    FixedI64, Perbill,
};
use sp_std::{
    cell::RefCell,
    convert::{TryFrom, TryInto},
    marker::PhantomData,
};

pub(crate) use crate as pallet_subsocial_proxy;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(super) type AccountId = u64;
pub(super) type Balance = u64;
type BlockNumber = u64;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Proxy: pallet_proxy,
        SubsocialProxy: pallet_subsocial_proxy,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
    pub MockBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(0)
        .for_class(DispatchClass::all(), |weights| {
            // we set it to 0 to have a predictable and easy to write weight to fee function
            weights.base_extrinsic = 0;
            weights.max_extrinsic = 1_000_000_000.into();
            weights.max_total = 1_000_000_000_000.into();
            weights.reserved = None;
        })
        .avg_block_initialization(Perbill::zero())
        .build_or_panic();
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = MockBlockWeights;
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
    pub static ExistentialDeposit: u64 = 1;
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
	pub static ProxyDepositBase: Balance = 10;
	pub static ProxyDepositFactor: Balance = 10;
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = 9999999999999;
	pub const AnnouncementDepositFactor: Balance = 9999999999999;
	pub const MaxPending: u16 = 32;
}

impl pallet_subsocial_proxy::Config for Test {
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
}

impl pallet_proxy::Config for Test {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ProxyType = ();
    type ProxyDepositBase = pallet_subsocial_proxy::AdjustedProxyDepositBase<Test>;
    type ProxyDepositFactor = pallet_subsocial_proxy::AdjustedProxyDepositFactor<Test>;
    type MaxProxies = MaxProxies;
    type WeightInfo = ();
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}


pub struct ExtBuilder {
    deposit_base: Balance,
    deposit_factor: Balance,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder {
            deposit_factor: 10,
            deposit_base: 10,
        }
    }
}

impl ExtBuilder {
    pub(crate) fn deposit_base(mut self, deposit_base: Balance) -> Self {
        self.deposit_base = deposit_base;
        self
    }

    pub(crate) fn deposit_factor(mut self, deposit_factor: Balance) -> Self {
        self.deposit_factor = deposit_factor;
        self
    }

    fn set_configs(&self) {
        PROXY_DEPOSIT_BASE.with(|x| *x.borrow_mut() = self.deposit_base);
        PROXY_DEPOSIT_FACTOR.with(|x| *x.borrow_mut() = self.deposit_factor);
    }

    pub(crate) fn build(self) -> TestExternalities {
        self.set_configs();

        let storage = &mut frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| {
            System::set_block_number(1);
        });

        ext
    }
}