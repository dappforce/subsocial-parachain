//! RPC interface for the domains pallet.

use std::{fmt::Display, sync::Arc};

use codec::Codec;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::Block as BlockT,
};

pub use pallet_posts_rpc_runtime_api::PostsApi as PostsRuntimeApi;
use subsocial_support::SpaceId;

#[rpc(client, server)]
pub trait PostsApi<AccountId, BlockHash> {
    #[method(name = "posts_checkAccountCanCreatePost")]
    fn check_account_can_create_post(&self, account: AccountId, space_id: SpaceId, at: Option<BlockHash>) -> RpcResult<bool>;
}

/// Provides RPC method to query a domain price.
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
    fn check_account_can_create_post(
        &self,
        account: AccountId,
        space_id: SpaceId,
        at: Option<Block::Hash>,
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        fn map_err(error: impl ToString, desc: &'static str) -> CallError {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                desc,
                Some(error.to_string()),
            ))
        }

        let res = api
            .check_account_can_create_post(&at, account, space_id)
            .map_err(|e| map_err(e, "Unable to calculate price for domain."))?;

        Ok(res)
    }
}
