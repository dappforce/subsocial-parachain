// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
// pub mod rpc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use crate::weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use pallet_permissions::SpacePermissions;
    use subsocial_support::{
        traits::{ProfileManager, SpacePermissionsProvider, SpacesInterface},
        Content, SpaceId, SpacePermissionsInfo,
    };

    type SpacePermissionsInfoOf<T> =
        SpacePermissionsInfo<<T as frame_system::Config>::AccountId, SpacePermissions>;

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type SpacePermissionsProvider: SpacePermissionsProvider<
            Self::AccountId,
            SpacePermissionsInfoOf<Self>,
        >;

        type SpacesInterface: SpacesInterface<Self::AccountId, SpaceId>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn profile_space_id_by_account)]
    pub type ProfileSpaceIdByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, SpaceId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Profile's space id was updated for this account.
        ProfileUpdated { account: T::AccountId, space_id: Option<SpaceId> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// There is no space set as profile.
        NoSpaceSetAsProfile,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::set_profile())]
        pub fn set_profile(origin: OriginFor<T>, space_id: SpaceId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::do_set_profile(&sender, space_id)?;
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::reset_profile())]
        pub fn reset_profile(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::profile_space_id_by_account(&sender).is_some(),
                Error::<T>::NoSpaceSetAsProfile
            );

            <ProfileSpaceIdByAccount<T>>::remove(&sender);

            Self::deposit_event(Event::ProfileUpdated { account: sender, space_id: None });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::create_space_as_profile())]
        pub fn create_space_as_profile(origin: OriginFor<T>, content: Content) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let space_id = T::SpacesInterface::create_space(&sender, content)?;

            Self::do_set_profile(&sender, space_id)?;

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn do_set_profile(account: &T::AccountId, space_id: SpaceId) -> DispatchResult {
            T::SpacePermissionsProvider::ensure_space_owner(space_id, account)?;

            <ProfileSpaceIdByAccount<T>>::insert(account, space_id);

            Self::deposit_event(Event::ProfileUpdated {
                account: account.clone(),
                space_id: Some(space_id),
            });
            Ok(())
        }

        pub fn unlink_space_from_profile(account: &T::AccountId, space_id: SpaceId) {
            if let Some(profile_space_id) = Self::profile_space_id_by_account(account) {
                if profile_space_id == space_id {
                    <ProfileSpaceIdByAccount<T>>::remove(account);
                    Self::deposit_event(Event::ProfileUpdated {
                        account: account.clone(),
                        space_id: None,
                    });
                }
            }
        }
    }

    impl<T: Config> ProfileManager<T::AccountId> for Pallet<T> {
        fn unlink_space_from_profile(account: &T::AccountId, space_id: SpaceId) {
            Self::unlink_space_from_profile(account, space_id)
        }
    }
}
