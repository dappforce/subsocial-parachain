#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, ExistenceRequirement};

pub use pallet::*;
use pallet_spaces::Pallet as Spaces;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

// pub mod rpc;

pub mod types;

pub(crate) type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::pallet::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    use subsocial_support::{
        remove_from_vec,
        traits::{IsAccountBlocked, SpaceFollowsProvider},
        ModerationError, SpaceId,
    };

    use crate::types::{SpaceFollowSettings, SpaceSubscriberInfo};

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_spaces::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency type.
        type Currency: Currency<Self::AccountId>;

        type WeightInfo: WeightInfo;
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
        /// Caller account is not the owner of that space.
        NotSpaceOwner,
    }

    #[pallet::storage]
    #[pallet::getter(fn space_follow_settings)]
    pub type FollowSettingsForSpace<T: Config> =
        StorageMap<_, Blake2_128Concat, SpaceId, SpaceFollowSettings<BalanceOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn space_subscriber)]
    pub type SpaceSubscribers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SpaceId,
        Twox64Concat,
        T::AccountId,
        SpaceSubscriberInfo<BalanceOf<T>, T::BlockNumber>,
    >;

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
        SpaceFollowed { follower: T::AccountId, space_id: SpaceId },
        SpaceUnfollowed { follower: T::AccountId, space_id: SpaceId },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(<T as Config>::WeightInfo::set_space_follow_settings())]
        pub fn set_space_follow_settings(
            origin: OriginFor<T>,
            space_id: SpaceId,
            settings: SpaceFollowSettings<BalanceOf<T>>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            let space = Spaces::<T>::require_space(space_id)?;
            ensure!(space.owner == caller, Error::<T>::NotSpaceOwner);

            FollowSettingsForSpace::<T>::insert(space_id, settings);

            Ok(())
        }

        #[pallet::weight(<T as Config>::WeightInfo::unfollow_space())]
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

            let space_follow_settings = FollowSettingsForSpace::<T>::get(space_id);
            let maybe_subscription_info = SpaceSubscribers::<T>::get(space_id, follower.clone());

            if let Some(balance) = space_follow_settings.subscription {
                let current_block_number = <frame_system::Pallet<T>>::block_number();
                let should_subscribe = matches!(
                    maybe_subscription_info,
                    Some(SpaceSubscriberInfo {
                        subscribed_on,
                        expires_on,
                        ..
                    })
                    if subscribed_on >= current_block_number &&
                    (expires_on.is_none() || matches!(expires_on, Some(block) if current_block_number < block))
                );

                if should_subscribe {
                    T::Currency::transfer(
                        &follower,
                        &space.owner,
                        balance,
                        ExistenceRequirement::KeepAlive,
                    )?;
                    SpaceSubscribers::<T>::insert(
                        space_id,
                        follower.clone(),
                        SpaceSubscriberInfo {
                            subscribed_on: current_block_number,
                            expires_on: None,
                            subscription: balance,
                        },
                    );
                }
            }

            Self::add_space_follower(follower, space_id);

            Ok(())
        }

        #[pallet::weight(<T as Config>::WeightInfo::unfollow_space())]
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

            Self::deposit_event(Event::SpaceFollowed { follower, space_id });
        }

        pub fn remove_space_follower(follower: T::AccountId, space_id: SpaceId) -> DispatchResult {
            SpacesFollowedByAccount::<T>::mutate(follower.clone(), |space_ids| {
                remove_from_vec(space_ids, space_id)
            });
            SpaceFollowers::<T>::mutate(space_id, |account_ids| {
                remove_from_vec(account_ids, follower.clone())
            });
            SpaceFollowedByAccount::<T>::remove((follower.clone(), space_id));

            Self::deposit_event(Event::SpaceUnfollowed { follower, space_id });
            Ok(())
        }
    }

    impl<T: Config> SpaceFollowsProvider for Pallet<T> {
        type AccountId = T::AccountId;

        fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool {
            Pallet::<T>::space_followed_by_account((account, space_id))
        }
    }
}
