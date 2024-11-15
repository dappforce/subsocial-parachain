// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::{assert_ok, pallet_prelude::*};
use sp_core::storage::Storage;
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;
use pallet_ownership::{OwnableEntity, Event as OwnershipEvent};

use pallet_posts::PostExtension;
use pallet_spaces::*;
use subsocial_support::{Content, PostId, SpaceId};
use subsocial_support::mock_functions::valid_content_ipfs;

use crate::mock::*;

////// Ext Builder

pub struct ExtBuilder;

impl ExtBuilder {
    fn configure_storages(storage: &mut Storage) {
        let mut accounts = Vec::new();
        for account in ACCOUNT1..=ACCOUNT3 {
            accounts.push(account);
        }

        let _ = pallet_balances::GenesisConfig::<Test> {
            balances: accounts.iter().cloned().map(|k| (k, 1000)).collect(),
        }
        .assimilate_storage(storage);
    }

    /// Default ext configuration with BlockNumber 1
    pub fn build() -> TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

        Self::configure_storages(&mut storage);

        let mut ext: TestExternalities = storage.into();
        ext.execute_with(|| {
            let _m = use_static_mock();
            System::set_block_number(1);
        });

        ext
    }

    pub fn add_default_space() {
        assert_ok!(_create_default_space());
    }

    pub fn add_default_post(space_id_opt: Option<SpaceId>) {
        assert_ok!(_create_default_post(space_id_opt));
    }

    pub fn add_default_domain() {
        assert_ok!(_create_default_domain());
    }

    /// Custom ext configuration with SpaceId 1001, PostId 1 and Domain "dappforce.sub"
    pub fn build_with_all_enitities() -> TestExternalities {
        let mut ext = Self::build();
        ext.execute_with(|| {
            Self::add_default_space();
            let space_id = NextSpaceId::<Test>::get().saturating_sub(1);
            Self::add_default_post(Some(space_id));
            Self::add_default_domain();
        });
        ext
    }

    /// Custom ext configuration with SpaceId 1001, PostId 1 and Domain "dappforce.sub"
    /// and pending transfers for all entities (current owner: `ACCOUNT1`, new owner: `ACCOUNT2`)
    pub fn build_with_pending_transfers() -> TestExternalities {
        let mut ext = Self::build_with_all_enitities();
        ext.execute_with(|| {
            assert_create_transfers_ok();
        });
        ext
    }
}

// Constants

pub(crate) const ACCOUNT1: AccountId = 1;
pub(crate) const ACCOUNT2: AccountId = 2;
pub(crate) const ACCOUNT3: AccountId = 3;

pub(crate) const SPACE1: SpaceId = 1001;
pub(crate) const POST1: PostId = 1;

// Other pallets utils

// Data

pub(crate) fn default_domain() -> BoundedVec<u8, MaxDomainLength> {
    "dappforce.sub".as_bytes().to_vec().try_into().unwrap()
}

pub(crate) fn default_domain_entity() -> OwnableEntity<Test> {
    OwnableEntity::Domain(default_domain())
}

pub(crate) const fn default_space_entity() -> OwnableEntity<Test> {
    OwnableEntity::Space(SPACE1)
}

pub(crate) const fn default_post_entity() -> OwnableEntity<Test> {
    OwnableEntity::Post(POST1)
}

// Extrinsics

pub(crate) fn _create_default_space() -> DispatchResult {
    Spaces::create_space(
        RuntimeOrigin::signed(ACCOUNT1),
        valid_content_ipfs(),
        None,
    )
}

pub(crate) fn _create_default_post(space_id_opt: Option<SpaceId>) -> DispatchResult {
    Posts::create_post(
        RuntimeOrigin::signed(ACCOUNT1),
        space_id_opt,
        PostExtension::RegularPost,
        valid_content_ipfs(),
    )
}

pub(crate) fn _create_default_domain() -> DispatchResult {
    Domains::register_domain(
        RuntimeOrigin::signed(ACCOUNT1),
        None,
        default_domain(),
        Content::None,
        RegistrationPeriodLimit::get(),
    )
}

// Space ownership utils

pub(crate) fn _transfer_ownership(
    account: AccountId,
    entity: OwnableEntity<Test>,
    transfer_to: AccountId,
) -> DispatchResult {
    Ownership::transfer_ownership(
        RuntimeOrigin::signed(account),
        entity,
        transfer_to,
    )
}

pub(crate) fn _accept_pending_ownership(
    account: AccountId,
    entity: OwnableEntity<Test>,
) -> DispatchResult {
    Ownership::accept_pending_ownership(
        RuntimeOrigin::signed(account),
        entity,
    )
}

pub(crate) fn _reject_pending_ownership(
    account: AccountId,
    entity: OwnableEntity<Test>,
) -> DispatchResult {
    Ownership::reject_pending_ownership(
        RuntimeOrigin::signed(account),
        entity,
    )
}

pub(crate) fn assert_create_transfers_ok() {
    let _m = use_static_mock();
    // `is_creator_active` should return `is_active`.
    let creator_staking_ctx = MockCreatorStaking::is_creator_active_context();
    creator_staking_ctx.expect().returning(|_| false).once();

    // Mock a transfer from account 1 to account 2 for a space entity
    assert_ok!(Ownership::transfer_ownership(
        RuntimeOrigin::signed(ACCOUNT1),
        OwnableEntity::Space(SPACE1),
        ACCOUNT2
    ));

    // Mock a transfer from account 1 to account 2 for a post entity
    assert_ok!(Ownership::transfer_ownership(
        RuntimeOrigin::signed(ACCOUNT1),
        OwnableEntity::Post(POST1),
        ACCOUNT2
    ));

    // Mock a transfer from account 1 to account 2 for a domain entity
    assert_ok!(Ownership::transfer_ownership(
        RuntimeOrigin::signed(ACCOUNT1),
        OwnableEntity::Domain(default_domain()),
        ACCOUNT2
    ));

    System::assert_has_event(OwnershipEvent::OwnershipTransferCreated {
        current_owner: ACCOUNT1,
        entity: default_space_entity(),
        new_owner: ACCOUNT2,
    }.into());

    System::assert_has_event(OwnershipEvent::OwnershipTransferCreated {
        current_owner: ACCOUNT1,
        entity: default_post_entity(),
        new_owner: ACCOUNT2,
    }.into());

    System::assert_has_event(OwnershipEvent::OwnershipTransferCreated {
        current_owner: ACCOUNT1,
        entity: default_domain_entity(),
        new_owner: ACCOUNT2,
    }.into());
}

pub(crate) fn assert_reject_transfers_ok(account: AccountId) {
    // Mock rejecting ownership transfer for a space entity
    assert_ok!(Ownership::reject_pending_ownership(
        RuntimeOrigin::signed(account),
        default_space_entity(),
    ));

    // Mock rejecting ownership transfer for a post entity
    assert_ok!(Ownership::reject_pending_ownership(
        RuntimeOrigin::signed(account),
        default_post_entity(),
    ));

    // Mock rejecting ownership transfer for a domain entity
    assert_ok!(Ownership::reject_pending_ownership(
        RuntimeOrigin::signed(account),
        default_domain_entity(),
    ));

    System::assert_has_event(OwnershipEvent::OwnershipTransferRejected {
        account,
        entity: default_space_entity(),
    }.into());

    System::assert_has_event(OwnershipEvent::OwnershipTransferRejected {
        account,
        entity: default_post_entity(),
    }.into());

    System::assert_has_event(OwnershipEvent::OwnershipTransferRejected {
        account,
        entity: default_domain_entity(),
    }.into());
}
