use super::*;

pub const FIRST_POST_ID: u64 = 1;

/// Information about a post's owner, its' related space, content, and visibility.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Post<T: Config> {
    /// Unique sequential identifier of a post. Examples of post ids: `1`, `2`, `3`, and so on.
    pub id: PostId,

    pub created: WhoAndWhenOf<T>,
    pub updated: Option<WhoAndWhenOf<T>>,

    /// The current owner of a given post.
    pub owner: T::AccountId,

    /// Through post extension you can provide specific information necessary for different kinds
    /// of posts such as regular posts, comments, and shared posts.
    pub extension: PostExtension,

    /// An id of a space which contains a given post.
    pub space_id: Option<SpaceId>,

    pub content: Content,

    /// Hidden field is used to recommend to end clients (web and mobile apps) that a particular
    /// posts and its' comments should not be shown.
    pub hidden: bool,

    /// The total number of replies for a given post.
    pub replies_count: u16,

    /// The number of hidden replies for a given post.
    pub hidden_replies_count: u16,

    /// The number of times a given post has been shared.
    pub shares_count: u16,

    /// The number of times a given post has been upvoted.
    pub upvotes_count: u16,

    /// The number of times a given post has been downvoted.
    pub downvotes_count: u16,

    pub score: i32,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PostUpdate {
    /// Deprecated: This field has no effect in `fn update_post()` extrinsic.
    /// See `fn move_post()` extrinsic if you want to move a post to another space.
    pub space_id: Option<SpaceId>,

    pub content: Option<Content>,
    pub hidden: Option<bool>,
}

/// Post extension provides specific information necessary for different kinds
/// of posts such as regular posts, comments, and shared posts.
#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(untagged))]
pub enum PostExtension {
    RegularPost,
    Comment(Comment),
    SharedPost(PostId),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Comment {
    pub parent_id: Option<PostId>,
    pub root_post_id: PostId,
}

impl Default for PostExtension {
    fn default() -> Self {
        PostExtension::RegularPost
    }
}