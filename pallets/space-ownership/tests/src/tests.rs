// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use frame_support::{assert_noop, assert_ok};

use pallet_ownership::{EntityWithOwnership, Error as OwnershipError, Event as OwnershipEvent};

use crate::{mock::*, tests_utils::*};

#[test]
fn transfer_ownership_works() {
    ExtBuilder::build_with_pending_transfers();
}

#[test]
fn accept_pending_ownership_works() {
    ExtBuilder::build_with_pending_transfers().execute_with(|| {
        let _m = use_static_mock();
        let creator_staking_ctx = MockCreatorStaking::is_creator_active_context();

        // `is_creator_active` should return `false`.
        creator_staking_ctx.expect().returning(|_| false).once();

        // Mock accepting ownership transfer for a space entity
        assert_ok!(Ownership::accept_pending_ownership(
            RuntimeOrigin::signed(ACCOUNT2),
            default_space_entity(),
        ));

        // Mock accepting ownership transfer for a post entity
        assert_ok!(Ownership::accept_pending_ownership(
            RuntimeOrigin::signed(ACCOUNT2),
            default_post_entity(),
        ));

        // Mock accepting ownership transfer for a domain entity
        assert_ok!(Ownership::accept_pending_ownership(
            RuntimeOrigin::signed(ACCOUNT2),
            default_domain_entity(),
        ));

        System::assert_has_event(OwnershipEvent::OwnershipTransferAccepted {
            account: ACCOUNT2,
            entity: default_space_entity(),
        }.into());

        System::assert_has_event(OwnershipEvent::OwnershipTransferAccepted {
            account: ACCOUNT2,
            entity: default_post_entity(),
        }.into());

        System::assert_has_event(OwnershipEvent::OwnershipTransferAccepted {
            account: ACCOUNT2,
            entity: default_domain_entity(),
        }.into());
    });
}

#[test]
fn reject_pending_ownership_works() {
    ExtBuilder::build_with_pending_transfers().execute_with(|| {
        // Rejecting ownership transfer by the current owner
        assert_reject_transfers_ok(ACCOUNT1);

        // Re-create the pending transfers
        assert_create_transfers_ok();

        // Rejecting ownership transfer by a target account
        assert_reject_transfers_ok(ACCOUNT2);
    });
}

#[test]
fn transfer_ownership_should_not_allow_transfer_to_current_owner() {
    ExtBuilder::build_with_all_enitities().execute_with(|| {
        // Mock a transfer from account 1 to itself for a space entity
        assert_noop!(
            Ownership::transfer_ownership(
                RuntimeOrigin::signed(ACCOUNT1),
                EntityWithOwnership::Space(SPACE1),
                ACCOUNT1
            ),
            OwnershipError::<Test>::CannotTransferToCurrentOwner
        );
    });
}

#[test]
fn transfer_ownership_should_not_allow_active_creator_to_transfer_space_ownership() {
    ExtBuilder::build_with_all_enitities().execute_with(|| {
        let _m = use_static_mock();
        let creator_staking_ctx = MockCreatorStaking::is_creator_active_context();

        // `is_creator_active` should return `true`.
        creator_staking_ctx.expect().returning(|_| true).once();

        // Mock a transfer from an active creator (creator is active in this test)
        assert_noop!(
            Ownership::transfer_ownership(
                RuntimeOrigin::signed(ACCOUNT1),
                EntityWithOwnership::Space(SPACE1),
                ACCOUNT2
            ),
            OwnershipError::<Test>::ActiveCreatorCannotTransferOwnership
        );
    });
}

#[test]
fn accept_pending_ownership_should_fail_if_no_pending_transfer() {
    ExtBuilder::build_with_all_enitities().execute_with(|| {
        // Mock accepting ownership transfer for a space entity with no pending transfer
        assert_noop!(
            Ownership::accept_pending_ownership(
                RuntimeOrigin::signed(ACCOUNT2),
                EntityWithOwnership::Space(SPACE1)
            ),
            OwnershipError::<Test>::NoPendingTransfer
        );
    });
}

#[test]
fn accept_pending_ownership_should_not_allow_non_target_to_accept() {
    ExtBuilder::build_with_pending_transfers().execute_with(|| {
        let _m = use_static_mock();
        let creator_staking_ctx = MockCreatorStaking::is_creator_active_context();

        // `is_creator_active` should return `true` once.
        creator_staking_ctx.expect().returning(|_| false).once();

        // Mock accepting ownership transfer for a space entity by a non-owner
        assert_noop!(
            Ownership::accept_pending_ownership(
                RuntimeOrigin::signed(ACCOUNT3),
                EntityWithOwnership::Space(SPACE1)
            ),
            OwnershipError::<Test>::NotAllowedToAcceptOwnershipTransfer
        );

        assert_ok!(
            Ownership::accept_pending_ownership(
                RuntimeOrigin::signed(ACCOUNT2),
                EntityWithOwnership::Space(SPACE1)
            )
        );
    });
}

#[test]
fn reject_pending_ownership_should_fail_if_no_pending_transfer() {
    ExtBuilder::build_with_all_enitities().execute_with(|| {
        // Mock rejecting ownership transfer for a space entity with no pending transfer
        assert_noop!(
            Ownership::reject_pending_ownership(
                RuntimeOrigin::signed(ACCOUNT1),
                EntityWithOwnership::Space(SPACE1)
            ),
            OwnershipError::<Test>::NoPendingTransfer
        );
    });
}

#[test]
fn reject_pending_ownership_should_not_allow_non_owner_to_reject() {
    ExtBuilder::build_with_pending_transfers().execute_with(|| {
        // Mock rejecting ownership transfer for a space entity by a non-owner
        assert_noop!(
            Ownership::reject_pending_ownership(
                RuntimeOrigin::signed(ACCOUNT3),
                EntityWithOwnership::Space(SPACE1)
            ),
            OwnershipError::<Test>::NotAllowedToRejectOwnershipTransfer
        );

        assert_ok!(
            Ownership::reject_pending_ownership(
                RuntimeOrigin::signed(ACCOUNT1),
                EntityWithOwnership::Space(SPACE1)
            )
        );
    });
}
