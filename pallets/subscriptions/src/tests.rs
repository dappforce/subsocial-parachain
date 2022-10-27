use frame_support::{assert_noop, assert_ok};
use mockall::lazy_static;
use sp_runtime::DispatchError;
use sp_std::sync::{Mutex, MutexGuard};

use crate::{mock::*, types::SubscriptionSettings, Error, Event};

lazy_static! {
    static ref MTX: Mutex<()> = Mutex::new(());
}

/// mockall create static method mocking required this synchronization.
fn use_static_mock() -> MutexGuard<'static, ()> {
    match MTX.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[test]
fn update_subscription_settings_should_fail_when_caller_not_signed() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Subscriptions::update_subscription_settings(
                Origin::none(),
                0,
                SubscriptionSettings { subscription: 55, disabled: false, role_id: 22 },
            ),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn update_subscription_settings_should_fail_when_caller_not_space_owner() {
    ExtBuilder::default().build().execute_with(|| {
        let _z = use_static_mock();

        let space_owner: AccountId = 1;
        let not_space_owner: AccountId = 10;
        let space_id: SpaceId = 66;

        let ctx = MockSpaces::get_space_owner_context();
        ctx.expect().return_const(Ok(space_owner));

        assert_noop!(
            Subscriptions::update_subscription_settings(
                Origin::signed(not_space_owner),
                space_id,
                SubscriptionSettings { subscription: 55, disabled: false, role_id: 22 },
            ),
            Error::<Test>::NotSpaceOwner,
        );
    });
}

#[test]
fn update_subscription_settings_should_fail_when_role_not_in_space() {
    ExtBuilder::default().build().execute_with(|| {
        let _z = use_static_mock();

        let space_owner: AccountId = 1;
        let space1_id: SpaceId = 66;
        let space2_id: SpaceId = 45;
        let role_id_in_space2: RoleId = 10;

        let ctx1 = MockSpaces::get_space_owner_context();
        ctx1.expect().return_const(Ok(space_owner));

        let ctx2 = MockRoles::get_role_space_context();
        ctx2.expect().return_const(Ok(space2_id));

        assert_noop!(
            Subscriptions::update_subscription_settings(
                Origin::signed(space_owner),
                space1_id,
                SubscriptionSettings {
                    subscription: 55,
                    disabled: false,
                    role_id: role_id_in_space2
                },
            ),
            Error::<Test>::RoleNotInSpace,
        );
    });
}

#[test]
fn update_subscription_settings_should_work_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let _z = use_static_mock();

        let space_owner: AccountId = 1;
        let space_id: SpaceId = 66;
        let role_id: RoleId = 10;

        let ctx1 = MockSpaces::get_space_owner_context();
        ctx1.expect().return_const(Ok(space_owner));

        let ctx2 = MockRoles::get_role_space_context();
        ctx2.expect().return_const(Ok(space_id));

        let subscription_settings =
            SubscriptionSettings { subscription: 55, disabled: false, role_id };

        assert_ok!(Subscriptions::update_subscription_settings(
            Origin::signed(space_owner),
            space_id,
            subscription_settings,
        ),);

        System::assert_last_event(
            Event::SubscriptionSettingsChanged { space: space_id, account: space_owner }.into(),
        );
    });
}
