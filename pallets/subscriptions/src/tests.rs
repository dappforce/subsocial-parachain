use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;
use sp_std::sync::{Mutex, MutexGuard};

use crate::{mock::*, types::SubscriptionSettings, Error, Event};

#[test]
fn update_subscription_settings_should_fail_when_caller_not_signed() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Subscriptions::update_subscription_settings(
                RuntimeOrigin::none(),
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
        let space_owner: AccountId = 1;
        let not_space_owner: AccountId = 10;
        let space_id: SpaceId = 66;

        get_space_owner__return::set(Ok(space_owner));

        assert_noop!(
            Subscriptions::update_subscription_settings(
                RuntimeOrigin::signed(not_space_owner),
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
        let space_owner: AccountId = 1;
        let space1_id: SpaceId = 66;
        let space2_id: SpaceId = 45;
        let role_id_in_space2: RoleId = 10;

        get_space_owner__return::set(Ok(space_owner));

        get_role_space__return::set(Ok(space2_id));


        assert_noop!(
            Subscriptions::update_subscription_settings(
                RuntimeOrigin::signed(space_owner),
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
        let space_owner: AccountId = 1;
        let space_id: SpaceId = 66;
        let role_id: RoleId = 10;

        get_space_owner__return::set(Ok(space_owner));

        get_role_space__return::set(Ok(space_id));

        let subscription_settings =
            SubscriptionSettings { subscription: 55, disabled: false, role_id };

        assert_ok!(Subscriptions::update_subscription_settings(
            RuntimeOrigin::signed(space_owner),
            space_id,
            subscription_settings,
        ),);

        System::assert_last_event(
            Event::SubscriptionSettingsChanged { space: space_id, account: space_owner }.into(),
        );
    });
}
