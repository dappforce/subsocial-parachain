#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// pub use crate::weights::WeightInfo;
//
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
//
// pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::RawOrigin, pallet_prelude::*, traits::Currency, transactional};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{StaticLookup, Zero};
    use sp_std::vec::Vec;

    use pallet_posts::{NextPostId, PostExtension};
    use subsocial_support::{Content, PostId, SpaceId};
    use sp_std::{convert::TryInto, fmt::Debug};

    // use crate::weights::WeightInfo;

    type ResourceId<T> = BoundedVec<u8, <T as Config>::MaxResourcesIdLength>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_posts::Config + pallet_spaces::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The id of the space, where all resource posts will reside.
        type ResourcesSpaceId: Get<SpaceId>;

        /// The maximum number of characters in a resource id.
        #[pallet::constant]
        type MaxResourcesIdLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ResourcePostCreated { resource_id: ResourceId<T>, post_id: PostId },
    }

    #[pallet::error]
    pub enum Error<T> {
        ResourcePostAlreadyCreated,
    }

    #[pallet::storage]
    #[pallet::getter(fn resource_post)]
    pub type ResourcePost<T: Config> = StorageMap<_, Twox64Concat, ResourceId<T>, PostId>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        #[transactional]
        pub fn create_resource_post(
            origin: OriginFor<T>,
            resource_id: ResourceId<T>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            ensure!(
                ResourcePost::<T>::contains_key(resource_id.clone()),
                Error::<T>::ResourcePostAlreadyCreated,
            );

            let resource_space =
                pallet_spaces::Pallet::<T>::require_space(T::ResourcesSpaceId::get())?;

            pallet_posts::Pallet::<T>::create_post(
                RawOrigin::Signed(resource_space.owner).into(),
                Some(resource_space.id),
                PostExtension::RegularPost,
                Content::None,
            )?;

            let post_id = NextPostId::<T>::get();

            ResourcePost::<T>::insert(resource_id.clone(), post_id);

            Self::deposit_event(Event::ResourcePostCreated { resource_id, post_id });

            Ok(())
        }
    }
}
