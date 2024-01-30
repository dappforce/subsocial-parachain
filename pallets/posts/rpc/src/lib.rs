// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

//! RPC interface for the posts pallet.

use std::{fmt::Display, sync::Arc};

use codec::Codec;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{DispatchResult, traits::Block as BlockT};

pub use pallet_posts_rpc_runtime_api::PostsApi as PostsRuntimeApi;
use subsocial_support::{Content, PostId, SpaceId};

#[rpc(client, server)]
pub trait PostsApi<AccountId, BlockHash> {
    #[method(name = "posts_canAccountCreatePost")]
    fn can_account_create_post(
        &self,
        account: AccountId,
        space_id: SpaceId,
        content_opt: Option<Content>,
        at: Option<BlockHash>,
    ) -> RpcResult<DispatchResult>;

    #[method(name = "posts_canAccountCreateComment")]
    fn can_account_create_comment(
        &self,
        account: AccountId,
        root_post_id: PostId,
        parent_id_opt: Option<PostId>,
        content_opt: Option<Content>,
        at: Option<BlockHash>,
    ) -> RpcResult<DispatchResult>;
}

/// Provides RPC methods for posts pallet.
pub struct Posts<C, P> {
    /// Shared reference to the client.
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> Posts<C, P> {
    /// Creates a new instance of the Posts Rpc helper.
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
        }
    }
}

impl<C, Block, AccountId>
PostsApiServer<
    AccountId,
    <Block as BlockT>::Hash,
> for Posts<C, Block>
    where
        Block: BlockT,
        C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
        C::Api: PostsRuntimeApi<Block, AccountId>,
        AccountId: Clone + Display + Codec + Send + 'static,
{
    fn can_account_create_post(
        &self,
        account: AccountId,
        space_id: SpaceId,
        content_opt: Option<Content>,
        at: Option<Block::Hash>,
    ) -> RpcResult<DispatchResult> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        let res = api
            .can_account_create_post(at_hash, account, space_id, content_opt)
            .map_err(|e| map_err(e, "Unable to validate post creation."))?;

        Ok(res)
    }

    fn can_account_create_comment(
        &self,
        account: AccountId,
        root_post_id: PostId,
        parent_id_opt: Option<PostId>,
        content_opt: Option<Content>,
        at: Option<Block::Hash>,
    ) -> RpcResult<DispatchResult> {
        let api = self.client.runtime_api();
        let at_hash = at.unwrap_or_else(|| self.client.info().best_hash);

        let res = api
            .can_account_create_comment(at_hash, account, root_post_id, parent_id_opt, content_opt)
            .map_err(|e| map_err(e, "Unable to validate comment creation."))?;

        Ok(res)
    }
}

fn map_err(error: impl ToString, desc: &'static str) -> CallError {
    CallError::Custom(ErrorObject::owned(
        Error::RuntimeError.into(),
        desc,
        Some(error.to_string()),
    ))
}
