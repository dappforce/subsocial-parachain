use super::*;

pub type RoleId = u64;

pub const FIRST_ROLE_ID: u64 = 1;

/// Information about a role's permissions, its' containing space, and its' content.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Role<T: Config> {
    pub created: WhoAndWhenOf<T>,

    /// Unique sequential identifier of a role. Examples of role ids: `1`, `2`, `3`, and so on.
    pub id: RoleId,

    /// An id of a space that contains this role.
    pub space_id: SpaceId,

    /// If `true` then the permissions associated with a given role will have no affect.
    /// This is useful if you would like to temporarily disable permissions from a given role,
    /// without removing the role from its' owners
    pub disabled: bool,

    /// An optional block number at which this role will expire. If `expires_at` is `Some`
    /// and the current block is greater or equal to its value, the permissions associated
    /// with a given role will have no affect.
    pub expires_at: Option<T::BlockNumber>,

    /// Content can optionally contain additional information associated with a role,
    /// such as a name, description, and image for a role. This may be useful for end users.
    pub content: Content,

    /// A set of permisions granted to owners of a particular role which are valid
    /// only within the space containing this role
    pub permissions: SpacePermissionSet,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct RoleUpdate {
    pub disabled: Option<bool>,
    pub content: Option<Content>,
    pub permissions: Option<SpacePermissionSet>,
}
