use frame_support::dispatch::DispatchResult;
use sp_runtime::traits::Saturating;

use subsocial_support::{remove_from_vec, SpaceId};

use super::*;

impl<T: Config> Post<T> {
    pub fn new(
        id: PostId,
        created_by: T::AccountId,
        space_id_opt: Option<SpaceId>,
        extension: PostExtension,
        content: Content,
    ) -> Self {
        Post {
            id,
            created: new_who_and_when::<T>(created_by.clone()),
            updated: None,
            owner: created_by,
            extension,
            space_id: space_id_opt,
            content,
            hidden: false,
            upvotes_count: 0,
            downvotes_count: 0,
        }
    }

    pub fn ensure_owner(&self, account: &T::AccountId) -> DispatchResult {
        ensure!(self.is_owner(account), Error::<T>::NotAPostOwner);
        Ok(())
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn is_root_post(&self) -> bool {
        !self.is_comment()
    }

    pub fn is_regular_post(&self) -> bool {
        matches!(self.extension, PostExtension::RegularPost(_))
    }

    pub fn is_comment(&self) -> bool {
        matches!(self.extension, PostExtension::Comment(_))
    }

    pub fn is_shared_post(&self) -> bool {
        matches!(self.extension, PostExtension::SharedPost(_))
    }

    pub fn get_comment_ext(&self) -> Result<Comment, DispatchError> {
        match self.extension {
            PostExtension::Comment(comment_ext) => Ok(comment_ext),
            _ => Err(Error::<T>::NotComment.into()),
        }
    }

    pub fn get_original_post_id(&self) -> Result<PostId, DispatchError> {
        match self.extension {
            PostExtension::SharedPost(shared_ext) => Ok(shared_ext.original_post_id),
            _ => Err(Error::<T>::NotASharedPost.into()),
        }
    }

    pub fn get_root_post(&self) -> Result<Post<T>, DispatchError> {
        match self.extension {
            PostExtension::RegularPost(_) | PostExtension::SharedPost(_) => Ok(self.clone()),
            PostExtension::Comment(comment) => Pallet::<T>::require_post(comment.root_post_id),
        }
    }

    pub fn get_space_id(&self) -> Result<SpaceId, DispatchError> {
        Self::try_get_space_id(self).ok_or_else(|| Error::<T>::PostHasNoSpaceId.into())
    }

    pub fn try_get_space_id(&self) -> Option<SpaceId> {
        if let Ok(root_post) = self.get_root_post() {
            return root_post.space_id
        }

        None
    }

    pub fn get_space(&self) -> Result<Space<T>, DispatchError> {
        let root_post = self.get_root_post()?;
        let space_id = root_post.space_id.ok_or(Error::<T>::PostHasNoSpaceId)?;
        Spaces::require_space(space_id)
    }

    pub fn try_get_space(&self) -> Option<Space<T>> {
        if let Ok(root_post) = self.get_root_post() {
            return root_post.space_id.and_then(|space_id| Spaces::require_space(space_id).ok())
        }

        None
    }

    pub fn try_get_parent_id(&self) -> Option<PostId> {
        match self.extension {
            PostExtension::Comment(comment_ext) => comment_ext.parent_id,
            _ => None,
        }
    }

    pub fn inc_replies(&mut self) {
        match &mut self.extension {
            PostExtension::RegularPost(ext) => ext.total_replies_count.saturating_inc(),
            PostExtension::SharedPost(ext) => ext.total_replies_count.saturating_inc(),
            PostExtension::Comment(ext) => ext.direct_replies_count.saturating_inc(),
        }
    }

    pub fn dec_replies(&mut self) {
        match &mut self.extension {
            PostExtension::RegularPost(ext) => ext.total_replies_count.saturating_dec(),
            PostExtension::SharedPost(ext) => ext.total_replies_count.saturating_dec(),
            PostExtension::Comment(ext) => ext.direct_replies_count.saturating_dec(),
        }
    }

    pub fn inc_upvotes(&mut self) {
        self.upvotes_count = self.upvotes_count.saturating_add(1);
    }

    pub fn dec_upvotes(&mut self) {
        self.upvotes_count = self.upvotes_count.saturating_sub(1);
    }

    pub fn inc_downvotes(&mut self) {
        self.downvotes_count = self.downvotes_count.saturating_add(1);
    }

    pub fn dec_downvotes(&mut self) {
        self.downvotes_count = self.downvotes_count.saturating_sub(1);
    }

    pub fn is_public(&self) -> bool {
        !self.hidden && self.content.is_some()
    }

    pub fn is_unlisted(&self) -> bool {
        !self.is_public()
    }
}

impl<T: Config> Pallet<T> {
    pub fn ensure_account_can_update_post(
        editor: &T::AccountId,
        post: &Post<T>,
        space: &Space<T>,
    ) -> DispatchResult {
        let is_owner = post.is_owner(editor);
        let is_comment = post.is_comment();

        let permission_to_check: SpacePermission;
        let permission_error: DispatchError;

        if is_comment {
            if is_owner {
                permission_to_check = SpacePermission::UpdateOwnComments;
                permission_error = Error::<T>::NoPermissionToUpdateOwnComments.into();
            } else {
                fail!(Error::<T>::NotACommentAuthor);
            }
        } else {
            // Not a comment

            if is_owner {
                permission_to_check = SpacePermission::UpdateOwnPosts;
                permission_error = Error::<T>::NoPermissionToUpdateOwnPosts.into();
            } else {
                permission_to_check = SpacePermission::UpdateAnyPost;
                permission_error = Error::<T>::NoPermissionToUpdateAnyPost.into();
            }
        }

        Spaces::ensure_account_has_space_permission(
            editor.clone(),
            space,
            permission_to_check,
            permission_error,
        )
    }

