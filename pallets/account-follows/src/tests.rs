use frame_support::{assert_noop, assert_ok};

use crate::{mock::*, pallet::*, Error, Event};

#[test]
fn follow_account_should_fail_if_account_tries_to_follow_himself() {
    ExtBuilder::build().execute_with(|| {
        let account = 1;
        assert_noop!(
            AccountFollows::follow_account(Origin::signed(account), account),
            Error::<TestRuntime>::AccountCannotFollowItself,
        );
    });
}

#[test]
fn follow_account_should_fail_if_already_a_follower() {
    ExtBuilder::build().execute_with(|| {
        let follower = 1;
        let followee = 2;

        assert_ok!(AccountFollows::follow_account(Origin::signed(follower), followee));

        assert_noop!(
            AccountFollows::follow_account(Origin::signed(follower), followee),
            Error::<TestRuntime>::AlreadyAccountFollower,
        );
    });
}

#[test]
fn follow_account_should_work() {
    ExtBuilder::build().execute_with(|| {
        let follower = 1;
        let followee = 2;

        assert_ok!(AccountFollows::follow_account(Origin::signed(follower), followee));

        assert!(AccountsFollowedByAccount::<TestRuntime>::get(follower.clone()).contains(&followee));
        assert!(AccountFollowers::<TestRuntime>::get(followee.clone()).contains(&follower));
        assert!(AccountFollowedByAccount::<TestRuntime>::get((follower.clone(), followee.clone())));

        System::assert_last_event(Event::AccountFollowed {
            follower,
            account: followee,
        }.into());
    });
}

#[test]
fn unfollow_account_should_fail_if_account_tries_to_unfollow_himself() {
    ExtBuilder::build().execute_with(|| {
        let account = 1;
        assert_noop!(
            AccountFollows::unfollow_account(Origin::signed(account), account),
            Error::<TestRuntime>::AccountCannotUnfollowItself,
        );
    });
}

#[test]
fn unfollow_account_should_fail_if_account_not_a_follower() {
    ExtBuilder::build().execute_with(|| {
        let follower = 1;
        let followee = 2;

        assert_noop!(
            AccountFollows::unfollow_account(Origin::signed(follower), followee),
            Error::<TestRuntime>::NotAccountFollower,
        );
    });
}

#[test]
fn unfollow_account_should_work() {
    ExtBuilder::build().execute_with(|| {
        let follower = 1;
        let followee = 2;

        assert_ok!(AccountFollows::follow_account(Origin::signed(follower), followee));

        assert_ok!(AccountFollows::unfollow_account(Origin::signed(follower), followee));

        assert!(
            !AccountsFollowedByAccount::<TestRuntime>::get(follower.clone()).contains(&followee)
        );
        assert!(!AccountFollowers::<TestRuntime>::get(followee.clone()).contains(&follower));
        assert!(!AccountFollowedByAccount::<TestRuntime>::get((
            follower.clone(),
            followee.clone()
        )));

        System::assert_last_event(Event::AccountUnfollowed { follower, account: followee }.into());
    });
}
