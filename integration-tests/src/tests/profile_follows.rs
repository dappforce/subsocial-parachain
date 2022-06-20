use frame_support::{assert_noop, assert_ok};

use pallet_profile_follows::Error as ProfileFollowsError;

use crate::mock::*;
use crate::utils::*;
use crate::utils::profile_follows_utils::*;

#[test]
fn follow_account_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_default_follow_account()); // Follow ACCOUNT1 by ACCOUNT2

        assert_eq!(
            ProfileFollows::accounts_followed_by_account(ACCOUNT2),
            vec![ACCOUNT1]
        );
        assert_eq!(ProfileFollows::account_followers(ACCOUNT1), vec![ACCOUNT2]);
        assert_eq!(
            ProfileFollows::account_followed_by_account((ACCOUNT2, ACCOUNT1)),
            true
        );
    });
}

#[test]
fn follow_account_should_fail_when_account_tries_to_follow_themself() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _follow_account(None, Some(ACCOUNT2)),
            ProfileFollowsError::<TestRuntime>::AccountCannotFollowItself
        );
    });
}

#[test]
fn follow_account_should_fail_when_account_is_already_following_account() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_default_follow_account());

        assert_noop!(
            _default_follow_account(),
            ProfileFollowsError::<TestRuntime>::AlreadyAccountFollower
        );
    });
}

#[test]
fn unfollow_account_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_default_follow_account());
        // Follow ACCOUNT1 by ACCOUNT2
        assert_ok!(_default_unfollow_account());

        assert!(ProfileFollows::accounts_followed_by_account(ACCOUNT2).is_empty());
        assert!(ProfileFollows::account_followers(ACCOUNT1).is_empty());
        assert_eq!(
            ProfileFollows::account_followed_by_account((ACCOUNT2, ACCOUNT1)),
            false
        );
    });
}

#[test]
fn unfollow_account_should_fail_when_account_tries_to_unfollow_themself() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _unfollow_account(None, Some(ACCOUNT2)),
            ProfileFollowsError::<TestRuntime>::AccountCannotUnfollowItself
        );
    });
}

#[test]
fn unfollow_account_should_fail_when_account_is_not_following_another_account_yet() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_default_follow_account());
        assert_ok!(_default_unfollow_account());

        assert_noop!(
            _default_unfollow_account(),
            ProfileFollowsError::<TestRuntime>::NotAccountFollower
        );
    });
}