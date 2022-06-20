use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use sp_runtime::traits::Zero;

use pallet_utils::Error as UtilsError;
use pallet_spaces::Error as SpacesError;
use pallet_permissions::SpacePermission as SP;
use pallet_utils::mock_functions::*;

use crate::mock::*;
use crate::utils::*;
use crate::utils::posts_utils::*;
use crate::utils::spaces_utils::*;
use crate::utils::permissions_utils::*;
use crate::utils::moderation_utils::*;
use crate::utils::roles_utils::*;
use crate::utils::space_follows_utils::*;

#[test]
fn create_subspace_should_fail_when_content_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        block_content_in_space_1();
        assert_noop!(
            _create_subspace(
                None,
                Some(Some(SPACE1)),
                None,
                Some(valid_content_ipfs()),
                None,
            ),
            UtilsError::<TestRuntime>::ContentIsBlocked
        );
    });
}


#[test]
fn create_subspace_should_fail_when_account_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        block_account_in_space_1();
        assert_noop!(
            _create_subspace(
                None,
                Some(Some(SPACE1)),
                Some(Some(space_handle_2())),
                None,
                None,
            ),
            UtilsError::<TestRuntime>::AccountIsBlocked
        );
    });
}

#[test]
fn update_space_should_fail_when_account_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        block_account_in_space_1();
        assert_noop!(
            _update_space(
                None,
                None,
                Some(update_for_space_handle(Some(space_handle_2())))
            ),
            UtilsError::<TestRuntime>::AccountIsBlocked
        );
    });
}

#[test]
fn update_space_should_fail_when_content_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        block_content_in_space_1();
        assert_noop!(
            _update_space(
                None,
                None,
                Some(space_update(None, Some(valid_content_ipfs()), None))
            ),
            UtilsError::<TestRuntime>::ContentIsBlocked
        );
    });
}

#[test]
fn create_space_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_space()); // SpaceId 1

        // Check storages
        assert_eq!(Spaces::space_ids_by_owner(ACCOUNT1), vec![SPACE1]);
        assert_eq!(find_space_id_by_handle(space_handle()), Some(SPACE1));
        assert_eq!(Spaces::next_space_id(), SPACE2);

        // Check whether data stored correctly
        let space = Spaces::space_by_id(SPACE1).unwrap();

        assert_eq!(space.created.account, ACCOUNT1);
        assert!(space.updated.is_none());
        assert_eq!(space.hidden, false);

        assert_eq!(space.owner, ACCOUNT1);
        assert_eq!(space.handle, Some(space_handle()));
        assert_eq!(space.content, space_content_ipfs());

        assert_eq!(space.posts_count, 0);
        assert_eq!(space.followers_count, 1);
        assert!(SpaceHistory::edit_history(space.id).is_empty());

        // Check that the handle deposit has been reserved:
        let reserved_balance = Balances::reserved_balance(ACCOUNT1);
        assert_eq!(reserved_balance, HANDLE_DEPOSIT);
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
            assert_ok!(_create_post(
            Some(Origin::signed(ACCOUNT2)),
            None,
            None,
            None
        ));
        });
}

#[test]
fn create_post_should_work_overridden_space_permission_for_followers() {
    ExtBuilder::build_with_space_and_custom_permissions(
        permissions_where_follower_can_create_post(),
    )
        .execute_with(|| {
            assert_ok!(_default_follow_space());

            assert_ok!(_create_post(
            Some(Origin::signed(ACCOUNT2)),
            None,
            None,
            None
        ));
        });
}

#[test]
fn create_space_should_store_handle_lowercase() {
    ExtBuilder::build().execute_with(|| {
        let new_handle: Vec<u8> = b"sPaCe_hAnDlE".to_vec();

        assert_ok!(_create_space(
            None,
            Some(Some(new_handle.clone())),
            None,
            None
        )); // SpaceId 1

        // Handle should be lowercase in storage and original in struct
        let space = Spaces::space_by_id(SPACE1).unwrap();
        assert_eq!(space.handle, Some(new_handle.clone()));
        assert_eq!(find_space_id_by_handle(new_handle), Some(SPACE1));
    });
}

