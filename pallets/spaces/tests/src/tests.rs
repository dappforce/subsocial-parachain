use frame_support::{assert_noop, assert_ok};

use pallet_permissions::SpacePermission as SP;
use pallet_spaces::Error as SpacesError;
use subsocial_support::{mock_functions::*, ContentError, ModerationError};

use crate::{mock::*, tests_utils::*};

#[test]
fn update_space_should_fail_when_account_is_blocked() {
    ExtBuilder::build_with_space().execute_with(|| {
        block_account_in_space_1();
        assert_noop!(
            _update_space(None, None, Some(update_for_space_content(updated_space_content()))),
            ModerationError::AccountIsBlocked,
        );
    });
}

#[test]
fn update_space_should_fail_when_content_is_blocked() {
    ExtBuilder::build_with_space().execute_with(|| {
        block_content_in_space_1();
        assert_noop!(
            _update_space(None, None, Some(space_update(Some(valid_content_ipfs()), None))),
            ModerationError::ContentIsBlocked,
        );
    });
}

#[test]
fn create_space_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_space()); // SpaceId 1

        // Check storages
        assert_eq!(Spaces::space_ids_by_owner(ACCOUNT1), vec![SPACE1]);
        assert_eq!(Spaces::next_space_id(), SPACE2);

        // Check whether data stored correctly
        let space = Spaces::space_by_id(SPACE1).unwrap();

        assert_eq!(space.created.account, ACCOUNT1);
        assert!(!space.edited);
        assert!(!space.hidden);

        assert_eq!(space.owner, ACCOUNT1);
        assert_eq!(space.content, space_content_ipfs());
    });
}

#[test]
fn create_space_should_work_with_permissions_override() {
    let perms = permissions_where_everyone_can_create_post();
    ExtBuilder::build_with_space_and_custom_permissions(perms.clone()).execute_with(|| {
        let space = Spaces::space_by_id(SPACE1).unwrap();
        assert_eq!(space.permissions, Some(perms));
    });
}

#[test]
fn create_post_should_work_overridden_space_permission_for_everyone() {
    ExtBuilder::build_with_space_and_custom_permissions(
        permissions_where_everyone_can_create_post(),
    )
    .execute_with(|| {
        assert_ok!(_create_post(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None, None));
    });
}

#[test]
fn create_post_should_work_overridden_space_permission_for_followers() {
    ExtBuilder::build_with_space_and_custom_permissions(
        permissions_where_follower_can_create_post(),
    )
    .execute_with(|| {
        assert_ok!(_default_follow_space());

        assert_ok!(_create_post(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None, None));
    });
}

#[test]
fn create_space_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error creating a space with invalid content
        assert_noop!(
            _create_space(None, Some(invalid_content_ipfs()), None),
            ContentError::InvalidIpfsCid,
        );
    });
}

#[test]
fn update_space_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        let expected_content_ipfs = updated_space_content();
        // Space update with ID 1 should be fine

        assert_ok!(_update_space(
            None, // From ACCOUNT1 (has permission as he's an owner)
            None,
            Some(space_update(Some(expected_content_ipfs.clone()), Some(true),))
        ));

        // Check whether space updates correctly
        let space = Spaces::space_by_id(SPACE1).unwrap();
        assert_eq!(space.content, expected_content_ipfs);
        assert!(space.hidden);
    });
}

#[test]
fn update_space_should_work_when_one_of_roles_is_permitted() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateSpace]).execute_with(
        || {
            let space_update = space_update(Some(updated_space_content()), Some(true));

            assert_ok!(_update_space(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                Some(SPACE1),
                Some(space_update)
            ));
        },
    );
}

#[test]
fn update_space_should_fail_when_no_updates_for_space_provided() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Try to catch an error updating a space with no changes
        assert_noop!(_update_space(None, None, None), SpacesError::<Test>::NoUpdatesForSpace);
    });
}

#[test]
fn update_space_should_fail_when_space_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Try to catch an error updating a space with wrong space ID
        assert_noop!(
            _update_space(
                None,
                Some(SPACE2),
                Some(update_for_space_content(updated_space_content()))
            ),
            SpacesError::<Test>::SpaceNotFound
        );
    });
}

