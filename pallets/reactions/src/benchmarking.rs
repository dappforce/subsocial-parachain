// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

//! Reactions pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::dispatch::DispatchError;
use frame_system::RawOrigin;
use pallet_posts::{Post, PostExtension};
use pallet_spaces::types::Space;
use sp_std::vec;
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

fn create_dummy_post<T: Config>(origin: RawOrigin<T::AccountId>) -> Result<Post<T>, DispatchError> {
    let post_id = pallet_posts::NextPostId::<T>::get();
    let space = create_dummy_space::<T>(origin.clone())?;

    pallet_posts::Pallet::<T>::create_post(
        origin.clone().into(),
        Some(space.id),
        PostExtension::RegularPost,
        Content::None,
    )?;

    let post = PostById::<T>::get(post_id).ok_or(DispatchError::Other("Post not found"))?;

    Ok(post)
}

fn create_dummy_post_reaction<T: Config>(
    origin: RawOrigin<T::AccountId>,
) -> Result<(Post<T>, Reaction<T>), DispatchError> {
    let post = create_dummy_post::<T>(origin.clone())?;
    let reaction_id = NextReactionId::<T>::get();

    Pallet::<T>::create_post_reaction(origin.clone().into(), post.id, ReactionKind::Upvote)?;

    let reaction = ReactionById::<T>::get(reaction_id)
        .ok_or(DispatchError::Other("Reaction not found"))?;

    Ok((post, reaction))
}

benchmarks! {
    create_post_reaction {
        let origin = RawOrigin::Signed(whitelisted_caller());
        let post = create_dummy_post::<T>(origin.clone())?;
        let reaction_kind = ReactionKind::Upvote;
        let reaction_id = NextReactionId::<T>::get();

    }: _(origin, post.id, reaction_kind)
    verify {
        ensure!(ReactionIdsByPostId::<T>::get(post.id) == vec![reaction_id], "Incorrect reaction in storage");
        ensure!(
            ReactionById::<T>::get(reaction_id)
                .expect("Reaction not found")
                .kind == reaction_kind,
            "Reaction kind does not match"
        );
    }

    update_post_reaction {
        let origin = RawOrigin::Signed(whitelisted_caller());
        let (post, reaction) = create_dummy_post_reaction::<T>(origin.clone())?;
        let other_kind = match reaction.kind {
            ReactionKind::Upvote => ReactionKind::Downvote,
            ReactionKind::Downvote => ReactionKind::Upvote,
        };
    }: _(origin, post.id, reaction.id, other_kind)
    verify {
        ensure!(
            ReactionById::<T>::get(reaction.id)
                .expect("Reaction not found")
                .kind == other_kind,
            "Reaction kind does not match"
        );
    }

    delete_post_reaction {
        let origin = RawOrigin::Signed(whitelisted_caller());
        let (post, reaction) = create_dummy_post_reaction::<T>(origin.clone())?;

        ensure!(ReactionIdsByPostId::<T>::get(post.id) == vec![reaction.id], "Incorrect reaction in storage");
    }: _(origin, post.id, reaction.id)
    verify {
        ensure!(ReactionIdsByPostId::<T>::get(post.id).is_empty(), "Reaction was not deleted by post id");
        ensure!(ReactionById::<T>::get(reaction.id) == None, "Reaction was not deleted");
    }
}
