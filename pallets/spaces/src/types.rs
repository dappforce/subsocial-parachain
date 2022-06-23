use frame_support::pallet_prelude::*;

use pallet_parachain_utils::{WhoAndWhenOf, new_who_and_when};

use super::*;

pub const FIRST_SPACE_ID: u64 = 1;
pub const RESERVED_SPACE_COUNT: u64 = 1000;
pub const DEFAULT_MAX_HANDLE_LEN: u32 = 50;

pub(crate) type Handle<T> = BoundedVec<u8, <T as Config>::MaxHandleLen>;
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

    /// Unique alpha-numeric identifier that can be used in a space's URL.
    /// Handle can only contain numbers, letter and underscore: `0`-`9`, `a`-`z`, `_`.
    pub handle: Option<Handle<T>>,

    pub content: Content,

    /// Hidden field is used to recommend to end clients (web and mobile apps) that a particular
    /// space and its' posts should not be shown.
    pub hidden: bool,

    /// The total number of posts in a given space.
    pub posts_count: u32,

    /// The number of hidden posts in a given space.
    pub hidden_posts_count: u32,

    /// The number of account following a given space.
    pub followers_count: u32,

    pub(super) score: i32,

    /// This allows you to override Subsocial's default permissions by enabling or disabling role
    /// permissions.
    pub permissions: Option<SpacePermissions>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, RuntimeDebug, TypeInfo)]
pub struct SpaceUpdate {
    pub parent_id: Option<Option<SpaceId>>,
    pub handle: Option<Option<Vec<u8>>>,
    pub content: Option<Content>,
    pub hidden: Option<bool>,
    pub permissions: Option<Option<SpacePermissions>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SpacesSettings {
    pub handles_enabled: bool,
}

impl Default for SpacesSettings {
    fn default() -> Self {
        Self {
            handles_enabled: true,
        }
    }
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
            handle: Default::default(),
            content,
            hidden: false,
            posts_count: 0,
            hidden_posts_count: 0,
            followers_count: 0,
            score: 0,
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

    pub fn inc_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_add(1);
    }

    pub fn dec_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_sub(1);
    }

    pub fn inc_hidden_posts(&mut self) {
        self.hidden_posts_count = self.hidden_posts_count.saturating_add(1);
    }

    pub fn dec_hidden_posts(&mut self) {
        self.hidden_posts_count = self.hidden_posts_count.saturating_sub(1);
    }

    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
    }

    pub fn try_get_parent(&self) -> Result<SpaceId, DispatchError> {
        self.parent_id
            .ok_or_else(|| Error::<T>::SpaceIsAtRoot.into())
    }

    pub fn is_public(&self) -> bool {
        !self.hidden && self.content.is_some()
    }

    pub fn is_unlisted(&self) -> bool {
        !self.is_public()
    }
}