#[test]
fn create_space_should_fail_when_too_short_handle_provided() {
    ExtBuilder::build().execute_with(|| {
        let short_handle: Vec<u8> = vec![65; (MinHandleLen::get() - 1) as usize];

        // Try to catch an error creating a space with too short handle
        assert_noop!(
            _create_space(None, Some(Some(short_handle)), None, None),
            UtilsError::<TestRuntime>::HandleIsTooShort
        );
    });
}

#[test]
fn create_space_should_fail_when_too_long_handle_provided() {
    ExtBuilder::build().execute_with(|| {
        let long_handle: Vec<u8> = vec![65; (MaxHandleLen::get() + 1) as usize];

        // Try to catch an error creating a space with too long handle
        assert_noop!(
            _create_space(None, Some(Some(long_handle)), None, None),
            UtilsError::<TestRuntime>::HandleIsTooLong
        );
    });
}

#[test]
fn create_space_should_fail_when_not_unique_handle_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_space());
        // SpaceId 1
        // Try to catch an error creating a space with not unique handle
        assert_noop!(
            _create_default_space(),
            SpacesError::<TestRuntime>::SpaceHandleIsNotUnique
        );
    });
}

#[test]
fn create_space_should_fail_when_handle_contains_at_char() {
    ExtBuilder::build().execute_with(|| {
        let invalid_handle: Vec<u8> = b"@space_handle".to_vec();

        assert_noop!(
            _create_space(None, Some(Some(invalid_handle)), None, None),
            UtilsError::<TestRuntime>::HandleContainsInvalidChars
        );
    });
}

#[test]
fn create_space_should_fail_when_handle_contains_minus_char() {
    ExtBuilder::build().execute_with(|| {
        let invalid_handle: Vec<u8> = b"space-handle".to_vec();

        assert_noop!(
            _create_space(None, Some(Some(invalid_handle)), None, None),
            UtilsError::<TestRuntime>::HandleContainsInvalidChars
        );
    });
}

#[test]
fn create_space_should_fail_when_handle_contains_space_char() {
    ExtBuilder::build().execute_with(|| {
        let invalid_handle: Vec<u8> = b"space handle".to_vec();

        assert_noop!(
            _create_space(None, Some(Some(invalid_handle)), None, None),
            UtilsError::<TestRuntime>::HandleContainsInvalidChars
        );
    });
}

#[test]
fn create_space_should_fail_when_handle_contains_unicode() {
    ExtBuilder::build().execute_with(|| {
        let invalid_handle: Vec<u8> = String::from("блог_хендл").into_bytes().to_vec();

        assert_noop!(
            _create_space(None, Some(Some(invalid_handle)), None, None),
            UtilsError::<TestRuntime>::HandleContainsInvalidChars
        );
    });
}

#[test]
fn create_space_should_fail_when_handles_are_disabled() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_update_space_settings_with_handles_disabled());

        assert_noop!(
            _create_default_space(),
            SpacesError::<TestRuntime>::HandlesAreDisabled
        );
    });
}

#[test]
fn create_space_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error creating a space with invalid content
        assert_noop!(
            _create_space(None, None, Some(invalid_content_ipfs()), None),
            UtilsError::<TestRuntime>::InvalidIpfsCid
        );
    });
}

#[test]
fn update_space_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        let new_handle: Vec<u8> = b"new_handle".to_vec();
        let expected_content_ipfs = updated_space_content();
        // Space update with ID 1 should be fine

        assert_ok!(_update_space(
            None, // From ACCOUNT1 (has permission as he's an owner)
            None,
            Some(space_update(
                Some(Some(new_handle.clone())),
                Some(expected_content_ipfs.clone()),
                Some(true),
            ))
        ));

        // Check whether space updates correctly
        let space = Spaces::space_by_id(SPACE1).unwrap();
        assert_eq!(space.handle, Some(new_handle.clone()));
        assert_eq!(space.content, expected_content_ipfs);
        assert_eq!(space.hidden, true);

        // Check whether history recorded correctly
        let edit_history = &SpaceHistory::edit_history(space.id)[0];
        assert_eq!(edit_history.old_data.handle, Some(Some(space_handle())));
        assert_eq!(edit_history.old_data.content, Some(space_content_ipfs()));
        assert_eq!(edit_history.old_data.hidden, Some(false));

        assert_eq!(find_space_id_by_handle(space_handle()), None);
        assert_eq!(find_space_id_by_handle(new_handle), Some(SPACE1));

        // Check that the handle deposit has been reserved:
        let reserved_balance = Balances::reserved_balance(ACCOUNT1);
        assert_eq!(reserved_balance, HANDLE_DEPOSIT);
    });
}

