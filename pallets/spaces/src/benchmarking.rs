//! Spaces pallet benchmarking.

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, ensure, pallet_prelude::Get};
use frame_system::RawOrigin;

use crate::{types::*, Config};

use super::*;

fn dummy_space_content() -> Content {
    subsocial_support::mock_functions::valid_content_ipfs()
}

fn create_dummy_space<T: Config>(caller: T::AccountId) -> Space<T> {
    assert_ok!(Pallet::<T>::create_space(RawOrigin::Signed(caller).into(), Content::None, None));
    let id = Pallet::<T>::next_space_id() - 1;

    SpaceById::<T>::get(id).expect("qed; space should exist")
}

benchmarks! {
    create_space {
        let caller = whitelisted_caller::<T::AccountId>();

        let parent_space = create_dummy_space::<T>(caller.clone());
        let new_space_id = NextSpaceId::<T>::get();

        let content = dummy_space_content();
        let permissions_opt = None;
    }: _(RawOrigin::Signed(caller), content, permissions_opt)
    verify {
        ensure!(SpaceById::<T>::get(new_space_id).is_some(), "Created space should exist");
    }

    update_space {
        let caller = whitelisted_caller::<T::AccountId>();

        let space = create_dummy_space::<T>(caller.clone());
        let new_parent_space = create_dummy_space::<T>(caller.clone());

        assert!(space.content.is_none());
        assert!(space.permissions.is_none());

        let space_update = SpaceUpdate {
            content: dummy_space_content().into(),
            hidden: true.into(),
            permissions: Some(Some(<T as pallet_permissions::Config>::DefaultSpacePermissions::get())),
        };
    }: _(RawOrigin::Signed(caller), space.id, space_update)
    verify {
        let space_from_storage = SpaceById::<T>::get(space.id).expect("Updated space should exist");
        assert!(space_from_storage.content.is_some());
        assert!(space_from_storage.edited);
        assert!(space_from_storage.permissions.is_some());
    }
}
