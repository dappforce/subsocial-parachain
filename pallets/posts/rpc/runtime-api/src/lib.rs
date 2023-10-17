//! Runtime API definition for posts pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::DispatchResult;
use sp_runtime::traits::MaybeDisplay;

use subsocial_support::{Content, PostId, SpaceId};

sp_api::decl_runtime_apis! {
    pub trait PostsApi<AccountId>
        where
            AccountId: Codec + MaybeDisplay,
    {
        fn check_account_can_create_post(
            account: AccountId,
            space_id: SpaceId,
            content_opt: Option<Content>,
        ) -> DispatchResult;

        fn check_account_can_create_comment(
            account: AccountId,
            root_post_id: PostId,
            parent_id_opt: Option<PostId>,
            content_opt: Option<Content>
        ) -> DispatchResult;
    }
}