#[test]
fn update_space_should_fail_when_account_has_no_permission_to_update_space() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Try to catch an error updating a space with an account that it not permitted
        assert_noop!(
            _update_space(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                None,
                Some(update_for_space_content(updated_space_content()))
            ),
            SpacesError::<Test>::NoPermissionToUpdateSpace
        );
    });
}

#[test]
fn update_space_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Try to catch an error updating a space with invalid content
        assert_noop!(
            _update_space(None, None, Some(space_update(Some(invalid_content_ipfs()), None,))),
            ContentError::InvalidIpfsCid,
        );
    });
}

#[test]
fn update_space_should_fail_when_no_right_permission_in_account_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateSpace]).execute_with(
        || {
            let space_update = space_update(Some(updated_space_content()), Some(true));

            assert_ok!(_delete_default_role());

            assert_noop!(
                _update_space(Some(RuntimeOrigin::signed(ACCOUNT2)), Some(SPACE1), Some(space_update)),
                SpacesError::<Test>::NoPermissionToUpdateSpace
            );
        },
    );
}

// TODO: refactor or remove. Deprecated tests
// Find public space ids tests
// --------------------------------------------------------------------------------------------
/*#[test]
fn find_public_space_ids_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_space(None, None, Some(Some(space_handle1())), None, None));

        let space_ids = Spaces::find_public_space_ids(0, 3);
        assert_eq!(space_ids, vec![SPACE1, SPACE2]);
    });
}

#[test]
fn find_public_space_ids_should_work_with_zero_offset() {
    ExtBuilder::build_with_space().execute_with(|| {
        let space_ids = Spaces::find_public_space_ids(0, 1);
        assert_eq!(space_ids, vec![SPACE1]);
    });
}

#[test]
fn find_public_space_ids_should_work_with_zero_limit() {
    ExtBuilder::build_with_space().execute_with(|| {
        let space_ids = Spaces::find_public_space_ids(1, 0);
        assert_eq!(space_ids, vec![SPACE1]);
    });
}

#[test]
fn find_public_space_ids_should_work_with_zero_offset_and_zero_limit() {
    ExtBuilder::build_with_space().execute_with(|| {
        let space_ids = Spaces::find_public_space_ids(0, 0);
        assert_eq!(space_ids, vec![]);
    });
}

// Find unlisted space ids tests
// --------------------------------------------------------------------------------------------

#[test]
fn find_unlisted_space_ids_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_space(None, None, Some(Some(space_handle1())), None, None));
        assert_ok!(
            _update_space(
                None,
                Some(SPACE1),
                Some(
                    space_update(
                        None,
                        None,
                        Some(Content::None),
                        Some(true),
                        None
                    )
                )
            )
        );

        assert_ok!(
            _update_space(
                None,
                Some(SPACE2),
                Some(
                    space_update(
                        None,
                        None,
                        Some(Content::None),
                        Some(true),
                        None
                    )
                )
            )
        );


        let space_ids = Spaces::find_unlisted_space_ids(0, 2);
        assert_eq!(space_ids, vec![SPACE1, SPACE2]);
    });
}

#[test]
fn find_unlisted_space_ids_should_work_with_zero_offset() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(
            _update_space(
                None,
                Some(SPACE1),
                Some(
                    space_update(
                        None,
                        None,
                        Some(Content::None),
                        Some(true),
                        None
                    )
                )
            )
        );

        let space_ids = Spaces::find_unlisted_space_ids(0, 1);
        assert_eq!(space_ids, vec![SPACE1]);
    });
}

#[test]
fn find_unlisted_space_ids_should_work_with_zero_limit() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(
            _update_space(
                None,
                Some(SPACE1),
                Some(
                    space_update(
                        None,
                        None,
                        Some(Content::None),
                        Some(true),
                        None
                    )
                )
            )
        );

        let space_ids = Spaces::find_unlisted_space_ids(1, 0);
        assert_eq!(space_ids, vec![]);
    });
}

#[test]
fn find_unlisted_space_ids_should_work_with_zero_offset_and_zero_limit() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(
            _update_space(
                None,
                Some(SPACE1),
                Some(
                    space_update(
                        None,
                        None,
                        Some(Content::None),
                        Some(true),
                        None
                    )
                )
            )
        );

        let space_ids = Spaces::find_unlisted_space_ids(0, 0);
        assert_eq!(space_ids, vec![]);
    });
}*/
