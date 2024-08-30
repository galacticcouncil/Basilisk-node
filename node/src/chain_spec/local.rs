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

use super::*;

const INITIAL_BALANCE: u128 = 10_000;
const INITIAL_TOKEN_BALANCE: Balance = 1_000 * UNITS;

pub fn parachain_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	let mut properties = Map::new();
	properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

	let genesis_json = parachain_genesis(
		// initial_authorities
		(
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
			// candidacy bond
			10_000 * UNITS,
		),
		// endowed_accounts
		vec![
			(get_account_id_from_seed::<sr25519::Public>("Alice"), INITIAL_BALANCE),
			(get_account_id_from_seed::<sr25519::Public>("Bob"), INITIAL_BALANCE),
			(get_account_id_from_seed::<sr25519::Public>("Charlie"), INITIAL_BALANCE),
			(get_account_id_from_seed::<sr25519::Public>("Dave"), INITIAL_BALANCE),
			(get_account_id_from_seed::<sr25519::Public>("Eve"), INITIAL_BALANCE),
			(get_account_id_from_seed::<sr25519::Public>("Ferdie"), INITIAL_BALANCE),
			(
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				INITIAL_BALANCE,
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				INITIAL_BALANCE,
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				INITIAL_BALANCE,
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				INITIAL_BALANCE,
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				INITIAL_BALANCE,
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				INITIAL_BALANCE,
			),
		],
		// council_members
		vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
		// tech_committee_members
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
		],
		// registered_assets
		vec![
			(b"KSM".to_vec(), 1_000u128, Some(1u32)),
			(b"KUSD".to_vec(), 1_000u128, Some(2u32)),
		],
		// accepted_assets
		vec![(1, Price::from_float(0.0000212)), (2, Price::from_float(0.000806))],
		// token_balances
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![(1, INITIAL_TOKEN_BALANCE), (2, INITIAL_TOKEN_BALANCE)],
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				vec![(1, INITIAL_TOKEN_BALANCE), (2, INITIAL_TOKEN_BALANCE)],
			),
		],
		// elections
		vec![(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			INITIAL_TOKEN_BALANCE,
		)],
		// parachain ID
		PARA_ID.into(),
	);

	let chain_spec = ChainSpec::builder(
		wasm_binary,
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: PARA_ID,
		},
	)
	.with_name("Basilisk Testnet (Local)")
	.with_id("testnet_local")
	.with_chain_type(ChainType::Local)
	.with_boot_nodes(vec![])
	.with_properties(properties)
	.with_protocol_id(PROTOCOL_ID)
	.with_genesis_config_patch(genesis_json)
	.build();

	Ok(chain_spec)
}
