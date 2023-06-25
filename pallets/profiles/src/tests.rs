// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

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
        let result = Profiles::set_profile(RuntimeOrigin::signed(account), space_id);

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
        let result = Profiles::set_profile(RuntimeOrigin::signed(account), space_id);

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

        assert_ok!(Profiles::set_profile(RuntimeOrigin::signed(account), space_id));

        // when
        let result = Profiles::reset_profile(RuntimeOrigin::signed(account));

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
        let result = Profiles::reset_profile(RuntimeOrigin::signed(account));

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
        let result = Profiles::create_space_as_profile(RuntimeOrigin::signed(account), content);

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
        let result = Profiles::create_space_as_profile(RuntimeOrigin::signed(account), content);

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

        // `ensure_space_owner` in `set_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().return_const(Ok(()));

        assert_ok!(Profiles::set_profile(RuntimeOrigin::signed(account), space_id));

        // when
        Profiles::unlink_space_from_profile(&account, space_id);

        // then
        assert!(Profiles::profile_space_id_by_account(account).is_none());
    });
}

#[test]
fn unlink_space_from_profile_should_work_when_no_space_set_as_profile() {
    ExtBuilder::build().execute_with(|| {
        // given
        let account = 1;
        let space_id = 1;

        // when
        Profiles::unlink_space_from_profile(&account, space_id);

        // then
        assert!(Profiles::profile_space_id_by_account(account).is_none());
    });
}

#[test]
fn unlink_space_from_profile_should_work_when_provided_space_differs_from_profile_space() {
    ExtBuilder::build().execute_with(|| {
        let _m = use_static_mock();

        // given
        let account = 1;
        let space_id = 1;
        let space_id_wrong = 2;

        // `ensure_space_owner` in `set_profile` should return Ok.
        let space_owner_ctx = MockSpaces::ensure_space_owner_context();
        space_owner_ctx.expect().return_const(Ok(()));

        assert_ok!(Profiles::set_profile(RuntimeOrigin::signed(account), space_id));

        // when
        Profiles::unlink_space_from_profile(&account, space_id_wrong);

        // then
        assert_eq!(Profiles::profile_space_id_by_account(account), Some(space_id));
    });
}
