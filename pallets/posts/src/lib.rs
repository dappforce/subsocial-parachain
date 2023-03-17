//! # Posts Module
//!
//! Posts are the second crucial component of Subsocial after Spaces. This module allows you to
//! create, update, move (between spaces), and hide posts as well as manage owner(s).
//!
//! Posts can be compared to existing entities on web 2.0 platforms such as:
//! - Posts on Facebook,
//! - Tweets on Twitter,
//! - Images on Instagram,
//! - Articles on Medium,
//! - Shared links on Reddit,
//! - Questions and answers on Stack Overflow.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure, fail,
    traits::Get,
};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use pallet_permissions::SpacePermission;
use pallet_spaces::{types::Space, Pallet as Spaces};
use subsocial_support::{
    ensure_content_is_valid, new_who_and_when, remove_from_vec,
    traits::{IsAccountBlocked, IsContentBlocked, IsPostBlocked},
    Content, ModerationError, PostId, SpaceId, WhoAndWhen, WhoAndWhenOf,
};

pub use pallet::*;
pub mod functions;

pub mod types;
pub use types::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

// pub mod rpc;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    use crate::weights::WeightInfo;
    use frame_support::{pallet_prelude::*, traits::IsType};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_space_follows::Config
        + pallet_spaces::Config
        + pallet_timestamp::Config
    {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Max comments depth
        #[pallet::constant]
        type MaxCommentDepth: Get<u32>;

        type IsPostBlocked: IsPostBlocked<PostId>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::type_value]
    pub fn DefaultForNextPostId() -> PostId {
        FIRST_POST_ID
    }

    /// The next post id.
    #[pallet::storage]
    #[pallet::getter(fn next_post_id)]
    pub type NextPostId<T: Config> = StorageValue<_, PostId, ValueQuery, DefaultForNextPostId>;

    /// Get the details of a post by its' id.
    #[pallet::storage]
    #[pallet::getter(fn post_by_id)]
    pub type PostById<T: Config> = StorageMap<_, Twox64Concat, PostId, Post<T>>;

    /// Get the ids of all direct replies by their parent's post id.
    #[pallet::storage]
    #[pallet::getter(fn reply_ids_by_post_id)]
    pub type ReplyIdsByPostId<T: Config> =
        StorageMap<_, Twox64Concat, PostId, Vec<PostId>, ValueQuery>;

    /// Get the ids of all posts in a given space, by the space's id.
    #[pallet::storage]
    #[pallet::getter(fn post_ids_by_space_id)]
    pub type PostIdsBySpaceId<T: Config> =
        StorageMap<_, Twox64Concat, SpaceId, Vec<PostId>, ValueQuery>;

    /// Get the ids of all posts that have shared a given original post id.
    #[pallet::storage]
    #[pallet::getter(fn shared_post_ids_by_original_post_id)]
    pub type SharedPostIdsByOriginalPostId<T: Config> =
        StorageMap<_, Twox64Concat, PostId, Vec<PostId>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        PostCreated {
            account: T::AccountId,
            post_id: PostId,
        },
        PostUpdated {
            account: T::AccountId,
            post_id: PostId,
        },
        PostMoved {
            account: T::AccountId,
            post_id: PostId,
            from_space: Option<SpaceId>,
            to_space: Option<SpaceId>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        // Post related errors:
        /// Post was not found by id.
        PostNotFound,
        /// An account is not a post owner.
        NotAPostOwner,
        /// Nothing to update in this post.
        NoUpdatesForPost,
        /// Root post should have a space id.
        PostHasNoSpaceId,
        /// Not allowed to create a post/comment when a scope (space or root post) is hidden.
        CannotCreateInHiddenScope,
        /// Post has no replies.
        NoRepliesOnPost,
        /// Cannot move a post to the same space.
        CannotMoveToSameSpace,

        // Share related errors:
        /// Cannot share, because the original post was not found.
        OriginalPostNotFound,
        /// Cannot share a post that is sharing another post.
        CannotShareSharedPost,
        /// This post's extension is not a `SharedPost`.
        NotASharedPost,

        // Comment related errors:
        /// Unknown parent comment id.
        UnknownParentComment,
        /// Post by `parent_id` is not of a `Comment` extension.
        NotACommentByParentId,
        /// Cannot update space id of a comment.
        CannotUpdateSpaceIdOnComment,
        /// Max comment depth reached.
        MaxCommentDepthReached,
        /// Only comment owner can update this comment.
        NotACommentAuthor,
        /// This post's extension is not a `Comment`.
        NotComment,

        // Permissions related errors:
        /// User has no permission to create root posts in this space.
        NoPermissionToCreatePosts,
        /// User has no permission to create comments (aka replies) in this space.
        NoPermissionToCreateComments,
        /// User has no permission to share posts/comments from this space to another space.
        NoPermissionToShare,
        /// User has no permission to update any posts in this space.
        NoPermissionToUpdateAnyPost,
        /// A post owner is not allowed to update their own posts in this space.
        NoPermissionToUpdateOwnPosts,
        /// A comment owner is not allowed to update their own comments in this space.
        NoPermissionToUpdateOwnComments,

        /// `force_create_post` failed, because this post already exists.
        /// Consider removing the post with `force_remove_post` first.
        PostAlreadyExists,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(
            match extension {
                PostExtension::RegularPost => <T as Config>::WeightInfo::create_post__regular(),
                PostExtension::Comment(..) => <T as Config>::WeightInfo::create_post__comment(),
                PostExtension::SharedPost(..) => <T as Config>::WeightInfo::create_post__shared(),
            }
        )]
        pub fn create_post(
            origin: OriginFor<T>,
            space_id_opt: Option<SpaceId>,
            extension: PostExtension,
            content: Content,
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;

            ensure_content_is_valid(content.clone())?;

            let new_post_id = Self::next_post_id();
            let new_post: Post<T> =
                Post::new(new_post_id, creator.clone(), space_id_opt, extension, content.clone());

            // Get space from either space_id_opt or Comment if a comment provided
            let space = &new_post.get_space()?;
            ensure!(!space.hidden, Error::<T>::CannotCreateInHiddenScope);

            ensure!(
                T::IsAccountBlocked::is_allowed_account(creator.clone(), space.id),
                ModerationError::AccountIsBlocked
            );
            ensure!(
                T::IsContentBlocked::is_allowed_content(content, space.id),
                ModerationError::ContentIsBlocked
            );

            let root_post = &mut new_post.get_root_post()?;
            ensure!(!root_post.hidden, Error::<T>::CannotCreateInHiddenScope);

            // Check whether account has permission to create Post (by extension)
            let mut permission_to_check = SpacePermission::CreatePosts;
            let mut error_on_permission_failed = Error::<T>::NoPermissionToCreatePosts;

            if let PostExtension::Comment(_) = extension {
                permission_to_check = SpacePermission::CreateComments;
                error_on_permission_failed = Error::<T>::NoPermissionToCreateComments;
            }

            Spaces::ensure_account_has_space_permission(
                creator.clone(),
                space,
                permission_to_check,
                error_on_permission_failed.into(),
            )?;

            match extension {
                PostExtension::SharedPost(original_post_id) =>
                    Self::create_shared_post(&creator, new_post_id, original_post_id)?,
                PostExtension::Comment(comment_ext) =>
                    Self::create_comment(new_post_id, comment_ext, root_post.id)?,
                _ => (),
            }

            if new_post.is_root_post() {
                PostIdsBySpaceId::<T>::mutate(space.id, |ids| ids.push(new_post_id));
            }

            PostById::insert(new_post_id, new_post);
            NextPostId::<T>::mutate(|n| {
                *n += 1;
            });

            Self::deposit_event(Event::PostCreated { account: creator, post_id: new_post_id });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_post())]
        pub fn update_post(
            origin: OriginFor<T>,
            post_id: PostId,
            update: PostUpdate,
        ) -> DispatchResult {
            let editor = ensure_signed(origin)?;

            let has_updates = update.content.is_some() || update.hidden.is_some();

            ensure!(has_updates, Error::<T>::NoUpdatesForPost);

            let mut post = Self::require_post(post_id)?;
            let space_opt = &post.try_get_space();

            if let Some(space) = space_opt {
                ensure!(
                    T::IsAccountBlocked::is_allowed_account(editor.clone(), space.id),
                    ModerationError::AccountIsBlocked
                );
                Self::ensure_account_can_update_post(&editor, &post, space)?;
            }

            let mut is_update_applied = false;

            if let Some(content) = update.content {
                if content != post.content {
                    ensure_content_is_valid(content.clone())?;

                    if let Some(space) = space_opt {
                        ensure!(
                            T::IsContentBlocked::is_allowed_content(content.clone(), space.id),
                            ModerationError::ContentIsBlocked
                        );
                    }

                    post.content = content;
                    post.edited = true;
                    is_update_applied = true;
                }
            }

            if let Some(hidden) = update.hidden {
                if hidden != post.hidden {
                    post.hidden = hidden;
                    is_update_applied = true;
                }
            }

            // Update this post only if at least one field should be updated:
            if is_update_applied {
                <PostById<T>>::insert(post.id, post);
                Self::deposit_event(Event::PostUpdated { account: editor, post_id });
            }
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::move_post())]
        pub fn move_post(
            origin: OriginFor<T>,
            post_id: PostId,
            new_space_id: Option<SpaceId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let post = &mut Self::require_post(post_id)?;

            ensure!(new_space_id != post.space_id, Error::<T>::CannotMoveToSameSpace);

            if let Some(space) = post.try_get_space() {
                Self::ensure_account_can_update_post(&who, post, &space)?;
            } else {
                post.ensure_owner(&who)?;
            }

            let old_space_id = post.space_id;

            if let Some(space_id) = new_space_id {
                Self::move_post_to_space(who.clone(), post, space_id)?;
            } else {
                Self::delete_post_from_space(post_id)?;
            }

            Self::deposit_event(Event::PostMoved {
                account: who,
                post_id,
                from_space: old_space_id,
                to_space: new_space_id,
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight((
            Weight::from_ref_time(50_000) + T::DbWeight::get().reads_writes(4, 3),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_create_post(
            origin: OriginFor<T>,
            post_id: PostId,
            created: WhoAndWhenOf<T>,
            owner: T::AccountId,
            extension: PostExtension,
            space_id_opt: Option<SpaceId>,
            content: Content,
            hidden: bool,
            upvotes_count: u32,
            downvotes_count: u32,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            ensure!(Self::require_post(post_id).is_err(), Error::<T>::PostAlreadyExists);

            let WhoAndWhen { account, time, .. } = created;
            let new_who_and_when =
                WhoAndWhen { account, block: frame_system::Pallet::<T>::block_number(), time };

            let new_post = Post::<T> {
                id: post_id,
                created: new_who_and_when,
                edited: false,
                owner: owner.clone(),
                extension,
                space_id: space_id_opt,
                content,
                hidden,
                upvotes_count,
                downvotes_count,
            };

            if new_post.is_root_post() {
                if let Some(space_id) = new_post.space_id {
                    PostIdsBySpaceId::<T>::mutate(space_id, |ids| ids.push(post_id));
                }
            }

            match new_post.extension {
                PostExtension::Comment(ext) => {
                    let commented_post_id = ext.parent_id.unwrap_or(ext.root_post_id);
                    ReplyIdsByPostId::<T>::mutate(commented_post_id, |reply_ids| {
                        reply_ids.push(post_id)
                    });
                },
                PostExtension::SharedPost(original_post_id) => {
                    SharedPostIdsByOriginalPostId::<T>::mutate(original_post_id, |ids| {
                        ids.push(post_id)
                    });
                },
                _ => (),
            }

            PostById::insert(post_id, new_post);

            Self::deposit_event(Event::PostCreated { account: owner, post_id });
            Ok(Pays::No.into())
        }

        #[pallet::call_index(4)]
        #[pallet::weight((
            Weight::from_ref_time(10_000) + T::DbWeight::get().reads_writes(2, 3),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_remove_post(
            origin: OriginFor<T>,
            post_id: PostId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            if let Ok(old_post) = Self::require_post(post_id) {
                if old_post.is_root_post() {
                    if let Some(space_id) = old_post.space_id {
                        PostIdsBySpaceId::<T>::mutate(space_id, |ids| {
                            remove_from_vec(ids, post_id)
                        });
                    }
                }

                match old_post.extension {
                    PostExtension::Comment(ext) => {
                        let commented_post_id = ext.parent_id.unwrap_or(ext.root_post_id);
                        ReplyIdsByPostId::<T>::mutate(commented_post_id, |reply_ids| {
                            remove_from_vec(reply_ids, post_id)
                        });
                    },
                    PostExtension::SharedPost(original_post_id) => {
                        SharedPostIdsByOriginalPostId::<T>::mutate(original_post_id, |ids| {
                            remove_from_vec(ids, post_id)
                        });
                    },
                    _ => (),
                }
                PostById::<T>::remove(post_id);
            }

            Ok(Pays::No.into())
        }

        #[pallet::call_index(5)]
        #[pallet::weight((
            Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1),
            DispatchClass::Operational,
            Pays::Yes,
        ))]
        pub fn force_set_next_post_id(
            origin: OriginFor<T>,
            post_id: PostId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            NextPostId::<T>::put(post_id);
            Ok(Pays::No.into())
        }
    }
}
