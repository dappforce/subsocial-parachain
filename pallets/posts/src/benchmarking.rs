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
use pallet_spaces::types::Space;


fn create_dummy_space<T: Config>(origin: RawOrigin<T::AccountId>) -> Result<Space<T>, DispatchError> {
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


fn create_dummy_post<T: Config>(origin: RawOrigin<T::AccountId>, space: Space<T>) -> Result<Post<T>, DispatchError> {
    let post_id = NextPostId::<T>::get();

    Pallet::<T>::create_post(
        origin.clone().into(),
        Some(space.id),
        PostExtension::RegularPost,
        Content::None,
    )?;

    let post = PostById::<T>::get(post_id)
        .ok_or(DispatchError::Other("Post wasn't created"))?;

    Ok(post)
}

fn create_dummy_reply<T: Config>(origin: RawOrigin<T::AccountId>, space: Space<T>, post: Post<T>) -> Result<Post<T>, DispatchError> {
    let post_id = NextPostId::<T>::get();

    Pallet::<T>::create_post(
        origin.clone().into(),
        Some(space.id),
        PostExtension::Comment(Comment {
            parent_id: None,
            root_post_id: post.id,
        }),
        Content::None,
    )?;

    let post = PostById::<T>::get(post_id)
        .ok_or(DispatchError::Other("Reply wasn't created"))?;

    Ok(post)
}

benchmarks! {
    create_post__regular {
        let origin = RawOrigin::Signed(whitelisted_caller());
        let space = create_dummy_space::<T>(origin.clone())?;
        let post_id = NextPostId::<T>::get();

    }: create_post(origin, Some(space.id), PostExtension::RegularPost, Content::None)
    verify {
        let post = PostById::<T>::get(post_id)
            .ok_or(DispatchError::Other("Post wasn't created"))?;

        ensure!(post.space_id == Some(space.id), "Post wasn't created in the right space");
        ensure!(post.extension == PostExtension::RegularPost, "Post wasn't created with the right extension");
    }

    create_post__shared {
        let origin = RawOrigin::Signed(whitelisted_caller());
        let space = create_dummy_space::<T>(origin.clone())?;
        let original_post = create_dummy_post::<T>(origin.clone(), space.clone())?;
        let post_id = NextPostId::<T>::get();

    }: create_post(origin, Some(space.id), PostExtension::SharedPost(original_post.id), Content::None)
    verify {
        let post = PostById::<T>::get(post_id)
            .ok_or(DispatchError::Other("Post wasn't created"))?;

        ensure!(post.space_id == Some(space.id), "Post wasn't created in the right space");
        ensure!(post.extension == PostExtension::SharedPost(original_post.id), "Post wasn't created with the right extension");
    }

    create_post__comment {
        let origin = RawOrigin::Signed(whitelisted_caller());
        let space = create_dummy_space::<T>(origin.clone())?;
        let original_post = create_dummy_post::<T>(origin.clone(), space.clone())?;
        let reply = create_dummy_reply::<T>(origin.clone(), space.clone(), original_post.clone())?;
        let post_id = NextPostId::<T>::get();

        let ext = PostExtension::Comment(Comment {
            parent_id: Some(reply.id),
            root_post_id: original_post.id,
        });
    }: create_post(origin, Some(space.id), ext, Content::None)
    verify {
        let post = PostById::<T>::get(post_id)
            .ok_or(DispatchError::Other("Reply wasn't created"))?;

        ensure!(post.space_id == Some(space.id), "Reply wasn't created in the right space");
        ensure!(post.extension == ext, "Post wasn't created with the right extension");
    }
}