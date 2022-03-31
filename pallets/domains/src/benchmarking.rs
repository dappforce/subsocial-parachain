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
use pallet_parachain_utils::Content;

use pallet_parachain_utils::mock_functions::valid_content_ipfs;

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

fn create_domain_names<T: Config>(num: usize) -> Vec<DomainName<T>> {
	let mut domains = Vec::new();

	let max_domain_length = T::MaxDomainLength::get() as usize - TOP_LEVEL_DOMAIN.len() - 1;
	let mut domain = mock_domain::<T>();

	for i in 0..num {
		let idx = i % max_domain_length;

		let next_char = (domain[idx] + 1).clamp(65, 90);
		let _ = sp_std::mem::replace(&mut domain[idx], next_char);
		domains.push(domain.clone());
	}

	assert_eq!(num, domains.len());

	domains
}

fn mock_domain<T: Config>() -> DomainName<T> {
	let tld = &mut TOP_LEVEL_DOMAIN.to_vec();
	let mut domain_vec = vec![b'A'; T::MaxDomainLength::get() as usize - tld.len() - 1];

	domain_vec.push(b'.');
	domain_vec.append(tld);
	domain_vec.try_into().expect("domain exceeds max length")
}

fn add_domain<T: Config>(
	owner: <T::Lookup as StaticLookup>::Source,
) -> Result<DomainName<T>, DispatchErrorWithPostInfo> {
	let domain = mock_domain::<T>();

	let expires_in = T::RegistrationPeriodLimit::get();

	Pallet::<T>::force_register_domain(
		RawOrigin::Root.into(), owner, domain.clone(), valid_content_ipfs(), expires_in,
	)?;

	Ok(domain)
}

// TODO: replace with mock function when merged with other benchmarks.
fn valid_content_ipfs_2() -> Content {
	let mut new_content = Vec::new();
	if let Content::IPFS(mut content) = valid_content_ipfs() {
		content.swap_remove(0);
		content.push(b'a');
		new_content = content;
	}

	Content::IPFS(new_content)
}

benchmarks! {
	register_domain {
		let owner = account_with_balance::<T>();

		let full_domain = mock_domain::<T>();

		let expires_in = T::RegistrationPeriodLimit::get();
		let price = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Signed(owner), full_domain.clone(), valid_content_ipfs(), expires_in)
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(full_domain);
		ensure!(RegisteredDomains::<T>::get(&domain_lc).is_some(), "Domain was not purchased");
	}

	force_register_domain {
		let account_with_balance = account_with_balance::<T>();
		let owner = lookup_source_from_account::<T>(&account_with_balance);

		let full_domain = mock_domain::<T>();

		let expires_in = T::RegistrationPeriodLimit::get();
		let price = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Root, owner, full_domain.clone(), valid_content_ipfs(), expires_in)
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(full_domain);
		ensure!(RegisteredDomains::<T>::get(&domain_lc).is_some(), "Domain was not purchased");
	}

	set_inner_value {
		let owner = account_with_balance::<T>();
		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let value = Some(InnerValue::Account(owner.clone()));
	}: _(RawOrigin::Signed(owner), full_domain.clone(), value.clone())
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(full_domain);
		let DomainMeta { inner_value, .. } = RegisteredDomains::<T>::get(&domain_lc).unwrap();
		ensure!(value == inner_value, "Inner value was not updated.")
	}

	set_outer_value {
		let owner = account_with_balance::<T>();
		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let value = Some(
			vec![b'A'; T::MaxOuterValueLength::get() as usize]
				.try_into()
				.expect("outer value out of bounds")
		);

	}: _(RawOrigin::Signed(owner), full_domain.clone(), value.clone())
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(full_domain);
		let DomainMeta { outer_value, .. } = RegisteredDomains::<T>::get(&domain_lc).unwrap();
		ensure!(value == outer_value, "Outer value was not updated.")
	}

	set_domain_content {
		let owner = account_with_balance::<T>();

		let full_domain = add_domain::<T>(lookup_source_from_account::<T>(&owner))?;

		let new_content = valid_content_ipfs_2();
	}: _(RawOrigin::Signed(owner), full_domain.clone(), new_content.clone())
	verify {
		let domain_lc = Pallet::<T>::lower_domain_then_bound(full_domain);
		let DomainMeta { content, .. } = RegisteredDomains::<T>::get(&domain_lc).unwrap();
		ensure!(new_content == content, "Content was not updated.")
	}

	reserve_domains {
		let s in 1 .. T::DomainsInsertLimit::get() => ();
		let domains = create_domain_names::<T>(s as usize);
	}: _(RawOrigin::Root, domains)
	verify {
		ensure!(ReservedDomains::<T>::iter().count() as u32 == s, "Domains were not reserved.");
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::ExtBuilder::default().build(), crate::mock::Test);
}
