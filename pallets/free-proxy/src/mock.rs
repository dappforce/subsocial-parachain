// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use frame_support::{parameter_types, traits::Everything};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::{TryFrom, TryInto};

pub(crate) use crate as pallet_free_proxy;

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
        FreeProxy: pallet_free_proxy,
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
    pub static ProxyDepositBase: Balance = 10;
    pub static ProxyDepositFactor: Balance = 10;
    pub const MaxProxies: u16 = 32;
    pub const AnnouncementDepositBase: Balance = 9999999999999;
    pub const AnnouncementDepositFactor: Balance = 9999999999999;
    pub const MaxPending: u16 = 32;
}

impl pallet_free_proxy::Config for Test {
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type WeightInfo = ();
}

impl pallet_proxy::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = ();
    type ProxyDepositBase = pallet_free_proxy::AdjustedProxyDepositBase<Test>;
    type ProxyDepositFactor = pallet_free_proxy::AdjustedProxyDepositFactor<Test>;
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
        ExtBuilder { deposit_factor: 10, deposit_base: 10 }
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
