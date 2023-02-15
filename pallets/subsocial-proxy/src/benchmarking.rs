#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks, Zero};
use frame_support::ensure;
use frame_system::RawOrigin;
use pallet_proxy::ProxyDefinition;

use super::*;

benchmarks! {
    add_free_proxy {
        let delegator: T::AccountId = account("delegator", 24, 0);
        let proxy: T::AccountId = account("proxy", 65, 0);
    }: _(RawOrigin::Signed(delegator.clone()), proxy.clone(), Default::default(), 0u32.into())
    verify {
        let (proxies, deposits) = pallet_proxy::Proxies::<T>::get(&delegator);
        ensure!(deposits.is_zero(), "deposits should be zero");
        ensure!(proxies.len() == 1, "only one proxy should be found");
        let first_proxy = &proxies[0];
        ensure!(
            first_proxy == &ProxyDefinition {
                delegate: proxy,
                proxy_type: Default::default(),
                delay: 0u32.into(),
            },
            "deposits should be zero",
        );
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}
