#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks};
use frame_support::ensure;
use frame_system::RawOrigin;

use crate as pallet_account_follows;

use super::*;

benchmarks! {
    follow_account {
        let follower: T::AccountId = account("follower", 0, 0);
        let followee: T::AccountId = account("followee", 1, 1);

        let follower_origin = RawOrigin::Signed(follower.clone());

    }: _(follower_origin, followee.clone())
    verify {
        ensure!(
            AccountsFollowedByAccount::<T>::get(follower.clone()).contains(&followee),
            "AccountsFollowedByAccount didn't get updated",
        );
        ensure!(
            AccountFollowers::<T>::get(followee.clone()).contains(&follower),
            "AccountFollowers didn't get updated",
        );
        ensure!(
            AccountFollowedByAccount::<T>::get((follower.clone(), followee.clone())),
            "AccountFollowedByAccount didn't get updated",
        );
    }

    unfollow_account {
        let follower: T::AccountId = account("follower", 0, 0);
        let followee: T::AccountId = account("followee", 1, 1);

        let follower_origin = RawOrigin::Signed(follower.clone());

        ensure!(pallet_account_follows::Pallet::<T>::follow_account(
            follower_origin.clone().into(),
            followee.clone(),
        ).is_ok(), "follow_account call returned an err");

    }: _(follower_origin, followee.clone())
    verify {
        ensure!(
            !AccountsFollowedByAccount::<T>::get(follower.clone()).contains(&followee),
            "AccountsFollowedByAccount didn't get updated",
        );
        ensure!(
            !AccountFollowers::<T>::get(followee.clone()).contains(&follower),
            "AccountFollowers didn't get updated",
        );
        ensure!(
            !AccountFollowedByAccount::<T>::get((follower.clone(), followee.clone())),
            "AccountFollowedByAccount didn't get updated",
        );
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::build(),
        crate::mock::TestRuntime,
    );
}
