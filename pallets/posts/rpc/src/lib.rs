use std::{sync::Arc, collections::BTreeMap};
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;

use pallet_posts::rpc::{FlatPost, FlatPostKind, RepliesByPostId};
use pallet_utils::{PostId, SpaceId, rpc::map_rpc_error};
pub use posts_runtime_api::PostsApi as PostsRuntimeApi;

#[rpc]
pub trait PostsApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "posts_getPostsByIds")]
    fn get_posts_by_ids(
        &self,
        at: Option<BlockHash>,
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getPublicPosts")]
    fn get_public_posts(
        &self,
        at: Option<BlockHash>,
        kind_filter: Vec<FlatPostKind>,
        start_id: u64,
        limit: u16
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getPublicPostsBySpaceId")]
    fn get_public_posts_by_space_id(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getUnlistedPostsBySpaceId")]
    fn get_unlisted_posts_by_space_id(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getReplyIdsByParentId")]
    fn get_reply_ids_by_parent_id(
        &self,
        at: Option<BlockHash>,
        post_id: PostId,
    ) -> Result<Vec<PostId>>;

    #[rpc(name = "posts_getReplyIdsByParentIds")]
    fn get_reply_ids_by_parent_ids(
        &self,
        at: Option<BlockHash>,
        post_ids: Vec<PostId>,
    ) -> Result<BTreeMap<PostId, Vec<PostId>>>;

    #[rpc(name = "posts_getRepliesByParentId")]
    fn get_replies_by_parent_id(
        &self,
        at: Option<BlockHash>,
        parent_id: PostId,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getRepliesByParentIds")]
    fn get_replies_by_parent_ids(
        &self,
        at: Option<BlockHash>,
        parent_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Result<RepliesByPostId<AccountId, BlockNumber>>;

    #[rpc(name = "posts_getUnlistedPostIdsBySpaceId")]
    fn get_unlisted_post_ids_by_space_id(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
    ) -> Result<Vec<PostId>>;

    #[rpc(name = "posts_getPublicPostIdsBySpaceId")]
    fn get_public_post_ids_by_space_id(
        &self,
        at: Option<BlockHash>,
        space_id: SpaceId,
    ) -> Result<Vec<PostId>>;

    #[rpc(name = "posts_nextPostId")]
    fn get_next_post_id(&self, at: Option<BlockHash>) -> Result<PostId>;

    #[rpc(name = "posts_getFeed")]
    fn get_feed(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;
}

pub struct Posts<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Posts<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, BlockNumber> PostsApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Posts<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    BlockNumber: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: PostsRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_posts_by_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_posts_by_ids(&at, post_ids, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_posts(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        kind_filter: Vec<FlatPostKind>,
        start_id: u64,
        limit: u16
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_posts(&at, kind_filter, start_id, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_posts_by_space_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        space_id: u64,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_posts_by_space_id(&at, space_id, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_posts_by_space_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        space_id: u64,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_posts_by_space_id(&at, space_id, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_reply_ids_by_parent_id(&self, at: Option<<Block as BlockT>::Hash>, parent_id: PostId) -> Result<Vec<PostId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_reply_ids_by_parent_id(&at, parent_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_reply_ids_by_parent_ids(&self, at: Option<<Block as BlockT>::Hash>, parent_ids: Vec<PostId>) -> Result<BTreeMap<PostId, Vec<PostId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_reply_ids_by_parent_ids(&at, parent_ids);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_replies_by_parent_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        parent_id: PostId,
        offset: u64,
        limit: u16
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_replies_by_parent_id(&at, parent_id, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_replies_by_parent_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        parent_ids: Vec<PostId>,
        offset: u64,
        limit: u16
    ) -> Result<RepliesByPostId<AccountId, BlockNumber>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_replies_by_parent_ids(&at, parent_ids, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_post_ids_by_space_id(&self, at: Option<<Block as BlockT>::Hash>, space_id: u64) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_post_ids_by_space_id(&at, space_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_post_ids_by_space_id(&self, at: Option<<Block as BlockT>::Hash>, space_id: u64) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_post_ids_by_space_id(&at, space_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_next_post_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_next_post_id(&at);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_feed(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        account: AccountId,
        offset: u64,
        limit: u16
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_feed(&at, account, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }
}
