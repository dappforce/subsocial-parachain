//! Post follows pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks};
use frame_support::{dispatch::DispatchError, ensure};
use frame_system::RawOrigin;
use pallet_posts::types::Post;
use subsocial_support::Content;

fn create_dummy_post<T: Config>(
    origin: RawOrigin<T::AccountId>,
) -> Result<Post<T>, DispatchError> {
    let post_id = pallet_posts::NextPostId::<T>::get();

    pallet_posts::Pallet::<T>::create_post(origin.clone().into(), Content::None, None)?;

    let post = pallet_posts::PostById::<T>::get(post_id)
        .ok_or(DispatchError::Other("Post not found"))?;

    Ok(post)
}

benchmarks! {

    follow_post {
        let post_owner_origin = RawOrigin::Signed(account::<T::AccountId>("PostOwner", 2, 0));
        let post_follower = account::<T::AccountId>("PostFollower", 1, 0);

        let post = create_dummy_post::<T>(post_owner_origin.clone())?;
    }: _(RawOrigin::Signed(post_follower.clone()), post.id)
    verify {
        ensure!(PostFollowers::<T>::get(post.id).contains(&post_follower), "PostFollowers was not updated");
        ensure!(PostFollowedByAccount::<T>::get(&(post_follower.clone(), post.id)), "PostFollowedByAccount was not updated");
        ensure!(PostsFollowedByAccount::<T>::get(&post_follower).contains(&post.id), "PostsFollowedByAccount was not updated");
    }

    unfollow_post {
        let post_owner_origin = RawOrigin::Signed(account::<T::AccountId>("PostOwner", 2, 0));
        let post_follower = account::<T::AccountId>("PostFollower", 1, 0);

        let post = create_dummy_post::<T>(post_owner_origin.clone())?;
        Pallet::<T>::follow_post(RawOrigin::Signed(post_follower.clone()).into(),post.id)?;

    }: _(RawOrigin::Signed(post_follower.clone()), post.id)
    verify {
        ensure!(!PostFollowers::<T>::get(post.id).contains(&post_follower), "PostFollowers was not updated");
        ensure!(!PostFollowedByAccount::<T>::get(&(post_follower.clone(), post.id)), "PostFollowedByAccount was not updated");
        ensure!(!PostsFollowedByAccount::<T>::get(&post_follower).contains(&post.id), "PostsFollowedByAccount was not updated");
    }
}
