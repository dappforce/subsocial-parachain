use codec::Decode;
use frame_support::{
    dispatch::{RawOrigin, DispatchInfo},
    pallet_prelude::{DispatchClass, Pays, Weight},
    parameter_types,
    traits::{ConstU8, Currency, EnsureOrigin, Everything, Get, Imbalance, IsType},
    weights::{
        WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    },
};
use frame_support::weights::ConstantMultiplier;
use frame_support::weights::constants::ExtrinsicBaseWeight;
use pallet_balances::NegativeImbalance;
use pallet_transaction_payment::{ConstFeeMultiplier, CurrencyAdapter, OnChargeTransaction};
use smallvec::smallvec;
use sp_core::{ConstU32, H256};
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

pub(crate) use crate as pallet_evm_accounts;

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
        TransactionPayment: pallet_transaction_payment,
    }
);


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
    pub static ExistentialDeposit: u64 = 1;
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // Extrinsic base weight (smallest non-zero weight) is mapped to 10 MILLIUNIT
        let p = 10 * 10_000_000;
        let q = Balance::from(ExtrinsicBaseWeight::get().ref_time());
        smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
    }
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


parameter_types! {
    pub const TransactionByteFee: Balance = 0;
    pub const OperationalFeeMultiplier: u8 = 0;
}

impl pallet_transaction_payment::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = WeightToFee;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = ();
}

parameter_types! {
    pub static ValueCoefficient: FixedI64 = FixedI64::one();
    pub static TestUpdateOrigin: AccountId = 1235;
    pub static EnergyExistentialDeposit: Balance = 1;
}

pub(crate) fn account(id: AccountId) -> AccountId {
    id
}

pub(crate) fn account_with_balance(id: AccountId, balance: Balance) -> AccountId {
    let account = account(id);
    set_native_balance(account, balance);
    account
}

pub(crate) fn set_native_balance(id: AccountId, balance: Balance) {
    let _ = pallet_balances::Pallet::<Test>::make_free_balance_be(&id, balance);
}

pub struct ExtBuilder {

}

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder {

        }
    }
}

impl ExtBuilder {
    fn set_configs(&self) {

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