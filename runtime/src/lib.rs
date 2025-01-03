#![cfg_attr(not(feature = "std"), no_std)]
// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod weights;
pub mod xcm_config;

use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{create_runtime_str, generic, impl_opaque_keys, traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, IdentifyAccount, Verify}, transaction_validity::{TransactionSource, TransactionValidity}, ApplyExtrinsicResult, MultiSignature};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_support::{
	construct_runtime, parameter_types,
	dispatch::DispatchClass,
	traits::{ConstBool, ConstU32, ConstU64, ConstU8, Contains, Currency, OnUnbalanced, WithdrawReasons},
	weights::{
		constants::WEIGHT_REF_TIME_PER_SECOND, ConstantMultiplier, Weight,
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
	PalletId,
};
use frame_support::traits::{Imbalance, InstanceFilter};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureWithSuccess,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill, FixedI64, FixedPointNumber, DispatchResult};
use xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

use pallet_creator_staking::{CreatorId, EraIndex};
use pallet_domains::types::PricesConfigVec;

use subsocial_support::{Content, PostId, SpaceId};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Polkadot Imports
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

use weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight};

// XCM Imports
use xcm::latest::prelude::BodyId;
use xcm_executor::XcmExecutor;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	(
		pallet_collator_selection::migration::v1::MigrateToV1<Runtime>,
		pallet_multisig::migrations::v1::MigrateToV1<Runtime>,
		pallet_xcm::migration::v1::MigrateToV1<Runtime>,
		pallet_balances::migration::MigrateToTrackInactive<Runtime, xcm_config::CheckAccount>,
	),
>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// Extrinsic base weight (smallest non-zero weight) is mapped to 100 MILLIUNIT
		let p = 100 * MILLIUNIT;
		let q = Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{generic, traits::BlakeTwo256};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("subsocial-parachain"),
	impl_name: create_runtime_str!("subsocial-parachain"),
	authoring_version: 1,
	spec_version: 45,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 10,
	state_version: 0,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

mod currency {
	use super::Balance;

	// Unit = the base number of indivisible units for balances
	pub const UNIT: Balance = 10_000_000_000;
	pub const MILLIUNIT: Balance = UNIT / 1000;
	pub const MICROUNIT: Balance = MILLIUNIT / 1000;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 2 * UNIT + (bytes as Balance) * 300 * MICROUNIT
	}
}

pub use currency::*;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = 10 * MILLIUNIT;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_div(2),
	polkadot_primitives::MAX_POV_SIZE as u64,
);

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 28;
}

// Configure FRAME pallets to include in runtime.

pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
	fn contains(c: &RuntimeCall) -> bool {
		let is_set_balance = matches!(
			c,
			RuntimeCall::Balances(pallet_balances::Call::set_balance_deprecated { .. })
			| RuntimeCall::Balances(pallet_balances::Call::force_set_balance { .. })
		);
		let is_force_transfer =
			matches!(c, RuntimeCall::Balances(pallet_balances::Call::force_transfer { .. }));

		let is_treasury_spend =
			matches!(c, RuntimeCall::Treasury(pallet_treasury::Call::spend { .. }));
		let is_remove_treasury_approval =
			matches!(c, RuntimeCall::Treasury(pallet_treasury::Call::remove_approval { .. }));

		match *c {
			RuntimeCall::Balances(..) if is_set_balance || is_force_transfer => false,
			RuntimeCall::Treasury(..) if !is_treasury_spend && !is_remove_treasury_approval => false,
			_ => true,
		}
	}
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The block type.
	type Block = Block;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = BaseFilter;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = (Aura, CreatorStaking);
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = CollatorSelection;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<0>;
	type MaxFreezes = ConstU32<0>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = MILLIUNIT / 10;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// We process transaction fees with NativeOnChargeTransaction in the Energy pallet.
	type OnChargeTransaction = Energy;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 10000 * UNIT;
	pub const SpendPeriod: BlockNumber = 7 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TreasuryPalletId: PalletId = PalletId(*b"df/trsry");
	pub const MaxApprovals: u32 = 10;
	pub const MaxBalance: Balance = 10_000_000 * UNIT;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ();
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type PriceForSiblingDelivery = ();
	type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but let's be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinEligibleCollators: u32 = 3;
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const MaxInvulnerables: u32 = 20;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinEligibleCollators = MinEligibleCollators;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 1 * UNIT;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons = WithdrawReasons::except(
		WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE | WithdrawReasons::TIP
	);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	// `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
	// highest number of schedules that encodes less than 2^10.
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	codec::Encode,
	codec::Decode,
	sp_runtime::RuntimeDebug,
	codec::MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any,
	#[deprecated(note = "Will be removed in the next release")]
	// TODO: remove as it's not used
	DomainRegistrar,
	SocialActions,
	#[deprecated(note = "Will be removed in the next release")]
	// TODO: remove or either replace as it's not used
	Management,
	#[deprecated(note = "Will be removed in the next release")]
	// TODO: remove or either replace (if we have use-cases for it) as it's not used
	SocialActionsProxy,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		let is_social_action = matches!(
			c,
			RuntimeCall::AccountFollows(..)
			| RuntimeCall::Domains(..)
			| RuntimeCall::PostFollows(..)
			| RuntimeCall::Posts(..)
			| RuntimeCall::Profiles(..)
			| RuntimeCall::Reactions(..)
			| RuntimeCall::Roles(..)
			| RuntimeCall::SpaceFollows(..)
			| RuntimeCall::Spaces(..)
		);

		match self {
			ProxyType::Any => true,
			ProxyType::SocialActions => is_social_action,
			_ => false,
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			_ => false,
		}
	}
}

