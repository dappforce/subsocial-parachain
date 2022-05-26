use sp_std::marker::PhantomData;
use sp_std::cell::RefCell;
use frame_support::{
    assert_ok, dispatch::DispatchResult, parameter_types,
    traits::{Currency, Everything},
};
use frame_support::pallet_prelude::{DispatchClass, Pays, Weight};
use frame_support::traits::{ConstU8, Imbalance, IsType};
use frame_support::weights::{ConstantMultiplier, WeightToFeeCoefficients, WeightToFeeCoefficient, WeightToFeePolynomial, DispatchInfo};
use frame_support::weights::constants::ExtrinsicBaseWeight;
use pallet_transaction_payment::{ChargeTransactionPayment, CurrencyAdapter, MultiplierUpdate, OnChargeTransaction};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use pallet_balances::NegativeImbalance;
use smallvec::smallvec;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{Perbill, testing::Header, traits::{BlakeTwo256, IdentityLookup}};
use sp_runtime::traits::{DispatchInfoOf, PostDispatchInfoOf};
use sp_runtime::transaction_validity::TransactionValidityError;
use sp_std::convert::{TryInto, TryFrom};

pub(crate) use crate as pallet_energy;


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
		Energy: pallet_energy,
	}
);

// fn a () {
// 	let MockBlockWeights: BlockWeights = BlockWeights::builder()
// 		.base_block(0)
// 		.for_class(DispatchClass::all(), |weights| {
// 			weights.max_total = 1_000_000_000_000.into();
// 		})
// 		.avg_block_initialization(Perbill::zero())
// 		.build_or_panic();
// }

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub MockBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(0)
		.for_class(DispatchClass::all(), |weights| {
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
    pub const ExistentialDeposit: u64 = 0;
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


/// It returns the input weight as the result.
///
/// Equals to: f(x) = x
pub struct IdentityWeightToFeePolynomial;

impl WeightToFeePolynomial for IdentityWeightToFeePolynomial {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::zero(),
			coeff_integer: 1,
		}]
    }
}

/// It returns zero as the result no matter what the input is.
///
/// Equals to: f(x) = 0
pub struct ZeroWeightToFeePolynomial;

impl WeightToFeePolynomial for ZeroWeightToFeePolynomial {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        smallvec![WeightToFeeCoefficient {
			degree: 0,
			negative: false,
			coeff_frac: Perbill::zero(),
			coeff_integer: 0,
		}]
    }
}

parameter_types! {
	pub const TransactionByteFee: Balance = 0;
	pub const OperationalFeeMultiplier: u8 = 0;
}

impl pallet_transaction_payment::Config for Test {
    type OnChargeTransaction = Energy;
    type OperationalFeeMultiplier = ConstU8<0>;
    type WeightToFee = IdentityWeightToFeePolynomial;
    type LengthToFee = ZeroWeightToFeePolynomial;
    type FeeMultiplierUpdate = ();
}

#[test]
fn test_that_pallet_transaction_payment_works_as_expected() {
    assert_eq!(ZeroWeightToFeePolynomial::calc(&4000), 0);
    assert_eq!(ZeroWeightToFeePolynomial::calc(&1), 0);

    assert_eq!(IdentityWeightToFeePolynomial::calc(&4000), 4000);
    assert_eq!(IdentityWeightToFeePolynomial::calc(&1), 1);

    fn compute_fee(len: u32, weight: Weight, tip: Balance) -> Balance {
        ExtBuilder::default().build().execute_with(|| {
            pallet_transaction_payment::Pallet::<Test>::compute_fee(
                len,
                &DispatchInfo {
                    weight,
                    class: DispatchClass::Normal,
                    pays_fee: Pays::Yes,
                },
                tip,
            )
        })
    }

    assert_eq!(compute_fee(0, 0, 0), 0);
    assert_eq!(compute_fee(0, 1, 0), 1);
    assert_eq!(compute_fee(0, 1, 1), 2);
    assert_eq!(compute_fee(1, 1, 1), 2);
    assert_eq!(compute_fee(10_000, 1, 1), 2);
    assert_eq!(compute_fee(10_000, 10_000, 1), 10_001);
    assert_eq!(compute_fee(100_000, 10_000, 10_000), 20_000);
}


