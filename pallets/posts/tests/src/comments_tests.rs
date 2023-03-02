use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

use pallet_posts::Error as PostsError;
use subsocial_support::{mock_functions::*, ContentError, PostId};

use crate::{mock::*, tests_utils::*};

#[test]
fn create_comment_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_default_comment()); // PostId 2 by ACCOUNT1 which is permitted by default

        // Check storages
        assert_eq!(Posts::reply_ids_by_post_id(POST1), vec![POST2]);

        // Check whether data stored correctly
        let comment = Posts::post_by_id(POST2).unwrap();
        let comment_ext = comment.get_comment_ext().unwrap();

        assert!(comment_ext.parent_id.is_none());
        assert_eq!(comment_ext.root_post_id, POST1);
        assert_eq!(comment.created.account, ACCOUNT1);
        assert!(!comment.edited);
        assert_eq!(comment.content, comment_content_ipfs());

        assert_eq!(comment.upvotes_count, 0);
        assert_eq!(comment.downvotes_count, 0);
    });
}

#[test]
fn create_comment_should_work_when_comment_has_parents() {
    ExtBuilder::build_with_comment().execute_with(|| {
        let first_comment_id = 2;
        let last_comment_id = first_comment_id + (MaxCommentDepth::get() as PostId) - 1;

        // Create
        for parent_id in first_comment_id..last_comment_id {
            assert_ok!(_create_comment(None, None, Some(Some(parent_id)), None));
        }

        let parent_id = last_comment_id - 1;
        let parent_id_by_one = parent_id - 1;

        // We should check that counters were increased by 1 for all ancestor comments
        // except the last parent.
        for comment_id in first_comment_id..parent_id_by_one {
            // All of comments has 1 reply as they respond to each other.
            assert_eq!(Posts::reply_ids_by_post_id(comment_id), vec![comment_id + 1]);
        }

        assert_eq!(Posts::reply_ids_by_post_id(parent_id), vec![last_comment_id]);

        assert!(Posts::reply_ids_by_post_id(last_comment_id).is_empty());
    });
}

#[test]
fn create_comment_should_fail_when_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error creating a comment with wrong post
        assert_noop!(_create_default_comment(), PostsError::<Test>::PostNotFound);
    });
}

#[test]
fn create_comment_should_fail_when_parent_comment_is_unknown() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Try to catch an error creating a comment with wrong parent
        assert_noop!(
            _create_comment(None, None, Some(Some(POST2)), None),
            PostsError::<Test>::UnknownParentComment
        );
    });
}

#[test]
fn create_comment_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Try to catch an error creating a comment with wrong parent
        assert_noop!(
            _create_comment(None, None, None, Some(invalid_content_ipfs())),
            DispatchError::from(ContentError::InvalidIpfsCid)
        );
    });
}

#[test]
fn create_comment_should_fail_when_trying_to_create_in_hidden_space_scope() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_update_space(None, None, Some(space_update(None, Some(true)))));

        assert_noop!(_create_default_comment(), PostsError::<Test>::CannotCreateInHiddenScope);
    });
}

#[test]
fn create_comment_should_fail_when_trying_create_in_hidden_post_scope() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_update_post(None, None, Some(post_update(None, None, Some(true)))));

        assert_noop!(_create_default_comment(), PostsError::<Test>::CannotCreateInHiddenScope);
    });
}

#[test]
fn create_comment_should_fail_when_max_comment_depth_reached() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_comment(None, None, Some(None), None)); // PostId 2

        for parent_id in 2..11_u64 {
            assert_ok!(_create_comment(None, None, Some(Some(parent_id)), None));
            // PostId N (last = 10)
        }

        // Some(Some(11)) - here is parent_id 11 of type PostId
        assert_noop!(
            _create_comment(None, None, Some(Some(11)), None),
            PostsError::<Test>::MaxCommentDepthReached
        );
    });
}

#[test]
fn update_comment_should_work() {
    ExtBuilder::build_with_comment().execute_with(|| {
        // Post update with ID 1 should be fine
        assert_ok!(_update_comment(None, None, None));

        // Check whether post updates correctly
        let comment = Posts::post_by_id(POST2).unwrap();
        assert_eq!(comment.content, reply_content_ipfs());
    });
}

#[test]
fn update_comment_hidden_should_work_when_comment_has_parents() {
    ExtBuilder::build_with_comment().execute_with(|| {
        let first_comment_id = 2;
        let last_comment_id = first_comment_id + (MaxCommentDepth::get() as PostId) - 1;

        // Create comments from 3 to 11
        for parent_id in first_comment_id..last_comment_id {
            assert_ok!(_create_comment(None, None, Some(Some(parent_id)), None));
        }

        let should_hide_id = last_comment_id - 3;
        let should_hide_by_one_id = should_hide_id - 1;

        assert_ok!(_update_comment(
            None,
            Some(should_hide_id),
            Some(post_update(
                None,
                None,
                Some(true) // make comment hidden
            ))
        ));

        // We should check that counters weren't increased for all ancestor comments
        // except the last before hidden.
        for comment_id in first_comment_id..should_hide_by_one_id {
            // All of comments has 1 replies as they reply to each other.
            assert_eq!(Posts::reply_ids_by_post_id(comment_id), vec![comment_id + 1]);
        }

        assert_eq!(
            Posts::reply_ids_by_post_id(should_hide_by_one_id),
            vec![should_hide_by_one_id + 1]
        );
        assert_eq!(Posts::reply_ids_by_post_id(should_hide_id), vec![should_hide_id + 1]);
    });
}

#[test]
// `PostNotFound` here: Post with Comment extension. Means that comment wasn't found.
fn update_comment_should_fail_when_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error updating a comment with wrong PostId
        assert_noop!(_update_comment(None, None, None), PostsError::<Test>::PostNotFound);
    });
}

#[test]
fn update_comment_should_fail_when_account_is_not_a_comment_author() {
    ExtBuilder::build_with_comment().execute_with(|| {
        // Try to catch an error updating a comment with wrong Account
        assert_noop!(
            _update_comment(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None),
            PostsError::<Test>::NotACommentAuthor
        );
    });
}

#[test]
fn update_comment_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build_with_comment().execute_with(|| {
        // Try to catch an error updating a comment with invalid content
        assert_noop!(
            _update_comment(
                None,
                None,
                Some(post_update(None, Some(invalid_content_ipfs()), None))
            ),
            DispatchError::from(ContentError::InvalidIpfsCid)
        );
    });
}
