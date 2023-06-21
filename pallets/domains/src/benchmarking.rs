//! Benchmarking for pallet-domains

use super::*;
use types::*;

use crate::Pallet as Pallet;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{
	ensure, assert_ok,
	dispatch::DispatchErrorWithPostInfo,
	traits::{Currency, Get},
};
use frame_system::RawOrigin;

use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::{convert::TryInto, vec};

fn account_with_balance<T: Config>() -> T::AccountId {
	let owner: T::AccountId = whitelisted_caller();
	<T as Config>::Currency::make_free_balance_be(&owner, BalanceOf::<T>::max_value());

	owner
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
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
	Pallet::<T>::bound_domain(b"sub".to_vec())
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

fn add_domain<T: Config>(owner: &T::AccountId) -> Result<DomainName<T>, DispatchErrorWithPostInfo> {
	add_default_tld::<T>().map_err(|e| e.error)?;
	let domain = mock_domain::<T>();
	let expires_in = T::RegistrationPeriodLimit::get();
	let owner_lookup = lookup_source_from_account::<T>(owner);

	Pallet::<T>::force_register_domain(
		RawOrigin::Root.into(), owner_lookup, domain.clone(), expires_in,
	)?;

	Ok(domain)
}

benchmarks! {
	register_domain {
		add_default_tld::<T>()?;

		let who = account_with_balance::<T>();
		let domain = mock_domain::<T>();

		let expires_in = T::RegistrationPeriodLimit::get();
		let price = BalanceOf::<T>::max_value();

	}: _(RawOrigin::Signed(who.clone()), domain.clone(), expires_in)
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

	}: _(RawOrigin::Root, owner_lookup, domain.clone(), expires_in)
	verify {
		assert_last_event::<T>(
			Event::DomainRegistered { who, domain }.into()
		);
	}

    set_record {
        let who = account_with_balance::<T>();
        let owner_origin = RawOrigin::Signed(who.clone());

        let full_domain = add_domain::<T>(&who)?;

        let key: DomainRecordKey<T> = b"key".to_vec().try_into().unwrap();
        let value: DomainRecordValue<T> = b"value".to_vec().try_into().unwrap();
    }: _(owner_origin, full_domain.clone(), key.clone(), Some(value.clone()))
    verify {
        let full_domain = Pallet::<T>::lower_domain_then_bound(&full_domain);
        assert_last_event::<T>(
            Event::DomainRecordUpdated {
				account: who.clone(),
				domain: full_domain.clone(),
				key: key.clone(),
				value: Some(value.clone()),
			}.into(),
        );
        let found_value = RecordsByDomain::<T>::get(full_domain, key).map(|val_with_deposit| val_with_deposit.record_value);
        assert_eq!(found_value, Some(value.clone()));
        ensure!(found_value == Some(value), "Value isn't correct");
    }

    force_set_record {
        let who = account_with_balance::<T>();

        let full_domain = add_domain::<T>(&who)?;

        let key: DomainRecordKey<T> = b"key".to_vec().try_into().unwrap();
        let value: DomainRecordValue<T> = b"value".to_vec().try_into().unwrap();
    }: _(RawOrigin::Root, full_domain.clone(), key.clone(), Some(value.clone()))
    verify {
        let full_domain = Pallet::<T>::lower_domain_then_bound(&full_domain);
        assert_last_event::<T>(
            Event::DomainRecordUpdated {
				account: who.clone(),
				domain: full_domain.clone(),
				key: key.clone(),
				value: Some(value.clone()),
			}.into(),
        );
        let found_value = RecordsByDomain::<T>::get(full_domain, key).map(|val_with_deposit| val_with_deposit.record_value);
        assert_eq!(found_value, Some(value.clone()));
        ensure!(found_value == Some(value), "Value isn't correct");
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
