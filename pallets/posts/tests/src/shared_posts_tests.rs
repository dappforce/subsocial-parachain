use frame_support::{assert_noop, assert_ok};

use pallet_permissions::SpacePermission as SP;
use pallet_posts::Error as PostsError;

use crate::{mock::*, tests_utils::*};

#[test]
fn share_post_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None)); // SpaceId 2 by ACCOUNT2

        assert_ok!(_create_post(
            Some(RuntimeOrigin::signed(ACCOUNT2)),
            Some(Some(SPACE2)),
            Some(extension_shared_post(POST1)),
            None
        )); // Share PostId 1 on SpaceId 2 by ACCOUNT2 which is permitted by default in both spaces

        // Check storages
        assert_eq!(Posts::post_ids_by_space_id(SPACE1), vec![POST1]);
        assert_eq!(Posts::post_ids_by_space_id(SPACE2), vec![POST2]);
        assert_eq!(Posts::next_post_id(), POST3);

        assert_eq!(Posts::shared_post_ids_by_original_post_id(POST1), vec![POST2]);

        let shared_post = Posts::post_by_id(POST2).unwrap();

        assert_eq!(shared_post.space_id, Some(SPACE2));
        assert_eq!(shared_post.created.account, ACCOUNT2);
        assert_eq!(shared_post.extension, extension_shared_post(POST1));
    });
}

#[test]
fn share_post_should_work_when_one_of_roles_is_permitted() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(
        || {
            assert_ok!(_create_space(
                None,       // From ACCOUNT1
                None,       // With default space content,
                None
            ));
            // SpaceId 2
            assert_ok!(_create_post(
                None, // From ACCOUNT1
                Some(Some(SPACE2)),
                None, // With RegularPost extension
                None  // With default post content
            )); // PostId 1 on SpaceId 2

            assert_ok!(_create_post(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                Some(Some(SPACE1)),
                Some(extension_shared_post(POST1)),
                None
            )); // Share PostId 1 on SpaceId 1 by ACCOUNT2 which is permitted by RoleId 1 from ext
        },
    );
}

#[test]
fn share_post_should_work_for_share_own_post_in_same_own_space() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post(
            Some(RuntimeOrigin::signed(ACCOUNT1)),
            Some(Some(SPACE1)),
            Some(extension_shared_post(POST1)),
            None
        )); // Share PostId 1

        // Check storages
        assert_eq!(Posts::post_ids_by_space_id(SPACE1), vec![POST1, POST2]);
        assert_eq!(Posts::next_post_id(), POST3);

        assert_eq!(Posts::shared_post_ids_by_original_post_id(POST1), vec![POST2]);

        let shared_post = Posts::post_by_id(POST2).unwrap();
        assert_eq!(shared_post.space_id, Some(SPACE1));
        assert_eq!(shared_post.created.account, ACCOUNT1);
        assert_eq!(shared_post.extension, extension_shared_post(POST1));
    });
}

#[test]
fn share_post_should_fail_when_original_post_not_found() {
    ExtBuilder::build_with_space().execute_with(|| {
        assert_ok!(_create_space(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None)); // SpaceId 2 by ACCOUNT2

        // Skipped creating PostId 1
        assert_noop!(
            _create_post(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(extension_shared_post(POST1)),
                None
            ),
            PostsError::<Test>::OriginalPostNotFound
        );
    });
}

#[test]
fn share_post_should_fail_when_trying_to_share_shared_post() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None)); // SpaceId 2 by ACCOUNT2

        assert_ok!(_create_post(
            Some(RuntimeOrigin::signed(ACCOUNT2)),
            Some(Some(SPACE2)),
            Some(extension_shared_post(POST1)),
            None
        ));

        // Try to share post with extension SharedPost
        assert_noop!(
            _create_post(
                Some(RuntimeOrigin::signed(ACCOUNT1)),
                Some(Some(SPACE1)),
                Some(extension_shared_post(POST2)),
                None
            ),
            PostsError::<Test>::CannotShareSharedPost
        );
    });
}

#[test]
fn share_post_should_fail_when_account_has_no_permission_to_create_posts_in_new_space() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_space(
            Some(RuntimeOrigin::signed(ACCOUNT1)),
            None,       // Default space content,
            None
        )); // SpaceId 2 by ACCOUNT1

        // Try to share post with extension SharedPost
        assert_noop!(
            _create_post(
                Some(RuntimeOrigin::signed(ACCOUNT2)),
                Some(Some(SPACE2)),
                Some(extension_shared_post(POST1)),
                None
            ),
            PostsError::<Test>::NoPermissionToCreatePosts
        );
    });
}

#[test]
fn share_post_should_fail_when_no_right_permission_in_account_roles() {
    ExtBuilder::build_with_a_few_roles_granted_to_account2(vec![SP::CreatePosts]).execute_with(
        || {
            assert_ok!(_create_space(
                None,       // From ACCOUNT1
                None,       // With default space content
                None
            ));
            // SpaceId 2
            assert_ok!(_create_post(
                None, // From ACCOUNT1
                Some(Some(SPACE2)),
                None, // With RegularPost extension
                None  // With default post content
            )); // PostId 1 on SpaceId 2

            assert_ok!(_delete_default_role());

            assert_noop!(
                _create_post(
                    Some(RuntimeOrigin::signed(ACCOUNT2)),
                    Some(Some(SPACE1)),
                    Some(extension_shared_post(POST1)),
                    None
                ),
                PostsError::<Test>::NoPermissionToCreatePosts
            );
        },
    );
}