    /// Check that there is a `Post` with such `post_id` in the storage
    /// or return`PostNotFound` error.
    pub fn ensure_post_exists(post_id: PostId) -> DispatchResult {
        ensure!(PostById::<T>::contains_key(post_id), Error::<T>::PostNotFound);
        Ok(())
    }

    /// Get `Post` by id from the storage or return `PostNotFound` error.
    pub fn require_post(post_id: SpaceId) -> Result<Post<T>, DispatchError> {
        Ok(Self::post_by_id(post_id).ok_or(Error::<T>::PostNotFound)?)
    }

    fn share_post(
        account: T::AccountId,
        original_post_id: PostId,
        shared_post_id: PostId,
    ) -> DispatchResult {
        SharedPostIdsByOriginalPostId::<T>::mutate(original_post_id, |ids| {
            ids.push(shared_post_id)
        });

        Self::deposit_event(Event::PostShared { account, original_post_id, shared_post_id });

        Ok(())
    }

    pub fn is_root_post_hidden(post_id: PostId) -> Result<bool, DispatchError> {
        let post = Self::require_post(post_id)?;
        let root_post = post.get_root_post()?;
        Ok(root_post.hidden)
    }

    pub fn is_root_post_visible(post_id: PostId) -> Result<bool, DispatchError> {
        Self::is_root_post_hidden(post_id).map(|v| !v)
    }

    pub fn mutate_post_by_id<F: FnOnce(&mut Post<T>)>(
        post_id: PostId,
        f: F,
    ) -> Result<Post<T>, DispatchError> {
        PostById::<T>::mutate(post_id, |post_opt| {
            if let Some(ref mut post) = post_opt.clone() {
                f(post);
                *post_opt = Some(post.clone());

                return Ok(post.clone())
            }

            Err(Error::<T>::PostNotFound.into())
        })
    }

    // TODO refactor to a tail recursion
    /// Get all post ancestors (parent_id) including this post
    pub fn get_post_ancestors(post_id: PostId) -> Vec<Post<T>> {
        let mut ancestors: Vec<Post<T>> = Vec::new();

        if let Some(post) = Self::post_by_id(post_id) {
            ancestors.push(post.clone());
            if let Some(parent_id) = post.get_comment_ext().ok().unwrap().parent_id {
                ancestors.extend(Self::get_post_ancestors(parent_id).iter().cloned());
            }
        }

        ancestors
    }

