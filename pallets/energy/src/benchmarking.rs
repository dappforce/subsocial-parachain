#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::account;
use frame_benchmarking::benchmarks;
use frame_support::traits::Currency;
use frame_support::traits::EnsureOrigin;
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, StaticLookup};
use sp_runtime::FixedI64;
use sp_runtime::FixedPointNumber;

use super::*;

benchmarks! {
    update_value_coefficient {
        let ratio = FixedI64::checked_from_rational(2_65, 100).unwrap();
        let origin = T::UpdateOrigin::successful_origin();
    }: _<T::Origin>(origin, ratio)
    verify {
        let stored_ratio = ValueCoefficient::<T>::get();
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
        let conversion_ratio = ValueCoefficient::<T>::get();
        let energy = burn_amount;
        assert_eq!(Pallet::<T>::energy_balance(&receiver), energy);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().value_coefficient(1.5).update_origin(1).build(),
        crate::mock::Test,
    );
}
