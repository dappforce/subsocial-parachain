// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

//! Post follows pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks};
use frame_support::{dispatch::DispatchError, ensure};
use frame_system::RawOrigin;

use pallet_posts::{types::Post, PostExtension, PostById, NextPostId};
use pallet_spaces::types::Space;
use subsocial_support::{Content, SpaceId};

use super::*;

fn create_dummy_space<T: Config>(
    origin: RawOrigin<T::AccountId>,
) -> Result<Space<T>, DispatchError> {
    let space_id = pallet_spaces::NextSpaceId::<T>::get();

    pallet_spaces::Pallet::<T>::create_space(origin.into(), Content::None, None)?;

    let space = pallet_spaces::SpaceById::<T>::get(space_id)
        .ok_or(DispatchError::Other("Space not found"))?;

    Ok(space)
}

fn create_dummy_post<T: Config>(
    origin: RawOrigin<T::AccountId>,
    space_id: SpaceId,
) -> Result<Post<T>, DispatchError> {
    let post_id = NextPostId::<T>::get();

    pallet_posts::Pallet::<T>::create_post(
        origin.into(),
        Some(space_id),
        PostExtension::RegularPost,
        Content::None,
    )?;

    let post = PostById::<T>::get(post_id).ok_or(DispatchError::Other("Post wasn't created"))?;

    Ok(post)
}

benchmarks! {

    follow_post {
        let owner_origin = RawOrigin::Signed(account::<T::AccountId>("PostOwner", 2, 0));
        let post_follower = account::<T::AccountId>("PostFollower", 1, 0);

        let space = create_dummy_space::<T>(owner_origin.clone())?;
        let post = create_dummy_post::<T>(owner_origin.clone(), space.id)?;
    }: _(RawOrigin::Signed(post_follower.clone()), post.id)
    verify {
        ensure!(PostFollowers::<T>::get(post.id).contains(&post_follower), "PostFollowers was not updated");
        ensure!(PostFollowedByAccount::<T>::get(&(post_follower.clone(), post.id)), "PostFollowedByAccount was not updated");
        ensure!(PostsFollowedByAccount::<T>::get(&post_follower).contains(&post.id), "PostsFollowedByAccount was not updated");
    }

    unfollow_post {
        let owner_origin = RawOrigin::Signed(account::<T::AccountId>("PostOwner", 2, 0));
        let post_follower = account::<T::AccountId>("PostFollower", 1, 0);

        let space = create_dummy_space::<T>(owner_origin.clone())?;
        let post = create_dummy_post::<T>(owner_origin.clone(), space.id)?;
        Pallet::<T>::follow_post(RawOrigin::Signed(post_follower.clone()).into(),post.id)?;

    }: _(RawOrigin::Signed(post_follower.clone()), post.id)
    verify {
        ensure!(!PostFollowers::<T>::get(post.id).contains(&post_follower), "PostFollowers was not updated");
        ensure!(!PostFollowedByAccount::<T>::get(&(post_follower.clone(), post.id)), "PostFollowedByAccount was not updated");
        ensure!(!PostsFollowedByAccount::<T>::get(&post_follower).contains(&post.id), "PostsFollowedByAccount was not updated");
    }
}
