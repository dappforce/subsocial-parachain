// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

use super::*;

use crate::Pallet as Profiles;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::dispatch::DispatchError;
use frame_system::RawOrigin;
use subsocial_support::{traits::SpacesProvider, Content, SpaceId};

fn create_space<T: Config>(
    owner: &T::AccountId,
    content: Content,
) -> Result<SpaceId, DispatchError> {
    let space_id = T::SpacesProvider::create_space(owner, content)?;
    Ok(space_id)
}

benchmarks! {
    set_profile {
        let caller: T::AccountId = whitelisted_caller();
        let space_id = create_space::<T>(&caller, Content::default())?;
    }: _(RawOrigin::Signed(caller.clone()), space_id)
    verify {
        assert_eq!(Profiles::<T>::profile_space_id_by_account(&caller), Some(space_id));
    }

    reset_profile {
        let caller: T::AccountId = whitelisted_caller();
        let space_id = create_space::<T>(&caller, Content::default())?;
        Profiles::<T>::set_profile(RawOrigin::Signed(caller.clone()).into(), space_id)?;
    }: _(RawOrigin::Signed(caller.clone()))
    verify {
        assert_eq!(Profiles::<T>::profile_space_id_by_account(&caller), None);
    }

    create_space_as_profile {
        let caller: T::AccountId = whitelisted_caller();
        let content = Content::default();
    }: _(RawOrigin::Signed(caller.clone()), content)
    verify {
        assert!(Profiles::<T>::profile_space_id_by_account(&caller).is_some());
    }

    // impl_benchmark_test_suite!(Profiles, crate::mock::ExtBuilder::build(), crate::mock::Test);
}
