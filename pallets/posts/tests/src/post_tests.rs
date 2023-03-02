use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

use pallet_permissions::SpacePermission as SP;
use pallet_posts::{Error as PostsError, Post};
use pallet_spaces::Error as SpacesError;
use subsocial_support::{mock_functions::*, ContentError, ModerationError, PostId, SpaceId};

use crate::{mock::*, tests_utils::*};

#[test]
fn create_post_should_fail_when_content_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        block_content_in_space_1();
        assert_noop!(
            _create_post(None, None, None, Some(valid_content_ipfs()),),
            DispatchError::Other(ModerationError::ContentIsBlocked.into()),
        );
    });
}

#[test]
fn create_post_should_fail_when_account_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        block_account_in_space_1();
        assert_noop!(
            _create_post(None, None, None, Some(valid_content_ipfs()),),
            DispatchError::Other(ModerationError::AccountIsBlocked.into()),
        );
    });
}

#[test]
fn update_post_should_fail_when_content_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        block_content_in_space_1();
        assert_noop!(
            _update_post(
                None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
                None,
                Some(post_update(None, Some(valid_content_ipfs()), Some(true)))
            ),
            DispatchError::Other(ModerationError::ContentIsBlocked.into())
        );
    });
}

#[test]
fn update_post_should_fail_when_account_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        block_account_in_space_1();
        assert_noop!(
            _update_post(
                None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
                None,
                Some(post_update(None, Some(valid_content_ipfs()), Some(true)))
            ),
            DispatchError::Other(ModerationError::AccountIsBlocked.into())
        );
    });
}

// FIXME: uncomment when `update_post` will be able to move post from one space to another
/*
#[test]
fn update_post_should_fail_when_post_is_blocked() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(
            _update_entity_status(
                None,
                Some(EntityId::Post(POST1)),
                Some(SPACE1),
                Some(Some(EntityStatus::Blocked))
            )
        );
        assert_noop!(
            _update_post(
                None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
                Some(POST1),
                Some(
                    post_update(
                        Some(SPACE1),
                        None,
                        None
                    )
                )
            ), ModerationError::PostIsBlocked.into()
        );
    });
}
*/

#[test]
fn create_post_should_work() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_default_post()); // PostId 1 by ACCOUNT1 which is permitted by default

        // Check storages
        assert_eq!(Posts::post_ids_by_space_id(SPACE1), vec![POST1]);
        assert_eq!(Posts::next_post_id(), POST2);

        // Check whether data stored correctly
        let post = Posts::post_by_id(POST1).unwrap();

        assert_eq!(post.created.account, ACCOUNT1);
        assert!(!post.edited);
        assert!(!post.hidden);

        assert_eq!(post.space_id, Some(SPACE1));
        assert_eq!(post.extension, extension_regular_post());

        assert_eq!(post.content, post_content_ipfs());

        assert_eq!(post.upvotes_count, 0);
        assert_eq!(post.downvotes_count, 0);
    });
}

#[test]
fn create_post_should_work_when_one_of_roles_is_permitted() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(
        || {
            assert_ok!(_create_post(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                None, // SpaceId 1,
                None, // RegularPost extension
                None, // Default post content
            ));
        },
    );
}

#[test]
fn create_post_should_fail_when_post_has_no_space_id() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _create_post(None, Some(None), None, None),
            PostsError::<Test>::PostHasNoSpaceId
        );
    });
}

#[test]
fn create_post_should_fail_when_space_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(_create_default_post(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn create_post_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build_with_space().execute_with(|| {
        // Try to catch an error creating a regular post with invalid content
        assert_noop!(
            _create_post(None, None, None, Some(invalid_content_ipfs())),
            DispatchError::from(ContentError::InvalidIpfsCid)
        );
    });
}

#[test]
fn create_post_should_fail_when_account_has_no_permission() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_noop!(
            _create_post(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None, None),
            PostsError::<Test>::NoPermissionToCreatePosts
        );
    });
}

