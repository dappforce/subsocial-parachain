#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    ensure, assert_ok,
    dispatch::{DispatchError, DispatchErrorWithPostInfo},
    traits::{Currency, Get},
};
use frame_system::RawOrigin;
use sp_runtime::FixedI64;
use frame_support::traits::EnsureOrigin;
use frame_benchmarking::account;
use sp_runtime::FixedPointNumber;
use sp_runtime::traits::{StaticLookup, Bounded};
use super::FixedFromFloat;

benchmarks! {
    update_conversion_ratio {
        let ratio = FixedI64::from_f64(2.65);
        let origin = T::UpdateOrigin::successful_origin();
    }: _<T::Origin>(origin, ratio)
    verify {
        let stored_ratio = ConversionRatio::<T>::get();
        assert_eq!(ratio, stored_ratio);
    }

    generate_energy {
        let generator: T::AccountId = account("generator", 24, 0);
        let generator_balance = BalanceOf::<T>::max_value();
        <T as Config>::Currency::make_free_balance_be(&generator, generator_balance.clone());
        let receiver: T::AccountId = account("receiver", 36, 0);
        let burn_amount = 700_000u32.into();
    }: _(RawOrigin::Signed(generator.clone()), T::Lookup::unlookup(receiver.clone()), burn_amount)
    verify {
        let conversion_ratio = ConversionRatio::<T>::get();
        let energy = conversion_ratio.checked_mul_int(burn_amount).unwrap();
        assert_eq!(Pallet::<T>::energy_balance(&receiver), energy);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().conversion_ratio(1.5).update_origin(1).build(),
        crate::mock::Test,
    );
}