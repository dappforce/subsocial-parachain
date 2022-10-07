#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod types;

#[frame_support::pallet]
pub mod pallet {
    use std::fmt::Debug;

    use codec::EncodeLike;
    use frame_support::fail;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::{Currency, ExistenceRequirement};
    use frame_system::pallet_prelude::*;

    use crate::types::*;

    use super::*;

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::pallet::Config>::AccountId,
    >>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency trait.
        type Currency: Currency<Self::AccountId>;

        type SpaceId: MaxEncodedLen + Copy + Decode + TypeInfo + EncodeLike + Eq + Debug;

        type SpacesInterface: SubscriptionSpacesInterface<Self::AccountId, Self::SpaceId>;

        type RoleId: MaxEncodedLen + Copy + Decode + TypeInfo + EncodeLike + Eq + Debug;

        type RolesInterface: SubscriptionRolesInterface<
            Self::RoleId,
            Self::SpaceId,
            Self::AccountId,
        >;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Account is not the space owner.
        NotSpaceOwner,
        /// Role cannot be found in the space.
        RoleNotInSpace,
        /// User have already subscribed to this space.
        AlreadySubscribed,
        /// User have already unsubscribed or have no subscriptions to this space.
        AlreadyNotSubscribed,
        /// Cannot subscribe to the space that does not have subscriptions settings.
        SubscriptionNotEnabled,
        /// Space was not found by id.
        SpaceNotFound,
    }

    #[pallet::storage]
    pub type SubscriptionSettingsBySpace<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::SpaceId,
        SubscriptionSettings<BalanceOf<T>, T::RoleId>,
    >;

    #[pallet::storage]
    pub type SpaceSubscribers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::SpaceId,
        Twox64Concat,
        T::AccountId,
        SubscriberInfo<BalanceOf<T>, T::RoleId, T::BlockNumber>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SubscriptionSettingsChanged {
            /// Space owner
            account: T::AccountId,
            space: T::SpaceId,
        },
        UserSubscribed {
            /// Subscriber
            account: T::AccountId,
            space: T::SpaceId,
            granted_role: T::RoleId,
            price: BalanceOf<T>,
        },
        UserUnSubscribed {
            account: T::AccountId,
            space: T::SpaceId,
        },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // // TODO think
        // // 1. Create role -> Enable subscriptions.
        // // ... close the tab.
        // // 2. Set up subsc. for the 1-st time.
        // pub fn create_subscription_settings() {
        //     // Create a role
        //     // Set up a subscriptions settings for the first time.
        // }

        #[pallet::weight(100_000_000)]
        pub fn update_subscription_settings(
            origin: OriginFor<T>,
            space_id: T::SpaceId,
            settings: SubscriptionSettings<BalanceOf<T>, T::RoleId>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            ensure!(
                T::SpacesInterface::is_space_owner(owner.clone(), space_id),
                Error::<T>::NotSpaceOwner
            );

            ensure!(
                T::RolesInterface::does_role_exist_in_space(settings.role_id, space_id),
                Error::<T>::RoleNotInSpace,
            );

            SubscriptionSettingsBySpace::<T>::insert(space_id, settings);

            Self::deposit_event(Event::SubscriptionSettingsChanged {
                account: owner,
                space: space_id,
            });

            Ok(())
        }

        #[pallet::weight(100_000_000)]
        pub fn subscribe(origin: OriginFor<T>, space_id: T::SpaceId) -> DispatchResult {
            let subscriber = ensure_signed(origin)?;

            if matches!(SpaceSubscribers::<T>::get(space_id, subscriber.clone()), Some(info) if !info.unsubscribed) {
                fail!(Error::<T>::AlreadySubscribed);
            }

            let settings = match SubscriptionSettingsBySpace::<T>::get(space_id) {
                Some(settings) if !settings.disabled => settings,
                _ => fail!(Error::<T>::SubscriptionNotEnabled),
            };

            let space_owner =
                T::SpacesInterface::get_space_owner(space_id).ok_or(Error::<T>::SpaceNotFound)?;

            T::Currency::transfer(
                &subscriber,
                &space_owner,
                settings.subscription,
                ExistenceRequirement::KeepAlive,
            )?;

            T::RolesInterface::grant_role(subscriber.clone(), settings.role_id);

            SpaceSubscribers::<T>::insert(
                space_id,
                subscriber.clone(),
                SubscriberInfo {
                    subscribed_on: <frame_system::Pallet<T>>::block_number(),
                    subscription: settings.subscription,
                    granted_role_id: settings.role_id,
                    unsubscribed: false,
                },
            );

            Self::deposit_event(Event::UserSubscribed {
                account: subscriber,
                space: space_id,
                granted_role: settings.role_id,
                price: settings.subscription,
            });

            Ok(())
        }

        #[pallet::weight(100_000_000)]
        pub fn unsubscribe(origin: OriginFor<T>, space_id: T::SpaceId) -> DispatchResult {
            let subscriber = ensure_signed(origin)?;

            let mut subscriber_info = match SpaceSubscribers::<T>::get(space_id, subscriber.clone())
            {
                Some(info) if !info.unsubscribed => info,
                _ => fail!(Error::<T>::AlreadyNotSubscribed),
            };

            subscriber_info.unsubscribed = true;

            SpaceSubscribers::<T>::insert(space_id, subscriber.clone(), subscriber_info);

            Self::deposit_event(Event::UserUnSubscribed { account: subscriber, space: space_id });

            Ok(())
        }
    }
}