    pub fn try_get_post_replies(post_id: PostId) -> Vec<Post<T>> {
        let mut replies: Vec<Post<T>> = Vec::new();

        if let Some(post) = Self::post_by_id(post_id) {
            replies.push(post);
            for reply_id in Self::reply_ids_by_post_id(post_id).iter() {
                replies.extend(Self::try_get_post_replies(*reply_id).iter().cloned());
            }
        }

        replies
    }

    /// Recursively get all nested post replies (reply_ids_by_post_id)
    pub fn get_post_replies(post_id: PostId) -> Result<Vec<Post<T>>, DispatchError> {
        let reply_ids = Self::reply_ids_by_post_id(post_id);
        ensure!(!reply_ids.is_empty(), Error::<T>::NoRepliesOnPost);

        let mut replies: Vec<Post<T>> = Vec::new();
        for reply_id in reply_ids.iter() {
            replies.extend(Self::try_get_post_replies(*reply_id));
        }
        Ok(replies)
    }

    fn get_nested_replies(reply_ids: Vec<PostId>) -> u32 {
        reply_ids.iter().fold(0, |acc, reply_id| {
            acc + Self::get_nested_replies(Self::reply_ids_by_post_id(*reply_id))
        })
    }

    pub fn get_post_replies_count_by_id(post_id: PostId) -> RepliesCount {
        Self::get_nested_replies(Self::reply_ids_by_post_id(post_id))
    }

    pub(crate) fn create_comment(
        new_post_id: PostId,
        comment_ext: Comment,
        root_post: &mut Post<T>,
    ) -> DispatchResult {
        let mut commented_post_id = root_post.id;

        if let Some(parent_id) = comment_ext.parent_id {
            let mut parent_comment =
                Self::post_by_id(parent_id).ok_or(Error::<T>::UnknownParentComment)?;

            ensure!(parent_comment.is_comment(), Error::<T>::NotACommentByParentId);

            let ancestors = Self::get_post_ancestors(parent_id);
            ensure!(
                ancestors.len() < T::MaxCommentDepth::get() as usize,
                Error::<T>::MaxCommentDepthReached
            );

            parent_comment.inc_replies();

            commented_post_id = parent_id;
            PostById::insert(parent_id, parent_comment);
        }

        root_post.inc_replies();

        PostById::insert(root_post.id, root_post);
        ReplyIdsByPostId::<T>::mutate(commented_post_id, |reply_ids| reply_ids.push(new_post_id));

        Ok(())
    }

    pub(crate) fn create_shared_post(
        creator: &T::AccountId,
        new_post_id: PostId,
        original_post_id: PostId,
        space: &mut Space<T>,
    ) -> DispatchResult {
        let original_post =
            &mut Self::post_by_id(original_post_id).ok_or(Error::<T>::OriginalPostNotFound)?;

        ensure!(!original_post.is_shared_post(), Error::<T>::CannotShareSharedPost);

        // Check if it's allowed to share a post from the space of original post.
        Spaces::ensure_account_has_space_permission(
            creator.clone(),
            &original_post.get_space()?,
            SpacePermission::Share,
            Error::<T>::NoPermissionToShare.into(),
        )?;

        space.inc_posts();

        Self::share_post(creator.clone(), original_post_id, new_post_id)
    }

    fn mutate_posts_count_on_space<F: FnMut(&mut u32) + Copy>(
        space_id: SpaceId,
        is_post_hidden: bool,
        mut f: F,
    ) -> DispatchResult {
        Spaces::<T>::mutate_space_by_id(space_id, |space: &mut Space<T>| {
            if !is_post_hidden {
                f(&mut space.posts_count);
            }
        })
        .map(|_| ())
    }

