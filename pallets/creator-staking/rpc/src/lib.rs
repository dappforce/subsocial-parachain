//! RPC interface for the domains pallet.

use std::{convert::TryInto, fmt::Display, sync::Arc};

use codec::Codec;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, MaybeDisplay},
};
use sp_std::vec::Vec;

use subsocial_support::SpaceId;
pub use pallet_creator_staking_rpc_runtime_api::CreatorStakingApi as CreatorStakingRuntimeApi;

#[rpc(client, server)]
pub trait CreatorStakingApi<BlockHash, AccountId, GenericResponseType> {
    #[method(name = "creatorStaking_estimatedStakerRewardsByCreator")]
    fn estimated_staker_rewards_by_creators(
        &self,
        staker: AccountId,
        creators: Vec<SpaceId>,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(SpaceId, GenericResponseType)>>;

    #[method(name = "creatorStaking_withdrawableAmountsFromInactiveCreators")]
    fn withdrawable_amounts_from_inactive_creators(
        &self,
        staker: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(SpaceId, GenericResponseType)>>;

    #[method(name = "posts_availableClaimsByStaker")]
    fn available_claims_by_staker(
        &self,
        staker: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(SpaceId, u32)>>;
}

/// Provides RPC method to query a domain price.
pub struct CreatorStaking<C, P> {
    /// Shared reference to the client.
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> CreatorStaking<C, P> {
    /// Creates a new instance of the Domains Rpc helper.
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

impl<C, Block, AccountId, Balance>
CreatorStakingApiServer<
    <Block as BlockT>::Hash,
    AccountId,
    Balance,
> for CreatorStaking<C, Block>
    where
        Block: BlockT,
        C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
        C::Api: CreatorStakingRuntimeApi<Block, AccountId, Balance>,
        AccountId: Clone + Display + Codec + Send + 'static,
        Balance: Codec + MaybeDisplay + Copy + TryInto<NumberOrHex> + Send + Sync + 'static,
{
    fn estimated_staker_rewards_by_creators(
        &self,
        staker: AccountId,
        creators: Vec<SpaceId>,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(SpaceId, Balance)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let res = api
            .estimated_staker_rewards_by_creators(&at, staker, creators)
            .map_err(|e| map_err(e, "Unable to get estimated rewards by creator."))?;

        Ok(res)
    }

    fn withdrawable_amounts_from_inactive_creators(
        &self,
        staker: AccountId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(SpaceId, Balance)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let res = api
            .withdrawable_amounts_from_inactive_creators(&at, staker)
            .map_err(|e| map_err(e, "Unable to get withdrawable amounts from inactive creators."))?;

        Ok(res)
    }

    fn available_claims_by_staker(
        &self,
        staker: AccountId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(SpaceId, u32)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let res = api
            .available_claims_by_staker(&at, staker)
            .map_err(|e| map_err(e, "Unable to get claims number by staker."))?;

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
