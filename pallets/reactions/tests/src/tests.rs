use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::OriginFor;

use pallet_posts::{Error as PostsError, Post, PostById, PostExtension, PostUpdate};
use pallet_reactions::{Error as ReactionsError, ReactionKind};
use pallet_reactions::Event::{PostReactionCreated, PostReactionDeleted, PostReactionUpdated};
use pallet_spaces::types::{Space, SpaceUpdate};
use subsocial_support::{Content, SpaceId};

use crate::mock::*;

fn create_space(origin: OriginFor<Test>) -> Space<Test> {
    let space_id = pallet_spaces::NextSpaceId::<Test>::get();

    Spaces::create_space(origin.clone(), Content::None, None).expect("Spaces::create_space failed");

    let space = pallet_spaces::SpaceById::<Test>::get(space_id).expect("Space not found");

    space
}

fn create_post_in_space(origin: OriginFor<Test>, space_id: SpaceId) -> Post<Test> {
    let post_id = pallet_posts::NextPostId::<Test>::get();

    Posts::create_post(
        origin.clone().into(),
        Some(space_id),
        PostExtension::RegularPost,
        Content::None,
    )
    .expect("Posts::create_post failed");

    let post = PostById::<Test>::get(post_id).expect("Post not found");

    post
}

fn create_post(origin: OriginFor<Test>) -> Post<Test> {
    let space = create_space(origin.clone());

    create_post_in_space(origin, space.id)
}

fn create_hidden_post(origin: OriginFor<Test>) -> Post<Test> {
    let post = create_post(origin.clone());

    Posts::update_post(
        origin.clone(),
        post.id,
        PostUpdate { space_id: None, content: None, hidden: true.into() },
    )
    .expect("Couldn't hide post");

    post
}

fn create_post_in_hidden_space(origin: OriginFor<Test>) -> Post<Test> {
    let post = create_post(origin.clone());

    Spaces::update_space(
        origin.clone(),
        post.space_id.unwrap(),
        SpaceUpdate { content: None, hidden: true.into(), permissions: None },
    )
    .expect("Couldn't hide space");

    post
}

#[test]
fn create_post_reaction_should_work_upvote() {
    ExtBuilder::build().execute_with(|| {
        let account = 5;
        let origin = Origin::signed(account);
        let post = create_post(origin.clone());

        let reaction_id = Reactions::next_reaction_id();
        assert_ok!(Reactions::create_post_reaction(origin, post.id, ReactionKind::Upvote));
        let next_reaction_id = Reactions::next_reaction_id();

        // Check storages
        assert_eq!(Reactions::reaction_ids_by_post_id(post.id), vec![reaction_id]);
        assert_eq!(Reactions::next_reaction_id(), next_reaction_id);

        System::assert_last_event(PostReactionCreated {
            account,
            post_id: post.id,
            reaction_id,
            reaction_kind: ReactionKind::Upvote,
        }.into());

        // Check post reaction counters again
        let post = Posts::post_by_id(post.id).unwrap();
        assert_eq!(post.upvotes_count, 1);
        assert_eq!(post.downvotes_count, 0);

        // Check whether data stored correctly
        let reaction = Reactions::reaction_by_id(reaction_id).unwrap();
        assert_eq!(reaction.created.account, account);
        assert_eq!(reaction.kind, ReactionKind::Upvote);
    });
}