#[test]
fn create_post_should_fail_when_no_right_permission_in_account_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(
        || {
            assert_ok!(_delete_default_role());

            assert_noop!(
                _create_post(
                    Some(RuntimeOrigin::signed(ACCOUNT2)),
                    None, // SpaceId 1,
                    None, // RegularPost extension
                    None, // Default post content
                ),
                PostsError::<Test>::NoPermissionToCreatePosts
            );
        },
    );
}

#[test]
fn update_post_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        let expected_content_ipfs = updated_post_content();

        // Post update with ID 1 should be fine
        assert_ok!(_update_post(
            None, // From ACCOUNT1 (has default permission to UpdateOwnPosts)
            None,
            Some(post_update(None, Some(expected_content_ipfs.clone()), Some(true)))
        ));

        // Check whether post updates correctly
        let post = Posts::post_by_id(POST1).unwrap();
        assert_eq!(post.space_id, Some(SPACE1));
        assert_eq!(post.content, expected_content_ipfs);
        assert!(post.hidden);
    });
}

fn check_if_post_moved_correctly(moved_post_id: PostId, expected_new_space_id: SpaceId) {
    let post: Post<Test> = Posts::post_by_id(moved_post_id).unwrap(); // `POST2` is a comment
    let new_space_id = post.space_id.unwrap();

    // Check that space id of the post has been updated from 1 to 2
    assert_eq!(new_space_id, expected_new_space_id);
}

#[test]
fn move_post_should_work() {
    ExtBuilder::build_with_post_and_two_spaces().execute_with(|| {
        assert_ok!(_move_post_1_to_space_2());

        let moved_post_id = POST1;
        let old_space_id = SPACE1;
        let expected_new_space_id = SPACE2;
        check_if_post_moved_correctly(moved_post_id, expected_new_space_id);

        // Check that there are no posts ids in the old space
        assert!(Posts::post_ids_by_space_id(old_space_id).is_empty());

        // Check that there is the post id in the new space
        assert_eq!(Posts::post_ids_by_space_id(expected_new_space_id), vec![moved_post_id]);
    });
}

#[test]
fn move_post_should_work_when_space_id_none() {
    ExtBuilder::build_with_post_and_two_spaces().execute_with(|| {
        let moved_post_id = POST1;
        let old_space_id = SPACE1; // Where post were before moving to `SpaceId:None`
        let expected_new_space_id = SPACE2;

        assert_ok!(_move_post_to_nowhere(moved_post_id));
        assert_ok!(_move_post_1_to_space_2());

        check_if_post_moved_correctly(moved_post_id, expected_new_space_id);

        // Check that there are no posts ids in the old space
        assert!(Posts::post_ids_by_space_id(old_space_id).is_empty());

        // Check that there is the post id in the new space
        assert_eq!(Posts::post_ids_by_space_id(expected_new_space_id), vec![moved_post_id]);
    });
}

#[test]
fn move_hidden_post_should_work() {
    ExtBuilder::build_with_post_and_two_spaces().execute_with(|| {
        let moved_post_id = POST1;
        let old_space_id = SPACE1;
        let expected_new_space_id = SPACE2;

        // Hide the post before moving it
        assert_ok!(_update_post(
            None,
            Some(moved_post_id),
            Some(post_update(None, None, Some(true)))
        ));

        assert_ok!(_move_post_1_to_space_2());

        check_if_post_moved_correctly(moved_post_id, expected_new_space_id);

        // Check that there are no posts ids in the old space
        assert!(Posts::post_ids_by_space_id(old_space_id).is_empty());

        // Check that there is the post id in the new space
        assert_eq!(Posts::post_ids_by_space_id(expected_new_space_id), vec![moved_post_id]);
    });
}

#[test]
fn move_hidden_post_should_fail_when_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Note that we have not created a post that we are trying to move
        assert_noop!(_move_post_1_to_space_2(), PostsError::<Test>::PostNotFound);
    });
}

#[test]
fn move_hidden_post_should_fail_when_provided_space_not_found() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Note that we have not created a new space #2 before moving the post
        assert_noop!(_move_post_1_to_space_2(), SpacesError::<Test>::SpaceNotFound);
    });
}

