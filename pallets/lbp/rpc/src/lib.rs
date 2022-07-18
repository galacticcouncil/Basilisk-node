// This file is part of Basilisk-node.

// Copyright (C) 2020-2021  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::upper_case_acronyms)]

use codec::Codec;
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

pub use pallet_lbp_rpc_runtime_api::LBPApi as LBPRuntimeApi;

#[rpc(client, server)]
pub trait LBPApi<BlockHash, AccountId, AssetId> {
	#[method(name = "lbp_getPoolAccount")]
	fn get_pool_id(&self, asset_a: AssetId, asset_b: AssetId) -> RpcResult<AccountId>;
}

fn internal_err<T: ToString>(message: T) -> JsonRpseeError {
    JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
        ErrorCode::InternalError.code(),
        message.to_string(),
        None::<()>,
    )))
}

/// A struct that implements the [`XYKApi`].
pub struct LBP<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> LBP<C, B> {
	/// Create new `XYK` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		LBP {
			client,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

#[async_trait]
impl<C, Block, AccountId, AssetId> LBPApiServer<<Block as BlockT>::Hash, AccountId, AssetId> for LBP<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: LBPRuntimeApi<Block, AccountId, AssetId>,
	AccountId: Codec,
	AssetId: Codec,
{
	fn get_pool_id(&self, asset_a: AssetId, asset_b: AssetId) -> RpcResult<AccountId> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);

		api.get_pool_id(&at, asset_a, asset_b)
			.map_err(|e| internal_err(format!("Unable to retrieve pool account address: {:?}", e)))
	}
}
