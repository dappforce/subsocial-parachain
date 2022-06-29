#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_std::collections::btree_set::BTreeSet;

pub mod default_permissions;
mod types;

pub use types::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        type DefaultSpacePermissions: Get<SpacePermissions>;
    }

    impl<T: Config> Pallet<T> {
        fn get_overrides_or_defaults(
            overrides: Option<SpacePermissionSet>,
            defaults: Option<SpacePermissionSet>,
        ) -> Option<SpacePermissionSet> {
            if overrides.is_some() {
                overrides
            } else {
                defaults
            }
        }

        fn resolve_space_perms(space_perms: Option<SpacePermissions>) -> SpacePermissions {
            let defaults = T::DefaultSpacePermissions::get();
            let overrides = space_perms.unwrap_or_default();

            SpacePermissions {
                none: Self::get_overrides_or_defaults(overrides.none, defaults.none),
                everyone: Self::get_overrides_or_defaults(overrides.everyone, defaults.everyone),
                follower: Self::get_overrides_or_defaults(overrides.follower, defaults.follower),
                space_owner: Self::get_overrides_or_defaults(
                    overrides.space_owner,
                    defaults.space_owner,
                ),
            }
        }

        pub fn has_user_a_space_permission(
            ctx: SpacePermissionsContext,
            permission: SpacePermission,
        ) -> Option<bool> {
            let perms_by_role = Self::resolve_space_perms(ctx.space_perms);

            // Check if this permission is forbidden:
            if permission.is_present_in_role(perms_by_role.none) {
                return Some(false)
            }

            let is_space_owner = ctx.is_space_owner;
            let is_follower = is_space_owner || ctx.is_space_follower;

            if permission.is_present_in_role(perms_by_role.everyone) ||
                is_follower && permission.is_present_in_role(perms_by_role.follower) ||
                is_space_owner && permission.is_present_in_role(perms_by_role.space_owner)
            {
                return Some(true)
            }

            None
        }

        pub fn override_permissions(mut overrides: SpacePermissions) -> SpacePermissions {
            overrides.none = overrides.none.map(|mut none_permissions_set| {
                none_permissions_set
                    .extend(T::DefaultSpacePermissions::get().none.unwrap_or_default());
                none_permissions_set
            });

            overrides
        }
    }
}
