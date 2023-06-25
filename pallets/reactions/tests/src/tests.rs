// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use frame_support::{assert_noop, assert_ok};

use pallet_posts::Error as PostsError;
use pallet_reactions::Error as ReactionsError;

use crate::{mock::*, tests_utils::*};

#[test]
fn create_post_reaction_should_work_upvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(Some(RuntimeOrigin::signed(ACCOUNT2)), None, None)); // ReactionId 1 by ACCOUNT2 which is permitted by default

        // Check storages
        assert_eq!(Reactions::reaction_ids_by_post_id(POST1), vec![REACTION1]);
        assert_eq!(Reactions::next_reaction_id(), REACTION2);

        // Check post reaction counters
        let post = Posts::post_by_id(POST1).unwrap();
        assert_eq!(post.upvotes_count, 1);
        assert_eq!(post.downvotes_count, 0);

        // Check whether data stored correctly
        let reaction = Reactions::reaction_by_id(REACTION1).unwrap();
        assert_eq!(reaction.created.account, ACCOUNT2);
        assert_eq!(reaction.kind, reaction_upvote());
    });
}

#[test]
fn create_post_reaction_should_work_downvote() {
    ExtBuilder::build_with_post().execute_with(|| {
        assert_ok!(_create_post_reaction(
            Some(RuntimeOrigin::signed(ACCOUNT2)),
            None,
            Some(reaction_downvote())
        )); // ReactionId 1 by ACCOUNT2 which is permitted by default

        // Check storages
        assert_eq!(Reactions::reaction_ids_by_post_id(POST1), vec![REACTION1]);
        assert_eq!(Reactions::next_reaction_id(), REACTION2);

        // Check post reaction counters
        let post = Posts::post_by_id(POST1).unwrap();
        assert_eq!(post.upvotes_count, 0);
        assert_eq!(post.downvotes_count, 1);

        // Check whether data stored correctly
        let reaction = Reactions::reaction_by_id(REACTION1).unwrap();
        assert_eq!(reaction.created.account, ACCOUNT2);
        assert_eq!(reaction.kind, reaction_downvote());
    });
}

#[test]
fn create_post_reaction_should_fail_when_account_has_already_reacted() {
    ExtBuilder::build_with_reacted_post_and_two_spaces().execute_with(|| {
        // Try to catch an error creating reaction by the same account
        assert_noop!(
            _create_default_post_reaction(),
            ReactionsError::<Test>::AccountAlreadyReacted
        );
    });
}

#[test]
fn create_post_reaction_should_fail_when_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        // Try to catch an error creating reaction by the same account
        assert_noop!(_create_default_post_reaction(), PostsError::<Test>::PostNotFound);
    });
}

#[test]
fn create_post_reaction_should_fail_when_trying_to_react_in_hidden_space() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Hide the space
        assert_ok!(_update_space(None, None, Some(space_update(None, Some(true)))));

        assert_noop!(
            _create_default_post_reaction(),
            ReactionsError::<Test>::CannotReactWhenSpaceHidden
        );
    });
}

#[test]
fn create_post_reaction_should_fail_when_trying_to_react_on_hidden_post() {
    ExtBuilder::build_with_post().execute_with(|| {
        // Hide the post
        assert_ok!(_update_post(None, None, Some(post_update(None, None, Some(true)))));

        assert_noop!(
            _create_default_post_reaction(),
            ReactionsError::<Test>::CannotReactWhenPostHidden
        );
    });
}
