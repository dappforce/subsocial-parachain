use frame_support::{assert_noop, assert_ok};

use pallet_profiles::Error as ProfilesError;
use pallet_utils::Error as UtilsError;
use pallet_utils::mock_functions::*;

use crate::mock::*;
use crate::utils::*;
use crate::utils::spaces_utils::*;
use crate::utils::profile_utils::*;

#[test]
fn create_profile_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile()); // AccountId 1

        let profile = Profiles::social_account_by_id(ACCOUNT1)
            .unwrap()
            .profile
            .unwrap();
        assert_eq!(profile.created.account, ACCOUNT1);
        assert!(profile.updated.is_none());
        assert_eq!(profile.content, profile_content_ipfs());

        assert!(ProfileHistory::edit_history(ACCOUNT1).is_empty());
    });
}

#[test]
fn create_profile_should_fail_when_profile_is_already_created() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile());
        // AccountId 1
        assert_noop!(
            _create_default_profile(),
            ProfilesError::<TestRuntime>::ProfileAlreadyCreated
        );
    });
}

#[test]
fn create_profile_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _create_profile(None, Some(invalid_content_ipfs())),
            UtilsError::<TestRuntime>::InvalidIpfsCid
        );
    });
}

#[test]
fn update_profile_should_work() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile());
        // AccountId 1
        assert_ok!(_update_profile(None, Some(space_content_ipfs())));

        // Check whether profile updated correctly
        let profile = Profiles::social_account_by_id(ACCOUNT1)
            .unwrap()
            .profile
            .unwrap();
        assert!(profile.updated.is_some());
        assert_eq!(profile.content, space_content_ipfs());

        // Check whether profile history is written correctly
        let profile_history = ProfileHistory::edit_history(ACCOUNT1)[0].clone();
        assert_eq!(
            profile_history.old_data.content,
            Some(profile_content_ipfs())
        );
    });
}

#[test]
fn update_profile_should_fail_when_social_account_not_found() {
    ExtBuilder::build().execute_with(|| {
        assert_noop!(
            _update_profile(None, Some(profile_content_ipfs())),
            ProfilesError::<TestRuntime>::SocialAccountNotFound
        );
    });
}

#[test]
fn update_profile_should_fail_when_account_has_no_profile() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(ProfileFollows::follow_account(
            Origin::signed(ACCOUNT1),
            ACCOUNT2
        ));
        assert_noop!(
            _update_profile(None, Some(profile_content_ipfs())),
            ProfilesError::<TestRuntime>::AccountHasNoProfile
        );
    });
}

#[test]
fn update_profile_should_fail_when_no_updates_for_profile_provided() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile());
        // AccountId 1
        assert_noop!(
            _update_profile(None, None),
            ProfilesError::<TestRuntime>::NoUpdatesForProfile
        );
    });
}

#[test]
fn update_profile_should_fail_when_ipfs_cid_is_invalid() {
    ExtBuilder::build().execute_with(|| {
        assert_ok!(_create_default_profile());
        assert_noop!(
            _update_profile(None, Some(invalid_content_ipfs())),
            UtilsError::<TestRuntime>::InvalidIpfsCid
        );
    });
}