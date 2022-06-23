//! Posts pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::dispatch::DispatchError;
use frame_support::ensure;
use frame_system::RawOrigin;
use sp_std::vec;

use pallet_spaces::Module as Spaces;
use pallet_utils::Config as UtilsConfig;
use pallet_utils::mock_functions::{updated_content_ipfs, valid_content_ipfs};
use pallet_utils::mock_functions::bench::caller_with_balance;

use crate::Module as Posts;

const SPACE1: SpaceId = 1001;
const SPACE2: SpaceId = 1002;
const POST: PostId = 1;

fn add_space<T: Config>(origin: RawOrigin<T::AccountId>) -> DispatchResult {
    Spaces::<T>::create_space(
        origin.into(), None, None, valid_content_ipfs(), None
    )
}

fn add_origin_with_space_post_and_balance<T: Config>() -> Result<RawOrigin<T::AccountId>, DispatchError> {
    let caller = caller_with_balance::<T::AccountId, <T as UtilsConfig>::Currency>();
    let origin = RawOrigin::Signed(caller);

    add_space::<T>(origin.clone())?;
    Posts::<T>::create_post(
        origin.clone().into(),
        Some(SPACE1),
        PostExtension::RegularPost,
        valid_content_ipfs(),
    )?;

    Ok(origin)
}

benchmarks! {
    create_post {
        let caller = caller_with_balance::<T::AccountId, <T as UtilsConfig>::Currency>();
        let origin = RawOrigin::Signed(caller.clone());

        add_space::<T>(origin.clone())?;
    }: _(origin, Some(SPACE1), PostExtension::RegularPost, valid_content_ipfs())
    verify {
        ensure!(PostById::<T>::get(POST).is_some(), "Post not added");
    }

    update_post {
        let origin = add_origin_with_space_post_and_balance::<T>()?;

        let post_update: PostUpdate = PostUpdate {
            space_id: None,
            content: Some(updated_content_ipfs()),
            hidden: Some(true),
        };

    }: _(origin, POST, post_update)
    verify {
        let post: Post<T> = PostById::<T>::get(POST).unwrap();
        ensure!(post.content == updated_content_ipfs(), "Post content was not updated");
        ensure!(post.hidden == true, "Post hidden status was not updated");
    }

    move_post {
        let origin = add_origin_with_space_post_and_balance::<T>()?;
        add_space::<T>(origin.clone())?;
    }: _(origin, POST, Some(SPACE2))
    verify {
        ensure!(Posts::<T>::post_ids_by_space_id(SPACE1).is_empty(), "Post wasn't moved out of the old space");
        ensure!(Posts::<T>::post_ids_by_space_id(SPACE2) == vec![POST], "Post wasn't moved correctly to the new space");
    }
}

impl_benchmark_test_suite!(
    Posts,
    pallet_utils::mock_functions::ext_builder::DefaultExtBuilder::<crate::mock::Test>::build(),
    crate::mock::Test,
);
