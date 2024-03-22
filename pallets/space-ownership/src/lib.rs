// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

//! # Ownership Module
//!
//! This module allows the transfer of ownership of entities such as spaces, posts, and domains.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::ensure_signed;
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub mod migration;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_permissions::SpacePermissions;

    use subsocial_support::{PostId, SpaceId, SpacePermissionsInfo, traits::{CreatorStakingProvider, DomainsProvider, ProfileManager, SpacesProvider, PostsProvider, SpacePermissionsProvider}};

    pub(crate) type DomainLengthOf<T> = 
        <<T as Config>::DomainsProvider as DomainsProvider<<T as frame_system::Config>::AccountId>>::DomainLength;
    
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub enum OwnableEntity<T: Config> {
        Space(SpaceId),
        Post(PostId),
        Domain(BoundedVec<u8, DomainLengthOf<T>>),
    }
    
    pub(crate) type SpacePermissionsInfoOf<T> =
        SpacePermissionsInfo<<T as frame_system::Config>::AccountId, SpacePermissions>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_permissions::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type ProfileManager: ProfileManager<Self::AccountId>;

        type SpacesProvider: SpacesProvider<Self::AccountId, SpaceId>;

        type SpacePermissionsProvider: SpacePermissionsProvider<Self::AccountId, SpacePermissionsInfoOf<Self>>;

        type CreatorStakingProvider: CreatorStakingProvider<Self::AccountId>;

        type DomainsProvider: DomainsProvider<Self::AccountId>;
        
        type PostsProvider: PostsProvider<Self::AccountId>;

        type Currency: frame_support::traits::Currency<Self::AccountId>;

        type WeightInfo: WeightInfo;
    }

    /// The current storage version
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::error]
    pub enum Error<T> {
        /// The current entity owner cannot transfer ownership to themselves.
        CannotTransferToCurrentOwner,
        /// Cannot transfer ownership, because a space is registered as an active creator.
        ActiveCreatorCannotTransferOwnership,
        /// There is no pending ownership transfer for a given entity.
        NoPendingTransfer,
        /// Account is not allowed to accept ownership transfer.
        CurrentOwnerCannotAcceptOwnershipTransfer,
        /// Account is not allowed to reject ownership transfer.
        NotAllowedToRejectOwnershipTransfer,
    }

    #[pallet::storage]
    #[pallet::getter(fn pending_ownership_transfer)]
    pub type PendingOwnershipTransfers<T: Config> =
        StorageMap<_, Twox64Concat, OwnableEntity<T>, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        OwnershipTransferCreated {
            current_owner: T::AccountId,
            entity: OwnableEntity<T>,
            new_owner: T::AccountId,
        },
        OwnershipTransferAccepted {
            account: T::AccountId,
            entity: OwnableEntity<T>,
        },
        OwnershipTransferRejected {
            account: T::AccountId,
            entity: OwnableEntity<T>,
        },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(
            match entity {
                OwnableEntity::Space(_) => T::WeightInfo::transfer_space_ownership(),
                OwnableEntity::Post(_) => T::WeightInfo::transfer_post_ownership(),
                OwnableEntity::Domain(_) => T::WeightInfo::transfer_domain_ownership(),
            }
        )]
        pub fn transfer_ownership(
            origin: OriginFor<T>,
            entity: OwnableEntity<T>,
            transfer_to: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            match entity.clone() {
                OwnableEntity::Space(space_id) => {
                    ensure!(who != transfer_to, Error::<T>::CannotTransferToCurrentOwner);

                    T::SpacePermissionsProvider::ensure_space_owner(space_id, &who)?;
                    // TODO: ensure we need this kind of moderation checks
                    //  ensure!(
                    //      T::IsAccountBlocked::is_allowed_account(transfer_to.clone(), space_id),
                    //      ModerationError::AccountIsBlocked
                    //  );

                    Self::ensure_not_active_creator(space_id)?;
                }
                OwnableEntity::Post(post_id) =>
                    T::PostsProvider::ensure_allowed_to_update_post(&who, post_id)?,
                OwnableEntity::Domain(domain) =>
                    T::DomainsProvider::ensure_allowed_to_update_domain(&who, &domain)?,
            }

            PendingOwnershipTransfers::<T>::insert(&entity, transfer_to.clone());

            Self::deposit_event(Event::OwnershipTransferCreated {
                current_owner: who,
                entity,
                new_owner: transfer_to,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(
            match entity {
                OwnableEntity::Space(_) => T::WeightInfo::accept_pending_space_ownership_transfer(),
                OwnableEntity::Post(_) => T::WeightInfo::accept_pending_post_ownership_transfer(),
                OwnableEntity::Domain(_) => T::WeightInfo::accept_pending_domain_ownership_transfer(),
            }
        )]
        pub fn accept_pending_ownership(origin: OriginFor<T>, entity: OwnableEntity<T>) -> DispatchResult {
            let new_owner = ensure_signed(origin)?;

            let transfer_to =
                Self::pending_ownership_transfer(&entity).ok_or(Error::<T>::NoPendingTransfer)?;

            ensure!(new_owner == transfer_to, Error::<T>::CurrentOwnerCannotAcceptOwnershipTransfer);

            match entity.clone() {
                OwnableEntity::Space(space_id) => {
                    let old_space_owner = Self::get_entity_owner(&entity)?;

                    Self::ensure_not_active_creator(space_id)?;
                    T::SpacesProvider::update_space_owner(space_id, transfer_to.clone())?;
                    T::ProfileManager::unlink_space_from_profile(&old_space_owner, space_id);
                }
                OwnableEntity::Post(post_id) =>
                    T::PostsProvider::update_post_owner(post_id, &new_owner)?,
                OwnableEntity::Domain(domain) =>
                    T::DomainsProvider::update_domain_owner(&domain, &new_owner)?,
            }

            PendingOwnershipTransfers::<T>::remove(&entity);

            Self::deposit_event(Event::OwnershipTransferAccepted {
                account: new_owner,
                entity,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::reject_pending_ownership())]
        pub fn reject_pending_ownership(origin: OriginFor<T>, entity: OwnableEntity<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let transfer_to =
                Self::pending_ownership_transfer(&entity).ok_or(Error::<T>::NoPendingTransfer)?;
            let entity_owner = Self::get_entity_owner(&entity)?;

            ensure!(
                who == transfer_to || who == entity_owner,
                Error::<T>::NotAllowedToRejectOwnershipTransfer
            );

            PendingOwnershipTransfers::<T>::remove(&entity);

            Self::deposit_event(Event::OwnershipTransferRejected { account: who, entity });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_not_active_creator(creator_id: SpaceId) -> DispatchResult {
            ensure!(
                !T::CreatorStakingProvider::is_creator_active(creator_id),
                Error::<T>::ActiveCreatorCannotTransferOwnership,
            );

            Ok(())
        }

        fn get_entity_owner(entity: &OwnableEntity<T>) -> Result<T::AccountId, DispatchError> {
            match entity {
                OwnableEntity::Space(space_id) => T::SpacesProvider::get_space_owner(*space_id),
                OwnableEntity::Post(post_id) => T::PostsProvider::get_post_owner(*post_id),
                OwnableEntity::Domain(domain) => T::DomainsProvider::get_domain_owner(domain),
            }
        }
    }
}
