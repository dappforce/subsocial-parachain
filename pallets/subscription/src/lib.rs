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
        /// User have already subscribed.
        AlreadySubscribed,
        /// Space cannot be subscribed to.
        CannotSubscribeToSpace,
        /// Space Id given is invalid.
        InvalidSpaceId,
    }

    #[pallet::storage]
    pub type SubscriptionSettingsForSpace<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::SpaceId,
        SpaceSubscriptionSettings<BalanceOf<T>, T::RoleId>,
    >;

    #[pallet::storage]
    pub type SpaceSubscribers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::SpaceId,
        Twox64Concat,
        T::AccountId,
        SpaceSubscriberInfo<BalanceOf<T>, T::RoleId, T::BlockNumber>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SpaceSubscriptionSettingsChanged { space_id: T::SpaceId },
        UserSubscribed { space_id: T::SpaceId, granted_role: T::RoleId, subscription: BalanceOf<T> },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000_000)]
        pub fn set_space_subscription_settings(
            origin: OriginFor<T>,
            space_id: T::SpaceId,
            subscription_settings: SpaceSubscriptionSettings<BalanceOf<T>, T::RoleId>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            ensure!(T::SpacesInterface::is_space_owner(owner, space_id), Error::<T>::NotSpaceOwner);

            ensure!(
                T::RolesInterface::does_role_exist_in_space(
                    subscription_settings.role_id,
                    space_id
                ),
                Error::<T>::RoleNotInSpace,
            );

            SubscriptionSettingsForSpace::<T>::insert(space_id, subscription_settings);

            Self::deposit_event(Event::SpaceSubscriptionSettingsChanged { space_id });

            Ok(())
        }

        #[pallet::weight(100_000_000)]
        pub fn subscribe(origin: OriginFor<T>, space_id: T::SpaceId) -> DispatchResult {
            let subscriber = ensure_signed(origin)?;

            ensure!(
                SpaceSubscribers::<T>::get(space_id, subscriber.clone()) == None,
                Error::<T>::AlreadySubscribed,
            );

            let settings = match SubscriptionSettingsForSpace::<T>::get(space_id) {
                None => fail!(Error::<T>::CannotSubscribeToSpace),
                Some(settings) => settings,
            };

            let space_owner =
                T::SpacesInterface::get_space_owner(space_id).ok_or(Error::<T>::InvalidSpaceId)?;

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
                SpaceSubscriberInfo {
                    subscribed_on: <frame_system::Pallet<T>>::block_number(),
                    subscription: settings.subscription,
                    granted_role_id: settings.role_id,
                },
            );

            Self::deposit_event(Event::UserSubscribed {
                space_id,
                granted_role: settings.role_id,
                subscription: settings.subscription,
            });

            Ok(())
        }
    }
}
