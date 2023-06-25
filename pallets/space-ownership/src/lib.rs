// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::ensure_signed;
use sp_std::prelude::*;

use pallet_spaces::{Pallet as Spaces, SpaceById, SpaceIdsByOwner};
use subsocial_support::{
    remove_from_bounded_vec, traits::IsAccountBlocked, ModerationError, SpaceId,
};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use subsocial_support::traits::ProfileManager;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_spaces::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type ProfileManager: ProfileManager<Self::AccountId>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::error]
    pub enum Error<T> {
        /// The current space owner cannot transfer ownership to themself.
        CannotTransferToCurrentOwner,
        /// Account is already an owner of a space.
        AlreadyASpaceOwner,
        /// There is no pending ownership transfer for a given space.
        NoPendingTransferOnSpace,
        /// Account is not allowed to accept ownership transfer.
        NotAllowedToAcceptOwnershipTransfer,
        /// Account is not allowed to reject ownership transfer.
        NotAllowedToRejectOwnershipTransfer,
    }

    #[pallet::storage]
    #[pallet::getter(fn pending_space_owner)]
    pub type PendingSpaceOwner<T: Config> = StorageMap<_, Twox64Concat, SpaceId, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SpaceOwnershipTransferCreated {
            current_owner: T::AccountId,
            space_id: SpaceId,
            new_owner: T::AccountId,
        },
        SpaceOwnershipTransferAccepted {
            account: T::AccountId,
            space_id: SpaceId,
        },
        SpaceOwnershipTransferRejected {
            account: T::AccountId,
            space_id: SpaceId,
        },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::transfer_space_ownership())]
        pub fn transfer_space_ownership(
            origin: OriginFor<T>,
            space_id: SpaceId,
            transfer_to: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let space = Spaces::<T>::require_space(space_id)?;
            space.ensure_space_owner(who.clone())?;

            ensure!(who != transfer_to, Error::<T>::CannotTransferToCurrentOwner);
            ensure!(
                T::IsAccountBlocked::is_allowed_account(transfer_to.clone(), space_id),
                ModerationError::AccountIsBlocked
            );

            PendingSpaceOwner::<T>::insert(space_id, transfer_to.clone());

            Self::deposit_event(Event::SpaceOwnershipTransferCreated {
                current_owner: who,
                space_id,
                new_owner: transfer_to,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::accept_pending_ownership())]
        pub fn accept_pending_ownership(origin: OriginFor<T>, space_id: SpaceId) -> DispatchResult {
            let new_owner = ensure_signed(origin)?;

            let mut space = Spaces::require_space(space_id)?;
            ensure!(!space.is_owner(&new_owner), Error::<T>::AlreadyASpaceOwner);

            let transfer_to =
                Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;

            ensure!(new_owner == transfer_to, Error::<T>::NotAllowedToAcceptOwnershipTransfer);

            Spaces::<T>::ensure_space_limit_not_reached(&transfer_to)?;

            // Here we know that the origin is eligible to become a new owner of this space.
            PendingSpaceOwner::<T>::remove(space_id);

            let old_owner = space.owner;
            space.owner = new_owner.clone();
            SpaceById::<T>::insert(space_id, space);

            T::ProfileManager::unlink_space_from_profile(&old_owner, space_id);

            // Remove space id from the list of spaces by old owner
            SpaceIdsByOwner::<T>::mutate(old_owner, |space_ids| {
                remove_from_bounded_vec(space_ids, space_id)
            });

            // Add space id to the list of spaces by new owner
            SpaceIdsByOwner::<T>::mutate(new_owner.clone(), |ids| {
                ids.try_push(space_id).expect("qed; too many spaces per account")
            });

            // TODO add a new owner as a space follower? See
            // T::BeforeSpaceCreated::before_space_created(new_owner.clone(), space)?;

            Self::deposit_event(Event::SpaceOwnershipTransferAccepted {
                account: new_owner,
                space_id,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::reject_pending_ownership())]
        pub fn reject_pending_ownership(origin: OriginFor<T>, space_id: SpaceId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let space = Spaces::<T>::require_space(space_id)?;
            let transfer_to =
                Self::pending_space_owner(space_id).ok_or(Error::<T>::NoPendingTransferOnSpace)?;
            ensure!(
                who == transfer_to || who == space.owner,
                Error::<T>::NotAllowedToRejectOwnershipTransfer
            );

            PendingSpaceOwner::<T>::remove(space_id);

            Self::deposit_event(Event::SpaceOwnershipTransferRejected { account: who, space_id });
            Ok(())
        }
    }
}
