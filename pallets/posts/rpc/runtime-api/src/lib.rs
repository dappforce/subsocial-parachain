//! Runtime API definition for posts pallet.

#![cfg_attr(not(feature = "std"), no_std)]
// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE


use codec::Codec;
use sp_runtime::DispatchResult;
use sp_runtime::traits::MaybeDisplay;

use subsocial_support::{Content, PostId, SpaceId};

sp_api::decl_runtime_apis! {
    pub trait PostsApi<AccountId>
        where
            AccountId: Codec + MaybeDisplay,
    {
        fn can_create_post(
            account: AccountId,
            space_id: SpaceId,
            content_opt: Option<Content>,
        ) -> DispatchResult;

        fn can_create_comment(
            account: AccountId,
            root_post_id: PostId,
            parent_id_opt: Option<PostId>,
            content_opt: Option<Content>
        ) -> DispatchResult;
    }
}
