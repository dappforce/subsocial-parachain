//! Roles pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use subsocial_support::{Content, User};
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
        origin.into(),
        None,
        Content::None,
        None,
    ).map_err(|e| e.error)?;

    let space = pallet_spaces::SpaceById::<T>::get(space_id)
        .ok_or(DispatchError::Other("Space not found"))?;

    Ok(space)
}


fn create_dummy_role<T: Config>(
    origin: RawOrigin<T::AccountId>,
    space_id: SpaceId,
) -> Result<Role<T>, DispatchError> {
    let role_id = NextRoleId::<T>::get();

    Pallet::<T>::create_role(
        origin.into(),
        space_id,
        Some(100u32.into()),
        Content::None,
        vec![SP::ManageRoles],
    )?;

    let role = RoleById::<T>::get(role_id)
        .ok_or(DispatchError::Other("Role not found"))?;

    Ok(role)
}

benchmarks! {
    where_clause { where T: pallet_spaces::Config }

    create_role {
        let caller_origin = RawOrigin::Signed(account::<T::AccountId>("Acc1", 1, 0));
        let space = create_dummy_space::<T>(caller_origin.clone())?;
        let time_to_live: Option<T::BlockNumber> = Some(100u32.into());
        let content = Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec());
        let perms = vec![SP::ManageRoles];
        let role_id = NextRoleId::<T>::get();
    }: _(caller_origin, space.id, time_to_live, content, perms)
    verify {
        let role = RoleById::<T>::get(role_id).unwrap();
        let space_roles_ids = RoleIdsBySpaceId::<T>::get(space.id);

        ensure!(role.id == role_id, "Role id doesn't match");
        ensure!(space_roles_ids.contains(&role_id), "Role id not in space roles");
    }

    update_role {
        let caller_origin = RawOrigin::Signed(account::<T::AccountId>("Acc1", 1, 0));
        let space = create_dummy_space::<T>(caller_origin.clone())?;
        let role = create_dummy_role::<T>(caller_origin.clone(), space.id)?;

        ensure!(!role.disabled, "Role should be enabled");

        let update = RoleUpdate {
             disabled: true.into(),
             content: Content::IPFS(b"QmRAQB6YaCyidP37UdDnjFY5vQuiBrcqdyoW1CuDgwxkD4".to_vec()).into(),
             permissions: None
        };
    }: _(caller_origin, role.id, update)
    verify {
        let role = RoleById::<T>::get(role.id).unwrap();
        ensure!(role.disabled, "Role should be disabled");
    }

    delete_role {
        let caller_origin = RawOrigin::Signed(account::<T::AccountId>("Acc1", 1, 0));
        let space = create_dummy_space::<T>(caller_origin.clone())?;
        let role = create_dummy_role::<T>(caller_origin.clone(), space.id)?;
    }: _(caller_origin, role.id)
    verify {
        let deleted = RoleById::<T>::get(role.id).is_none();
        ensure!(deleted, "Role should be deleted");
    }

    grant_role {
        let caller_origin = RawOrigin::Signed(account::<T::AccountId>("Acc1", 1, 0));
        let space = create_dummy_space::<T>(caller_origin.clone())?;
        let role = create_dummy_role::<T>(caller_origin.clone(), space.id)?;

        let users_to_grant = vec![User::Account(account::<T::AccountId>("Acc2", 2, 0))];
    }: _(caller_origin, role.id, users_to_grant.clone())
    verify {
        let granted_users = UsersByRoleId::<T>::get(role.id);
        ensure!(granted_users == users_to_grant, "Role should be deleted");
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::build(),
        crate::mock::Test,
    );
}