impl pallet_free_proxy::Config for Runtime {
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type WeightInfo = pallet_free_proxy::weights::SubstrateWeight<Runtime>;
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = pallet_free_proxy::AdjustedProxyDepositBase<Runtime>;
	type ProxyDepositFactor = pallet_free_proxy::AdjustedProxyDepositFactor<Runtime>;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<100>;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const MinDomainLength: u32 = 4;
    pub const MaxDomainLength: u32 = 63;

    pub const MaxDomainsPerAccount: u32 = 100;

	// TODO: replace with a calculation
	// 	(([MAXIMUM_BLOCK_WEIGHT] * 0.75) / ("function_weight")) * 0.33
    pub const DomainsInsertLimit: u32 = 2860;
    pub const RegistrationPeriodLimit: BlockNumber = 365 * DAYS;
    pub const MaxOuterValueLength: u32 = 261;

    pub const BaseDomainDeposit: Balance = 10 * UNIT;
    pub const OuterValueByteDeposit: Balance = 10 * MILLIUNIT;

	pub InitialPaymentBeneficiary: AccountId = pallet_sudo::Pallet::<Runtime>::key()
		.unwrap_or(PalletId(*b"df/dmnbe").into_account_truncating());

	pub InitialPricesConfig: PricesConfigVec<Runtime> = vec![
		(4, 10_000 * UNIT),
		(5, 2_000 * UNIT),
		(6, 400 * UNIT),
		(7, 100 * UNIT),
	];
}

impl pallet_domains::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinDomainLength = MinDomainLength;
	type MaxDomainLength = MaxDomainLength;
	type MaxDomainsPerAccount = MaxDomainsPerAccount;
	type DomainsInsertLimit = DomainsInsertLimit;
	type RegistrationPeriodLimit = RegistrationPeriodLimit;
	type MaxOuterValueLength = MaxOuterValueLength;
	type BaseDomainDeposit = BaseDomainDeposit;
	type OuterValueByteDeposit = OuterValueByteDeposit;
	type InitialPaymentBeneficiary = InitialPaymentBeneficiary;
	type InitialPricesConfig = InitialPricesConfig;
	type WeightInfo = pallet_domains::weights::SubstrateWeight<Runtime>;
}

use pallet_permissions::default_permissions::DefaultSpacePermissions;

impl pallet_permissions::Config for Runtime {
	type DefaultSpacePermissions = DefaultSpacePermissions;
}

impl pallet_post_follows::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_post_follows::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
  pub const MaxCommentDepth: u32 = 10;
}

impl pallet_posts::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxCommentDepth = MaxCommentDepth;
	type IsPostBlocked = ()/*Moderation*/;
	type WeightInfo = pallet_posts::weights::SubstrateWeight<Runtime>;
}

impl pallet_reactions::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_reactions::weights::SubstrateWeight<Runtime>;
}

impl pallet_profiles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SpacePermissionsProvider = Spaces;
	type SpacesProvider = Spaces;
	type WeightInfo = pallet_profiles::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
  pub const MaxUsersToProcessPerDeleteRole: u16 = 40;
}