#[test]
fn update_space_should_work_when_one_of_roles_is_permitted() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateSpace]).execute_with(
        || {
            let space_update = space_update(
                Some(Some(b"new_handle".to_vec())),
                Some(updated_space_content()),
                Some(true),
            );

            assert_ok!(_update_space(
                Some(Origin::signed(ACCOUNT2)),
                Some(SPACE1),
                Some(space_update)
            ));
        },
    );
}

#[test]
fn update_space_should_work_when_unreserving_handle() {
    ExtBuilder::build_with_space().execute_with(|| {
        let no_handle = None;
        let space_update = update_for_space_handle(no_handle);
        assert_ok!(_update_space(None, None, Some(space_update)));

        // Check that the space handle is unreserved after this update:
        let space = Spaces::space_by_id(SPACE1).unwrap();
        assert_eq!(space.handle, None);

        // Check that the previous space handle has been added to the space history:
        let edit_history = &SpaceHistory::edit_history(space.id)[0];
        assert_eq!(edit_history.old_data.handle, Some(Some(space_handle())));

        // Check that the previous space handle is not reserved in storage anymore:
        assert_eq!(find_space_id_by_handle(space_handle()), None);

        // Check that the handle deposit has been unreserved:
        let reserved_balance = Balances::reserved_balance(ACCOUNT1);
        assert!(reserved_balance.is_zero());
    });
}

#[test]
fn should_update_space_content_when_handles_disabled() {
    ExtBuilder::build_with_space_then_disable_handles().execute_with(|| {
        let space_update = update_for_space_content(updated_space_content());
        assert_ok!(_update_space(None, None, Some(space_update)));
    });
}

#[test]
fn should_fail_to_update_space_handle_when_handles_disabled() {
    ExtBuilder::build_with_space_then_disable_handles().execute_with(|| {
        let space_update = update_for_space_handle(Some(space_handle_2()));
        assert_noop!(
            _update_space(None, None, Some(space_update)),
            SpacesError::<TestRuntime>::HandlesAreDisabled
        );
    });
}

#[test]
fn update_space_should_fail_when_no_updates_for_space_provided() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Try to catch an error updating a space with no changes
        assert_noop!(
            _update_space(None, None, None),
            SpacesError::<TestRuntime>::NoUpdatesForSpace
        );
    });
}

#[test]
fn update_space_should_fail_when_space_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        let new_handle: Vec<u8> = b"new_handle".to_vec();

        // Try to catch an error updating a space with wrong space ID
        assert_noop!(
            _update_space(
                None,
                Some(SPACE2),
                Some(update_for_space_handle(Some(new_handle)))
            ),
            SpacesError::<TestRuntime>::SpaceNotFound
        );
    });
}

#[test]
fn update_space_should_fail_when_account_has_no_permission_to_update_space() {
    ExtBuilder::build_with_space().execute_with(|| {
        let new_handle: Vec<u8> = b"new_handle".to_vec();

        // Try to catch an error updating a space with an account that it not permitted
        assert_noop!(
            _update_space(
                Some(Origin::signed(ACCOUNT2)),
                None,
                Some(update_for_space_handle(Some(new_handle)))
            ),
            SpacesError::<TestRuntime>::NoPermissionToUpdateSpace
        );
    });
}

#[test]
fn update_space_should_fail_when_too_short_handle_provided() {
    ExtBuilder::build_with_space().execute_with(|| {
        let short_handle: Vec<u8> = vec![65; (MinHandleLen::get() - 1) as usize];

        // Try to catch an error updating a space with too short handle
        assert_noop!(
            _update_space(
                None,
                None,
                Some(update_for_space_handle(Some(short_handle)))
            ),
            UtilsError::<TestRuntime>::HandleIsTooShort
        );
    });
}

#[test]
fn update_space_should_fail_when_too_long_handle_provided() {
    ExtBuilder::build_with_space().execute_with(|| {
        let long_handle: Vec<u8> = vec![65; (MaxHandleLen::get() + 1) as usize];

        // Try to catch an error updating a space with too long handle
        assert_noop!(
            _update_space(None, None, Some(update_for_space_handle(Some(long_handle)))),
            UtilsError::<TestRuntime>::HandleIsTooLong
        );
    });
}

