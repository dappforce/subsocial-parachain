//! Space follows pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::vec;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, account, whitelist};
use subsocial_support::Content;
use frame_support::{
    dispatch::DispatchError,
    traits::Currency,
    ensure,
};
use pallet_spaces::types::Space;

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

    follow_space {
        let space_owner_origin = RawOrigin::Signed(account::<T::AccountId>("SpaceOwner", 2, 0));
        let space_follower = account::<T::AccountId>("SpaceFollower", 1, 0);

        let space = create_dummy_space::<T>(space_owner_origin.clone())?;
    }: {
        Pallet::<T>::follow_space(
            RawOrigin::Signed(space_follower.clone()).into(),
            space.id,
        )?;

        // Cleanup
        whitelist!(space_follower);
        Pallet::<T>::unfollow_space(
            RawOrigin::Signed(space_follower.clone()).into(),
            space.id,
        )?;
    }
    verify {
        let space = pallet_spaces::SpaceById::<T>::get(space.id)
            .ok_or(DispatchError::Other("Space not found"))?;

        ensure!(SpaceFollowers::<T>::get(space.id).contains(&space_follower), "SpaceFollowers didn't update");
        ensure!(SpaceFollowedByAccount::<T>::get(&(space_follower.clone(), space.id)), "SpaceFollowedByAccount didn't update");
        ensure!(SpacesFollowedByAccount::<T>::get(&space_follower).contains(&space.id), "SpacesFollowedByAccount didn't update");
    }

    unfollow_space {
        let space_owner_origin = RawOrigin::Signed(account::<T::AccountId>("SpaceOwner", 2, 0));
        let space_follower = account::<T::AccountId>("SpaceFollower", 1, 0);

        let space = create_dummy_space::<T>(space_owner_origin.clone())?;
        Pallet::<T>::follow_space(RawOrigin::Signed(space_follower.clone()).into(),space.id)?;

    }: _(RawOrigin::Signed(space_follower.clone()), space.id)
    verify {
        let space = pallet_spaces::SpaceById::<T>::get(space.id)
            .ok_or(DispatchError::Other("Space not found"))?;

        ensure!(!SpaceFollowers::<T>::get(space.id).contains(&space_follower), "SpaceFollowers didn't update");
        ensure!(!SpaceFollowedByAccount::<T>::get(&(space_follower.clone(), space.id)), "SpaceFollowedByAccount didn't update");
        ensure!(!SpacesFollowedByAccount::<T>::get(&space_follower).contains(&space.id), "SpacesFollowedByAccount didn't update");
    }
}