impl pallet_energy::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type FallbackOnChargeTransaction = ProxiedOnChargeTransaction<CurrencyAdapter<Balances, ()>>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct WithdrawFeeArgs {
    who: AccountId,
    fee: Balance,
    tip: Balance,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct CorrectAndDepositFeeArgs {
    who: AccountId,
    corrected_fee: Balance,
    already_withdrawn: Option<Balance>,
}

thread_local! {
    pub(crate) static CapturedWithdrawFeeArgs: RefCell<Option<WithdrawFeeArgs>> = RefCell::new(None);
    pub(crate) static CapturedCorrectAndDepositFeeArgs: RefCell<Option<CorrectAndDepositFeeArgs>> = RefCell::new(None);
}

pub(crate) fn get_captured_withdraw_fee_args() -> Option<WithdrawFeeArgs> {
    CapturedWithdrawFeeArgs.with(|r| r.borrow().clone())
}

pub(crate) fn get_corrected_and_deposit_fee_args() -> Option<CorrectAndDepositFeeArgs> {
    CapturedCorrectAndDepositFeeArgs.with(|r| r.borrow().clone())
}

pub(crate) fn set_withdraw_fee_args(args: WithdrawFeeArgs) {
    CapturedWithdrawFeeArgs.with(|r| *r.borrow_mut() = Some(args));
}

pub(crate) fn set_corrected_and_deposit_fee_args(args: CorrectAndDepositFeeArgs) {
    CapturedCorrectAndDepositFeeArgs.with(|r| *r.borrow_mut() = Some(args));
}

pub(crate) fn clear_withdraw_fee_args() {
    CapturedWithdrawFeeArgs.with(|r| *r.borrow_mut() = None);
}

pub(crate) fn clear_corrected_and_deposit_fee_args() {
    CapturedCorrectAndDepositFeeArgs.with(|r| *r.borrow_mut() = None);
}

pub struct ProxiedOnChargeTransaction<Real>(PhantomData<Real>);

impl<Real> OnChargeTransaction<Test> for ProxiedOnChargeTransaction<Real>
    where Real: OnChargeTransaction<Test>,
          Real::Balance: IsType<Balance>,
          Real::LiquidityInfo: IsType<Option<NegativeImbalance<Test>>>,
{
    type Balance = Real::Balance;
    type LiquidityInfo = Real::LiquidityInfo;

    fn withdraw_fee(
        who: &AccountId,
        call: &<Test as frame_system::Config>::Call,
        dispatch_info: &DispatchInfoOf<<Test as frame_system::Config>::Call>,
        fee: Self::Balance,
        tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        set_withdraw_fee_args(WithdrawFeeArgs {
            who: who.clone(),
            fee: fee.into(),
            tip: tip.into(),
        });
        Real::withdraw_fee(who, call, dispatch_info, fee, tip)
    }

    fn correct_and_deposit_fee(
        who: &AccountId,
        dispatch_info: &DispatchInfoOf<<Test as frame_system::Config>::Call>,
        post_info: &PostDispatchInfoOf<<Test as frame_system::Config>::Call>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        set_corrected_and_deposit_fee_args(CorrectAndDepositFeeArgs {
            who: who.clone(),
            corrected_fee: corrected_fee.into(),
            already_withdrawn: already_withdrawn.into_ref().as_ref().map(|val| val.peek().clone()),
        });
        Real::correct_and_deposit_fee(who, dispatch_info, post_info, corrected_fee, tip, already_withdrawn)
    }
}

pub(crate) fn account(id: AccountId) -> AccountId {
    id
}

pub(crate) fn account_with_balance(id: AccountId, balance: Balance) -> AccountId {
    let account = account(id);
    let _ = pallet_balances::Pallet::<Test>::make_free_balance_be(&account, balance);
    account
}


pub struct ExtBuilder {}

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder {}
    }
}

impl ExtBuilder {
    pub(crate) fn build(self) -> TestExternalities {
        let storage = &mut frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| {
            System::set_block_number(1);
        });

        ext
    }
}