#[test]
fn create_post_reaction_should_work_downvote() {
    ExtBuilder::build().execute_with(|| {
        let account = 5;
        let origin = Origin::signed(account);
        let post = create_post(origin.clone());

        let reaction_id = Reactions::next_reaction_id();
        assert_ok!(Reactions::create_post_reaction(origin, post.id, ReactionKind::Downvote));
        let next_reaction_id = Reactions::next_reaction_id();

        System::assert_last_event(PostReactionCreated {
            account,
            post_id: post.id,
            reaction_id,
            reaction_kind: ReactionKind::Downvote,
        }.into());

        // Check storages
        assert_eq!(Reactions::reaction_ids_by_post_id(post.id), vec![reaction_id]);
        assert_eq!(Reactions::next_reaction_id(), next_reaction_id);

        // Check post reaction counters
        let post = Posts::post_by_id(post.id).unwrap();
        assert_eq!(post.upvotes_count, 0);
        assert_eq!(post.downvotes_count, 1);

        // Check whether data stored correctly
        let reaction = Reactions::reaction_by_id(reaction_id).unwrap();
        assert_eq!(reaction.created.account, account);
        assert_eq!(reaction.kind, ReactionKind::Downvote);
    });
}

#[test]
fn create_post_reaction_should_fail_when_account_has_already_reacted() {
    ExtBuilder::build().execute_with(|| {
        let account = 5;
        let origin = Origin::signed(account);
        let post = create_post(origin.clone());

        assert_ok!(Reactions::create_post_reaction(origin.clone(), post.id, ReactionKind::Upvote));

        assert_noop!(
            Reactions::create_post_reaction(origin.clone(), post.id, ReactionKind::Downvote),
            ReactionsError::<Test>::AccountAlreadyReacted
        );

        assert_noop!(
            Reactions::create_post_reaction(origin.clone(), post.id, ReactionKind::Upvote),
            ReactionsError::<Test>::AccountAlreadyReacted
        );
    });
}

#[test]
fn create_post_reaction_should_fail_when_post_not_found() {
    ExtBuilder::build().execute_with(|| {
        let origin = Origin::signed(34);
        let non_existing_post_id = 56;

        assert_noop!(
            Reactions::create_post_reaction(origin, non_existing_post_id, ReactionKind::Upvote),
            PostsError::<Test>::PostNotFound,
        );
    });
}

#[test]
fn create_post_reaction_should_fail_when_trying_to_react_in_hidden_space() {
    ExtBuilder::build().execute_with(|| {
        let origin = Origin::signed(8);
        let post = create_post_in_hidden_space(origin.clone());

        assert_noop!(
            Reactions::create_post_reaction(origin, post.id, ReactionKind::Downvote),
            ReactionsError::<Test>::CannotReactWhenSpaceHidden
        );
    });
}

#[test]
fn create_post_reaction_should_fail_when_trying_to_react_on_hidden_post() {
    ExtBuilder::build().execute_with(|| {
        let origin = Origin::signed(2);
        let post = create_hidden_post(origin.clone());

        assert_noop!(
            Reactions::create_post_reaction(origin, post.id, ReactionKind::Upvote),
            ReactionsError::<Test>::CannotReactWhenPostHidden
        );
    });
}


#[test]
fn update_post_reaction_should_fail_when_reaction_not_found() {
    ExtBuilder::build().execute_with(|| {
        let origin = Origin::signed(13);
        let post = create_post(origin.clone());
        let non_existing_reaction_id = 56;

        assert_noop!(
            Reactions::update_post_reaction(origin, post.id, non_existing_reaction_id, ReactionKind::Upvote),
            ReactionsError::<Test>::ReactionByAccountNotFound
        );
    });
}

#[test]
fn update_post_reaction_should_fail_when_not_reaction_owner() {
    ExtBuilder::build().execute_with(|| {
        let owner_origin = Origin::signed(13);
        let not_owner_origin = Origin::signed(123);

        let post = create_post(owner_origin.clone());

        let reaction_id = Reactions::next_reaction_id();
        assert_ok!(Reactions::create_post_reaction(owner_origin, post.id, ReactionKind::Upvote));


        assert_noop!(
            Reactions::update_post_reaction(not_owner_origin, post.id, reaction_id, ReactionKind::Downvote),
            ReactionsError::<Test>::ReactionByAccountNotFound
        );
    });
}

