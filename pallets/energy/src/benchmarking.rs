// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::{Currency, EnsureOrigin, Get};
use frame_system::RawOrigin;
use sp_runtime::{
    traits::{Bounded, StaticLookup},
    FixedI64, FixedPointNumber,
};

use super::*;

benchmarks! {
    update_value_coefficient {
        let origin = T::UpdateOrigin::successful_origin();
        let coefficient = FixedI64::checked_from_rational(2_65, 100).unwrap();
    }: _<T::RuntimeOrigin>(origin, coefficient)
    verify {
        let stored_coefficient = ValueCoefficient::<T>::get();
        assert_eq!(coefficient, stored_coefficient);
    }

    generate_energy {
        let generator: T::AccountId = account("generator", 24, 0);
        let generator_balance = BalanceOf::<T>::max_value();
        <T as Config>::Currency::make_free_balance_be(&generator, generator_balance);
        let receiver: T::AccountId = account("receiver", 36, 0);

        // The minimum amount of energy that can be generated is T::ExistentialDeposit
        let burn_amount = T::ExistentialDeposit::get();
    }: _(RawOrigin::Signed(generator.clone()), T::Lookup::unlookup(receiver.clone()), burn_amount)
    verify {
        let energy = burn_amount;
        assert_eq!(Pallet::<T>::energy_balance(&receiver), energy);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().value_coefficient(1.5).update_origin(1).build(),
        crate::mock::Test,
    );
}