#[test]
fn move_hidden_post_should_fail_origin_has_no_permission_to_create_posts() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Create a space #2 from account #2
        assert_ok!(_create_space(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None));

        // Should not be possible to move the post b/c it's owner is account #1
        // when the space #2 is owned by account #2
        assert_noop!(_move_post_1_to_space_2(), PostsError::<Test>::NoPermissionToCreatePosts);
    });
}

#[test]
fn move_post_should_fail_when_account_has_no_permission() {
    ExtBuilder::build_with_post_and_two_spaces().execute_with(|| {
        assert_noop!(
            _move_post(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None),
            PostsError::<Test>::NoPermissionToUpdateAnyPost
        );
    });
}

#[test]
fn move_post_should_fail_when_space_none_and_account_is_not_post_owner() {
    ExtBuilder::build_with_post_and_two_spaces().execute_with(|| {
        assert_ok!(_move_post_to_nowhere(POST1));
        assert_noop!(
            _move_post(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None),
            PostsError::<Test>::NotAPostOwner
        );
    });
}

#[test]
fn should_fail_when_trying_to_move_comment() {
    ExtBuilder::build_with_comment().execute_with(|| {
        assert_ok!(_create_space(None, None, None));

        // Comments cannot be moved, they stick to their parent post
        assert_noop!(
            _move_post(None, Some(POST2), None),
            PostsError::<Test>::CannotUpdateSpaceIdOnComment
        );
    });
}

#[test]
fn update_post_should_work_after_transfer_space_ownership() {
    ExtBuilder::build_with_post().execute_with(|| {
        let post_update = post_update(None, Some(updated_post_content()), Some(true));

        assert_ok!(_transfer_default_space_ownership());

        // Post update with ID 1 should be fine
        assert_ok!(_update_post(None, None, Some(post_update)));
    });
}

#[test]
fn update_any_post_should_work_when_account_has_default_permission() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(
        || {
            let post_update = post_update(None, Some(updated_post_content()), Some(true));
            assert_ok!(_create_post(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                None, // SpaceId 1
                None, // RegularPost extension
                None  // Default post content
            )); // PostId 1

            // Post update with ID 1 should be fine
            assert_ok!(_update_post(
                None, // From ACCOUNT1 (has default permission to UpdateAnyPosts as SpaceOwner)
                Some(POST1),
                Some(post_update)
            ));
        },
    );
}

#[test]
fn update_any_post_should_work_when_one_of_roles_is_permitted() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateAnyPost]).execute_with(
        || {
            let post_update = post_update(None, Some(updated_post_content()), Some(true));
            assert_ok!(_create_default_post()); // PostId 1

            // Post update with ID 1 should be fine
            assert_ok!(_update_post(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                Some(POST1),
                Some(post_update)
            ));
        },
    );
}

#[test]
fn update_post_should_fail_when_no_updates_for_post_provided() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Try to catch an error updating a post with no changes
        assert_noop!(_update_post(None, None, None), PostsError::<Test>::NoUpdatesForPost);
    });
}

#[test]
fn update_post_should_fail_when_post_not_found() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(None, None, None)); // SpaceId 2

        // Try to catch an error updating a post with wrong post ID
        assert_noop!(
            _update_post(
                None,
                Some(POST2),
                Some(post_update(
                    // FIXME: when Post's `space_id` update is fully implemented
                    None, /* Some(SPACE2) */
                    None,
                    Some(true) /* None */
                ))
            ),
            PostsError::<Test>::PostNotFound
        );
    });
}

#[test]
fn update_post_should_fail_when_account_has_no_permission_to_update_any_post() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(None, None, None)); // SpaceId 2

        // Try to catch an error updating a post with different account
        assert_noop!(
            _update_post(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                None,
                Some(post_update(
                    // FIXME: when Post's `space_id` update is fully implemented
                    None, /* Some(SPACE2) */
                    None,
                    Some(true) /* None */
                ))
            ),
            PostsError::<Test>::NoPermissionToUpdateAnyPost
        );
    });
}

#[test]
fn update_post_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Try to catch an error updating a post with invalid content
        assert_noop!(
            _update_post(None, None, Some(post_update(None, Some(invalid_content_ipfs()), None))),
            DispatchError::from(ContentError::InvalidIpfsCid)
        );
    });
}

