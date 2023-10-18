//! RPC interface for the creator-staking pallet.

use std::{convert::TryInto, fmt::Display, sync::Arc, vec::Vec};

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

use pallet_creator_staking::{CreatorId, EraIndex};
pub use pallet_creator_staking_rpc_runtime_api::CreatorStakingApi as CreatorStakingRuntimeApi;

#[rpc(client, server)]
pub trait CreatorStakingApi<BlockHash, AccountId, GenericResponseType> {
    #[method(name = "creatorStaking_estimatedBackerRewardsByCreator")]
    fn estimated_backer_rewards_by_creators(
        &self,
        backer: AccountId,
        creators: Vec<CreatorId>,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(CreatorId, GenericResponseType)>>;

    #[method(name = "creatorStaking_withdrawableAmountsFromInactiveCreators")]
    fn withdrawable_amounts_from_inactive_creators(
        &self,
        backer: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(CreatorId, GenericResponseType)>>;

    #[method(name = "creatorStaking_availableClaimsByBacker")]
    fn available_claims_by_backer(
        &self,
        backer: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(CreatorId, u32)>>;

    #[method(name = "creatorStaking_estimatedCreatorRewards")]
    fn estimated_creator_rewards(
        &self,
        creator: CreatorId,
        at: Option<BlockHash>,
    ) -> RpcResult<GenericResponseType>;

    #[method(name = "creatorStaking_availableClaimsByCreator")]
    fn available_claims_by_creator(
        &self,
        creator: CreatorId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<EraIndex>>;
}

/// Provides RPC method to query a domain price.
pub struct CreatorStaking<C, P> {
    /// Shared reference to the client.
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> CreatorStaking<C, P> {
    /// Creates a new instance of the CreatorStaking Rpc helper.
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
    fn estimated_backer_rewards_by_creators(
        &self,
        backer: AccountId,
        creators: Vec<CreatorId>,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(CreatorId, Balance)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let res = api
            .estimated_backer_rewards_by_creators(&at, backer, creators)
            .map_err(|e| map_err(e, "Unable to get estimated rewards by creator."))?;

        Ok(res)
    }

    fn withdrawable_amounts_from_inactive_creators(
        &self,
        backer: AccountId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(CreatorId, Balance)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let res = api
            .withdrawable_amounts_from_inactive_creators(&at, backer)
            .map_err(|e| map_err(e, "Unable to get withdrawable amounts from inactive creators."))?;

        Ok(res)
    }

    fn available_claims_by_backer(
        &self,
        backer: AccountId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(CreatorId, u32)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let res = api
            .available_claims_by_backer(&at, backer)
            .map_err(|e| map_err(e, "Unable to get claims number by backer."))?;

        Ok(res)
    }

    fn estimated_creator_rewards(
        &self,
        creator: CreatorId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Balance> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let res = api
            .estimated_creator_rewards(&at, creator)
            .map_err(|e| map_err(e, "Unable to get claims number by backer."))?;

        Ok(res)
    }

    fn available_claims_by_creator(
        &self,
        creator: CreatorId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<EraIndex>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let res = api
            .available_claims_by_creator(&at, creator)
            .map_err(|e| map_err(e, "Unable to get claims number by backer."))?;

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
