// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::{
    parameter_types,
    traits::{ConstU8, Everything},
    weights::{
        constants::ExtrinsicBaseWeight, ConstantMultiplier, WeightToFeeCoefficient,
        WeightToFeeCoefficients, WeightToFeePolynomial,
    },
};
use smallvec::smallvec;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{generic, traits::{BlakeTwo256, IdentityLookup}, BuildStorage, Perbill};
use sp_std::convert::{TryFrom, TryInto};

pub(crate) use crate as pallet_evm_accounts;

type SignedExtra = (
    pallet_transaction_payment::ChargeTransactionPayment<Test>,
    // pallet_evm_accounts::ChargeTransactionPaymentEvmMapped<Test>,
);
type Signature = ();
type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, SignedExtra>;
type Block = generic::Block<generic::Header<u64, BlakeTwo256>, UncheckedExtrinsic>;

pub(super) type AccountId = u64;
pub(super) type Balance = u64;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        EvmAccounts: pallet_evm_accounts,
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
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
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
    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type MaxHolds = ();
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
    pub static MaxLinkedAccounts: u32 = 1;
}

impl pallet_evm_accounts::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

pub(crate) fn account(id: AccountId) -> AccountId {
    id
}

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
    pub(crate) fn build(self) -> TestExternalities {
        let mut ext: TestExternalities = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
        ext.execute_with(|| {
            System::set_block_number(1);
        });

        ext
    }
}
