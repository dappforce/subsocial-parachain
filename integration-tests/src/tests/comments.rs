use frame_support::{assert_noop, assert_ok};

use pallet_posts::{Error as PostsError, HasReplies, PostExtension};
use subsocial_support::{
    mock_functions::*, PostId,
};

use crate::mock::*;
use crate::utils::*;
use crate::utils::posts_utils::*;
use crate::utils::spaces_utils::*;

#[test]
fn create_comment_should_work() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_default_comment()); // PostId 2 by ACCOUNT1 which is permitted by default

        // Check storages
        let root_post = Posts::post_by_id(POST1).unwrap();
        assert_eq!(Posts::reply_ids_by_post_id(POST1), vec![POST2]);
        let PostExtension::Post(post_ext) =
            root_post.extension else { panic!("qed; not a regular post") };

        assert_eq!(post_ext.total_replies_count.get(), 1);

        // Check whether data stored correctly
        let comment = Posts::post_by_id(POST2).unwrap();
        let comment_ext = comment.get_comment_ext().unwrap();

        assert!(comment_ext.parent_id.is_none());
        assert_eq!(comment_ext.root_post_id, POST1);
        assert_eq!(comment.created.account, ACCOUNT1);
        assert!(comment.updated.is_none());
        assert_eq!(comment.content, comment_content_ipfs());

        let PostExtension::Comment(comment_ext) =
            comment.extension else { panic!("qed; not a comment") };

        assert_eq!(comment_ext.replies_count.get(), 0);

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
            let comment = Posts::post_by_id(comment_id).unwrap();
            let PostExtension::Comment(comment_ext) =
                comment.extension else { panic!("qed; not a comment") };

            // All of comments has 1 replies as they reply to each other.
            assert_eq!(comment_ext.replies_count.get(), 1);
            assert_eq!(Posts::reply_ids_by_post_id(comment_id), vec![comment_id + 1]);
        }

        let last_parent = Posts::post_by_id(parent_id).unwrap();
        let PostExtension::Comment(last_parent_ext) =
            last_parent.extension else { panic!("qed; not a comment") };

        assert_eq!(last_parent_ext.replies_count.get(), 1);
        assert_eq!(Posts::reply_ids_by_post_id(parent_id), vec![last_comment_id]);

        let last_comment = Posts::post_by_id(last_comment_id).unwrap();
        let PostExtension::Comment(last_comment_ext) =
            last_comment.extension else { panic!("qed; not a comment") };

        assert_eq!(last_comment_ext.replies_count.get(), 0);
        assert!(Posts::reply_ids_by_post_id(last_comment_id).is_empty());

        let PostExtension::Post(root_post_ext) =
            Posts::post_by_id(last_comment_ext.root_post_id).unwrap().extension else { panic!("qed; not a regular post") };

        assert_eq!(root_post_ext.total_replies_count.get(), MaxCommentDepth::get());
    });
}

#[test]
fn create_comment_should_fail_when_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error creating a comment with wrong post
        assert_noop!(
            _create_default_comment(),
            PostsError::<TestRuntime>::PostNotFound
        );
    });
}

#[test]
fn create_comment_should_fail_when_parent_comment_is_unknown() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Try to catch an error creating a comment with wrong parent
        assert_noop!(
            _create_comment(None, None, Some(Some(POST2)), None),
            PostsError::<TestRuntime>::UnknownParentComment
        );
    });
}

#[test]
fn create_comment_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Try to catch an error creating a comment with wrong parent
        assert_noop!(
            _create_comment(None, None, None, Some(invalid_content_ipfs())),
            ContentError::InvalidIpfsCid,
        );
    });
}

#[test]
fn create_comment_should_fail_when_trying_to_create_in_hidden_space_scope() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_update_space(
            None,
            None,
            Some(space_update(None, Some(true)))
        ));

        assert_noop!(
            _create_default_comment(),
            PostsError::<TestRuntime>::CannotCreateInHiddenScope
        );
    });
}

#[test]
fn create_comment_should_fail_when_trying_create_in_hidden_post_scope() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_update_post(
            None,
            None,
            Some(post_update(None, None, Some(true)))
        ));

        assert_noop!(
            _create_default_comment(),
            PostsError::<TestRuntime>::CannotCreateInHiddenScope
        );
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
            PostsError::<TestRuntime>::MaxCommentDepthReached
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

        for parent_id in first_comment_id..last_comment_id {
            assert_ok!(_create_comment(None, None, Some(Some(parent_id)), None));
        }

        assert_ok!(_update_comment(
            None,
            Some(last_comment_id),
            Some(post_update(
                None,
                None,
                Some(true) // make comment hidden
            ))
        ));

        let parent_id = last_comment_id - 1;
        let parent_id_by_one = parent_id - 1;

        // We should check that counters weren't increased for all ancestor comments
        // except the last parent.
        for comment_id in first_comment_id..parent_id_by_one {
            let comment = Posts::post_by_id(comment_id).unwrap();
            let PostExtension::Comment(comment_ext) =
                comment.extension else { panic!("qed; not a comment") };

            // All of comments has 1 replies as they reply to each other.
            assert_eq!(comment_ext.replies_count.get(), 1);
            assert_eq!(Posts::reply_ids_by_post_id(comment_id), vec![comment_id + 1]);
        }

        let last_parent = Posts::post_by_id(parent_id).unwrap();
        let PostExtension::Comment(last_parent_ext) =
            last_parent.extension else { panic!("qed; not a comment") };

        assert_eq!(last_parent_ext.replies_count.get(), 0);
        assert!(Posts::reply_ids_by_post_id(last_comment_id).is_empty());

        let last_comment = Posts::post_by_id(last_comment_id).unwrap();
        let PostExtension::Comment(last_comment_ext) =
            last_comment.extension else { panic!("qed; not a comment") };

        assert_eq!(last_comment_ext.replies_count.get(), 0);
        assert!(Posts::reply_ids_by_post_id(last_comment_id).is_empty());

        let PostExtension::Post(root_post_ext) =
            Posts::post_by_id(last_comment_ext.root_post_id).unwrap().extension else { panic!("qed; not a regular post") };

        assert_eq!(root_post_ext.total_replies_count.get(), MaxCommentDepth::get() - 1);
    });
}

#[test]
// `PostNotFound` here: Post with Comment extension. Means that comment wasn't found.
fn update_comment_should_fail_when_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error updating a comment with wrong PostId
        assert_noop!(
            _update_comment(None, None, None),
            PostsError::<TestRuntime>::PostNotFound
        );
    });
}

#[test]
fn update_comment_should_fail_when_account_is_not_a_comment_author() {
    ExtBuilder::build_with_comment().execute_with(|| {
        // Try to catch an error updating a comment with wrong Account
        assert_noop!(
            _update_comment(Some(Origin::signed(ACCOUNT2)), None, None),
            PostsError::<TestRuntime>::NotACommentAuthor
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
            ContentError::InvalidIpfsCid,
        );
    });
}
