use std::convert::TryFrom;

use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

use subsocial_support::{Content, PostId, SpaceId};

use crate::{mock::*, pallet::ResourceId, Event, ResourceDiscussion};

fn resource_id(resource_id: &[u8]) -> ResourceId<Test> {
    ResourceId::<Test>::try_from(Vec::from(resource_id))
        .expect("given resource id is longer than Config::MaxResourceIdLength")
}

fn account(account: AccountId) -> AccountId {
    account
}

fn create_space(owner: AccountId) -> SpaceId {
    let space_id = pallet_spaces::NextSpaceId::<Test>::get();

    assert_ok!(Spaces::create_space(RuntimeOrigin::signed(owner), Content::None, None));

    pallet_spaces::SpaceById::<Test>::get(space_id)
        .expect("space didn't get created")
        .id
}

fn create_post(owner: AccountId, space_id: SpaceId) -> PostId {
    let post_id = pallet_posts::NextPostId::<Test>::get();

    assert_ok!(Posts::create_post(
        RuntimeOrigin::signed(owner),
        Some(space_id),
        pallet_posts::PostExtension::RegularPost,
        Content::None
    ));

    pallet_posts::PostById::<Test>::get(post_id)
        .expect("post didn't get created")
        .id
}

#[test]
fn link_post_to_resource_should_fail_when_caller_is_unsigned() {
    ExtBuilder::default().build().execute_with(|| {
        let post_id = 213;
        let resource_id = resource_id(b"test");

        assert_noop!(
            ResourceDiscussions::link_post_to_resource(
                RuntimeOrigin::none(),
                resource_id.clone(),
                post_id,
            ),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn link_post_to_resource_should_fail_when_caller_is_not_post_owner() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = account(1);
        let post_id = create_post(owner, create_space(owner));
        let resource_id = resource_id(b"test");

        let not_owner = account(2);

        assert_noop!(
            ResourceDiscussions::link_post_to_resource(
                RuntimeOrigin::signed(not_owner),
                resource_id.clone(),
                post_id,
            ),
            pallet_posts::Error::<Test>::NotAPostOwner,
        );
    });
}

#[test]
fn link_post_to_resource_should_fail_when_post_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account(1);
        let post_id = 12;
        let resource_id = resource_id(b"test");

        assert_noop!(
            ResourceDiscussions::link_post_to_resource(
                RuntimeOrigin::signed(caller),
                resource_id.clone(),
                post_id,
            ),
            pallet_posts::Error::<Test>::PostNotFound,
        );
    });
}

#[test]
fn link_post_to_resource_should_link_post_to_new_resource_id() {
    ExtBuilder::default().build().execute_with(|| {
        let caller = account(1);
        let post_id = create_post(caller, create_space(caller));
        let resource_id = resource_id(b"test");

        assert_ok!(ResourceDiscussions::link_post_to_resource(
            RuntimeOrigin::signed(caller),
            resource_id.clone(),
            post_id,
        ));

        assert_eq!(ResourceDiscussion::<Test>::get(resource_id.clone(), caller), Some(post_id));
        System::assert_last_event(
            Event::ResourceDiscussionLinked {
                resource_id: resource_id.clone(),
                account_id: caller,
                post_id,
            }
            .into(),
        );
    });
}
