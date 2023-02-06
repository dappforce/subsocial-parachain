use frame_support::dispatch::DispatchError;

use pallet_permissions::SpacePermissionsContext;
use subsocial_support::traits::RolesInterface;

use super::*;

impl<T: Config> Pallet<T> {
    /// Check that there is a `Role` with such `role_id` in the storage
    /// or return`RoleNotFound` error.
    pub fn ensure_role_exists(role_id: RoleId) -> DispatchResult {
        ensure!(<RoleById<T>>::contains_key(role_id), Error::<T>::RoleNotFound);
        Ok(())
    }

    /// Get `Role` by id from the storage or return `RoleNotFound` error.
    pub fn require_role(role_id: RoleId) -> Result<Role<T>, DispatchError> {
        Ok(Self::role_by_id(role_id).ok_or(Error::<T>::RoleNotFound)?)
    }

    /// Ensure that this account is not blocked and has 'ManageRoles' permission in a given space
    pub fn ensure_role_manager(account: T::AccountId, space_id: SpaceId) -> DispatchResult {
        ensure!(
            T::IsAccountBlocked::is_allowed_account(account.clone(), space_id),
            ModerationError::AccountIsBlocked
        );
        Self::ensure_user_has_space_permission_with_load_space(
            User::Account(account),
            space_id,
            SpacePermission::ManageRoles,
            Error::<T>::NoPermissionToManageRoles.into(),
        )
    }