impl pallet_roles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxUsersToProcessPerDeleteRole = MaxUsersToProcessPerDeleteRole;
	type SpacePermissionsProvider = Spaces;
	type SpaceFollows = SpaceFollows;
	type IsAccountBlocked = ()/*Moderation*/;
	type IsContentBlocked = ()/*Moderation*/;
	type WeightInfo = pallet_roles::weights::SubstrateWeight<Runtime>;
}

impl pallet_space_follows::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_space_follows::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MaxSpacesPerAccount: u32 = 4096;
}

impl pallet_spaces::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Roles = Roles;
	type SpaceFollows = SpaceFollows;
	type IsAccountBlocked = ()/*Moderation*/;
	type IsContentBlocked = ()/*Moderation*/;
	type MaxSpacesPerAccount = MaxSpacesPerAccount;
	type WeightInfo = pallet_spaces::weights::SubstrateWeight<Runtime>;
}

impl pallet_ownership::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ProfileManager = Profiles;
	type SpacesProvider = Spaces;
	type SpacePermissionsProvider = Spaces;
	type CreatorStakingProvider = CreatorStaking;
	type DomainsProvider = Domains;
	type PostsProvider = Posts;
	type Currency = Balances;
	type WeightInfo = pallet_ownership::weights::SubstrateWeight<Runtime>;
}

impl pallet_account_follows::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}
			// for fees and tips, 100% to treasury
			Treasury::on_unbalanced(fees);
		}
	}
}

parameter_types! {
	pub DefaultValueCoefficient: FixedI64 = FixedI64::checked_from_rational(1_25, 100).unwrap();
}

impl pallet_energy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type Balance = Balance;
	type DefaultValueCoefficient = DefaultValueCoefficient;
	type UpdateOrigin = EnsureRoot<AccountId>;
	type NativeOnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
	type ExistentialDeposit = ExistentialDeposit;
	type WeightInfo = pallet_energy::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const BlockPerEra: BlockNumber = 1 * DAYS;
	pub const StakeExpirationInEras: EraIndex = 60 * DAYS / BlockPerEra::get();
	pub const UnbondingPeriodInEras: EraIndex = 7 * DAYS / BlockPerEra::get();

	pub const CreatorStakingPalletId: PalletId = PalletId(*b"df/crtst");
	pub const CreatorRegistrationDeposit: Balance = 10 * UNIT;
	pub const MinimumTotalStake: Balance = 2000 * UNIT;
	pub const MinimumRemainingFreeBalance: Balance = 10 * UNIT;

	pub const InitialRewardPerBlock: Balance = 6 * UNIT;
	pub const BlocksPerYear: BlockNumber = 365 * DAYS;
	pub TreasuryAccount: AccountId = pallet_treasury::Pallet::<Runtime>::account_id();
}

impl pallet_creator_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = CreatorStakingPalletId;
	type BlockPerEra = BlockPerEra;
	type Currency = Balances;
	type SpacesProvider = Spaces;
	type SpacePermissionsProvider = Spaces;
	type CreatorRegistrationDeposit = CreatorRegistrationDeposit;
	type MinimumTotalStake = MinimumTotalStake;
	type MinimumRemainingFreeBalance = MinimumRemainingFreeBalance;
	type MaxNumberOfBackersPerCreator = ConstU32<8000>;
	type MaxEraStakeItems = ConstU32<10>;
	type StakeExpirationInEras = StakeExpirationInEras;
	type UnbondingPeriodInEras = UnbondingPeriodInEras;
	type MaxUnbondingChunks = ConstU32<32>;
	type InitialRewardPerBlock = InitialRewardPerBlock;
	type BlocksPerYear = BlocksPerYear;
	type TreasuryAccount = TreasuryAccount;
}

impl pallet_evm_addresses::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_evm_addresses::weights::SubstrateWeight<Runtime>;
}

