//! Benchmarking for pallet-domains

use super::*;
use types::*;

use crate::Pallet as Pallet;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{
	ensure,
	dispatch::DispatchErrorWithPostInfo,
	traits::{Currency, Get},
};
use frame_system::RawOrigin;

use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::{convert::TryInto, vec, vec::Vec};

use pallet_parachain_utils::mock_functions::{another_valid_content_ipfs, valid_content_ipfs};

fn account_with_balance<T: Config>() -> T::AccountId {
	let owner: T::AccountId = whitelisted_caller();
	<T as Config>::Currency::make_free_balance_be(&owner, BalanceOf::<T>::max_value());

	owner
}

fn lookup_source_from_account<T: Config>(
	account: &T::AccountId,
) -> <T::Lookup as StaticLookup>::Source {
	T::Lookup::unlookup(account.clone())
}

fn mock_words_array<T: Config>(length: usize) -> Vec<DomainName<T>> {
	let mut words = Vec::new();

	let max_domain_length = T::MaxDomainLength::get() as usize;
	let mut word: DomainName<T> = mock_word::<T>(T::MaxDomainLength::get() as usize);

	for i in 0..length {
		let idx = i % max_domain_length;

		let next_char = (word[idx] + 1).clamp(65, 90);
		let _ = sp_std::mem::replace(&mut word[idx], next_char);
		words.push(word.clone());
	}

	assert_eq!(length, words.len());

	words
}

fn mock_tld<T: Config>() -> DomainName<T> {
	b"tld".to_vec().try_into().expect("qed; domain exceeds max length")
}

fn add_default_tld<T: Config>() -> Result<DomainName<T>, DispatchErrorWithPostInfo> {
	let tld = mock_tld::<T>();
	Pallet::<T>::add_tld(RawOrigin::Root.into(), vec![tld.clone()])?;
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

fn add_domain<T: Config>(
	owner: <T::Lookup as StaticLookup>::Source,
) -> Result<DomainName<T>, DispatchErrorWithPostInfo> {
	add_default_tld::<T>()?;
	let domain = mock_domain::<T>();

	let expires_in = T::RegistrationPeriodLimit::get();

	Pallet::<T>::force_register_domain(
		RawOrigin::Root.into(), owner, domain.clone(), valid_content_ipfs(), expires_in,
	)?;

	Ok(domain)
}

benchmarks! {
	register_domain {
		add_default_tld::<T>()?;

		let owner = account_with_balance::<T>();

		let full_domain = mock_domain::<T>();

		let expires_in = T::RegistrationPeriodLimit::get();
		let price = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Signed(owner), full_domain.clone(), valid_content_ipfs(), expires_in)
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(&full_domain);
		ensure!(RegisteredDomains::<T>::get(&domain_lc).is_some(), "Domain was not purchased");
	}

	force_register_domain {
		add_default_tld::<T>()?;

		let account_with_balance = account_with_balance::<T>();
		let owner = lookup_source_from_account::<T>(&account_with_balance);

		let full_domain = mock_domain::<T>();

		let expires_in = T::RegistrationPeriodLimit::get();
		let price = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Root, owner, full_domain.clone(), valid_content_ipfs(), expires_in)
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(&full_domain);
		ensure!(RegisteredDomains::<T>::get(&domain_lc).is_some(), "Domain was not purchased");
	}

	set_inner_value {
		let owner = account_with_balance::<T>();
		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let value = Some(InnerValue::Account(owner.clone()));
	}: _(RawOrigin::Signed(owner), full_domain.clone(), value.clone())
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(&full_domain);
		let DomainMeta { inner_value, .. } = RegisteredDomains::<T>::get(&domain_lc).unwrap();
		ensure!(value == inner_value, "Inner value was not updated.")
	}

	set_outer_value {
		let owner = account_with_balance::<T>();
		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let value = Some(
			vec![b'A'; T::MaxOuterValueLength::get() as usize]
				.try_into()
				.expect("qed; outer value exceeds max length")
		);

	}: _(RawOrigin::Signed(owner), full_domain.clone(), value.clone())
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(&full_domain);
		let DomainMeta { outer_value, .. } = RegisteredDomains::<T>::get(&domain_lc).unwrap();
		ensure!(value == outer_value, "Outer value was not updated.")
	}

	set_domain_content {
		let owner = account_with_balance::<T>();

		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let new_content = another_valid_content_ipfs();
	}: _(RawOrigin::Signed(owner), full_domain.clone(), new_content.clone())
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(&full_domain);
		let DomainMeta { content, .. } = RegisteredDomains::<T>::get(&domain_lc).unwrap();
		ensure!(new_content == content, "Content was not updated.")
	}

	reserve_words {
		let s in 1 .. T::DomainsInsertLimit::get() => ();
		let words = mock_words_array::<T>(s as usize);
	}: _(RawOrigin::Root, words)
	verify {
		ensure!(ReservedWords::<T>::iter().count() as u32 == s, "Domains were not reserved.");
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::ExtBuilder::default().build(), crate::mock::Test);
}
