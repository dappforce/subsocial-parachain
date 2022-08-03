#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::dispatch::DispatchResult;

use pallet_spaces::{BeforeSpaceCreated, Pallet as Spaces, types::Space};

// pub mod rpc;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use subsocial_support::{
        traits::{IsAccountBlocked, SpaceFollowsProvider},
        ModerationError, SpaceId, remove_from_vec,
    };
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_spaces::Config
    {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Account is already a space follower.
        AlreadySpaceFollower,
        /// Account is not a space follower.
        NotSpaceFollower,
        /// Not allowed to follow a hidden space.
        CannotFollowHiddenSpace,
    }

    #[pallet::storage]
    #[pallet::getter(fn space_followers)]
    pub type SpaceFollowers<T: Config> =
        StorageMap<_, Twox64Concat, SpaceId, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn space_followed_by_account)]
    pub type SpaceFollowedByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, SpaceId), bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn spaces_followed_by_account)]
    pub type SpacesFollowedByAccount<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<SpaceId>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SpaceFollowed(
            /* follower */ T::AccountId,
            /* following */ SpaceId,
        ),
        SpaceUnfollowed(
            /* follower */ T::AccountId,
            /* unfollowing */ SpaceId,
        ),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(5, 5))]
        pub fn follow_space(origin: OriginFor<T>, space_id: SpaceId) -> DispatchResult {
            let follower = ensure_signed(origin)?;

            ensure!(
                !Self::space_followed_by_account((follower.clone(), space_id)),
                Error::<T>::AlreadySpaceFollower
            );

            let space = Spaces::<T>::require_space(space_id)?;
            ensure!(!space.hidden, Error::<T>::CannotFollowHiddenSpace);

            ensure!(
                T::IsAccountBlocked::is_allowed_account(follower.clone(), space.id),
                ModerationError::AccountIsBlocked
            );

            Self::add_space_follower(follower, space_id);

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(5, 5))]
        pub fn unfollow_space(origin: OriginFor<T>, space_id: SpaceId) -> DispatchResult {
            let follower = ensure_signed(origin)?;

            Spaces::<T>::ensure_space_exists(space_id)?;

            ensure!(
                Self::space_followed_by_account((follower.clone(), space_id)),
                Error::<T>::NotSpaceFollower
            );

            Self::remove_space_follower(follower, space_id)
        }

        #[pallet::weight((
            100_000 + T::DbWeight::get().reads_writes(3, 4),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_follow_space(
            origin: OriginFor<T>,
            follower: T::AccountId,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            ensure!(
                !Self::space_followed_by_account((follower.clone(), space_id)),
                Error::<T>::AlreadySpaceFollower
            );

            Self::add_space_follower(follower, space_id);

            Ok(Pays::No.into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn add_space_follower(follower: T::AccountId, space_id: SpaceId) {
            SpaceFollowers::<T>::mutate(space_id, |followers| followers.push(follower.clone()));
            SpaceFollowedByAccount::<T>::insert((follower.clone(), space_id), true);
            SpacesFollowedByAccount::<T>::mutate(follower.clone(), |space_ids| {
                space_ids.push(space_id)
            });

            Self::deposit_event(Event::SpaceFollowed(follower, space_id));
        }

        pub fn remove_space_follower(follower: T::AccountId, space_id: SpaceId) -> DispatchResult {
            SpacesFollowedByAccount::<T>::mutate(follower.clone(), |space_ids| {
                remove_from_vec(space_ids, space_id)
            });
            SpaceFollowers::<T>::mutate(space_id, |account_ids| {
                remove_from_vec(account_ids, follower.clone())
            });
            SpaceFollowedByAccount::<T>::remove((follower.clone(), space_id));

            Self::deposit_event(Event::SpaceUnfollowed(follower, space_id));
            Ok(())
        }
    }

    impl<T: Config> SpaceFollowsProvider for Pallet<T> {
        type AccountId = T::AccountId;

        fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool {
            Pallet::<T>::space_followed_by_account((account, space_id))
        }
    }

    impl<T: Config> BeforeSpaceCreated<T> for Pallet<T> {
        fn before_space_created(creator: T::AccountId, space: &mut Space<T>) -> DispatchResult {
            // Make a space creator the first follower of this space:
            Pallet::<T>::add_space_follower(creator, space.id);
            Ok(())
        }
    }
}

/// Handler that will be called right before the space is followed.
pub trait BeforeSpaceFollowed<T: Config> {
    fn before_space_followed(
        follower: T::AccountId,
        space: &mut Space<T>,
    ) -> DispatchResult;
}

impl<T: Config> BeforeSpaceFollowed<T> for () {
    fn before_space_followed(
        _follower: T::AccountId,
        _space: &mut Space<T>,
    ) -> DispatchResult {
        Ok(())
    }
}

/// Handler that will be called right before the space is unfollowed.
pub trait BeforeSpaceUnfollowed<T: Config> {
    fn before_space_unfollowed(follower: T::AccountId, space: &mut Space<T>) -> DispatchResult;
}

impl<T: Config> BeforeSpaceUnfollowed<T> for () {
    fn before_space_unfollowed(_follower: T::AccountId, _space: &mut Space<T>) -> DispatchResult {
        Ok(())
    }
}