    pub(crate) fn move_post_to_space(
        editor: T::AccountId,
        post: &mut Post<T>,
        new_space_id: SpaceId,
    ) -> DispatchResult {
        let old_space_id_opt = post.try_get_space_id();
        let new_space = Spaces::<T>::require_space(new_space_id)?;

        ensure!(
            T::IsAccountBlocked::is_allowed_account(editor.clone(), new_space_id),
            ModerationError::AccountIsBlocked
        );
        Spaces::ensure_account_has_space_permission(
            editor,
            &new_space,
            SpacePermission::CreatePosts,
            Error::<T>::NoPermissionToCreatePosts.into(),
        )?;
        ensure!(
            T::IsPostBlocked::is_allowed_post(post.id, new_space_id),
            ModerationError::PostIsBlocked
        );
        ensure!(
            T::IsContentBlocked::is_allowed_content(post.content.clone(), new_space_id),
            ModerationError::ContentIsBlocked
        );

        match post.extension {
            PostExtension::RegularPost(_) | PostExtension::SharedPost(_) => {
                if let Some(old_space_id) = old_space_id_opt {
                    // Decrease the number of posts on the old space
                    Self::mutate_posts_count_on_space(old_space_id, post.hidden, |counter| {
                        *counter = counter.saturating_sub(1)
                    })?;

                    PostIdsBySpaceId::<T>::mutate(old_space_id, |post_ids| {
                        remove_from_vec(post_ids, post.id)
                    });
                }

                // Increase the number of posts on the new space
                Self::mutate_posts_count_on_space(new_space_id, post.hidden, |counter| {
                    *counter = counter.saturating_add(1)
                })?;

                PostIdsBySpaceId::<T>::mutate(new_space_id, |post_ids| post_ids.push(post.id));

                post.space_id = Some(new_space_id);
                PostById::<T>::insert(post.id, post);

                Ok(())
            },
            _ => fail!(Error::<T>::CannotUpdateSpaceIdOnComment),
        }
    }

    pub fn delete_post_from_space(post_id: PostId) -> DispatchResult {
        let mut post = Self::require_post(post_id)?;

        if let PostExtension::Comment(comment_ext) = post.extension {
            let comment_total_replies = Self::get_post_replies_count_by_id(post_id);
            post.extension =
                PostExtension::RegularPost(RegularPost { total_replies_count: comment_total_replies });

            if post.hidden {
                PostById::insert(post.id, post);
                return Ok(())
            }

            let root_post = &mut Self::require_post(comment_ext.root_post_id)?;
            let parent_id = comment_ext.parent_id.unwrap_or_default();

            if let Ok(mut parent_comment) = Self::require_post(parent_id) {
                parent_comment.dec_replies();
                PostById::insert(parent_id, parent_comment);
            }

            if let PostExtension::RegularPost(post_ext) = &mut root_post.extension {
                post_ext.total_replies_count =
                    post_ext.total_replies_count.saturating_sub(comment_total_replies);
            }

            PostById::insert(root_post.id, root_post);
        } else {
            // If post is not a comment:

            let space_id = post.get_space_id()?;

            // Decrease the number of posts on the space
            Self::mutate_posts_count_on_space(space_id, post.hidden, |counter| {
                *counter = counter.saturating_sub(1)
            })?;

            post.space_id = None;
            PostIdsBySpaceId::<T>::mutate(space_id, |post_ids| remove_from_vec(post_ids, post_id));
        }

        PostById::insert(post.id, post);

        Ok(())
    }

    /// Rewrite ancestor counters when Post hidden status changes
    /// Warning: This will affect storage state!
    pub(crate) fn update_counters_on_comment_hidden_change(
        comment_ext: &Comment,
        becomes_hidden: bool,
    ) -> DispatchResult {
        let root_post = &mut Self::require_post(comment_ext.root_post_id)?;
        let parent_comment_id: PostId = comment_ext.parent_id.unwrap_or_default();

        let mut update_replies: fn(&mut Post<T>) = Post::inc_replies;
        if becomes_hidden {
            update_replies = Post::dec_replies;
        }

        // TODO: refactor this: doesn't look in the best way possible, but it works.
        if let Ok(parent_comment) = &mut Self::require_post(parent_comment_id) {
            if parent_comment.is_comment() {
                update_replies(parent_comment);
                PostById::<T>::insert(parent_comment_id, parent_comment);
            }
        }

        if root_post.is_root_post() {
            update_replies(root_post);
            PostById::<T>::insert(root_post.id, root_post);
        }

        Ok(())
    }
}
