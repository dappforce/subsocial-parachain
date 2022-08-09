use pallet_permissions::{
    SpacePermissions,
    SpacePermission,
    SpacePermission as SP,
};
use pallet_permissions::default_permissions::DefaultSpacePermissions;

pub(crate) fn permissions_where_everyone_can_create_post() -> SpacePermissions {
    let mut default_permissions = DefaultSpacePermissions::get();
    default_permissions.everyone = default_permissions.everyone
        .map(|mut permissions| {
            permissions.insert(SP::CreatePosts);
            permissions
        });

    default_permissions
}

pub(crate) fn permissions_where_follower_can_create_post() -> SpacePermissions {
    let mut default_permissions = DefaultSpacePermissions::get();
    default_permissions.follower = Some(vec![SP::CreatePosts].into_iter().collect());

    default_permissions
}


/// Permissions Set that includes next permission: ManageRoles
pub(crate) fn permission_set_default() -> Vec<SpacePermission> {
    vec![SP::ManageRoles]
}