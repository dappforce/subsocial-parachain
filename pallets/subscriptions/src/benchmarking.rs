use codec::Decode;
use frame_benchmarking::{account, benchmarks};
use frame_support::{
    ensure,
    sp_runtime::traits::{Bounded, Saturating},
    traits::{Currency, Get},
};
use frame_system::RawOrigin;
use sp_std::vec;

use pallet_permissions::SpacePermission;
use subsocial_support::{
    mock_functions::valid_content_ipfs,
    traits::{RolesInterface, SpacesInterface},
    Content,
};

use crate::types::{SubscriberInfo, SubscriptionSettings};

use super::*;

fn ed_multiplied_by<T: pallet_balances::Config + Config>(multiplier: u32) -> BalanceOf<T>
where
    BalanceOf<T>: From<<T as pallet_balances::Config>::Balance>,
{
    T::ExistentialDeposit::get().saturating_mul(multiplier.into()).into()
}

benchmarks! {
    where_clause {  where
        T: pallet_balances::Config,
        BalanceOf<T>: From<<T as pallet_balances::Config>::Balance>,
    }

    update_subscription_settings {
        let owner = T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap();
        let space_id = T::SpacesInterface::create_space(&owner, valid_content_ipfs())?;
        let role_id = T::RolesInterface::create_role(
            &owner,
            space_id,
            None,
            Content::None,
            vec![
                SpacePermission::CreatePosts,
                SpacePermission::UpdateOwnPosts,
                SpacePermission::UpdateAnyPost,
                SpacePermission::UpdateEntityStatus,
            ],
        )?;

        let settings = SubscriptionSettings::<BalanceOf<T>, T::RoleId> {
            subscription: ed_multiplied_by::<T>(10),
            disabled: false,
            role_id: role_id,
        };
    }: _(RawOrigin::Signed(owner.clone()), space_id, settings.clone())
    verify {
        let maybe_settings = SubscriptionSettingsBySpace::<T>::get(space_id);
        ensure!(maybe_settings == Some(settings), "Settings isn't updated");
    }

    subscribe {
        let owner: T::AccountId = account("owner", 24, 0);
        let space_id = T::SpacesInterface::create_space(&owner, valid_content_ipfs())?;
        let role_id = T::RolesInterface::create_role(
            &owner,
            space_id,
            None,
            Content::None,
            vec![
                SpacePermission::CreatePosts,
                SpacePermission::UpdateOwnPosts,
                SpacePermission::UpdateAnyPost,
                SpacePermission::UpdateEntityStatus,
            ],
        )?;

        SubscriptionSettingsBySpace::<T>::insert(space_id, SubscriptionSettings::<BalanceOf<T>, T::RoleId> {
            subscription: ed_multiplied_by::<T>(15),
            disabled: false,
            role_id: role_id,
        });

        let subscriber: T::AccountId = account("subscriber", 24, 0);
        <T as Config>::Currency::make_free_balance_be(&subscriber, BalanceOf::<T>::max_value());
    }: _(RawOrigin::Signed(subscriber.clone()), space_id)
    verify {
        let info = SpaceSubscribers::<T>::get(space_id, subscriber).ok_or("Info wasn't set")?;
        ensure!(info.subscription == ed_multiplied_by::<T>(15), "subscription didn't match");
        ensure!(info.granted_role_id == role_id, "granted_role_id didn't match");
        ensure!(!info.unsubscribed, "account should be subscribed");
    }


    unsubscribe {
        let subscriber: T::AccountId = account("subscriber", 24, 0);
        let space_id = T::SpaceId::default();
        let role_id = T::RoleId::default();

        SpaceSubscribers::<T>::insert(space_id.clone(), subscriber.clone(), SubscriberInfo::<BalanceOf<T>, T::RoleId, T::BlockNumber> {
            subscribed_on: <frame_system::Pallet<T>>::block_number(),
            subscription: ed_multiplied_by::<T>(3),
            granted_role_id: role_id,
            unsubscribed: false,
        });
    }: _(RawOrigin::Signed(subscriber.clone()), space_id.clone())
    verify {
        let info = SpaceSubscribers::<T>::get(space_id, subscriber).ok_or("Info wasn't set")?;
        ensure!(info.unsubscribed, "account should be unsubscribed");
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Test,
    );
}