#[test]
fn update_post_reaction_should_fail_when_setting_same_reaction() {
    ExtBuilder::build().execute_with(|| {
        let origin = Origin::signed(432);

        let post = create_post(origin.clone());
        let reaction_id = Reactions::next_reaction_id();
        assert_ok!(Reactions::create_post_reaction(origin.clone(), post.id, ReactionKind::Upvote));


        assert_noop!(
            Reactions::update_post_reaction(origin, post.id, reaction_id, ReactionKind::Upvote),
            ReactionsError::<Test>::SameReaction
        );
    });
}

#[test]
fn update_post_reaction_should_work() {
    ExtBuilder::build().execute_with(|| {
        let account = 423;
        let origin = Origin::signed(account);

        let post = create_post(origin.clone());
        let reaction_id = Reactions::next_reaction_id();
        assert_ok!(Reactions::create_post_reaction(origin.clone(), post.id, ReactionKind::Upvote));

        let post = Posts::post_by_id(post.id).unwrap();
        assert_eq!(post.upvotes_count, 1);
        assert_eq!(post.downvotes_count, 0);

        assert_ok!(
            Reactions::update_post_reaction(origin, post.id, reaction_id, ReactionKind::Downvote),
        );

        System::assert_last_event(PostReactionUpdated {
            account,
            post_id: post.id,
            reaction_id,
            reaction_kind: ReactionKind::Downvote,
        }.into());

        let post = Posts::post_by_id(post.id).unwrap();
        assert_eq!(post.upvotes_count, 0);
        assert_eq!(post.downvotes_count, 1);

        // Check whether data stored correctly
        let reaction = Reactions::reaction_by_id(reaction_id).unwrap();
        assert_eq!(reaction.created.account, account);
        assert_eq!(reaction.kind, ReactionKind::Downvote);
    });
}

//////


#[test]
fn delete_post_reaction_should_fail_when_reaction_not_found() {
    ExtBuilder::build().execute_with(|| {
        let origin = Origin::signed(13);
        let post = create_post(origin.clone());
        let non_existing_reaction_id = 56;

        assert_noop!(
            Reactions::delete_post_reaction(origin, post.id, non_existing_reaction_id),
            ReactionsError::<Test>::ReactionByAccountNotFound
        );
    });
}

#[test]
fn delete_post_reaction_should_fail_when_not_reaction_owner() {
    ExtBuilder::build().execute_with(|| {
        let owner_origin = Origin::signed(13);
        let not_owner_origin = Origin::signed(123);

        let post = create_post(owner_origin.clone());

        let reaction_id = Reactions::next_reaction_id();
        assert_ok!(Reactions::create_post_reaction(owner_origin, post.id, ReactionKind::Upvote));


        assert_noop!(
            Reactions::delete_post_reaction(not_owner_origin, post.id, reaction_id),
            ReactionsError::<Test>::ReactionByAccountNotFound
        );
    });
}

#[test]
fn delete_post_reaction_should_work() {
    ExtBuilder::build().execute_with(|| {
        let account = 423;
        let origin = Origin::signed(account);

        let post = create_post(origin.clone());
        let reaction_id = Reactions::next_reaction_id();
        assert_ok!(Reactions::create_post_reaction(origin.clone(), post.id, ReactionKind::Upvote));

        let post = Posts::post_by_id(post.id).unwrap();
        assert_eq!(post.upvotes_count, 1);
        assert_eq!(post.downvotes_count, 0);

        assert_ok!(
            Reactions::delete_post_reaction(origin, post.id, reaction_id),
        );

        System::assert_last_event(PostReactionDeleted {
            account,
            post_id: post.id,
            reaction_id,
            reaction_kind: ReactionKind::Upvote,
        }.into());

        let post = Posts::post_by_id(post.id).unwrap();
        assert_eq!(post.upvotes_count, 0);
        assert_eq!(post.downvotes_count, 0);

        // Check whether data is deleted
        assert!(Reactions::reaction_by_id(reaction_id).is_none());
    });
}