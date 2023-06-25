// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::dispatch::DispatchResult;

use pallet_posts::Pallet as Posts;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

// pub mod rpc;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use subsocial_support::{
        remove_from_vec,
        traits::{IsAccountBlocked, PostFollowsProvider},
        ModerationError, PostId,
    };

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_posts::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Account is already a post follower.
        AlreadyPostFollower,
        /// Account is not a post follower.
        NotPostFollower,
        /// Not allowed to follow a hidden post.
        CannotFollowHiddenPost,
    }

    #[pallet::storage]
    #[pallet::getter(fn post_followers)]
    pub type PostFollowers<T: Config> =
        StorageMap<_, Twox64Concat, PostId, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn post_followed_by_account)]
    pub type PostFollowedByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, PostId), bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn posts_followed_by_account)]
    pub type PostsFollowedByAccount<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<PostId>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PostFollowed { follower: T::AccountId, post_id: PostId },
        PostUnfollowed { follower: T::AccountId, post_id: PostId },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::follow_post())]
        pub fn follow_post(origin: OriginFor<T>, post_id: PostId) -> DispatchResult {
            let follower = ensure_signed(origin)?;

            ensure!(
                !Self::post_followed_by_account((follower.clone(), post_id)),
                Error::<T>::AlreadyPostFollower
            );

            let post = Posts::<T>::require_post(post_id)?;
            ensure!(!post.hidden, Error::<T>::CannotFollowHiddenPost);

            ensure!(
                T::IsAccountBlocked::is_allowed_account(follower.clone(), post.id),
                ModerationError::AccountIsBlocked
            );

            Self::add_post_follower(follower, post_id);

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::unfollow_post())]
        pub fn unfollow_post(origin: OriginFor<T>, post_id: PostId) -> DispatchResult {
            let follower = ensure_signed(origin)?;

            Posts::<T>::ensure_post_exists(post_id)?;

            ensure!(
                Self::post_followed_by_account((follower.clone(), post_id)),
                Error::<T>::NotPostFollower
            );

            Self::remove_post_follower(follower, post_id)
        }
    }

    impl<T: Config> Pallet<T> {
        fn add_post_follower(follower: T::AccountId, post_id: PostId) {
            PostFollowers::<T>::mutate(post_id, |followers| followers.push(follower.clone()));
            PostFollowedByAccount::<T>::insert((follower.clone(), post_id), true);
            PostsFollowedByAccount::<T>::mutate(follower.clone(), |post_ids| {
                post_ids.push(post_id)
            });

            Self::deposit_event(Event::PostFollowed { follower, post_id });
        }

        pub fn remove_post_follower(follower: T::AccountId, post_id: PostId) -> DispatchResult {
            PostsFollowedByAccount::<T>::mutate(follower.clone(), |post_ids| {
                remove_from_vec(post_ids, post_id)
            });
            PostFollowers::<T>::mutate(post_id, |account_ids| {
                remove_from_vec(account_ids, follower.clone())
            });
            PostFollowedByAccount::<T>::remove((follower.clone(), post_id));

            Self::deposit_event(Event::PostUnfollowed { follower, post_id });
            Ok(())
        }
    }

    impl<T: Config> PostFollowsProvider for Pallet<T> {
        type AccountId = T::AccountId;

        fn is_post_follower(account: Self::AccountId, post_id: PostId) -> bool {
            Pallet::<T>::post_followed_by_account((account, post_id))
        }
    }
}
