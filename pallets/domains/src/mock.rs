use frame_support::{
    assert_ok, parameter_types, dispatch::DispatchResultWithPostInfo,
    traits::Everything,
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    Storage, testing::Header, traits::{BlakeTwo256, IdentityLookup}, traits::Zero,
};
use sp_std::convert::TryInto;

use pallet_parachain_utils::Content;
use pallet_parachain_utils::mock_functions::valid_content_ipfs;

use crate as pallet_domains;
use crate::types::*;

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
		Domains: pallet_domains,
	}
);

pub(crate) type AccountId = u64;
type Balance = u64;
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
    pub const MinDomainLength: u32 = 3;
    pub const MaxDomainLength: u32 = 63;

    pub const MaxDomainsPerAccount: u32 = 10;

    pub const DomainsInsertLimit: u32 = 100;
    pub const ReservationPeriodLimit: BlockNumber = 100;
    pub const OuterValueLimit: u16 = 256;

    pub const DomainDeposit: Balance = 10;
    pub const OuterValueByteDeposit: Balance = 1;
}

impl pallet_domains::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MinDomainLength = MinDomainLength;
    type MaxDomainLength = MaxDomainLength;
    type MaxDomainsPerAccount = MaxDomainsPerAccount;
    type DomainsInsertLimit = DomainsInsertLimit;
    type ReservationPeriodLimit = ReservationPeriodLimit;
    type OuterValueLimit = OuterValueLimit;
    type DomainDeposit = DomainDeposit;
    type OuterValueByteDeposit = OuterValueByteDeposit;
    type WeightInfo = ();
}

pub(crate) const DOMAIN_OWNER: u64 = 1;

fn default_domain() -> DomainName<Test> {
    vec![b'A'; MaxDomainLength::get() as usize].try_into().expect("domain exceeds max length")
}

pub(crate) fn default_domain_lc() -> DomainName<Test> {
    Domains::lower_domain_then_bound(default_domain())
}

pub(crate) fn _register_domain_with_full_domain(
    domain: DomainName<Test>,
) -> DispatchResultWithPostInfo {
    _register_domain(None, None, Some(domain), None, None)
}

pub(crate) fn _register_default_domain() -> DispatchResultWithPostInfo {
    _register_domain(None, None, None, None, None)
}

fn _register_domain(
    origin: Option<Origin>,
    owner: Option<AccountId>,
    domain: Option<DomainName<Test>>,
    content: Option<Content>,
    expires_in: Option<BlockNumber>,
) -> DispatchResultWithPostInfo {
    Domains::register_domain(
        origin.unwrap_or_else(Origin::root),
        owner.unwrap_or(DOMAIN_OWNER),
        domain.unwrap_or_else(default_domain),
        content.unwrap_or_else(valid_content_ipfs),
        expires_in.unwrap_or_else(ReservationPeriodLimit::get),
    )
}

pub struct ExtBuilder;

impl ExtBuilder {
    fn set_domain_owner_balance(balance: Balance) -> impl Fn(&mut Storage) {
        move |storage: &mut Storage| {
            let _ = pallet_balances::GenesisConfig::<Test> {
                balances: [DOMAIN_OWNER].iter().cloned().map(|acc| (acc, balance)).collect(),
            }.assimilate_storage(storage);
        }
    }

    fn build_with_custom_balance_for_domain_owner(balance: Balance) -> TestExternalities {
        let storage = &mut frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        Self::set_domain_owner_balance(balance)(storage);

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| System::set_block_number(1));

        ext
    }

    pub(crate) fn build() -> TestExternalities {
        Self::build_with_custom_balance_for_domain_owner(BalanceOf::<Test>::MAX)
    }

    pub(crate) fn build_with_no_balance() -> TestExternalities {
        Self::build_with_custom_balance_for_domain_owner(Zero::zero())
    }
}
