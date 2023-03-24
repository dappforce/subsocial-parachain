//! # Spaces Module
//!
//! Spaces are the primary components of Subsocial. This module allows you to create a Space
//! and customize it by updating its' owner(s), content, and permissions.
//!
//! To understand how Spaces fit into the Subsocial ecosystem, you can think of how
//! folders and files work in a file system. Spaces are similar to folders, that can contain Posts,
//! in this sense. The permissions of the Space and Posts can be customized so that a Space
//! could be as simple as a personal blog (think of a page on Facebook) or as complex as community
//! (think of a subreddit) governed DAO.
//!
//! Spaces can be compared to existing entities on web 2.0 platforms such as:
//!
//! - Blogs on Blogger,
//! - Publications on Medium,
//! - Groups or pages on Facebook,
//! - Accounts on Twitter and Instagram,
//! - Channels on YouTube,
//! - Servers on Discord,
//! - Forums on Discourse.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use pallet_permissions::{SpacePermission, SpacePermissions};
use subsocial_support::{traits::SpaceFollowsProvider, Content, SpaceId};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

// pub mod rpc;
pub mod types;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    use pallet_permissions::{
        Pallet as Permissions, PermissionChecker, SpacePermissionsContext, SpacePermissionsInfoOf,
    };
    use subsocial_support::{
        ensure_content_is_valid, remove_from_bounded_vec,
        traits::{IsAccountBlocked, IsContentBlocked, SpacePermissionsProvider, SpacesInterface},
        ModerationError, SpacePermissionsInfo, WhoAndWhen, WhoAndWhenOf,
    };
    use types::*;

    pub use crate::weights::WeightInfo;

    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_permissions::Config + pallet_timestamp::Config
    {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Roles: PermissionChecker<AccountId = Self::AccountId>;

        type SpaceFollows: SpaceFollowsProvider<AccountId = Self::AccountId>;

        type IsAccountBlocked: IsAccountBlocked<Self::AccountId>;

        type IsContentBlocked: IsContentBlocked;

        #[pallet::constant]
        type MaxSpacesPerAccount: Get<u32>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        SpaceCreated { account: T::AccountId, space_id: SpaceId },
        SpaceUpdated { account: T::AccountId, space_id: SpaceId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Space was not found by id.
        SpaceNotFound,
        /// Nothing to update in this space.
        NoUpdatesForSpace,
        /// Only space owners can manage this space.
        NotASpaceOwner,
        /// User has no permission to update this space.
        NoPermissionToUpdateSpace,
        /// User has no permission to create subspaces within this space.
        NoPermissionToCreateSubspaces,
        /// Space is at root level, no `parent_id` specified.
        SpaceIsAtRoot,
        /// New spaces' settings don't differ from the old ones.
        NoUpdatesForSpacesSettings,
        /// There are too many spaces created by this account already
        TooManySpacesPerAccount,
    }

    #[pallet::type_value]
    pub fn DefaultForNextSpaceId() -> SpaceId {
        RESERVED_SPACE_COUNT + 1
    }

    /// The next space id.
    #[pallet::storage]
    #[pallet::getter(fn next_space_id)]
    pub type NextSpaceId<T: Config> = StorageValue<_, SpaceId, ValueQuery, DefaultForNextSpaceId>;

    /// Get the details of a space by its' id.
    #[pallet::storage]
    #[pallet::getter(fn space_by_id)]
    pub type SpaceById<T: Config> = StorageMap<_, Twox64Concat, SpaceId, Space<T>>;

    /// Find the ids of all spaces owned, by a given account.
    #[pallet::storage]
    #[pallet::getter(fn space_ids_by_owner)]
    pub type SpaceIdsByOwner<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, SpacesByAccount<T>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub endowed_account: Option<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { endowed_account: None }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::init_pallet(self.endowed_account.as_ref());
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(< T as Config >::WeightInfo::create_space())]
        pub fn create_space(
            origin: OriginFor<T>,
            content: Content,
            permissions_opt: Option<SpacePermissions>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            Self::do_create_space(&owner, content, permissions_opt)?;
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(< T as Config >::WeightInfo::update_space())]
        pub fn update_space(
            origin: OriginFor<T>,
            space_id: SpaceId,
            update: SpaceUpdate,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            let has_updates =
                update.content.is_some() || update.hidden.is_some() || update.permissions.is_some();

            ensure!(has_updates, Error::<T>::NoUpdatesForSpace);

            let mut space = Self::require_space(space_id)?;

            ensure!(
                T::IsAccountBlocked::is_allowed_account(owner.clone(), space.id),
                ModerationError::AccountIsBlocked
            );

            Self::ensure_account_has_space_permission(
                owner.clone(),
                &space,
                SpacePermission::UpdateSpace,
                Error::<T>::NoPermissionToUpdateSpace.into(),
            )?;

            let mut is_update_applied = false;

            if let Some(content) = update.content {
                if content != space.content {
                    ensure_content_is_valid(content.clone())?;

                    ensure!(
                        T::IsContentBlocked::is_allowed_content(content.clone(), space.id),
                        ModerationError::ContentIsBlocked
                    );

                    space.content = content;
                    space.edited = true;
                    is_update_applied = true;
                }
            }

            if let Some(hidden) = update.hidden {
                if hidden != space.hidden {
                    space.hidden = hidden;
                    is_update_applied = true;
                }
            }

            if let Some(overrides_opt) = update.permissions {
                if space.permissions != overrides_opt {
                    if let Some(overrides) = overrides_opt.clone() {
                        space.permissions = Some(Permissions::<T>::override_permissions(overrides));
                    } else {
                        space.permissions = overrides_opt;
                    }

                    is_update_applied = true;
                }
            }

            // Update this space only if at least one field should be updated:
            if is_update_applied {
                SpaceById::<T>::insert(space_id, space);
                Self::deposit_event(Event::SpaceUpdated { account: owner, space_id });
            }
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight((
            Weight::from_ref_time(1_000_000) + T::DbWeight::get().reads_writes(1, 3),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_create_space(
            origin: OriginFor<T>,
            space_id: SpaceId,
            created: WhoAndWhenOf<T>,
            owner: T::AccountId,
            content: Content,
            hidden: bool,
            permissions_opt: Option<SpacePermissions>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let permissions =
                permissions_opt.map(|perms| Permissions::<T>::override_permissions(perms));

            let WhoAndWhen { account, time, .. } = created;
            let new_who_and_when =
                WhoAndWhen { account, block: frame_system::Pallet::<T>::block_number(), time };

            let new_space = &mut Space {
                id: space_id,
                created: new_who_and_when,
                edited: false,
                owner: owner.clone(),
                content,
                hidden,
                permissions,
            };

            let add_new_space_id_by_owner = |owner: &T::AccountId, space_id: SpaceId| {
                SpaceIdsByOwner::<T>::mutate(&owner, |ids| {
                    ids.try_push(space_id).expect("qed; too many spaces per account")
                });
            };

            // To prevent incorrect [SpaceIdsByOwner] insertion,
            // we check if the space already exists.
            match Self::require_space(space_id) {
                Ok(space) if !space.is_owner(&owner) => {
                    SpaceIdsByOwner::<T>::mutate(&space.owner, |ids| {
                        remove_from_bounded_vec(ids, space_id)
                    });
                    add_new_space_id_by_owner(&owner, space_id);
                },
                Err(_) => add_new_space_id_by_owner(&owner, space_id),
                _ => (),
            }

            SpaceById::<T>::insert(space_id, new_space);

            Self::deposit_event(Event::SpaceCreated { account: owner, space_id });

            Ok(Pays::No.into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight((
            Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_set_next_space_id(
            origin: OriginFor<T>,
            space_id: SpaceId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            NextSpaceId::<T>::put(space_id);
            Ok(Pays::No.into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Create reserved spaces either on genesis build or when pallet is added to a runtime.
        pub fn init_pallet(endowed_account_opt: Option<&T::AccountId>) {
            if let Some(endowed_account) = endowed_account_opt {
                let mut spaces = Vec::new();

                for id in FIRST_SPACE_ID..=RESERVED_SPACE_COUNT {
                    spaces.push((
                        id,
                        Space::<T>::new(id, endowed_account.clone(), Content::None, None),
                    ));
                }
                spaces.iter().for_each(|(space_id, space)| {
                    SpaceById::<T>::insert(space_id, space);
                });
            }
        }

        fn do_create_space(
            owner: &T::AccountId,
            content: Content,
            permissions_opt: Option<SpacePermissions>,
        ) -> Result<SpaceId, DispatchError> {
            ensure_content_is_valid(content.clone())?;
            Self::ensure_space_limit_not_reached(owner)?;

            let permissions =
                permissions_opt.map(|perms| Permissions::<T>::override_permissions(perms));

            let space_id = Self::next_space_id();
            let new_space = &mut Space::new(space_id, owner.clone(), content, permissions);

            SpaceById::<T>::insert(space_id, new_space);
            SpaceIdsByOwner::<T>::mutate(owner, |ids| {
                ids.try_push(space_id).expect("qed; too many spaces per account")
            });
            NextSpaceId::<T>::mutate(|n| *n += 1);

            Self::deposit_event(Event::SpaceCreated { account: owner.clone(), space_id });
            Ok(space_id)
        }

        /// Check that there is a `Space` with such `space_id` in the storage
        /// or return`SpaceNotFound` error.
        pub fn ensure_space_exists(space_id: SpaceId) -> DispatchResult {
            ensure!(<SpaceById<T>>::contains_key(space_id), Error::<T>::SpaceNotFound);
            Ok(())
        }

        /// Get `Space` by id from the storage or return `SpaceNotFound` error.
        pub fn require_space(space_id: SpaceId) -> Result<Space<T>, DispatchError> {
            Ok(Self::space_by_id(space_id).ok_or(Error::<T>::SpaceNotFound)?)
        }

        pub fn ensure_account_has_space_permission(
            account: T::AccountId,
            space: &Space<T>,
            permission: SpacePermission,
            error: DispatchError,
        ) -> DispatchResult {
            let is_owner = space.is_owner(&account);
            let is_follower = space.is_follower(&account);

            let ctx = SpacePermissionsContext {
                space_id: space.id,
                is_space_owner: is_owner,
                is_space_follower: is_follower,
                space_perms: space.permissions.clone(),
            };

            T::Roles::ensure_account_has_space_permission(account, ctx, permission, error)
        }

        pub fn mutate_space_by_id<F: FnOnce(&mut Space<T>)>(
            space_id: SpaceId,
            f: F,
        ) -> Result<Space<T>, DispatchError> {
            <SpaceById<T>>::try_mutate(space_id, |space_opt| {
                if let Some(ref mut space) = space_opt.clone() {
                    f(space);
                    *space_opt = Some(space.clone());

                    return Ok(space.clone());
                }

                Err(Error::<T>::SpaceNotFound.into())
            })
        }

        pub fn ensure_space_limit_not_reached(owner: &T::AccountId) -> DispatchResult {
            ensure!(
                Self::space_ids_by_owner(&owner).len() < T::MaxSpacesPerAccount::get() as usize,
                Error::<T>::TooManySpacesPerAccount,
            );
            Ok(())
        }
    }

    impl<T: Config> SpacePermissionsProvider<T::AccountId, SpacePermissionsInfoOf<T>> for Pallet<T> {
        fn space_permissions_info(id: SpaceId) -> Result<SpacePermissionsInfoOf<T>, DispatchError> {
            let space = Pallet::<T>::require_space(id)?;

            Ok(SpacePermissionsInfo { owner: space.owner, permissions: space.permissions })
        }

        fn ensure_space_owner(id: SpaceId, account: &T::AccountId) -> DispatchResult {
            let space = Pallet::<T>::require_space(id)?;
            ensure!(space.is_owner(account), Error::<T>::NotASpaceOwner);
            Ok(())
        }
    }

    impl<T: Config> SpacesInterface<T::AccountId, SpaceId> for Pallet<T> {
        fn get_space_owner(space_id: SpaceId) -> Result<T::AccountId, DispatchError> {
            let space = Pallet::<T>::require_space(space_id)?;
            Ok(space.owner)
        }

        fn create_space(owner: &T::AccountId, content: Content) -> Result<SpaceId, DispatchError> {
            Self::do_create_space(owner, content, None)
        }
    }
}
