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

#![allow(clippy::or_fun_call)]
#![allow(clippy::too_many_arguments)]

use cumulus_primitives_core::ParaId;
use primitives::BlockNumber;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use serde_json::map::Map;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use testing_basilisk_runtime::{
	AccountId, AssetRegistryConfig, AuraId, Balance, BalancesConfig, CollatorSelectionConfig, CouncilConfig,
	DusterConfig, ElectionsConfig, GenesisConfig, MultiTransactionPaymentConfig, OrmlNftConfig, ParachainInfoConfig,
	SessionConfig, Signature, SudoConfig, SystemConfig, TechnicalCommitteeConfig, TokensConfig, VestingConfig,
	NATIVE_EXISTENTIAL_DEPOSIT, UNITS, WASM_BINARY,
};

const TOKEN_DECIMALS: u8 = 12;
const TOKEN_SYMBOL: &str = "BSX";
const PROTOCOL_ID: &str = "bsx";

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	#[allow(clippy::borrowed_box)]
	#[allow(dead_code)]
	pub fn try_get(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
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

pub fn get_vesting_config_for_test() -> Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)> {
	let vesting_list_json = &include_bytes!("../res/basilisk-vesting-lbp-test.json")[..];
	let vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)> =
		serde_json::from_slice(vesting_list_json).unwrap();

	// ensure no duplicates exist.
	let unique_vesting_accounts = vesting_list
		.iter()
		.map(|(x, _, _, _, _)| x)
		.cloned()
		.collect::<std::collections::BTreeSet<_>>();
	assert!(
		unique_vesting_accounts.len() == vesting_list.len(),
		"duplicate vesting accounts in genesis."
	);
	vesting_list
}

pub fn parachain_development_config(para_id: ParaId) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
	let mut properties = Map::new();
	properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Testing Basilisk Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_parachain_genesis(
				wasm_binary,
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				//initial authorities & invulnerables
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_from_seed::<AuraId>("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_from_seed::<AuraId>("Bob"),
					),
				],
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Duster"),
				],
				true,
				para_id,
				//council
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
				//technical_committe
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"), // SAME AS ROOT
				get_vesting_config_for_test(),
				vec![
					(b"hKSM".to_vec(), 1_000u128),
					(b"hDOT".to_vec(), 1_000u128),
					(b"hETH".to_vec(), 1_000u128),
					(b"hUSDT".to_vec(), 1_000u128),
				],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(PROTOCOL_ID),
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-dev".into(),
			para_id: para_id.into(),
		},
	))
}

pub fn local_parachain_config(para_id: ParaId) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	let mut properties = Map::new();
	properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Testing Basilisk Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_parachain_genesis(
				wasm_binary,
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				//initial authorities & invulnerables
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_from_seed::<AuraId>("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_from_seed::<AuraId>("Bob"),
					),
				],
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				true,
				para_id,
				//council
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
				//technical_committe
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"), // SAME AS ROOT
				get_vesting_config_for_test(),
				vec![
					(b"hKSM".to_vec(), 1_000u128),
					(b"hDOT".to_vec(), 1_000u128),
					(b"hETH".to_vec(), 1_000u128),
					(b"hUSDT".to_vec(), 1_000u128),
				],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(PROTOCOL_ID),
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: para_id.into(),
		},
	))
}

fn testnet_parachain_genesis(
	wasm_binary: &[u8],
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
	parachain_id: ParaId,
	council_members: Vec<AccountId>,
	tech_committee_members: Vec<AccountId>,
	tx_fee_payment_account: AccountId,
	vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)>,
	registered_assets: Vec<(Vec<u8>, Balance)>, // (Asset name, Existential deposit)
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of a lot.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1_000_000_000u128 * UNITS))
				.collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		collator_selection: CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: 10_000,
			..Default::default()
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                            // account id
						acc,                                                    // validator id
						testing_basilisk_runtime::opaque::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},

		// no need to pass anything, it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		asset_registry: AssetRegistryConfig {
			asset_names: registered_assets,
			native_asset_name: TOKEN_SYMBOL.as_bytes().to_vec(),
			native_existential_deposit: NATIVE_EXISTENTIAL_DEPOSIT,
		},
		multi_transaction_payment: MultiTransactionPaymentConfig {
			currencies: vec![],
			fallback_account: tx_fee_payment_account,
		},
		tokens: TokensConfig {
			balances: endowed_accounts
				.iter()
				.flat_map(|x| {
					vec![
						(x.clone(), 1, 1_000_000_000u128 * UNITS),
						(x.clone(), 2, 1_000_000_000u128 * UNITS),
						(x.clone(), 3, 1_000_000_000u128 * UNITS),
						(x.clone(), 4, 1_000_000_000u128 * UNITS),
					]
				})
				.collect(),
		},
		treasury: Default::default(),
		elections: ElectionsConfig {
			// Intergalactic elections
			members: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				100_000_000u128 * UNITS,
			)],
		},
		council: CouncilConfig {
			members: council_members,
			phantom: Default::default(),
		},
		technical_committee: TechnicalCommitteeConfig {
			members: tech_committee_members,
			phantom: Default::default(),
		},
		vesting: VestingConfig { vesting: vesting_list },
		orml_nft: OrmlNftConfig {
			tokens: Default::default(),
		},
		parachain_info: ParachainInfoConfig { parachain_id },
		aura_ext: Default::default(),
		duster: DusterConfig {
			account_blacklist: vec![get_account_id_from_seed::<sr25519::Public>("Duster")],
			reward_account: get_account_id_from_seed::<sr25519::Public>("Duster"),
			dust_account: get_account_id_from_seed::<sr25519::Public>("Duster"),
		},
	}
}
