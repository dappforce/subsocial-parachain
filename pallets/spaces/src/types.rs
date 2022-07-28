use frame_support::pallet_prelude::*;

use subsocial_support::{new_who_and_when, WhoAndWhenOf};

use super::*;

pub const FIRST_SPACE_ID: u64 = 1;
pub const RESERVED_SPACE_COUNT: u64 = 1000;

pub(crate) type SpacesByAccount<T> = BoundedVec<SpaceId, <T as Config>::MaxSpacesPerAccount>;

/// Information about a space's owner, its' content, visibility and custom permissions.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Space<T: Config> {
    /// Unique sequential identifier of a space. Examples of space ids: `1`, `2`, `3`, and so on.
    pub id: SpaceId,

    pub created: WhoAndWhenOf<T>,
    pub updated: Option<WhoAndWhenOf<T>>,

    /// The current owner of a given space.
    pub owner: T::AccountId,

    // The next fields can be updated by the owner:
    pub(super) parent_id: Option<SpaceId>,

    pub content: Content,

    /// Hidden field is used to recommend to end clients (web and mobile apps) that a particular
    /// space and its' posts should not be shown.
    pub hidden: bool,

    /// This allows you to override Subsocial's default permissions by enabling or disabling role
    /// permissions.
    pub permissions: Option<SpacePermissions>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, RuntimeDebug, TypeInfo)]
pub struct SpaceUpdate {
    pub parent_id: Option<Option<SpaceId>>,
    pub content: Option<Content>,
    pub hidden: Option<bool>,
    pub permissions: Option<Option<SpacePermissions>>,
}

impl<T: Config> Space<T> {
    pub fn new(
        id: SpaceId,
        parent_id: Option<SpaceId>,
        created_by: T::AccountId,
        content: Content,
        permissions: Option<SpacePermissions>,
    ) -> Self {
        Space {
            id,
            created: new_who_and_when::<T>(created_by.clone()),
            updated: None,
            owner: created_by,
            parent_id,
            content,
            hidden: false,
            permissions,
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn is_follower(&self, account: &T::AccountId) -> bool {
        T::SpaceFollows::is_space_follower(account.clone(), self.id)
    }

    pub fn ensure_space_owner(&self, account: T::AccountId) -> DispatchResult {
        ensure!(self.is_owner(&account), Error::<T>::NotASpaceOwner);
        Ok(())
    }

    pub fn try_get_parent(&self) -> Result<SpaceId, DispatchError> {
        self.parent_id.ok_or_else(|| Error::<T>::SpaceIsAtRoot.into())
    }

    pub fn is_public(&self) -> bool {
        !self.hidden && self.content.is_some()
    }

    pub fn is_unlisted(&self) -> bool {
        !self.is_public()
    }
}
