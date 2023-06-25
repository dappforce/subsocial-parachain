// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
// Copyright (C) 2021-2022 DAPPFORCE PTE. Ltd., aleksandr.siman@gmail.com.

// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/HEADER-GPL3. 
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE.

use std::sync::Arc;
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;

use pallet_utils::rpc::map_rpc_error;
pub use profile_follows_runtime_api::ProfileFollowsApi as ProfileFollowsRuntimeApi;

#[rpc]
pub trait ProfileFollowsApi<BlockHash, AccountId> {
    #[rpc(name = "profileFollows_filterFollowedAccounts")]
    fn filter_followed_accounts(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        maybe_following: Vec<AccountId>,
    ) -> Result<Vec<AccountId>>;
}

pub struct ProfileFollows<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> ProfileFollows<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> ProfileFollowsApi<<Block as BlockT>::Hash, AccountId>
    for ProfileFollows<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ProfileFollowsRuntimeApi<Block, AccountId>,
{
    fn filter_followed_accounts(
        &self, at:
        Option<<Block as BlockT>::Hash>,
        account: AccountId,
        maybe_following: Vec<AccountId>,
    ) -> Result<Vec<AccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.filter_followed_accounts(&at, account, maybe_following);
        runtime_api_result.map_err(map_rpc_error)
    }
}
