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


fn dummy_list_of_users<T: Config>(num_of_users: u32) -> Vec<User<T::AccountId>> {
    let mut users_to_grant = Vec::<User<T::AccountId>>::new();

    for i in 0..num_of_users {
        let user = account("user", i * 2 - 1, i * 2);
        users_to_grant.push(User::Account(user));
    }

    users_to_grant
}

fn create_dummy_role<T: Config>(
    origin: RawOrigin<T::AccountId>,
    space_id: SpaceId,
    num_of_users: u32,
) -> Result<(Role<T>, Vec<User<T::AccountId>>), DispatchError> {
    let role_id = NextRoleId::<T>::get();

    Pallet::<T>::create_role(
        origin.clone().into(),
        space_id,
        Some(100u32.into()),
        Content::None,
        vec![SP::ManageRoles],
    )?;

    let role = RoleById::<T>::get(role_id)
        .ok_or(DispatchError::Other("Role not found"))?;

    let users_to_grant = dummy_list_of_users::<T>(num_of_users);


    if !users_to_grant.is_empty() {
        Pallet::<T>::grant_role(
            origin.into(),
            role.id,
            users_to_grant.clone(),
        )?;
    }

    Ok((role, users_to_grant))
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
        let (role, _) = create_dummy_role::<T>(caller_origin.clone(), space.id, 10)?;

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
        let x in 0..T::MaxUsersToProcessPerDeleteRole::get().into();
        let caller_origin = RawOrigin::Signed(account::<T::AccountId>("Acc1", 1, 0));
        let space = create_dummy_space::<T>(caller_origin.clone())?;
        let (role, _) = create_dummy_role::<T>(caller_origin.clone(), space.id, x)?;
    }: _(caller_origin, role.id, x)
    verify {
        let deleted = RoleById::<T>::get(role.id).is_none();
        ensure!(deleted, "Role should be deleted");
    }

    grant_role {
        let x in 1..500;
        let caller_origin = RawOrigin::Signed(account::<T::AccountId>("Acc1", 1, 0));
        let space = create_dummy_space::<T>(caller_origin.clone())?;
        let (role, _) = create_dummy_role::<T>(caller_origin.clone(), space.id, 0)?;

        let users_to_grant = dummy_list_of_users::<T>(x);
    }: _(caller_origin, role.id, users_to_grant.clone())
    verify {
        let granted_users = UsersByRoleId::<T>::get(role.id);
        ensure!(granted_users == users_to_grant, "Role should be deleted");
    }

    revoke_role {
        let x in 1..500;
        let caller_origin = RawOrigin::Signed(account::<T::AccountId>("Acc1", 1, 0));
        let space = create_dummy_space::<T>(caller_origin.clone())?;
        let (role, users_to_revoke) = create_dummy_role::<T>(caller_origin.clone(), space.id, x)?;
    }: _(caller_origin, role.id, users_to_revoke)
    verify {
        let granted_users = UsersByRoleId::<T>::get(role.id);
        ensure!(granted_users.is_empty(), "Role should have zero users");
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::build(),
        crate::mock::Test,
    );
}