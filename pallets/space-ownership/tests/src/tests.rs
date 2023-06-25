// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::Zero;

use pallet_space_ownership::Error as SpaceOwnershipError;
use pallet_spaces::Error as SpacesError;

use crate::{mock::*, tests_utils::*};

#[test]
fn transfer_space_ownership_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2

        assert_eq!(SpaceOwnership::pending_space_owner(SPACE1).unwrap(), ACCOUNT2);
    });
}

#[test]
fn transfer_space_ownership_should_fail_when_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_transfer_default_space_ownership(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn transfer_space_ownership_should_fail_when_account_is_not_space_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _transfer_space_ownership(Some(RuntimeOrigin::signed(ACCOUNT2)), None, Some(ACCOUNT1)),
            SpacesError::<Test>::NotASpaceOwner
        );
    });
}

#[test]
fn transfer_space_ownership_should_fail_when_trying_to_transfer_to_current_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _transfer_space_ownership(Some(RuntimeOrigin::signed(ACCOUNT1)), None, Some(ACCOUNT1)),
            SpaceOwnershipError::<Test>::CannotTransferToCurrentOwner
        );
    });
}

#[test]
fn accept_pending_ownership_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2:
        assert_ok!(_transfer_default_space_ownership());

        // Account 2 accepts the transfer of ownership:
        assert_ok!(_accept_default_pending_ownership());

        // Check that Account 2 is a new space owner:
        let space = Spaces::space_by_id(SPACE1).unwrap();
        assert_eq!(space.owner, ACCOUNT2);

        // Check that pending storage is cleared:
        assert!(SpaceOwnership::pending_space_owner(SPACE1).is_none());

        assert!(Balances::reserved_balance(ACCOUNT1).is_zero());

        // assert_eq!(Balances::reserved_balance(ACCOUNT2), HANDLE_DEPOSIT);
    });
}

#[test]
fn accept_pending_ownership_should_fail_when_space_not_found() {
    ExtBuilder::build_with_pending_ownership_transfer_no_space().execute_with(|| {
        assert_noop!(_accept_default_pending_ownership(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn accept_pending_ownership_should_fail_when_no_pending_transfer_for_space() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _accept_default_pending_ownership(),
            SpaceOwnershipError::<Test>::NoPendingTransferOnSpace
        );
    });
}

#[test]
fn accept_pending_ownership_should_fail_if_origin_is_already_an_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership());

        assert_noop!(
            _accept_pending_ownership(Some(RuntimeOrigin::signed(ACCOUNT1)), None),
            SpaceOwnershipError::<Test>::AlreadyASpaceOwner
        );
    });
}

#[test]
fn accept_pending_ownership_should_fail_if_origin_is_not_equal_to_pending_account() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership());

        assert_noop!(
            _accept_pending_ownership(Some(RuntimeOrigin::signed(ACCOUNT3)), None),
            SpaceOwnershipError::<Test>::NotAllowedToAcceptOwnershipTransfer
        );
    });
}

#[test]
fn reject_pending_ownership_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership());
        // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2
        assert_ok!(_reject_default_pending_ownership()); // Rejecting a transfer from ACCOUNT2

        // Check whether owner was not changed
        let space = Spaces::space_by_id(SPACE1).unwrap();
        assert_eq!(space.owner, ACCOUNT1);

        // Check whether storage state is correct
        assert!(SpaceOwnership::pending_space_owner(SPACE1).is_none());
    });
}

#[test]
fn reject_pending_ownership_should_work_when_proposal_rejected_by_current_space_owner() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership());
        // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2
        assert_ok!(_reject_default_pending_ownership_by_current_owner()); // Rejecting a transfer from ACCOUNT2

        // Check whether owner was not changed
        let space = Spaces::space_by_id(SPACE1).unwrap();
        assert_eq!(space.owner, ACCOUNT1);

        // Check whether storage state is correct
        assert!(SpaceOwnership::pending_space_owner(SPACE1).is_none());
    });
}

#[test]
fn reject_pending_ownership_should_fail_when_space_not_found() {
    ExtBuilder::build_with_pending_ownership_transfer_no_space().execute_with(|| {
        assert_noop!(_reject_default_pending_ownership(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn reject_pending_ownership_should_fail_when_no_pending_transfer_on_space() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _reject_default_pending_ownership(),
            SpaceOwnershipError::<Test>::NoPendingTransferOnSpace
        ); // Rejecting a transfer from ACCOUNT2
    });
}

#[test]
fn reject_pending_ownership_should_fail_when_account_is_not_allowed_to_reject() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_transfer_default_space_ownership()); // Transfer SpaceId 1 owned by ACCOUNT1 to ACCOUNT2

        assert_noop!(
            _reject_pending_ownership(Some(RuntimeOrigin::signed(ACCOUNT3)), None),
            SpaceOwnershipError::<Test>::NotAllowedToRejectOwnershipTransfer
        ); // Rejecting a transfer from ACCOUNT2
    });
}
