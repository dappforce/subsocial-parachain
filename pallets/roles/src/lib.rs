//! # Roles Module
//!
//! This module allow you to create dynalic roles with an associated set of permissions
//! and grant them to users (accounts or space ids) within a given space.
//!
//! For example if you want to create a space that enables editors in a similar way to Medium,
//! you would create a role "Editor" with permissions such as `CreatePosts`, `UpdateAnyPost`,
//! and `HideAnyComment`. Then you would grant this role to the specific accounts you would like
//! to make editors.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get};
use frame_system::{self as system, ensure_signed};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

use pallet_permissions::{
    Pallet as Permissions, PermissionChecker, SpacePermission, SpacePermissionSet,
};
use subsocial_support::{
    convert_users_vec_to_btree_set, ensure_content_is_valid, new_who_and_when,
    traits::{IsAccountBlocked, IsContentBlocked, SpaceFollowsProvider, SpacePermissionsProvider},
    Content, ModerationError, SpaceId, User, WhoAndWhenOf,
};

pub use pallet::*;
pub mod functions;

pub mod types;
pub use types::*;
// pub mod rpc;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(all(test, not(feature = "runtime-benchmarks")))]
mod tests;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_permissions::SpacePermissionsInfoOf;
    use subsocial_support::{remove_from_vec, WhoAndWhen};

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_permissions::Config + pallet_timestamp::Config
    {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// When deleting a role via `delete_role()` dispatch, this parameter is checked.
        /// If the number of users that own a given role is greater or equal to this number,
        /// then `TooManyUsersToDeleteRole` error will be returned and the dispatch will fail.
        #[pallet::constant]
        type MaxUsersToProcessPerDeleteRole: Get<u16>;

        type SpacePermissionsProvider: SpacePermissionsProvider<
            Self::AccountId,
            SpacePermissionsInfoOf<Self>,
        >;

        type SpaceFollows: SpaceFollowsProvider<AccountId = Self::AccountId>;

        type IsAccountBlocked: IsAccountBlocked<Self::AccountId>;

        type IsContentBlocked: IsContentBlocked;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RoleCreated { account: T::AccountId, space_id: SpaceId, role_id: RoleId },
        RoleUpdated { account: T::AccountId, role_id: RoleId },
        RoleDeleted { account: T::AccountId, role_id: RoleId },
        RoleGranted { account: T::AccountId, role_id: RoleId, users: Vec<User<T::AccountId>> },
        RoleRevoked { account: T::AccountId, role_id: RoleId, users: Vec<User<T::AccountId>> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Role was not found by id.
        RoleNotFound,

        /// `NextRoleId` exceeds its maximum value.
        RoleIdOverflow,

        /// Account does not have permission to manage roles in this space.
        NoPermissionToManageRoles,

        /// Nothing to update in role.
        NoUpdatesProvided,

        /// No permissions provided when trying to create a new role.
        /// A role must have at least one permission.
        NoPermissionsProvided,

        /// No users provided when trying to grant a role.
        /// A role must be granted/revoked to/from at least one user.
        NoUsersProvided,

        /// Canot remove a role from this many users in a single transaction.
        /// See `MaxUsersToProcessPerDeleteRole` parameter of this trait.
        TooManyUsersToDeleteRole,

        /// The user count sent doesn't match the real user count.
        IncorrectUserCount,

        /// Cannot disable a role that is already disabled.
        RoleAlreadyDisabled,

        /// Cannot enable a role that is already enabled.
        RoleAlreadyEnabled,
    }

    #[pallet::type_value]
    pub fn DefaultForNextRoleId() -> RoleId {
        FIRST_ROLE_ID
    }

    /// The next role id.
    #[pallet::storage]
    #[pallet::getter(fn next_role_id)]
    pub type NextRoleId<T: Config> = StorageValue<_, RoleId, ValueQuery, DefaultForNextRoleId>;

    /// Get the details of a role by its' id.
    #[pallet::storage]
    #[pallet::getter(fn role_by_id)]
    pub type RoleById<T: Config> = StorageMap<_, Twox64Concat, RoleId, Role<T>>;

    /// Get a list of all users (account or space ids) that a given role has been granted to.
    #[pallet::storage]
    #[pallet::getter(fn users_by_role_id)]
    pub type UsersByRoleId<T: Config> =
        StorageMap<_, Twox64Concat, RoleId, Vec<User<T::AccountId>>, ValueQuery>;

    // TODO: maybe use BoundedVec here?
    /// Get a list of all role ids available in a given space.
    #[pallet::storage]
    #[pallet::getter(fn role_ids_by_space_id)]
    pub type RoleIdsBySpaceId<T: Config> =
        StorageMap<_, Twox64Concat, SpaceId, Vec<RoleId>, ValueQuery>;

    /// Get a list of all role ids owned by a given user (account or space id)
    /// within a given space.
    #[pallet::storage]
    #[pallet::getter(fn role_ids_by_user_in_space)]
    pub type RoleIdsByUserInSpace<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        User<T::AccountId>,
        Twox64Concat,
        SpaceId,
        Vec<RoleId>,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new role, with a list of permissions, within a given space.
        ///
        /// `content` can optionally contain additional information associated with a role,
        /// such as a name, description, and image for a role. This may be useful for end users.
        ///
        /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_role())]
        pub fn create_role(
            origin: OriginFor<T>,
            space_id: SpaceId,
            time_to_live: Option<T::BlockNumber>,
            content: Content,
            permissions: Vec<SpacePermission>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!permissions.is_empty(), Error::<T>::NoPermissionsProvided);

            ensure_content_is_valid(content.clone())?;
            ensure!(
                T::IsContentBlocked::is_allowed_content(content.clone(), space_id),
                ModerationError::ContentIsBlocked,
            );

            Self::ensure_role_manager(who.clone(), space_id)?;

            let permissions_set = permissions.into_iter().collect();
            let new_role =
                Role::<T>::new(who.clone(), space_id, time_to_live, content, permissions_set)?;

            // TODO review strange code:
            let next_role_id = new_role.id.checked_add(1).ok_or(Error::<T>::RoleIdOverflow)?;
            NextRoleId::<T>::put(next_role_id);

            RoleById::<T>::insert(new_role.id, new_role.clone());
            RoleIdsBySpaceId::<T>::mutate(space_id, |role_ids| role_ids.push(new_role.id));

            Self::deposit_event(Event::RoleCreated {
                account: who,
                space_id,
                role_id: new_role.id,
            });
            Ok(())
        }

        /// Update an existing role by a given id.
        /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_role())]
        pub fn update_role(
            origin: OriginFor<T>,
            role_id: RoleId,
            update: RoleUpdate,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let has_updates = update.disabled.is_some() ||
                update.content.is_some() ||
                update.permissions.is_some();

            ensure!(has_updates, Error::<T>::NoUpdatesProvided);

            let mut role = Self::require_role(role_id)?;

            Self::ensure_role_manager(who.clone(), role.space_id)?;

            let mut is_update_applied = false;

            if let Some(disabled) = update.disabled {
                if disabled != role.disabled {
                    role.set_disabled(disabled)?;
                    is_update_applied = true;
                }
            }

            if let Some(content) = update.content {
                if content != role.content {
                    ensure_content_is_valid(content.clone())?;
                    ensure!(
                        T::IsContentBlocked::is_allowed_content(content.clone(), role.space_id),
                        ModerationError::ContentIsBlocked
                    );

                    role.content = content;
                    is_update_applied = true;
                }
            }

            if let Some(permissions) = update.permissions {
                if !permissions.is_empty() {
                    let permissions_diff: Vec<_> =
                        permissions.symmetric_difference(&role.permissions).cloned().collect();

                    if !permissions_diff.is_empty() {
                        role.permissions = permissions;
                        is_update_applied = true;
                    }
                }
            }

            if is_update_applied {
                <RoleById<T>>::insert(role_id, role);
                Self::deposit_event(Event::RoleUpdated { account: who, role_id });
            }
            Ok(())
        }

        /// Delete a given role and clean all associated storage items.
        /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::delete_role(*user_count))]
        pub fn delete_role(
            origin: OriginFor<T>,
            role_id: RoleId,
            user_count: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let role = Self::require_role(role_id)?;

            Self::ensure_role_manager(who.clone(), role.space_id)?;

            let users = Self::users_by_role_id(role_id);
            ensure!(users.len() as u32 == user_count, Error::<T>::IncorrectUserCount);
            ensure!(
                users.len() <= T::MaxUsersToProcessPerDeleteRole::get() as usize,
                Error::<T>::TooManyUsersToDeleteRole
            );

            let role_idx_by_space_opt =
                Self::role_ids_by_space_id(role.space_id).iter().position(|x| *x == role_id);

            if let Some(role_idx) = role_idx_by_space_opt {
                RoleIdsBySpaceId::<T>::mutate(role.space_id, |n| n.swap_remove(role_idx));
            }

            role.revoke_from_users(users);

            <RoleById<T>>::remove(role_id);
            <UsersByRoleId<T>>::remove(role_id);

            Self::deposit_event(Event::RoleDeleted { account: who, role_id });
            Ok(())
        }

        /// Grant a given role to a list of users.
        /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::grant_role(users.len() as u32))]
        pub fn grant_role(
            origin: OriginFor<T>,
            role_id: RoleId,
            users: Vec<User<T::AccountId>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!users.is_empty(), Error::<T>::NoUsersProvided);
            let users_set: BTreeSet<User<T::AccountId>> = convert_users_vec_to_btree_set(users)?;

            let role = Self::require_role(role_id)?;

            Self::ensure_role_manager(who.clone(), role.space_id)?;

            for user in users_set.iter() {
                if !Self::users_by_role_id(role_id).contains(user) {
                    <UsersByRoleId<T>>::mutate(role_id, |users| {
                        users.push(user.clone());
                    });
                }
                if !Self::role_ids_by_user_in_space(user.clone(), role.space_id).contains(&role_id)
                {
                    <RoleIdsByUserInSpace<T>>::mutate(user.clone(), role.space_id, |roles| {
                        roles.push(role_id);
                    })
                }
            }

            Self::deposit_event(Event::RoleGranted {
                account: who,
                role_id,
                users: users_set.iter().cloned().collect(),
            });
            Ok(())
        }

        /// Revoke a given role from a list of users.
        /// Only the space owner or a user with `ManageRoles` permission can call this dispatch.
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_role(users.len() as u32))]
        pub fn revoke_role(
            origin: OriginFor<T>,
            role_id: RoleId,
            users: Vec<User<T::AccountId>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!users.is_empty(), Error::<T>::NoUsersProvided);

            let role = Self::require_role(role_id)?;

            Self::ensure_role_manager(who.clone(), role.space_id)?;

            role.revoke_from_users(users.clone());

            Self::deposit_event(Event::RoleRevoked { account: who, role_id, users });
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight((
            Weight::from_ref_time(25_000) + T::DbWeight::get().reads_writes(1, 2),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_create_role(
            origin: OriginFor<T>,
            created: WhoAndWhenOf<T>,
            role_id: RoleId,
            space_id: SpaceId,
            disabled: bool,
            content: Content,
            permissions: SpacePermissionSet,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let WhoAndWhen { account, time, .. } = created;
            let new_who_and_when = WhoAndWhen {
                account: account.clone(),
                block: frame_system::Pallet::<T>::block_number(),
                time,
            };

            let new_role = Role::<T> {
                created: new_who_and_when,
                id: role_id,
                space_id,
                disabled,
                expires_at: None,
                content,
                permissions,
            };

            if let Ok(role) = Self::require_role(role_id) {
                if role.space_id != space_id {
                    RoleIdsBySpaceId::<T>::mutate(role.space_id, |role_ids| {
                        remove_from_vec(role_ids, role_id)
                    });
                }
            }

            RoleById::<T>::insert(role_id, new_role);
            RoleIdsBySpaceId::<T>::mutate(space_id, |role_ids| role_ids.push(role_id));

            Self::deposit_event(Event::RoleCreated { account, space_id, role_id });

            Ok(Pays::No.into())
        }

        #[pallet::call_index(6)]
        #[pallet::weight((
            Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_grant_role(
            origin: OriginFor<T>,
            role_id: RoleId,
            users: Vec<User<T::AccountId>>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let space_id = Self::require_role(role_id)?.space_id;
            let space = T::SpacePermissionsProvider::space_permissions_info(space_id)?;

            let users_set: BTreeSet<User<T::AccountId>> = convert_users_vec_to_btree_set(users)?;

            for user in users_set.iter() {
                if !Self::users_by_role_id(role_id).contains(user) {
                    <UsersByRoleId<T>>::mutate(role_id, |users| {
                        users.push(user.clone());
                    });
                }
                if !Self::role_ids_by_user_in_space(user.clone(), space_id).contains(&role_id) {
                    <RoleIdsByUserInSpace<T>>::mutate(user.clone(), space_id, |roles| {
                        roles.push(role_id);
                    })
                }
            }

            Self::deposit_event(Event::RoleGranted {
                account: space.owner,
                role_id,
                users: users_set.iter().cloned().collect(),
            });
            Ok(Pays::No.into())
        }

        #[pallet::call_index(7)]
        #[pallet::weight((
            Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_set_next_role_id(
            origin: OriginFor<T>,
            role_id: RoleId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            NextRoleId::<T>::put(role_id);
            Ok(Pays::No.into())
        }
    }
}
