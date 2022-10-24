// This file is part of Basilisk-node.

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

use crate::chain_spec::Extensions;
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use primitives::{AssetId, BlockNumber, Price};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde_json::map::Map;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use testing_basilisk_runtime::{
	AccountId, AssetRegistryConfig, AuraId, Balance, BalancesConfig, CollatorSelectionConfig, CouncilConfig,
	DusterConfig, ElectionsConfig, GenesisConfig, MultiTransactionPaymentConfig, ParachainInfoConfig, SessionConfig,
	Signature, SudoConfig, SystemConfig, TechnicalCommitteeConfig, TokensConfig, VestingConfig,
	NATIVE_EXISTENTIAL_DEPOSIT, UNITS, WASM_BINARY,
};

const TOKEN_DECIMALS: u8 = 12;
const TOKEN_SYMBOL: &str = "BSX";
const PROTOCOL_ID: &str = "bsx";
// The URL for the telemetry server.
const TELEMETRY_URLS: [&str; 2] = [
	"wss://telemetry.polkadot.io/submit/",
	"wss://telemetry.hydradx.io:9000/submit/",
];
//Kusama parachain id
const PARA_ID: u32 = 2090;

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

pub fn parachain_development_config() -> Result<ChainSpec, String> {
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
				PARA_ID.into(),
				//council
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
				//technical_committe
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
				],
				get_vesting_config_for_test(),
				vec![(b"KSM".to_vec(), 1_000u128), (b"KUSD".to_vec(), 1_000u128)],
				vec![(1, Price::from_float(0.0000212)), (2, Price::from_float(0.000806))],
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(PROTOCOL_ID),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-dev".into(),
			para_id: PARA_ID,
		},
	))
}

pub fn local_parachain_config() -> Result<ChainSpec, String> {
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
				PARA_ID.into(),
				//council
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
				//technical_committe
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
				],
				get_vesting_config_for_test(),
				vec![(b"KSM".to_vec(), 1_000u128), (b"KUSD".to_vec(), 1_000u128)],
				vec![(1, Price::from_float(0.0000212)), (2, Price::from_float(0.000806))],
				vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(PROTOCOL_ID),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: PARA_ID,
		},
	))
}

pub fn k8s_testnet_parachain_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
	let mut properties = Map::new();
	properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Basilisk testnet",
		// ID
		"basilisk_testnet",
		ChainType::Live,
		move || {
			testnet_parachain_genesis(
				wasm_binary,
				// Sudo account
				hex!["a62f1daf8e490a1c0514c7d9f3a700999100f2aeb1d67a2ca68b241d3d6b3547"].into(),
				//initial authorities & invulnerables
				vec![
					(
						hex!["54d469d6141e56c0aa802b00732c38477e72d1ad9d8030f45f76b61aaebc1251"].into(),
						hex!["54d469d6141e56c0aa802b00732c38477e72d1ad9d8030f45f76b61aaebc1251"].unchecked_into(),
					),
					(
						hex!["f4969ff6c9b4b1219a1d329d7bdeff9857dd5fc33085fd98182856f7f781b043"].into(),
						hex!["f4969ff6c9b4b1219a1d329d7bdeff9857dd5fc33085fd98182856f7f781b043"].unchecked_into(),
					),
				],
				// Pre-funded accounts
				vec![
					hex!["a62f1daf8e490a1c0514c7d9f3a700999100f2aeb1d67a2ca68b241d3d6b3547"].into(),
					hex!["2e004ce52dd68ec64093d887dbbcc69c7a5333673a7acec6109b08056e89f538"].into(),
					hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"].into(), //acc from ../res/basilisk-vesting-lbp-test.json
					hex!["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"].into(), //acc from ../res/basilisk-vesting-lbp-test.json
				],
				true,
				PARA_ID.into(),
				//technical committee
				vec![hex!["a62f1daf8e490a1c0514c7d9f3a700999100f2aeb1d67a2ca68b241d3d6b3547"].into()],
				vec![],
				get_vesting_config_for_test(),
				vec![(b"KSM".to_vec(), 1_000u128), (b"KUSD".to_vec(), 1_000u128)],
				vec![(1, Price::from_float(0.0000212)), (2, Price::from_float(0.000806))],
				vec![hex!["a62f1daf8e490a1c0514c7d9f3a700999100f2aeb1d67a2ca68b241d3d6b3547"].into()],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some(PROTOCOL_ID),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "westend".into(),
			para_id: PARA_ID,
		},
	))
}

