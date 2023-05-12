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
    use frame_support::{dispatch::RawOrigin, pallet_prelude::*, transactional};
    use frame_system::pallet_prelude::*;
    use sp_std::{convert::TryInto};

    use pallet_posts::{NextPostId, PostExtension};
    use subsocial_support::{Content, PostId, SpaceId};

    // use crate::weights::WeightInfo;

    type ResourceId<T> = BoundedVec<u8, <T as Config>::MaxResourcesIdLength>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_posts::Config + pallet_spaces::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
        ResourceDiscussionLinked { resource_id: ResourceId<T>, account_id: T::AccountId, post_id: PostId },
    }

    #[pallet::error]
    pub enum Error<T> {
        ResourceDiscussionAlreadyLinked,
    }

    #[pallet::storage]
    #[pallet::getter(fn resource_post)]
    pub type ResourceDiscussion<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, ResourceId<T>, Twox64Concat, T::AccountId, PostId>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        #[transactional]
        pub fn link_post_to_resource(
            origin: OriginFor<T>,
            resource_id: ResourceId<T>,
            post_id: PostId,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Self::do_link_post_to_resource(caller, resource_id, post_id)
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_ref_time(10_000))]
        #[transactional]
        pub fn create_resource_discussion(
            origin: OriginFor<T>,
            resource_id: ResourceId<T>,
            space_id: PostId,
            content: Content,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            Self::do_create_resource_discussion(caller, resource_id, space_id, content)
        }
    }

    impl<T: Config> Pallet<T> {
        fn do_link_post_to_resource(
            caller: T::AccountId,
            resource_id: ResourceId<T>,
            post_id: PostId,
        ) -> DispatchResult {
            let post = pallet_posts::Pallet::<T>::require_post(post_id)?;
            post.ensure_owner(&caller)?;

            ResourceDiscussion::<T>::insert(resource_id.clone(), caller.clone(), post_id);

            Self::deposit_event(Event::ResourceDiscussionLinked {
                resource_id,
                account_id: caller.clone(),
                post_id,
            });

            Ok(())
        }

        fn do_create_resource_discussion(
            caller: T::AccountId,
            resource_id: ResourceId<T>,
            space_id: SpaceId,
            content: Content,
        ) -> DispatchResult {
            // doesnt make sense new discussion post if already exists, you can relink it using link_post_to_resource
            ensure!(
                ResourceDiscussion::<T>::contains_key(resource_id.clone(), caller.clone()),
                Error::<T>::ResourceDiscussionAlreadyLinked,
            );

            let space = pallet_spaces::Pallet::<T>::require_space(space_id)?;

            let post_id = NextPostId::<T>::get();

            // this call ensures that [caller] have the correct permissions to create posts in that space
            pallet_posts::Pallet::<T>::create_post(
                RawOrigin::Signed(caller.clone()).into(),
                Some(space.id),
                PostExtension::RegularPost,
                content,
            )?;

            Self::do_link_post_to_resource(caller, resource_id, post_id)
        }
    }
}