impl pallet_resource_discussions::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxResourceIdLength = ConstU32<256>;
	type WeightInfo = pallet_resource_discussions::weights::SubstrateWeight<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime {
		// System support stuff.
		System: frame_system = 0,
		ParachainSystem: cumulus_pallet_parachain_system = 1,
		RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 2,
		Timestamp: pallet_timestamp = 3,
		ParachainInfo: parachain_info = 4,

		// Monetary stuff.
		Balances: pallet_balances = 10,
		TransactionPayment: pallet_transaction_payment = 11,
		Treasury: pallet_treasury = 12,

		// Collator support. The order of these 5 is important and shall not change.
		Authorship: pallet_authorship = 20,
		CollatorSelection: pallet_collator_selection = 21,
		Session: pallet_session = 22,
		Aura: pallet_aura = 23,
		AuraExt: cumulus_pallet_aura_ext = 24,

		Vesting: pallet_vesting = 26,
		Proxy: pallet_proxy = 27,
		Utility: pallet_utility = 28,
		Multisig: pallet_multisig = 29,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue = 30,
		PolkadotXcm: pallet_xcm = 31,
		CumulusXcm: cumulus_pallet_xcm = 32,
		DmpQueue: cumulus_pallet_dmp_queue = 33,

		// Subsocial Pallets
		Domains: pallet_domains = 60,
		Energy: pallet_energy = 61,
		FreeProxy: pallet_free_proxy = 62,
		CreatorStaking: pallet_creator_staking = 63,
		EvmAddresses: pallet_evm_addresses = 64,
		ResourceDiscussions: pallet_resource_discussions = 65,

		Permissions: pallet_permissions = 70,
		Roles: pallet_roles = 71,
		AccountFollows: pallet_account_follows = 72,
		Profiles: pallet_profiles = 73,
		SpaceFollows: pallet_space_follows = 74,
		Ownership: pallet_ownership = 75,
		Spaces: pallet_spaces = 76,
		PostFollows: pallet_post_follows = 77,
		Posts: pallet_posts = 78,
		Reactions: pallet_reactions = 79,

		// Temporary
		Sudo: pallet_sudo = 255,
	}
);

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_vesting, Vesting]
		[pallet_proxy, Proxy]
		[pallet_utility, Utility]
		[pallet_collator_selection, CollatorSelection]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
		[pallet_xcm, PolkadotXcm]
		[pallet_domains, Domains]
		[pallet_energy, Energy]
		[pallet_evm_addresses, EvmAddresses]
		[pallet_profiles, Profiles]
		[pallet_reactions, Reactions]
		[pallet_roles, Roles]
		[pallet_space_follows, SpaceFollows]
		[pallet_ownership, Ownership]
		[pallet_spaces, Spaces]
		[pallet_post_follows, PostFollows]
		[pallet_posts, Posts]
		[pallet_resource_discussions, ResourceDiscussions]
		[pallet_free_proxy, FreeProxy]
	);
}

impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl pallet_creator_staking_rpc_runtime_api::CreatorStakingApi<Block, AccountId, Balance>
		for Runtime
	{
		fn estimated_backer_rewards_by_creators(
			backer: AccountId,
			creators: Vec<CreatorId>,
		) -> Vec<(CreatorId, Balance)> {
			CreatorStaking::estimated_backer_rewards_by_creators(backer, creators)
		}

		fn withdrawable_amounts_from_inactive_creators(
			backer: AccountId,
		) -> Vec<(CreatorId, Balance)> {
			CreatorStaking::withdrawable_amounts_from_inactive_creators(backer)
		}

		fn available_claims_by_backer(
			backer: AccountId,
		) -> Vec<(CreatorId, u32)> {
			CreatorStaking::available_claims_by_backer(backer)
		}

		fn estimated_creator_rewards(
			creator: CreatorId,
		) -> Balance {
			CreatorStaking::estimated_creator_rewards(creator)
		}

		fn available_claims_by_creator(
			creator: CreatorId,
		) -> Vec<EraIndex> {
			CreatorStaking::available_claims_by_creator(creator)
		}
	}

	impl pallet_domains_rpc_runtime_api::DomainsApi<Block, Balance> for Runtime {
		fn calculate_price(subdomain: Vec<u8>) -> Option<Balance> {
			Domains::calculate_price(&subdomain)
		}
	}

	impl pallet_posts_rpc_runtime_api::PostsApi<Block, AccountId> for Runtime {
		fn can_create_post(
			account: AccountId,
			space_id: SpaceId,
			content_opt: Option<Content>,
		) -> DispatchResult {
			Posts::can_create_regular_post(account, space_id, content_opt)
		}

		fn can_create_comment(
			account: AccountId,
			root_post_id: PostId,
			parent_id_opt: Option<PostId>,
			content_opt: Option<Content>
		) -> DispatchResult {
			Posts::can_create_comment(account, root_post_id, parent_id_opt, content_opt)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect,
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{BenchmarkError, Benchmarking, BenchmarkBatch};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {
				fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
					ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					Ok(())
				}

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