#[test]
fn update_post_should_fail_when_no_right_permission_in_account_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::UpdateAnyPost]).execute_with(
        || {
            let post_update = post_update(None, Some(updated_post_content()), Some(true));
            assert_ok!(_create_default_post());
            // PostId 1
            assert_ok!(_delete_default_role());

            // Post update with ID 1 should be fine
            assert_noop!(
                _update_post(Some(RuntimeOrigin::signed(ACCOUNT2)), Some(POST1), Some(post_update)),
                PostsError::<Test>::NoPermissionToUpdateAnyPost
            );
        },
    );
}

// TODO: refactor or remove. Deprecated tests
// Find public post ids tests
// --------------------------------------------------------------------------------------------
/*#[test]
fn find_public_post_ids_in_space_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post(None, Some(Some(SPACE1)), None, None));

        let post_ids = Posts::find_public_post_ids_in_space(SPACE1, 0, 3);
        assert_eq!(post_ids, vec![POST1, POST2]);
    });
}

#[test]
fn find_public_post_ids_in_space_should_work_with_zero_offset() {
    ExtBuilder::build_with_post().execute_with(|| {
        let post_ids = Posts::find_public_post_ids_in_space(SPACE1, 0, 1);
        assert_eq!(post_ids, vec![POST1]);
    });
}

#[test]
fn find_public_post_ids_in_space_should_work_with_zero_limit() {
    ExtBuilder::build_with_post().execute_with(|| {
        let post_ids = Posts::find_public_post_ids_in_space(SPACE1, 1, 0);
        assert_eq!(post_ids, vec![POST1]);
    });
}

#[test]
fn find_public_post_ids_in_space_should_work_with_zero_offset_and_zero_limit() {
    ExtBuilder::build_with_post().execute_with(|| {
        let post_ids = Posts::find_public_post_ids_in_space(SPACE1, 0, 0);
        assert_eq!(post_ids, vec![]);
    });
}

// Find unlisted post ids tests
// --------------------------------------------------------------------------------------------

#[test]
fn find_unlisted_post_ids_in_space_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post(None, Some(Some(SPACE1)), None, None));
        assert_ok!(
            _update_post(
                None,
                None,
                Some(
                    post_update(
                        None,
                        Some(Content::None),
                        Some(true))
                )
            )
        );
        assert_ok!(
            _update_post(
                None,
                Some(POST2),
                Some(
                    post_update(
                        None,
                        Some(Content::None),
                        Some(true))
                )
            )
        );

        let post_ids = Posts::find_unlisted_post_ids_in_space(SPACE1, 0, 3);
        assert_eq!(post_ids, vec![POST1, POST2]);
    });
}

#[test]
fn find_unlisted_post_ids_in_space_should_work_with_zero_offset() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(
            _update_post(
                None,
                None,
                Some(
                    post_update(
                        None,
                        Some(Content::None),
                        Some(true))
                )
            )
        );

        let post_ids = Posts::find_unlisted_post_ids_in_space(SPACE1, 0, 1);
        assert_eq!(post_ids, vec![POST1]);
    });
}

#[test]
fn find_unlisted_post_ids_in_space_should_work_with_zero_limit() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(
            _update_post(
                None,
                None,
                Some(
                    post_update(
                        None,
                        Some(Content::None),
                        Some(true))
                )
            )
        );

        let post_ids = Posts::find_unlisted_post_ids_in_space(SPACE1, 1, 0);
        assert_eq!(post_ids, vec![POST1]);
    });
}

#[test]
fn find_unlisted_post_ids_in_space_should_work_with_zero_offset_and_zero_limit() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(
            _update_post(
                None,
                None,
                Some(
                    post_update(
                        None,
                        Some(Content::None),
                        Some(true))
                )
            )
        );

        let post_ids = Posts::find_unlisted_post_ids_in_space(SPACE1, 0, 0);
        assert_eq!(post_ids, vec![]);
    });
}*/
// --------------------------------------------------------------------------------------------
