//! Benchmarking for pallet-domains

use super::*;
use types::*;

use crate::Pallet as Pallet;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{
	ensure, assert_ok,
	dispatch::{DispatchError, DispatchErrorWithPostInfo},
	traits::{Currency, Get},
};
use frame_system::RawOrigin;

use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::{convert::TryInto, vec};

use pallet_parachain_utils::mock_functions::{another_valid_content_ipfs, valid_content_ipfs};

fn account_with_balance<T: Config>() -> T::AccountId {
	let owner: T::AccountId = whitelisted_caller();
	<T as Config>::Currency::make_free_balance_be(&owner, BalanceOf::<T>::max_value());

	owner
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn lookup_source_from_account<T: Config>(
	account: &T::AccountId,
) -> <T::Lookup as StaticLookup>::Source {
	T::Lookup::unlookup(account.clone())
}

fn mock_bounded_string_array<T: Config>(length: usize) -> BoundedDomainsVec<T> {
	let mut words = BoundedDomainsVec::<T>::default();

	let max_domain_length = T::MaxDomainLength::get() as usize;
	let mut word: DomainName<T> = mock_word::<T>(T::MaxDomainLength::get() as usize);

	for i in 0..length {
		let idx = i % max_domain_length;

		let next_char = (word[idx] + 1).clamp(65, 90);
		let _ = sp_std::mem::replace(&mut word[idx], next_char);
		assert_ok!(words.try_push(word.clone()));
	}

	assert_eq!(length, words.len());

	words
}

fn mock_tld<T: Config>() -> DomainName<T> {
	Pallet::<T>::bound_domain(b"tld".to_vec())
}

fn add_default_tld<T: Config>() -> Result<DomainName<T>, DispatchErrorWithPostInfo> {
	let tld = mock_tld::<T>();
	Pallet::<T>::support_tlds(
		RawOrigin::Root.into(),
		vec![tld.clone()].try_into().expect("qed; domains vector exceeds the limit"),
	)?;
	Ok(tld)
}

fn mock_word<T: Config>(length: usize) -> DomainName<T> {
	vec![b'A'; length].try_into().expect("qed; word exceeds max domain length")
}

fn mock_domain<T: Config>() -> DomainName<T> {
	let tld = &mut mock_tld::<T>().to_vec();
	let domain_name = mock_word::<T>(T::MaxDomainLength::get() as usize - tld.len() - 1);

	domain_name.try_mutate(|vec| {
		vec.push(b'.');
		vec.append(tld);
	}).unwrap()
}

fn add_domain<T: Config>(owner: &T::AccountId) -> Result<DomainName<T>, DispatchError> {
	add_default_tld::<T>().map_err(|e| e.error)?;
	let domain = mock_domain::<T>();
	let expires_in = T::RegistrationPeriodLimit::get();
	let owner_lookup = lookup_source_from_account::<T>(owner);

	Pallet::<T>::force_register_domain(
		RawOrigin::Root.into(), owner_lookup, domain.clone(), valid_content_ipfs(), expires_in,
	)?;

	Ok(domain)
}

fn inner_value_owner_account<T: Config>(account: T::AccountId) -> Option<InnerValueOf<T>> {
	Some(InnerValue::Account(account))
}

 fn inner_value_space_id<T: Config>() -> Option<InnerValueOf<T>> {
	Some(InnerValue::Space(1))
}

benchmarks! {
	register_domain {
		add_default_tld::<T>()?;

		let who = account_with_balance::<T>();
		let domain = mock_domain::<T>();

		let expires_in = T::RegistrationPeriodLimit::get();
		let price = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Signed(who.clone()), domain.clone(), valid_content_ipfs(), expires_in)
	verify {
		assert_last_event::<T>(
			Event::DomainRegistered { who, domain }.into()
		);
	}

	force_register_domain {
		add_default_tld::<T>()?;

		let who = account_with_balance::<T>();
		let owner_lookup = lookup_source_from_account::<T>(&who);

		let domain = mock_domain::<T>();

		let expires_in = T::RegistrationPeriodLimit::get();
		let price = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Root, owner_lookup, domain.clone(), valid_content_ipfs(), expires_in)
	verify {
		assert_last_event::<T>(
			Event::DomainRegistered { who, domain }.into()
		);
	}

	set_inner_value {
		let who = account_with_balance::<T>();
		let owner_origin = RawOrigin::Signed(who.clone());

		let full_domain = add_domain::<T>(&who)?;

		let initial_value = inner_value_owner_account::<T>(who);
		Pallet::<T>::set_inner_value(
			owner_origin.clone().into(), full_domain.clone(), initial_value
		)?;

		let updated_value = inner_value_space_id::<T>();

	}: _(owner_origin, full_domain.clone(), updated_value.clone())
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(&full_domain);
		let DomainMeta { inner_value, .. } = RegisteredDomains::<T>::get(&domain_lc).unwrap();
		ensure!(updated_value == inner_value, "Inner value was not updated")
	}

	force_set_inner_value {
		let who = account_with_balance::<T>();
		let owner_origin = RawOrigin::Signed(who.clone());

		let full_domain = add_domain::<T>(&who)?;

		let initial_value = inner_value_owner_account::<T>(who);
		Pallet::<T>::set_inner_value(owner_origin.into(), full_domain.clone(), initial_value)?;

		let updated_value = inner_value_space_id::<T>();

	}: _(RawOrigin::Root, full_domain.clone(), updated_value.clone())
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(&full_domain);
		let DomainMeta { inner_value, .. } = RegisteredDomains::<T>::get(&domain_lc).unwrap();
		ensure!(updated_value == inner_value, "Inner value was not updated")
	}

	set_outer_value {
		let who = account_with_balance::<T>();
		let domain = add_domain::<T>(&who)?;

		let value = Some(
			vec![b'A'; T::MaxOuterValueLength::get() as usize]
				.try_into()
				.expect("qed; outer value exceeds max length")
		);

	}: _(RawOrigin::Signed(who.clone()), domain.clone(), value)
	verify {
		assert_last_event::<T>(Event::DomainMetaUpdated { who, domain }.into());
	}

	set_domain_content {
		let who = account_with_balance::<T>();
		let domain = add_domain::<T>(&who)?;
		let new_content = another_valid_content_ipfs();
	}: _(RawOrigin::Signed(who.clone()), domain.clone(), new_content)
	verify {
		assert_last_event::<T>(Event::DomainMetaUpdated { who, domain }.into());
	}

	reserve_words {
		let s in 1 .. T::DomainsInsertLimit::get() => ();
		let words = mock_bounded_string_array::<T>(s as usize);
	}: _(RawOrigin::Root, words)
	verify {
		assert_last_event::<T>(Event::NewWordsReserved { count: s }.into());
	}

	support_tlds {
		let s in 1 .. T::DomainsInsertLimit::get() => ();
		let tlds = mock_bounded_string_array::<T>(s as usize);
	}: _(RawOrigin::Root, tlds)
	verify {
		assert_last_event::<T>(Event::NewTldsSupported { count: s }.into());
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::ExtBuilder::default().build(), crate::mock::Test);
}
