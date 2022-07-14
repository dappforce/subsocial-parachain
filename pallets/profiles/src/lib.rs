#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
// pub mod rpc;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use pallet_permissions::SpacePermissions;
    use subsocial_support::{traits::SpacePermissionsProvider, SpaceId, SpacePermissionsInfo};

    type SpacePermissionsInfoOf<T> =
        SpacePermissionsInfo<<T as frame_system::Config>::AccountId, SpacePermissions>;

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type SpacePermissionsProvider: SpacePermissionsProvider<SpacePermissionsInfoOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn profile_space_by_account)]
    pub type ProfileSpaceByAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, SpaceId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Space was successfully assigned as a profile.
        SpaceAsProfileAssigned { account: T::AccountId, space: SpaceId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Social account was not found by id.
        SocialAccountNotFound,
        /// Account is not a space owner.
        NotSpaceOwner,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // FIXME: cover with tests
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_space_as_profile(origin: OriginFor<T>, space_id: SpaceId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::try_set_space_as_profile(&sender, space_id)?;

            Self::deposit_event(Event::SpaceAsProfileAssigned { account: sender, space: space_id });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // FIXME: cover with tests
        pub fn try_set_space_as_profile(
            account: &T::AccountId,
            space_id: SpaceId,
        ) -> DispatchResult {
            let space_permissions_info =
                T::SpacePermissionsProvider::space_permissions_info(space_id)?;

            ensure!(&space_permissions_info.owner == account, Error::<T>::NotSpaceOwner);

            <ProfileSpaceByAccount<T>>::insert(account, space_id);
            Ok(())
        }

        // FIXME: cover with tests
        pub fn unset_space_as_profile(account: &T::AccountId, space_id: SpaceId) {
            if let Some(profile_space_id) = Self::profile_space_by_account(account) {
                if profile_space_id == space_id {
                    <ProfileSpaceByAccount<T>>::remove(account);
                }
            }
        }
    }
}
