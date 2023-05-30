#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::{Currency, EnsureOrigin, Get};
use frame_support::ensure;
use frame_system::RawOrigin;
use sp_std::convert::TryFrom;
use subsocial_support::Content;
use sp_runtime::{
    traits::{Bounded, StaticLookup},
    FixedI64, FixedPointNumber,
};

use super::*;

benchmarks! {

    link_post_to_resource {
        let account: T::AccountId = account("account", 24, 0);
        let res_id = ResourceId::<T>::try_from(b"test".to_vec()).unwrap();

        let space_id = pallet_spaces::NextSpaceId::<T>::get();
        let post_id = pallet_posts::NextPostId::<T>::get();

        ensure!(pallet_spaces::Pallet::<T>::create_space(
            RawOrigin::Signed(account.clone()).into(),
            Content::None,
            None,
        ).is_ok(), "Space didn't get created");

        ensure!(pallet_posts::Pallet::<T>::create_post(
            RawOrigin::Signed(account.clone()).into(),
            Some(space_id),
            pallet_posts::PostExtension::RegularPost,
            Content::None
        ).is_ok(), "Post didn't get created");

    }: _(RawOrigin::Signed(account.clone()), res_id.clone(), post_id)
    verify {
        ensure!(ResourceDiscussion::<T>::get(res_id.clone(), account) == Some(post_id), "resource isn't linked");
    }

    create_resource_discussion {
        let account: T::AccountId = account("account", 24, 0);
        let res_id = ResourceId::<T>::try_from(b"test".to_vec()).unwrap();

        let space_id = pallet_spaces::NextSpaceId::<T>::get();
        let post_id = pallet_posts::NextPostId::<T>::get();

        ensure!(pallet_spaces::Pallet::<T>::create_space(
            RawOrigin::Signed(account.clone()).into(),
            Content::None,
            None,
        ).is_ok(), "Space didn't get created");
    }: _(RawOrigin::Signed(account.clone()), res_id.clone(), space_id, Content::None)
    verify {
        ensure!(ResourceDiscussion::<T>::get(res_id.clone(), account) == Some(post_id), "resource isn't linked");
    }

     impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}