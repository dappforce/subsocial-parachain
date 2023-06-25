// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use codec::Decode;
use frame_support::{
    dispatch::{RawOrigin, DispatchInfo},
    pallet_prelude::{DispatchClass, Pays, Weight},
    parameter_types,
    traits::{ConstU8, Currency, EnsureOrigin, Everything, Get, Imbalance, IsType},
    weights::{
        WeightToFee, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
    },
};
use pallet_balances::NegativeImbalance;
use pallet_transaction_payment::{CurrencyAdapter, OnChargeTransaction};
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

pub(crate) use crate as pallet_energy;
use crate::{EnergyBalance, TotalEnergy};

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

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
    pub MockBlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			frame_support::weights::Weight::from_ref_time(1_000_000)
                .set_proof_size(u64::MAX)
		);
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = MockBlockWeights;
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
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = Energy;
    type OperationalFeeMultiplier = ConstU8<0>;
    type WeightToFee = IdentityWeightToFeePolynomial;
    type LengthToFee = ZeroWeightToFeePolynomial;
    type FeeMultiplierUpdate = ();
}

#[test]
fn test_that_pallet_transaction_payment_works_as_expected() {
    assert_eq!(ZeroWeightToFeePolynomial::weight_to_fee(&Weight::from_ref_time(4000)), 0);
    assert_eq!(ZeroWeightToFeePolynomial::weight_to_fee(&Weight::from_ref_time(1)), 0);

    assert_eq!(IdentityWeightToFeePolynomial::weight_to_fee(&Weight::from_ref_time(4000)), 4000);
    assert_eq!(IdentityWeightToFeePolynomial::weight_to_fee(&Weight::from_ref_time(1)), 1);

    fn compute_fee(len: u32, weight: u64, tip: Balance) -> Balance {
        ExtBuilder::default().build().execute_with(|| {
            pallet_transaction_payment::Pallet::<Test>::compute_fee(
                len,
                &DispatchInfo {
                    weight: Weight::from_ref_time(weight),
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

parameter_types! {
    pub static ValueCoefficient: FixedI64 = FixedI64::one();
    pub static TestUpdateOrigin: AccountId = 1235;
    pub static EnergyExistentialDeposit: Balance = 1;
}

pub struct EnsureAccount<Account, AccountId>(PhantomData<(Account, AccountId)>);

impl<O, Account, AccountId> EnsureOrigin<O> for EnsureAccount<Account, AccountId>
where
    O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>,
    AccountId: PartialEq + Clone + Ord + Decode,
    Account: Get<AccountId>,
{
    type Success = AccountId;
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Signed(who) if who == Account::get() => Ok(who),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> O {
        O::from(RawOrigin::Signed(Account::get()))
    }
}

impl pallet_energy::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Balance = <Test as pallet_balances::Config>::Balance;
    type DefaultValueCoefficient = ValueCoefficient;
    type UpdateOrigin = EnsureAccount<TestUpdateOrigin, AccountId>;
    type NativeOnChargeTransaction = ProxiedOnChargeTransaction<CurrencyAdapter<Balances, ()>>;
    type ExistentialDeposit = EnergyExistentialDeposit;
    type WeightInfo = ();
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct WithdrawFeeArgs {
    pub(crate) who: AccountId,
    pub(crate) fee_with_tip: Balance,
    pub(crate) tip: Balance,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct CorrectAndDepositFeeArgs {
    pub(crate) who: AccountId,
    pub(crate) corrected_fee_with_tip: Balance,
    pub(crate) already_withdrawn: Option<Balance>,
}

thread_local! {
    pub(crate) static CAPTURED_WITHDRAW_FEE_ARGS: RefCell<Option<WithdrawFeeArgs>> = RefCell::new(None);
    pub(crate) static CAPTURED_CORRECT_AND_DEPOSIT_FEE_ARGS: RefCell<Option<CorrectAndDepositFeeArgs>> = RefCell::new(None);
}

pub(crate) fn get_captured_withdraw_fee_args() -> Option<WithdrawFeeArgs> {
    CAPTURED_WITHDRAW_FEE_ARGS.with(|r| r.borrow().clone())
}

pub(crate) fn get_corrected_and_deposit_fee_args() -> Option<CorrectAndDepositFeeArgs> {
    CAPTURED_CORRECT_AND_DEPOSIT_FEE_ARGS.with(|r| r.borrow().clone())
}

pub(crate) fn set_withdraw_fee_args(args: WithdrawFeeArgs) {
    CAPTURED_WITHDRAW_FEE_ARGS.with(|r| *r.borrow_mut() = Some(args));
}

pub(crate) fn set_corrected_and_deposit_fee_args(args: CorrectAndDepositFeeArgs) {
    CAPTURED_CORRECT_AND_DEPOSIT_FEE_ARGS.with(|r| *r.borrow_mut() = Some(args));
}

pub(crate) fn clear_withdraw_fee_args() {
    CAPTURED_WITHDRAW_FEE_ARGS.with(|r| *r.borrow_mut() = None);
}

pub(crate) fn clear_corrected_and_deposit_fee_args() {
    CAPTURED_CORRECT_AND_DEPOSIT_FEE_ARGS.with(|r| *r.borrow_mut() = None);
}

pub struct ProxiedOnChargeTransaction<Real>(PhantomData<Real>);

impl<Real> OnChargeTransaction<Test> for ProxiedOnChargeTransaction<Real>
where
    Real: OnChargeTransaction<Test>,
    Real::Balance: IsType<Balance>,
    Real::LiquidityInfo: IsType<Option<NegativeImbalance<Test>>>,
{
    type Balance = Real::Balance;
    type LiquidityInfo = Real::LiquidityInfo;

    fn withdraw_fee(
        who: &AccountId,
        call: &<Test as frame_system::Config>::RuntimeCall,
        dispatch_info: &DispatchInfoOf<<Test as frame_system::Config>::RuntimeCall>,
        fee: Self::Balance,
        tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        set_withdraw_fee_args(WithdrawFeeArgs {
            who: *who,
            fee_with_tip: fee.into(),
            tip: tip.into(),
        });
        Real::withdraw_fee(who, call, dispatch_info, fee, tip)
    }

    fn correct_and_deposit_fee(
        who: &AccountId,
        dispatch_info: &DispatchInfoOf<<Test as frame_system::Config>::RuntimeCall>,
        post_info: &PostDispatchInfoOf<<Test as frame_system::Config>::RuntimeCall>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        set_corrected_and_deposit_fee_args(CorrectAndDepositFeeArgs {
            who: *who,
            corrected_fee_with_tip: corrected_fee.into(),
            already_withdrawn: already_withdrawn.into_ref().as_ref().map(|val| val.peek()),
        });
        Real::correct_and_deposit_fee(
            who,
            dispatch_info,
            post_info,
            corrected_fee,
            tip,
            already_withdrawn,
        )
    }
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

pub(crate) fn set_energy_balance(id: AccountId, new_balance: Balance) {
    EnergyBalance::<Test>::mutate(id, |current_balance| {
        TotalEnergy::<Test>::mutate(|total| {
            if *current_balance > new_balance {
                *total -= *current_balance - new_balance;
            } else {
                *total += new_balance - *current_balance;
            };
        });
        *current_balance = new_balance
    });
}

pub struct ExtBuilder {
    value_coefficient: f64,
    update_origin: AccountId,
    existential_deposit: Balance,
    energy_existential_deposit: Balance,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder {
            value_coefficient: 1.0,
            update_origin: 1235,
            energy_existential_deposit: 1,
            existential_deposit: 1,
        }
    }
}

impl ExtBuilder {
    pub(crate) fn native_existential_deposit(mut self, existential_deposit: Balance) -> Self {
        self.existential_deposit = existential_deposit;
        self
    }

    pub(crate) fn energy_existential_deposit(mut self, new: Balance) -> Self {
        self.energy_existential_deposit = new;
        self
    }

    pub(crate) fn update_origin(mut self, update_origin: AccountId) -> Self {
        self.update_origin = update_origin;
        self
    }

    pub(crate) fn value_coefficient(mut self, value_coefficient: f64) -> Self {
        self.value_coefficient = value_coefficient;
        self
    }

    fn set_configs(&self) {
        VALUE_COEFFICIENT.with(|x| *x.borrow_mut() = FixedI64::from_float(self.value_coefficient));
        TEST_UPDATE_ORIGIN.with(|x| *x.borrow_mut() = self.update_origin);
        ENERGY_EXISTENTIAL_DEPOSIT.with(|x| *x.borrow_mut() = self.energy_existential_deposit);
        EXISTENTIAL_DEPOSIT.with(|x| *x.borrow_mut() = self.existential_deposit);
    }

    pub(crate) fn build(self) -> TestExternalities {
        self.set_configs();

        clear_withdraw_fee_args();
        clear_corrected_and_deposit_fee_args();

        let storage = &mut frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| {
            System::set_block_number(1);
        });

        ext
    }
}

macro_rules! assert_total_energy {
    ($expected_total_energy:expr) => {
        let total_energy = Energy::total_energy();
        assert_eq!(
            $expected_total_energy, total_energy,
            "Expected total energy to be {}, but found {}",
            $expected_total_energy, total_energy,
        );
    };
}
/// Asserts the total issuance of native token is equal to the given value.
macro_rules! assert_total_issuance {
    ($expected_issuance:expr) => {
        let total_issuance = Balances::total_issuance();
        assert_eq!(
            $expected_issuance, total_issuance,
            "Expected total issuance to be {}, but found {}",
            $expected_issuance, total_issuance,
        );
    };
}
macro_rules! assert_energy_balance {
    ($account:expr, $expected_energy_balance:expr) => {
        let energy_balance = EnergyBalance::<Test>::get($account);
        dbg!(energy_balance);
        assert_eq!(
            energy_balance,
            $expected_energy_balance,
            "Expected energy balance of {}={} to be {}, but found {}",
            stringify!($account),
            $account,
            $expected_energy_balance,
            energy_balance,
        );
    };
}
/// Asserts the balance of an account is equal to the given value.
macro_rules! assert_balance {
    ($account:expr, $expected_balance:expr) => {
        let balance = Balances::free_balance($account);
        assert_eq!(
            balance,
            $expected_balance,
            "Expected balance of {}={} to be {}, but found {}",
            stringify!($account),
            $account,
            $expected_balance,
            balance,
        );
    };
}

pub(crate) use assert_balance;
pub(crate) use assert_energy_balance;
pub(crate) use assert_total_energy;
pub(crate) use assert_total_issuance;
