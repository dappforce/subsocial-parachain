use frame_support::{
    assert_ok, dispatch::DispatchResult, parameter_types,
    traits::{Currency, Everything},
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header, traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::{TryInto, TryFrom};
pub(crate) use crate as pallet_domains;
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

    pub static MaxRecordKeySize: u32 = 250;
    pub static MaxRecordValueSize: u32 = 250;
    pub static RecordByteDeposit: Balance = 0;
}

impl pallet_domains::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MinDomainLength = MinDomainLength;
    type MaxDomainLength = MaxDomainLength;
    type MaxDomainsPerAccount = MaxDomainsPerAccount;
    type MaxPromoDomainsPerAccount = MaxPromoDomainsPerAccount;
    type DomainsInsertLimit = DomainsInsertLimit;
    type RegistrationPeriodLimit = ReservationPeriodLimit;
    type MaxOuterValueLength = MaxOuterValueLength;
    type MaxRecordKeySize = MaxRecordKeySize;
    type MaxRecordValueSize = MaxRecordValueSize;
    type BaseDomainDeposit = BaseDomainDeposit;
    type RecordByteDeposit = RecordByteDeposit;
    type WeightInfo = ();
}

pub(crate) const DOMAIN_OWNER: u64 = 1;
pub(crate) const DUMMY_ACCOUNT: u64 = 2;

pub(crate) const DEFAULT_TLD: [u8; 3] = *b"sub";

pub(crate) fn default_tld() -> DomainName<Test> {
    Domains::bound_domain(DEFAULT_TLD.to_vec())
}

pub(crate) fn default_domain() -> DomainName<Test> {
    let tld = default_tld();
    let mut domain_vec = vec![b'A'; MaxDomainLength::get() as usize - tld.len() - 1];

    domain_vec.push(b'.');
    domain_vec.append(&mut tld.to_vec());
    Domains::bound_domain(domain_vec)
}

pub(crate) fn domain_from(mut string: Vec<u8>) -> DomainName<Test> {
    string.push(b'.');
    string.append(&mut default_tld().to_vec());
    Domains::bound_domain(string)
}

pub(crate) fn split_domain_from(string: &[u8]) -> Vec<DomainName<Test>> {
    Domains::split_domain_by_dot(
        &Domains::bound_domain(string.to_vec())
    )
}

pub(crate) fn default_domain_lc() -> DomainName<Test> {
    Domains::lower_domain_then_bound(&default_domain())
}

pub(crate) fn _force_register_domain_with_origin(origin: Origin) -> DispatchResult {
    _force_register_domain(Some(origin), None, None, None)
}

pub(crate) fn _force_register_domain_with_expires_in(expires_in: BlockNumber) -> DispatchResult {
    _force_register_domain(None, None, None, Some(expires_in))
}

pub(crate) fn _force_register_domain_with_name(domain_name: DomainName<Test>) -> DispatchResult {
    _force_register_domain(None, None, Some(domain_name), None)
}

fn _force_register_domain(
    origin: Option<Origin>,
    owner: Option<AccountId>,
    domain: Option<DomainName<Test>>,
    expires_in: Option<BlockNumber>,
) -> DispatchResult {
    Domains::force_register_domain(
        origin.unwrap_or_else(Origin::root),
        owner.unwrap_or(DOMAIN_OWNER),
        domain.unwrap_or_else(default_domain),
        expires_in.unwrap_or(ExtBuilder::default().reservation_period_limit),
    )
}

pub(crate) fn _register_default_domain() -> DispatchResult {
    _register_domain(None, None, None)
}

fn _register_domain(
    origin: Option<Origin>,
    domain: Option<DomainName<Test>>,
    expires_in: Option<BlockNumber>,
) -> DispatchResult {
    Domains::register_domain(
        origin.unwrap_or_else(|| Origin::signed(DOMAIN_OWNER)),
        domain.unwrap_or_else(default_domain),
        expires_in.unwrap_or(ExtBuilder::default().reservation_period_limit),
    )
}

pub(crate) fn account_with_balance(id: AccountId, balance: Balance) -> AccountId {
    let account = account(id);
    let _ = <Test as pallet_domains::Config>::Currency::make_free_balance_be(&account, balance);
    account
}

pub(crate) fn account(id: AccountId) -> AccountId {
    id
}

pub(crate) fn get_reserved_balance(who: &AccountId) -> BalanceOf<Test> {
    <Test as pallet_domains::Config>::Currency::reserved_balance(who)
}

