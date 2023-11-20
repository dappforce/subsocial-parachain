// Copyright (C) DAPPFORCE PTE. LTD.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0.
//
// Full notice is available at https://github.com/dappforce/subsocial-parachain/blob/main/COPYRIGHT
// Full license is available at https://github.com/dappforce/subsocial-parachain/blob/main/LICENSE

//! RPC interface for the domains pallet.

use std::{convert::TryInto, sync::Arc};

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

pub use pallet_domains_rpc_runtime_api::DomainsApi as DomainsRuntimeApi;

#[rpc(client, server)]
pub trait DomainsApi<BlockHash, ResponseType> {
    #[method(name = "domains_calculatePrice")]
    fn calculate_price(&self, subdomain: Vec<u8>, at: Option<BlockHash>) -> RpcResult<Option<ResponseType>>;
}

/// Provides RPC method to query a domain price.
pub struct Domains<C, P> {
    /// Shared reference to the client.
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> Domains<C, P> {
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

impl<C, Block, Balance>
DomainsApiServer<
    <Block as BlockT>::Hash,
    Balance,
> for Domains<C, Block>
    where
        Block: BlockT,
        C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
        C::Api: DomainsRuntimeApi<Block, Balance>,
        Balance: Codec + MaybeDisplay + Copy + TryInto<NumberOrHex> + Send + Sync + 'static,
{
    fn calculate_price(
        &self,
        subdomain: Vec<u8>,
        at: Option<Block::Hash>,
    ) -> RpcResult<Option<Balance>> {
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
            .calculate_price(&at, subdomain)
            .map_err(|e| map_err(e, "Unable to calculate price for domain."))?;

        Ok(res)
    }
}
