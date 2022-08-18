//! Spaces pallet benchmarking.

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{ensure, pallet_prelude::Get};
use frame_system::RawOrigin;
use sp_std::vec;

use crate::{types::*, Config};

use pallet_permissions::{default_permissions::DefaultSpacePermissions, SpacePermissionSet};

fn dummy_space_content() -> Content {
    Content::IPFS(b"QmRAQB6YaCaidP37UdDnjFY5aQuiBrbqdyoW1CaDgwxkD4".to_vec())
}

fn get_next_space_id<T: Config>() -> SpaceId {
    NextSpaceId::<T>::get()
}

fn get_new_space_id<T: Config>() -> SpaceId {
    let space_id = get_next_space_id::<T>();
    NextSpaceId::<T>::mutate(|n| *n += 1);
    space_id
}

fn create_dummy_space<T: Config>() -> Space<T> {
    let id = get_new_space_id::<T>();
    let space = Space::new(id, whitelisted_caller::<T::AccountId>(), Content::None, None);
    SpaceById::<T>::insert(id, space);

    SpaceById::<T>::get(id).expect("Expected space to exist")
}

benchmarks! {
    create_space {
        let caller = whitelisted_caller::<T::AccountId>();

        let parent_space = create_dummy_space::<T>();
        let new_space_id = get_next_space_id::<T>();

        let content = Content::None;
        let permissions_opt = None;
    }: _(RawOrigin::Signed(caller), content, permissions_opt)
    verify {
        ensure!(SpaceById::<T>::get(new_space_id).is_some(), "Expected to find the created space");
    }

    update_space {
        let caller = whitelisted_caller::<T::AccountId>();

        let space = create_dummy_space::<T>();
        let new_parent_space = create_dummy_space::<T>();

        assert!(space.content.is_none());
        assert!(space.permissions.is_none());

        let space_update = SpaceUpdate {
            content: dummy_space_content().into(),
            hidden: None,
            permissions: Some(Some(<T as pallet_permissions::Config>::DefaultSpacePermissions::get())),
        };
    }: _(RawOrigin::Signed(caller), space.id, space_update)
    verify {
        let space_from_storage = SpaceById::<T>::get(space.id).expect("Expected space to exist");
        assert!(space_from_storage.content.is_some());
        assert!(space_from_storage.permissions.is_some());
    }
}
