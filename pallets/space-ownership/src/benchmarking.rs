// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

//! Space ownership pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks};
use frame_support::{dispatch::DispatchError, ensure, traits::Currency};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

use subsocial_support::{
    traits::{DomainsProvider, PostsProvider, SpacesInterface},
    Content,
};

use super::*;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

fn grab_accounts<T: Config>() -> (T::AccountId, T::AccountId) {
    let acc1 = account::<T::AccountId>("Acc1", 1, 0);
    let acc2 = account::<T::AccountId>("Acc2", 2, 0);
    T::Currency::make_free_balance_be(&acc1, BalanceOf::<T>::max_value());
    T::Currency::make_free_balance_be(&acc2, BalanceOf::<T>::max_value());

    (acc1, acc2)
}

fn create_dummy_space<T: Config>(
    owner: &T::AccountId,
) -> Result<EntityWithOwnership<T>, DispatchError> {
    let space_id = T::SpacesInterface::create_space(owner, Content::None)?;
    Ok(EntityWithOwnership::Space(space_id))
}

fn create_dummy_post<T: Config>(
    owner: &T::AccountId,
) -> Result<EntityWithOwnership<T>, DispatchError> {
    let space_id = T::SpacesInterface::create_space(owner, Content::None)?;
    let post_id = T::PostsProvider::create_post(owner, space_id, Content::None)?;
    Ok(EntityWithOwnership::Post(post_id))
}

fn create_dummy_domain<T: Config>(
    owner: &T::AccountId,
) -> Result<EntityWithOwnership<T>, DispatchError> {
    let domain = T::DomainsProvider::register_domain(owner, "dappforce.sub".as_bytes())?;
    let domain_bounded = domain.try_into().unwrap();
    Ok(EntityWithOwnership::Domain(domain_bounded))
}

benchmarks! {

    transfer_space_ownership {
        let (acc1, acc2) = grab_accounts::<T>();
        let space_entity = create_dummy_space::<T>(&acc1)?;
    }: transfer_ownership(RawOrigin::Signed(acc1.clone()), space_entity.clone(), acc2.clone())
    verify {
        ensure!(
            PendingOwnershipTransfers::<T>::get(&space_entity) == Some(acc2),
            "Request was not created",
        );
    }

    transfer_post_ownership {
        let (acc1, acc2) = grab_accounts::<T>();
        let post_entity = create_dummy_post::<T>(&acc1)?;
    }: transfer_ownership(RawOrigin::Signed(acc1.clone()), post_entity.clone(), acc2.clone())
    verify {
        ensure!(
            PendingOwnershipTransfers::<T>::get(&post_entity) == Some(acc2),
            "Request was not created",
        );
    }

    transfer_domain_ownership {
        let (acc1, acc2) = grab_accounts::<T>();
        let domain_entity = create_dummy_domain::<T>(&acc1)?;
    }: transfer_ownership(RawOrigin::Signed(acc1), domain_entity.clone(), acc2.clone())
    verify {
        ensure!(
            PendingOwnershipTransfers::<T>::get(&domain_entity) == Some(acc2),
            "Request was not created",
        );
    }

    accept_pending_space_ownership_transfer {
        let (acc1, acc2) = grab_accounts::<T>();
        let space_entity = create_dummy_space::<T>(&acc1)?;

        Pallet::<T>::transfer_ownership(
            RawOrigin::Signed(acc1).into(),
            space_entity.clone(),
            acc2.clone(),
        )?;
    }: accept_pending_ownership(RawOrigin::Signed(acc2), space_entity.clone())
    verify {
        ensure!(
            PendingOwnershipTransfers::<T>::get(&space_entity).is_none(),
            "Request was not cleaned",
        );
    }

    accept_pending_post_ownership_transfer {
        let (acc1, acc2) = grab_accounts::<T>();
        let post_entity = create_dummy_post::<T>(&acc1)?;

        Pallet::<T>::transfer_ownership(
            RawOrigin::Signed(acc1).into(),
            post_entity.clone(),
            acc2.clone(),
        )?;
    }: accept_pending_ownership(RawOrigin::Signed(acc2), post_entity.clone())
    verify {
        ensure!(
            PendingOwnershipTransfers::<T>::get(&post_entity).is_none(),
            "Request was not cleaned",
        );
    }

    accept_pending_domain_ownership_transfer {
        let (acc1, acc2) = grab_accounts::<T>();
        let domain_entity = create_dummy_domain::<T>(&acc1)?;

        Pallet::<T>::transfer_ownership(
            RawOrigin::Signed(acc1).into(),
            domain_entity.clone(),
            acc2.clone(),
        )?;
    }: accept_pending_ownership(RawOrigin::Signed(acc2), domain_entity.clone())
    verify {
        ensure!(
            PendingOwnershipTransfers::<T>::get(&domain_entity).is_none(),
            "Request was not cleaned",
        );
    }

    reject_pending_ownership {
        let (acc1, acc2) = grab_accounts::<T>();
        let space_entity = create_dummy_space::<T>(&acc1)?;

        Pallet::<T>::transfer_ownership(
            RawOrigin::Signed(acc1).into(),
            space_entity.clone(),
            acc2.clone(),
        )?;
    }: _(RawOrigin::Signed(acc2), space_entity.clone())
    verify {
        ensure!(
            PendingOwnershipTransfers::<T>::get(&space_entity).is_none(),
            "Request was not cleaned",
        );
    }

}
