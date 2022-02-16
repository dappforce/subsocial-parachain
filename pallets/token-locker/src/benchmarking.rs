//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as TokenLocker;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{dispatch::DispatchResult, traits::{Currency, Get}};
use frame_system::{RawOrigin, Pallet as System};
use sp_runtime::traits::{Bounded, StaticLookup};

fn get_caller_with_balance<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

	caller
}

fn _lock_sub<T: Config>(caller: T::AccountId) -> DispatchResult {
	let target_lookup = T::Lookup::unlookup(caller.clone());
	let amount = T::MaxLockAmount::get();
	TokenLocker::<T>::lock_sub(RawOrigin::Signed(caller).into(), amount, target_lookup)
}

benchmarks! {
	lock_sub {
		let caller = get_caller_with_balance::<T>();

		let target_lookup = T::Lookup::unlookup(caller.clone());
		let amount = T::MaxLockAmount::get();
	}: _(RawOrigin::Signed(caller.clone()), amount, target_lookup)
	verify {
		assert!(LockDetails::<T>::get(&caller).is_some());
	}

	request_unlock {
		let caller = get_caller_with_balance::<T>();
		_lock_sub::<T>(caller.clone())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(UnlockAt::<T>::get(&caller).is_some());
	}

	try_refund {
		let caller = get_caller_with_balance::<T>();
		_lock_sub::<T>(caller.clone())?;
		TokenLocker::<T>::request_unlock(RawOrigin::Signed(caller.clone()).into())?;

		System::<T>::set_block_number(System::<T>::block_number() + T::UnlockPeriod::get());
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(LockDetails::<T>::get(&caller).is_none());
		assert!(UnlockAt::<T>::get(&caller).is_none());
	}

	impl_benchmark_test_suite!(TokenLocker, crate::mock::new_test_ext(), crate::mock::Test,);
}
