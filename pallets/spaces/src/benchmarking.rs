//! Spaces pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::benchmarks;
use frame_support::ensure;
use frame_system::RawOrigin;
use sp_std::vec;

use pallet_permissions::{
    default_permissions::DefaultSpacePermissions,
    SpacePermissionSet,
};
use pallet_utils::Config as UtilsConfig;
use pallet_utils::mock_functions::{
    updated_content_ipfs, updated_max_length_handle,
    valid_content_ipfs, valid_max_length_handle,
};
use pallet_utils::mock_functions::bench::caller_with_balance;

const SPACE: SpaceId = 1001;

fn extend_none_permissions_with(
    permissions: &mut SpacePermissionSet, extends: Option<SpacePermissionSet>
) -> SpacePermissionSet {
    permissions.extend(extends.unwrap_or_default());
    permissions.clone()
}

fn space_permissions_overrides() -> SpacePermissions {
    let defaults = DefaultSpacePermissions::get();
    let mut all_permissions = defaults.clone();

    all_permissions.none = all_permissions.none.map(|mut perms| {
        extend_none_permissions_with(&mut perms, defaults.everyone);
        extend_none_permissions_with(&mut perms, defaults.follower);
        extend_none_permissions_with(&mut perms, defaults.space_owner)
    });

    all_permissions
}

fn default_space_update<T: Config>() -> SpaceUpdate {
    SpaceUpdate {
        parent_id: None,
        handle: Some(Some(updated_max_length_handle::<T>())),
        content: Some(updated_content_ipfs()),
        hidden: Some(true),
        permissions: Some(Some(space_permissions_overrides())),
    }
}

benchmarks! {
    create_space {
        let caller = caller_with_balance::<T::AccountId, <T as UtilsConfig>::Currency>();
    }: _(RawOrigin::Signed(caller), None, Some(valid_max_length_handle::<T>()), valid_content_ipfs(), Some(space_permissions_overrides()))
    verify {
        ensure!(SpaceById::<T>::get(SPACE).is_some(), Error::<T>::SpaceNotFound)
    }

    update_space {
        let caller = caller_with_balance::<T::AccountId, <T as UtilsConfig>::Currency>();
        let origin = RawOrigin::Signed(caller);
        let space_update = default_space_update::<T>();

        Module::<T>::create_space(origin.clone().into(), None, Some(valid_max_length_handle::<T>()), valid_content_ipfs(), None)?;
    }: _(origin, SPACE, space_update.clone())
    verify {
        let space: Space<T> = SpaceById::<T>::get(SPACE).unwrap();
        assert_eq!(space.handle, space_update.handle.unwrap_or_default());
        assert_eq!(space.content, updated_content_ipfs());
        assert_eq!(space.hidden, true);
    }
}

#[cfg(test)]
mod test {
    use frame_support::assert_ok;

    use pallet_utils::mock_functions::ext_builder::DefaultExtBuilder;

    use crate::mock::Test;

    use super::*;

    #[test]
    fn test_benchmarks() {
        DefaultExtBuilder::<Test>::build().execute_with(|| {
            assert_ok!(test_benchmark_create_space::<Test>());
            assert_ok!(test_benchmark_update_space::<Test>());
        });
    }
}
