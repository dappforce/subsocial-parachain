//! Space follows pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks};
use frame_support::{dispatch::DispatchError, ensure};
use frame_system::RawOrigin;
use pallet_spaces::types::Space;
use subsocial_support::Content;

fn create_dummy_space<T: Config>(
    origin: RawOrigin<T::AccountId>,
) -> Result<Space<T>, DispatchError> {
    let space_id = pallet_spaces::NextSpaceId::<T>::get();

    pallet_spaces::Pallet::<T>::create_space(origin.clone().into(), Content::None, None)?;

    let space = pallet_spaces::SpaceById::<T>::get(space_id)
        .ok_or(DispatchError::Other("Space not found"))?;

    Ok(space)
}

benchmarks! {

    follow_space {
        let space_owner_origin = RawOrigin::Signed(account::<T::AccountId>("SpaceOwner", 2, 0));
        let space_follower = account::<T::AccountId>("SpaceFollower", 1, 0);

        let space = create_dummy_space::<T>(space_owner_origin.clone())?;
    }: _(RawOrigin::Signed(space_follower.clone()), space.id)
    verify {
        ensure!(SpaceFollowers::<T>::get(space.id).contains(&space_follower), "SpaceFollowers was not updated");
        ensure!(SpaceFollowedByAccount::<T>::get(&(space_follower.clone(), space.id)), "SpaceFollowedByAccount was not updated");
        ensure!(SpacesFollowedByAccount::<T>::get(&space_follower).contains(&space.id), "SpacesFollowedByAccount was not updated");
    }

    unfollow_space {
        let space_owner_origin = RawOrigin::Signed(account::<T::AccountId>("SpaceOwner", 2, 0));
        let space_follower = account::<T::AccountId>("SpaceFollower", 1, 0);

        let space = create_dummy_space::<T>(space_owner_origin.clone())?;
        Pallet::<T>::follow_space(RawOrigin::Signed(space_follower.clone()).into(),space.id)?;

    }: _(RawOrigin::Signed(space_follower.clone()), space.id)
    verify {
        ensure!(!SpaceFollowers::<T>::get(space.id).contains(&space_follower), "SpaceFollowers was not updated");
        ensure!(!SpaceFollowedByAccount::<T>::get(&(space_follower.clone(), space.id)), "SpaceFollowedByAccount was not updated");
        ensure!(!SpacesFollowedByAccount::<T>::get(&space_follower).contains(&space.id), "SpacesFollowedByAccount was not updated");
    }
}
