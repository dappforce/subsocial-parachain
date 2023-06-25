// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub use crate::weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::RawOrigin, pallet_prelude::*, traits::Currency, transactional};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{StaticLookup, Zero};

    use crate::weights::WeightInfo;

    type BalanceOf<T> = <<T as pallet_proxy::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_proxy::Config {
        type ProxyDepositBase: Get<BalanceOf<Self>>;

        type ProxyDepositFactor: Get<BalanceOf<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        OnlyFirstProxyCanBeFree,
    }

    #[pallet::storage]
    #[pallet::getter(fn is_free_proxy)]
    pub type CanAddFreeProxy<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(< T as Config >::WeightInfo::add_free_proxy())]
        #[transactional]
        pub fn add_free_proxy(
            origin: OriginFor<T>,
            delegate: AccountIdLookupOf<T>,
            proxy_type: T::ProxyType,
            delay: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let proxy_count = pallet_proxy::Proxies::<T>::get(&who).0.len();

            ensure!(proxy_count == 0, Error::<T>::OnlyFirstProxyCanBeFree);

            CanAddFreeProxy::<T>::set(true);

            let add_proxy_res = pallet_proxy::Pallet::<T>::add_proxy(
                RawOrigin::Signed(who).into(),
                delegate,
                proxy_type,
                delay,
            );

            CanAddFreeProxy::<T>::kill();

            add_proxy_res
        }
    }

    pub struct AdjustedProxyDepositBase<T>(PhantomData<T>);
    impl<T: Config> Get<BalanceOf<T>> for AdjustedProxyDepositBase<T> {
        fn get() -> BalanceOf<T> {
            if CanAddFreeProxy::<T>::get() {
                Zero::zero()
            } else {
                <T as Config>::ProxyDepositBase::get()
            }
        }
    }

    pub struct AdjustedProxyDepositFactor<T>(PhantomData<T>);
    impl<T: Config> Get<BalanceOf<T>> for AdjustedProxyDepositFactor<T> {
        fn get() -> BalanceOf<T> {
            if CanAddFreeProxy::<T>::get() {
                Zero::zero()
            } else {
                <T as Config>::ProxyDepositFactor::get()
            }
        }
    }
}
