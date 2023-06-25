// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks, Zero};
use frame_support::ensure;
use frame_system::RawOrigin;
use pallet_proxy::ProxyDefinition;
use sp_runtime::traits::StaticLookup;

use super::*;

benchmarks! {
    add_free_proxy {
        let delegator: T::AccountId = account("delegator", 24, 0);
        let proxy: T::AccountId = account("proxy", 65, 0);
        let proxy_lookup = <<T as frame_system::Config>::Lookup as StaticLookup>::unlookup(proxy.clone());
    }: _(RawOrigin::Signed(delegator.clone()), proxy_lookup.clone(), Default::default(), 0u32.into())
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
            "proxy definition is not correct",
        );
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}
