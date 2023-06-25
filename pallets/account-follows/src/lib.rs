// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// pub mod rpc;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use subsocial_support::remove_from_vec;

    use sp_std::vec::Vec;

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn account_followers)]
    pub(super) type AccountFollowers<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_followed_by_account)]
    pub(super) type AccountFollowedByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn accounts_followed_by_account)]
    pub(super) type AccountsFollowedByAccount<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        AccountFollowed { follower: T::AccountId, account: T::AccountId },
        AccountUnfollowed { follower: T::AccountId, account: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Follower social account was not found by id.
        FollowerAccountNotFound,
        /// Social account that is being followed was not found by id.
        FollowedAccountNotFound,

        /// Account can not follow itself.
        AccountCannotFollowItself,
        /// Account can not unfollow itself.
        AccountCannotUnfollowItself,

        /// Account (Alice) is already a follower of another account (Bob).
        AlreadyAccountFollower,
        /// Account (Alice) is not a follower of another account (Bob).
        NotAccountFollower,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_ref_time(1_250_000) + T::DbWeight::get().reads_writes(2, 3))]
        pub fn follow_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            let follower = ensure_signed(origin)?;

            ensure!(follower != account, Error::<T>::AccountCannotFollowItself);
            ensure!(
                !<AccountFollowedByAccount<T>>::contains_key((follower.clone(), account.clone())),
                Error::<T>::AlreadyAccountFollower
            );

            AccountsFollowedByAccount::<T>::mutate(follower.clone(), |ids| {
                ids.push(account.clone())
            });
            AccountFollowers::<T>::mutate(account.clone(), |ids| ids.push(follower.clone()));
            AccountFollowedByAccount::<T>::insert((follower.clone(), account.clone()), true);

            Self::deposit_event(Event::AccountFollowed { follower, account });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_ref_time(1_250_000) + T::DbWeight::get().reads_writes(2, 3))]
        pub fn unfollow_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            let follower = ensure_signed(origin)?;

            ensure!(follower != account, Error::<T>::AccountCannotUnfollowItself);
            ensure!(
                <AccountFollowedByAccount<T>>::contains_key((follower.clone(), account.clone())),
                Error::<T>::NotAccountFollower
            );

            AccountsFollowedByAccount::<T>::mutate(follower.clone(), |account_ids| {
                remove_from_vec(account_ids, account.clone())
            });
            AccountFollowers::<T>::mutate(account.clone(), |account_ids| {
                remove_from_vec(account_ids, follower.clone())
            });
            AccountFollowedByAccount::<T>::remove((follower.clone(), account.clone()));

            Self::deposit_event(Event::AccountUnfollowed { follower, account });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight((
            Weight::from_ref_time(10_000) + T::DbWeight::get().reads_writes(4, 4),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_follow_account(
            origin: OriginFor<T>,
            follower: T::AccountId,
            following: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            ensure!(
                !Self::account_followed_by_account((follower.clone(), following.clone())),
                Error::<T>::AlreadyAccountFollower
            );

            AccountsFollowedByAccount::<T>::mutate(follower.clone(), |ids| {
                ids.push(following.clone())
            });
            AccountFollowers::<T>::mutate(following.clone(), |ids| ids.push(follower.clone()));
            AccountFollowedByAccount::<T>::insert((follower.clone(), following.clone()), true);

            Self::deposit_event(Event::AccountFollowed { follower, account: following });

            Ok(Pays::No.into())
        }
    }
}