#[derive(Clone)]
pub struct ExtBuilder {
    pub(crate) min_domain_length: u32,
    pub(crate) max_domains_per_account: u32,
    pub(crate) max_promo_domains_per_account: u32,
    pub(crate) reservation_period_limit: BlockNumber,
    pub(crate) base_domain_deposit: Balance,
    pub(crate) outer_value_byte_deposit: Balance,
    pub(crate) max_record_key_size: u32,
    pub(crate) max_record_value_size: u32,
    pub(crate) record_byte_deposit: Balance,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder {
            min_domain_length: 3,
            max_domains_per_account: 10,
            max_promo_domains_per_account: 10,
            reservation_period_limit: 1000,
            base_domain_deposit: 10,
            outer_value_byte_deposit: 1,
            max_record_key_size: 250,
            max_record_value_size: 250,
            record_byte_deposit: 0,
        }
    }
}

impl ExtBuilder {
    pub(crate) fn min_domain_length(mut self, min_domain_length: u32) -> Self {
        self.min_domain_length = min_domain_length;
        self
    }

    pub(crate) fn max_domains_per_account(mut self, max_domains_per_account: u32) -> Self {
        self.max_domains_per_account = max_domains_per_account;
        self
    }

    pub(crate) fn max_promo_domains_per_account(mut self, max_promo_domains_per_account: u32) -> Self {
        self.max_promo_domains_per_account = max_promo_domains_per_account;
        self
    }

    pub(crate) fn reservation_period_limit(mut self, reservation_period_limit: BlockNumber) -> Self {
        self.reservation_period_limit = reservation_period_limit;
        self
    }

    pub(crate) fn base_domain_deposit(mut self, domain_deposit: Balance) -> Self {
        self.base_domain_deposit = domain_deposit;
        self
    }

    pub(crate) fn outer_value_byte_deposit(mut self, outer_value_byte_deposit: Balance) -> Self {
        self.outer_value_byte_deposit = outer_value_byte_deposit;
        self
    }

    pub(crate) fn max_record_key_size(mut self, max_record_key_size: u32) -> Self {
        self.max_record_key_size = max_record_key_size;
        self
    }

    pub(crate) fn max_record_value_size(mut self, max_record_value_size: u32) -> Self {
        self.max_record_value_size = max_record_value_size;
        self
    }

    pub(crate) fn record_byte_deposit(mut self, record_byte_deposit: Balance) -> Self {
        self.record_byte_deposit = record_byte_deposit;
        self
    }

    fn set_configs(&self) {
        MIN_DOMAIN_LENGTH.with(|x| *x.borrow_mut() = self.min_domain_length);
        MAX_DOMAINS_PER_ACCOUNT.with(|x| *x.borrow_mut() = self.max_domains_per_account);
        MAX_PROMO_DOMAINS_PER_ACCOUNT.with(|x| *x.borrow_mut() = self.max_promo_domains_per_account);
        BASE_DOMAIN_DEPOSIT.with(|x| *x.borrow_mut() = self.base_domain_deposit);
        OUTER_VALUE_BYTE_DEPOSIT.with(|x| *x.borrow_mut() = self.outer_value_byte_deposit);
        RESERVATION_PERIOD_LIMIT.with(|x| *x.borrow_mut() = self.reservation_period_limit);
        MAX_RECORD_KEY_SIZE.with(|x| *x.borrow_mut() = self.max_record_key_size);
        MAX_RECORD_VALUE_SIZE.with(|x| *x.borrow_mut() = self.max_record_value_size);
        RECORD_BYTE_DEPOSIT.with(|x| *x.borrow_mut() = self.record_byte_deposit);
    }

    pub(crate) fn build(self) -> TestExternalities {
        self.set_configs();

        let storage = &mut frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let mut ext = TestExternalities::from(storage.clone());
        ext.execute_with(|| {
            System::set_block_number(1);
            assert_ok!(
                Domains::support_tlds(
                    Origin::root(),
                    vec![default_tld()].try_into().expect("qed; domains vector exceeds the limit"),
                )
            );
        });

        ext
    }

    pub(crate) fn build_with_default_domain_registered(self) -> TestExternalities {
        let mut ext = self.clone().build();
        ext.execute_with(|| {
            let _ = account_with_balance(DOMAIN_OWNER, self.base_domain_deposit);
            assert_ok!(_register_default_domain());
        });
        ext
    }
}