#[test]
fn update_space_should_fail_when_not_unique_handle_provided() {
    ExtBuilder::build_with_space().execute_with(|| {
        let handle: Vec<u8> = b"unique_handle".to_vec();

        assert_ok!(_create_space(None, Some(Some(handle.clone())), None, None)); // SpaceId 2 with a custom handle

        // Should fail when updating a space 1 with a handle of a space 2:
        assert_noop!(
            _update_space(
                None,
                Some(SPACE1),
                Some(update_for_space_handle(Some(handle)))
            ),
            SpacesError::<TestRuntime>::SpaceHandleIsNotUnique
        );
    });
}

#[test]
fn update_space_should_fail_when_handle_contains_at_char() {
    ExtBuilder::build_with_space().execute_with(|| {
        let invalid_handle: Vec<u8> = b"@space_handle".to_vec();

        assert_noop!(
            _update_space(
                None,
                None,
                Some(update_for_space_handle(Some(invalid_handle)))
            ),
            UtilsError::<TestRuntime>::HandleContainsInvalidChars
        );
    });
}

#[test]
fn update_space_should_fail_when_handle_contains_minus_char() {
    ExtBuilder::build_with_space().execute_with(|| {
        let invalid_handle: Vec<u8> = b"space-handle".to_vec();

        assert_noop!(
            _update_space(
                None,
                None,
                Some(update_for_space_handle(Some(invalid_handle)))
            ),
            UtilsError::<TestRuntime>::HandleContainsInvalidChars
        );
    });
}

#[test]
fn update_space_should_fail_when_handle_contains_space_char() {
    ExtBuilder::build_with_space().execute_with(|| {
        let invalid_handle: Vec<u8> = b"space handle".to_vec();

        assert_noop!(
            _update_space(
                None,
                None,
                Some(update_for_space_handle(Some(invalid_handle)))
            ),
            UtilsError::<TestRuntime>::HandleContainsInvalidChars
        );
    });
}

#[test]
fn update_space_should_fail_when_handle_contains_unicode() {
    ExtBuilder::build_with_space().execute_with(|| {
        let invalid_handle: Vec<u8> = String::from("блог_хендл").into_bytes().to_vec();

        assert_noop!(
            _update_space(
                None,
                None,
                Some(update_for_space_handle(Some(invalid_handle)))
            ),
            UtilsError::<TestRuntime>::HandleContainsInvalidChars
        );
    });
}

#[test]
fn update_space_should_fail_when_handles_are_disabled() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_update_space_settings_with_handles_disabled());
        let space_update = update_for_space_handle(Some(space_handle_2()));

        assert_noop!(
            _update_space(None, None, Some(space_update)),
            SpacesError::<TestRuntime>::HandlesAreDisabled
        );
    });
}

#[test]
fn update_space_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Try to catch an error updating a space with invalid content
        assert_noop!(
            _update_space(
                None,
                None,
                Some(space_update(None, Some(invalid_content_ipfs()), None,))
            ),
            UtilsError::<TestRuntime>::InvalidIpfsCid
        );
    });
}

#[test]
fn update_space_should_fail_when_no_right_permission_in_account_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateSpace]).execute_with(
        || {
            let space_update = space_update(
                Some(Some(b"new_handle".to_vec())),
                Some(updated_space_content()),
                Some(true),
            );

            assert_ok!(_delete_default_role());

            assert_noop!(
                _update_space(
                    Some(Origin::signed(ACCOUNT2)),
                    Some(SPACE1),
                    Some(space_update)
                ),
                SpacesError::<TestRuntime>::NoPermissionToUpdateSpace
            );
        },
    );
}

#[test]
fn update_space_settings_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_update_space_settings_with_handles_disabled());

        let spaces_settings = Spaces::settings();
        // Ensure that `handles_enabled` field is false
        assert!(!spaces_settings.handles_enabled);
    });
}

#[test]
fn update_space_settings_should_fail_when_account_is_not_root() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _update_space_settings(Some(Origin::signed(ACCOUNT1)), None),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn update_space_settings_should_fail_when_same_settings_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _update_space_settings_with_handles_enabled(),
            SpacesError::<TestRuntime>::NoUpdatesForSpacesSettings
        );
    });
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