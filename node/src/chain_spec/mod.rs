// This file is part of Basilisk-node

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
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

#![allow(clippy::or_fun_call)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::derive_partial_eq_without_eq)] //Needed due to bug 'https://github.com/rust-lang/rust-clippy/issues/8867'

pub mod basilisk;
pub mod local;
pub mod paseo;
pub mod rococo;

const PARA_ID: u32 = 2090;
const TOKEN_DECIMALS: u8 = 12;
const TOKEN_SYMBOL: &str = "BSX";
const PROTOCOL_ID: &str = "bsx";

use basilisk_runtime::{AccountId, AuraId, Balance, RuntimeGenesisConfig, Signature, WASM_BINARY};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use primitives::{
	constants::currency::{NATIVE_EXISTENTIAL_DEPOSIT, UNITS},
	AssetId, Price,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use serde_json::map::Map;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{seed}"), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

#[allow(clippy::type_complexity)]
pub fn parachain_genesis(
	initial_authorities: (Vec<(AccountId, AuraId)>, Balance), // (initial auths, candidacy bond)
	endowed_accounts: Vec<(AccountId, Balance)>,
	council_members: Vec<AccountId>,
	tech_committee_members: Vec<AccountId>,
	registered_assets: Vec<(Vec<u8>, Balance, Option<AssetId>)>,
	accepted_assets: Vec<(AssetId, Price)>, // (Asset id, Fallback price) - asset which fee can be paid with
	token_balances: Vec<(AccountId, Vec<(AssetId, Balance)>)>,
	elections: Vec<(AccountId, Balance)>,
	parachain_id: ParaId,
) -> serde_json::Value {
	serde_json::json!({
		"system": {},
		"session": {
			"keys": initial_authorities
				.0
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                   // account id
						acc,                                           // validator id
						basilisk_runtime::opaque::SessionKeys { aura }, // session keys
					)
				})
				.collect::<Vec<_>>(),
		},
	  "assetRegistry": {
			"registeredAssets": registered_assets.clone(),
			"nativeAssetName": TOKEN_SYMBOL.as_bytes().to_vec(),
			"nativeExistentialDeposit": NATIVE_EXISTENTIAL_DEPOSIT,
		},
		"aura": {
			"authorities": Vec::<sp_consensus_aura::sr25519::AuthorityId>::new()
		},
	  "auraExt": {
		},
	  "balances": {
		"balances": endowed_accounts
		  .iter()
		  .cloned()
		  .map(|k| (k.0.clone(), k.1 * UNITS))
		  .collect::<Vec<_>>(),
	  },
		"collatorSelection": {
			"invulnerables": initial_authorities.0.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": initial_authorities.1,
			"desiredCandidates": 0u32,
		},
		"council": {
			"members": council_members,
		},
	  "duster": {
		"accountBlacklist": vec![get_account_id_from_seed::<sr25519::Public>("Duster")],
			"rewardAccount": Some(get_account_id_from_seed::<sr25519::Public>("Duster")),
			"dustAccount": Some(get_account_id_from_seed::<sr25519::Public>("Duster"))
		},
	  "elections": {
		"members": elections,
		},
	  "emaOracle": {
		},
	  "multiTransactionPayment": {
			"currencies": accepted_assets,
			"accountCurrencies": Vec::<(AccountId, AssetId)>::new(),
		},
	  "parachainInfo": {
		"parachainId": parachain_id,
		},
	  "polkadotXcm": {
		},
		"technicalCommittee": {
			"members": tech_committee_members,
		},
		"tokens": {
			"balances": if registered_assets.is_empty() {
				vec![]
			} else {
				token_balances
					.iter()
					.flat_map(|x| {
						x.1.clone()
							.into_iter()
							.map(|(asset_id, amount)| (x.0.clone(), asset_id, amount))
					})
				.collect::<Vec<_>>()
			},
		},
		"treasury": {
	  },
	  "vesting": {
	  },
		"xykWarehouseLm": {
	  },
		"xykLiquidityMining": {
	  },
	}
	)
}