    fn ensure_user_has_space_permission_with_load_space(
        user: User<T::AccountId>,
        space_id: SpaceId,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {
        let space = T::SpacePermissionsProvider::space_permissions_info(space_id)?;

        let mut is_owner = false;
        let mut is_follower = false;

        match &user {
            User::Account(account) => {
                is_owner = *account == space.owner;

                // No need to check if a user is follower, if they already are an owner:
                is_follower =
                    is_owner || T::SpaceFollows::is_space_follower(account.clone(), space_id);
            },
            User::Space(_) => (/* Not implemented yet. */),
        }

        Self::ensure_user_has_space_permission(
            user,
            SpacePermissionsContext {
                space_id,
                is_space_owner: is_owner,
                is_space_follower: is_follower,
                space_perms: space.permissions,
            },
            permission,
            error,
        )
    }

    fn ensure_user_has_space_permission(
        user: User<T::AccountId>,
        ctx: SpacePermissionsContext,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {
        match Permissions::<T>::has_user_a_space_permission(ctx.clone(), permission.clone()) {
            Some(true) => return Ok(()),
            Some(false) => return Err(error),
            _ => (/* Need to check in dynamic roles */),
        }

        Self::has_permission_in_space_roles(user, ctx.space_id, permission, error)
    }

    fn has_permission_in_space_roles(
        user: User<T::AccountId>,
        space_id: SpaceId,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {
        let role_ids = Self::role_ids_by_user_in_space(user, space_id);

        for role_id in role_ids {
            if let Some(role) = Self::role_by_id(role_id) {
                if role.disabled {
                    continue
                }

                let mut is_expired = false;
                if let Some(expires_at) = role.expires_at {
                    if expires_at <= <system::Pallet<T>>::block_number() {
                        is_expired = true;
                    }
                }

                if !is_expired && role.permissions.contains(&permission) {
                    return Ok(())
                }
            }
        }

        Err(error)
    }

    pub fn do_create_role(
        space_owner: &T::AccountId,
        space_id: SpaceId,
        time_to_live: Option<T::BlockNumber>,
        content: Content,
        permissions: Vec<SpacePermission>,
    ) -> Result<RoleId, DispatchError> {
        ensure!(!permissions.is_empty(), Error::<T>::NoPermissionsProvided);

        ensure_content_is_valid(content.clone())?;
        ensure!(
            T::IsContentBlocked::is_allowed_content(content.clone(), space_id),
            ModerationError::ContentIsBlocked,
        );

        Self::ensure_role_manager(space_owner.clone(), space_id)?;

        let permissions_set = permissions.into_iter().collect();
        let new_role =
            Role::<T>::new(space_owner.clone(), space_id, time_to_live, content, permissions_set)?;

        // TODO review strange code:
        let next_role_id = new_role.id.checked_add(1).ok_or(Error::<T>::RoleIdOverflow)?;
        NextRoleId::<T>::put(next_role_id);

        RoleById::<T>::insert(new_role.id, new_role.clone());
        RoleIdsBySpaceId::<T>::mutate(space_id, |role_ids| role_ids.push(new_role.id));

        Ok(new_role.id)
    }

    pub fn do_grant_role(
        manager: Option<T::AccountId>,
        role_id: RoleId,
        users: Vec<User<T::AccountId>>,
    ) -> DispatchResult {
        let users_set: BTreeSet<User<T::AccountId>> = convert_users_vec_to_btree_set(users)?;

        let role = Self::require_role(role_id)?;

        if let Some(who) = manager.clone() {
            Self::ensure_role_manager(who, role.space_id)?;
        }

        for user in users_set.iter() {
            if !Self::users_by_role_id(role_id).contains(user) {
                <UsersByRoleId<T>>::mutate(role_id, |users| {
                    users.push(user.clone());
                });
            }
            if !Self::role_ids_by_user_in_space(user.clone(), role.space_id).contains(&role_id) {
                <RoleIdsByUserInSpace<T>>::mutate(user.clone(), role.space_id, |roles| {
                    roles.push(role_id);
                })
            }
        }

        Self::deposit_event(Event::RoleGranted {
            account: manager.unwrap_or_else(|| {
                T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()
            }),
            role_id,
            users: users_set.iter().cloned().collect(),
        });

        Ok(())
    }
}

impl<T: Config> Role<T> {
    pub fn new(
        created_by: T::AccountId,
        space_id: SpaceId,
        time_to_live: Option<T::BlockNumber>,
        content: Content,
        permissions: BTreeSet<SpacePermission>,
    ) -> Result<Self, DispatchError> {
        let role_id = Pallet::<T>::next_role_id();

        let mut expires_at: Option<T::BlockNumber> = None;
        if let Some(ttl) = time_to_live {
            expires_at = Some(ttl + <system::Pallet<T>>::block_number());
        }

        let new_role = Role::<T> {
            created: new_who_and_when::<T>(created_by),
            id: role_id,
            space_id,
            disabled: false,
            expires_at,
            content,
            permissions,
        };

        Ok(new_role)
    }

    pub fn set_disabled(&mut self, disable: bool) -> DispatchResult {
        if self.disabled && disable {
            return Err(Error::<T>::RoleAlreadyDisabled.into())
        } else if !self.disabled && !disable {
            return Err(Error::<T>::RoleAlreadyEnabled.into())
        }

        self.disabled = disable;

        Ok(())
    }

    pub fn revoke_from_users(&self, users: Vec<User<T::AccountId>>) {
        let mut users_by_role = <UsersByRoleId<T>>::take(self.id);

        for user in users.iter() {
            let role_idx_by_user_opt = Pallet::<T>::role_ids_by_user_in_space(&user, self.space_id)
                .iter()
                .position(|x| *x == self.id);

            if let Some(role_idx) = role_idx_by_user_opt {
                <RoleIdsByUserInSpace<T>>::mutate(user, self.space_id, |n| n.swap_remove(role_idx));
            }

            let user_idx_by_role_opt = users_by_role.iter().position(|x| x == user);

            if let Some(user_idx) = user_idx_by_role_opt {
                users_by_role.swap_remove(user_idx);
            }
        }
        <UsersByRoleId<T>>::insert(self.id, users_by_role);
    }
}

impl<T: Config> PermissionChecker for Pallet<T> {
    type AccountId = T::AccountId;

    fn ensure_user_has_space_permission(
        user: User<Self::AccountId>,
        ctx: SpacePermissionsContext,
        permission: SpacePermission,
        error: DispatchError,
    ) -> DispatchResult {
        Self::ensure_user_has_space_permission(user, ctx, permission, error)
    }
}

impl<T: Config> RolesInterface<RoleId, SpaceId, T::AccountId, SpacePermission, T::BlockNumber>
    for Pallet<T>
{
    fn get_role_space(role_id: RoleId) -> Result<SpaceId, DispatchError> {
        let role = Pallet::<T>::require_role(role_id)?;
        Ok(role.space_id)
    }

    fn grant_role(account_id: T::AccountId, role_id: RoleId) -> DispatchResult {
        Pallet::<T>::do_grant_role(None, role_id, vec![User::Account(account_id)])
    }

    fn create_role(
        space_owner: &T::AccountId,
        space_id: SpaceId,
        time_to_live: Option<T::BlockNumber>,
        content: Content,
        permissions: Vec<SpacePermission>,
    ) -> Result<RoleId, DispatchError> {
        Pallet::<T>::do_create_role(space_owner, space_id, time_to_live, content, permissions)
    }
}
