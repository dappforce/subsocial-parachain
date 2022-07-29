//! Roles pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use subsocial_support::Content;
use frame_support::{
    dispatch::DispatchError,
    traits::Currency,
};
use sp_std::{
    collections::btree_set::BTreeSet,
    prelude::Vec,
};
use pallet_permissions::{SpacePermission, SpacePermission as SP, SpacePermissions};
use pallet_spaces::types::Space;
use frame_benchmarking::account;

fn create_dummy_space<T: Config + pallet_spaces::Config>(origin: RawOrigin<T::AccountId>) -> Result<Space<T>, DispatchError> {
    let space_id = pallet_spaces::NextSpaceId::<T>::get();

    pallet_spaces::Pallet::<T>::create_space(
        origin.clone().into(),
        None,
        Content::None,
        None,
    ).map_err(|e| e.error)?;

    let space = pallet_spaces::SpaceById::<T>::get(space_id)
        .ok_or(DispatchError::Other("Space not found"))?;

    Ok(space)
}


benchmarks! {
    where_clause { where T: pallet_spaces::Config }

    create_role {
        let caller = account::<T::AccountId>("Acc1", 1, 0);
        let space = create_dummy_space::<T>(RawOrigin::Signed(caller.clone().into()))?;
        let time_to_live: Option<T::BlockNumber> = Some(100u32.into());
        let content = Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec());
        let perms = vec![SP::ManageRoles];
        let role_id = NextRoleId::<T>::get();
    }: _(RawOrigin::Signed(caller.clone()), space.id, time_to_live, content, perms)
    verify {
        let role = RoleById::<T>::get(role_id).unwrap();
        let space_roles_ids = RoleIdsBySpaceId::<T>::get(space.id);

        ensure!(role.id == role_id, "Role id doesn't match");
        ensure!(space_roles_ids.contains(&role_id), "Role id not in space roles");
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::build(),
        crate::mock::Test,
    );
}