pub fn moonbase_parachain_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
	let mut properties = Map::new();
	properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Basilisk testnet",
		// ID
		"basilisk_moonbase",
		ChainType::Live,
		move || {
			testnet_parachain_genesis(
				wasm_binary,
				// Sudo account
				// 5FemJdNUx75n3udXj5T2WhWE5jtKjxJhQJ1bRhcAGV15Z4Yo
				hex!["9eaea650948488ccc720491b8e40be7436359dc4213a6487ba758ed496f9e53f"].into(),
				//initial authorities & invulnerables
				vec![
					(
						// 5DcuZeiMHh2sZVPZeLaju73p3s7SiDx8P5WtTUTHkk9Xhojo
						hex!["44cb7e89dfbafaa0a90b1838ec588d53fb01f1789d9b638d4aa4cb7de3870463"].into(),
						hex!["44cb7e89dfbafaa0a90b1838ec588d53fb01f1789d9b638d4aa4cb7de3870463"].unchecked_into(),
					),
					(
						// 5FFHN8YyNDht5j6AsDTYQyCLLyMCYpMp5wjo2WFXHSegfh29
						hex!["8cc5f3c8e9480977f93c519522c12f5ec4cd8ab246be9eee6110aec498f22f36"].into(),
						hex!["8cc5f3c8e9480977f93c519522c12f5ec4cd8ab246be9eee6110aec498f22f36"].unchecked_into(),
					),
				],
				// Pre-funded accounts
				vec![
					hex!["9eaea650948488ccc720491b8e40be7436359dc4213a6487ba758ed496f9e53f"].into(), // sudo
				],
				true,
				PARA_ID.into(),
				//technical committee
				vec![hex!["9eaea650948488ccc720491b8e40be7436359dc4213a6487ba758ed496f9e53f"].into()], // same as sudo
				vec![],
				vec![],
				vec![],
				vec![],
				vec![hex!["9eaea650948488ccc720491b8e40be7436359dc4213a6487ba758ed496f9e53f"].into()], // same as sudo
			)
		},
		// Bootnodes
		vec![
			"/dns/p2p-01.basilisk-moonbase.hydradx.io/tcp/30333/p2p/12D3KooWEugVWasr2Zz8SBETvBwdG65RGo4CEp5tJJSASDarqgam"
				.parse()
				.unwrap(),
			"/dns/p2p-02.basilisk-moonbase.hydradx.io/tcp/30333/p2p/12D3KooWP1bEqFb6VxqoNkp4e5ifkxNLFYzzLxzAJCH76zNq2iQB"
				.parse()
				.unwrap()
		],
		// Telemetry
		Some(
			TelemetryEndpoints::new(vec![
				(TELEMETRY_URLS[0].to_string(), 0),
				(TELEMETRY_URLS[1].to_string(), 0),
			])
			.expect("Telemetry url is valid"),
		),
		// Protocol ID
		Some(PROTOCOL_ID),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "westend".into(),
			para_id: PARA_ID,
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
	vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)>,
	registered_assets: Vec<(Vec<u8>, Balance)>, // (Asset name, Existential deposit)
	accepted_assets: Vec<(AssetId, Price)>,     // (Asset id, Fallback price) - asset which fee can be paid with
	elections: Vec<AccountId>,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of a lot.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1_000_000_000_000u128 * UNITS))
				.collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
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
			asset_names: registered_assets.clone(),
			native_asset_name: TOKEN_SYMBOL.as_bytes().to_vec(),
			native_existential_deposit: NATIVE_EXISTENTIAL_DEPOSIT,
		},
		multi_transaction_payment: MultiTransactionPaymentConfig {
			currencies: accepted_assets,
			account_currencies: vec![],
		},
		tokens: TokensConfig {
			balances: if registered_assets.is_empty() {
				vec![]
			} else {
				endowed_accounts
					.iter()
					.flat_map(|x| {
						vec![
							(x.clone(), 1, 1_000_000_000_000u128 * UNITS),
							(x.clone(), 2, 1_000_000_000_000u128 * UNITS),
						]
					})
					.collect()
			},
		},
		treasury: Default::default(),
		elections: ElectionsConfig {
			members: elections
				.iter()
				.flat_map(|x| vec![(x.clone(), 100_000_000u128 * UNITS)])
				.collect(),
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
		parachain_info: ParachainInfoConfig { parachain_id },
		aura_ext: Default::default(),
		duster: DusterConfig {
			account_blacklist: vec![get_account_id_from_seed::<sr25519::Public>("Duster")],
			reward_account: Some(get_account_id_from_seed::<sr25519::Public>("Duster")),
			dust_account: Some(get_account_id_from_seed::<sr25519::Public>("Duster")),
		},
		polkadot_xcm: Default::default(),
	}
}
