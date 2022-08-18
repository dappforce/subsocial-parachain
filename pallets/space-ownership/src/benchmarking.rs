//! Space ownership pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, account, whitelist};
use frame_support::{
    dispatch::DispatchError,
    traits::Currency,
    ensure,
};
use pallet_spaces::types::Space;
use subsocial_support::Content;

fn create_dummy_space<T: Config>(origin: RawOrigin<T::AccountId>) -> Result<Space<T>, DispatchError> {
    let space_id = pallet_spaces::NextSpaceId::<T>::get();

    pallet_spaces::Pallet::<T>::create_space(
        origin.clone().into(),
        Content::None,
        None,
    ).map_err(|e| e.error)?;

    let space = pallet_spaces::SpaceById::<T>::get(space_id)
        .ok_or(DispatchError::Other("Space not found"))?;

    Ok(space)
}

benchmarks! {

    transfer_space_ownership {
        let acc1 = account::<T::AccountId>("Acc1", 1, 0);
        let acc2 = account::<T::AccountId>("Acc2", 2, 0);

        let space = create_dummy_space::<T>(RawOrigin::Signed(acc1.clone()))?;
    }: _(RawOrigin::Signed(acc1.clone()), space.id, acc2.clone())
    verify {
        ensure!(PendingSpaceOwner::<T>::get(&space.id) == Some(acc2), "Request is not found");
    }

    accept_pending_ownership {
        let acc1 = account::<T::AccountId>("Acc1", 1, 0);
        let acc2 = account::<T::AccountId>("Acc2", 2, 0);

        let space = create_dummy_space::<T>(RawOrigin::Signed(acc1.clone()))?;
        Pallet::<T>::transfer_space_ownership(
            RawOrigin::Signed(acc1.clone()).into(),
            space.id,
            acc2.clone(),
        )?;
    }: _(RawOrigin::Signed(acc2.clone()), space.id)
    verify {
        let space = pallet_spaces::SpaceById::<T>::get(space.id)
            .ok_or(DispatchError::Other("Space not found"))?;

        ensure!(PendingSpaceOwner::<T>::get(&space.id) == None, "Request is found");
        ensure!(space.owner == acc2, "Space owner is not updated");
    }

    reject_pending_ownership {
        let acc1 = account::<T::AccountId>("Acc1", 1, 0);
        let acc2 = account::<T::AccountId>("Acc2", 2, 0);

        let space = create_dummy_space::<T>(RawOrigin::Signed(acc1.clone()))?;
        Pallet::<T>::transfer_space_ownership(
            RawOrigin::Signed(acc1.clone()).into(),
            space.id,
            acc2.clone(),
        )?;
    }: _(RawOrigin::Signed(acc2.clone()), space.id)
    verify {
        let space = pallet_spaces::SpaceById::<T>::get(space.id)
            .ok_or(DispatchError::Other("Space not found"))?;

        ensure!(PendingSpaceOwner::<T>::get(&space.id) == None, "Request is found");
        ensure!(space.owner == acc1, "Space owner is updated");
    }

}