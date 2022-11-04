use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok};
use subsocial_support::mock_functions::valid_content_ipfs;

// For now, this one works for `do_set_profile` as well.
#[test]
fn set_profile_should_work() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account = 1;
        let space_id = 1;

        // `ensure_space_owner` in `set_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().return_const(Ok(()));

        // when
        let result = Profiles::set_profile(Origin::signed(account), space_id);

        // then
        assert_ok!(result);
        assert_eq!(Profiles::profile_space_id_by_account(account), Some(space_id));
    });
}

#[test]
fn set_profile_should_fail_when_not_space_owner() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account = 1;
        let space_id = 1;

        // `ensure_space_owner` in `set_profile` should return Error.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().return_const(Err("NotSpaceOwner".into()));

        // when
        let result = Profiles::set_profile(Origin::signed(account), space_id);

        // then:
        //  - expecting error here
        assert_err!(result, "NotSpaceOwner");
    });
}

#[test]
fn reset_profile_should_work() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account = 1;
        let space_id = 1;

        // `ensure_space_owner` in `set_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().return_const(Ok(()));

        assert_ok!(Profiles::set_profile(Origin::signed(account), space_id));

        // when
        let result = Profiles::reset_profile(Origin::signed(account));

        // then
        assert_ok!(result);
        assert!(Profiles::profile_space_id_by_account(account).is_none());
    });
}

#[test]
fn reset_profile_should_fail_when_no_space_set_as_profile() {
    ExtBuilder::build().execute_with(|| {
        // given
        let account = 1;
        assert!(Profiles::profile_space_id_by_account(account).is_none());

        // when
        let result = Profiles::reset_profile(Origin::signed(account));

        // then
        assert_err!(result, Error::<Test>::NoSpaceSetAsProfile);
    });
}

#[test]
fn create_space_as_profile_should_work() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account = 1;
        let space_id = 1;
        let content = valid_content_ipfs();

        // `ensure_space_owner` in `set_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().return_const(Ok(()));

        // `create_space` in `create_space_as_profile` should return Space id 1.
        let create_space_ctx = MockSpaces::create_space_context();
        create_space_ctx.expect().return_const(Ok(space_id));

        // when
        let result = Profiles::create_space_as_profile(Origin::signed(account), content);

        // then
        assert_ok!(result);
        assert_eq!(Profiles::profile_space_id_by_account(account), Some(space_id));
    });
}

#[test]
fn create_space_as_profile_should_fail_when_space_not_created() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account = 1;
        let content = valid_content_ipfs();

        // `create_space` in `create_space_as_profile` should return Error.
        let create_space_ctx = MockSpaces::create_space_context();
        create_space_ctx.expect().return_const(Err("UnableToCreateSpace".into()));

        // when
        let result = Profiles::create_space_as_profile(Origin::signed(account), content);

        // then
        //  - expecting error here
        assert_err!(result, "UnableToCreateSpace");
        assert!(Profiles::profile_space_id_by_account(account).is_none());
    });
}

#[test]
fn unlink_space_from_profile_should_work() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account = 1;
        let space_id = 1;

        // `ensure_space_owner` in `set_profile` and `unlink_space_from_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().times(2).return_const(Ok(()));

        assert_ok!(Profiles::set_profile(Origin::signed(account), space_id));

        // when
        let result = Profiles::unlink_space_from_profile(&account, space_id);

        // then
        assert_ok!(result);
        assert!(Profiles::profile_space_id_by_account(account).is_none());
    });
}

#[test]
fn unlink_space_from_profile_should_work_when_no_space_set_as_profile() {
    ExtBuilder::build().execute_with(|| {
        // given
        let account_owner = 1;
        let space_id = 1;

        // `ensure_space_owner` in `set_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().return_const(Ok(()));

        // when
        let result = Profiles::unlink_space_from_profile(&account_owner, space_id);

        // then
        assert_ok!(result);
        assert!(Profiles::profile_space_id_by_account(account_owner).is_none());
    });
}

#[test]
fn unlink_space_from_profile_should_work_when_provided_space_differs_from_profile_space() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account_owner = 1;
        let space_id = 1;
        let space_id_wrong = 2;

        // `ensure_space_owner` in `set_profile` and `unlink_space_from_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().times(2).return_const(Ok(()));

        assert_ok!(Profiles::set_profile(Origin::signed(account_owner), space_id));

        // when
        let result = Profiles::unlink_space_from_profile(&account_owner, space_id_wrong);

        // then
        assert_ok!(result);
        assert_eq!(Profiles::profile_space_id_by_account(account_owner), Some(space_id));
    });
}

#[test]
fn unlink_space_from_profile_should_fail_when_not_space_owner() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account_owner = 1;
        let space_id = 1;

        // `ensure_space_owner` in `set_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().times(1).return_const(Ok(()));

        assert_ok!(Profiles::set_profile(Origin::signed(account_owner), space_id));

        let account_not_owner = 2;

        // `ensure_space_owner` in `unlink_space_from_profile` should return Error.
        space_owner_ctx.expect().return_const(Err("NotSpaceOwner".into()));

        // when
        let result = Profiles::unlink_space_from_profile(&account_not_owner, space_id);

        // then
        //  - expecting error here
        assert_err!(result, "NotSpaceOwner");
        assert_eq!(Profiles::profile_space_id_by_account(account_owner), Some(space_id));
        assert!(Profiles::profile_space_id_by_account(account_not_owner).is_none());
    });
}
