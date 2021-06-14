#![allow(clippy::or_fun_call)]

use basilisk_runtime::{
	AccountId, AssetRegistryConfig, AuraConfig, AuraId, BalancesConfig, CouncilConfig, GenesisConfig, OrmlNftConfig,
	ParachainInfoConfig, Signature, SudoConfig, SystemConfig, TechnicalCommitteeConfig, TokensConfig, VestingConfig,
	CORE_ASSET_ID, WASM_BINARY, MultiTransactionPaymentConfig
};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use serde_json::map::Map;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

const TOKEN_DECIMALS: u8 = 12;
const TOKEN_SYMBOL: &str = "BSX";
const PROTOCOL_ID: &str = "bsx";
// The URL for the telemetry server.
const TELEMETRY_URLS: [&str; 2] = [
	"wss://telemetry.polkadot.io/submit/",
	"wss://telemetry.hydradx.io:9000/submit/",
];

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

pub fn testnet_parachain_config(para_id: ParaId) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
	let mut properties = Map::new();
	properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Basilisk Egg",
		// ID
		"basilisk_egg",
		ChainType::Live,
		move || {
			parachain_genesis(
				wasm_binary,
				// Sudo account
				hex!["30035c21ba9eda780130f2029a80c3e962f56588bc04c36be95a225cb536fb55"].into(),
				vec![
					hex!["da0fa4ab419def66fb4ac5224e594e82c34ee795268fc7787c8a096c4ff14f11"].unchecked_into(),
					hex!["ecd7a5439c6ab0cd6550bc2f1cef5299d425bb95bb6d7afb32aa3d95ee4f7f1f"].unchecked_into(),
					hex!["f0ad6f1aae7a445c1e80cac883096ec8177eda276fec53ad9ccbe570f3090a26"].unchecked_into(),
				],
				// Pre-funded accounts
				vec![hex!["30035c21ba9eda780130f2029a80c3e962f56588bc04c36be95a225cb536fb55"].into()],
				true,
				para_id,
				//council
				vec![hex!["30035c21ba9eda780130f2029a80c3e962f56588bc04c36be95a225cb536fb55"].into()],
				//technical committee
				vec![hex!["30035c21ba9eda780130f2029a80c3e962f56588bc04c36be95a225cb536fb55"].into()],
				hex!["30035c21ba9eda780130f2029a80c3e962f56588bc04c36be95a225cb536fb55"].into(), // SAME AS ROOT
			)
		},
		// Bootnodes
		vec![
            "/dns/p2p-01.basilisk-testnet.hydradx.io/tcp/30333/p2p/12D3KooW9qapYrocm6W1meShf8eQfeJzbry9PN2CN6SfBGbymxPL"
                .parse()
                .unwrap(),
            "/dns/p2p-02.basilisk-testnet.hydradx.io/tcp/30333/p2p/12D3KooWPS16BYW173YxmxEJpQBoDz1t3Ht4yaPwwg5qCTED7N66"
                .parse()
                .unwrap(),
            "/dns/p2p-03.basilisk-testnet.hydradx.io/tcp/30333/p2p/12D3KooWRMgQRtYrWsLvuwg3V3aQEvMgsbb88T29cKCTH6RAxTaj"
                .parse()
                .unwrap(),
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
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "westend".into(),
			para_id: para_id.into(),
		},
	))
}

pub fn parachain_development_config(para_id: ParaId) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
	let mut properties = Map::new();
	properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
	properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Basilisk Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			parachain_genesis(
				wasm_binary,
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
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
				get_account_id_from_seed::<sr25519::Public>("Alice"),  // SAME AS ROOT
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
		"Basilisk Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			parachain_genesis(
				wasm_binary,
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
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

/// Configure initial storage state for FRAME modules.
fn parachain_genesis(
	wasm_binary: &[u8],
	root_key: AccountId,
	initial_authorities: Vec<AuraId>,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
	parachain_id: ParaId,
	council_members: Vec<AccountId>,
	tech_committee_members: Vec<AccountId>,
	tx_fee_payment_account: AccountId,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: BalancesConfig {
			// Configure endowed accounts with initial balance of a lot.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1_000_000_000_000_000_000_000u128))
				.collect(),
		},
		pallet_sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		pallet_aura: AuraConfig {
			authorities: initial_authorities,
		},
		pallet_asset_registry: AssetRegistryConfig {
			core_asset_id: CORE_ASSET_ID,
			asset_ids: vec![
				(b"hKSM".to_vec(), 1),
				(b"hDOT".to_vec(), 2),
				(b"hETH".to_vec(), 3),
				(b"hUSDT".to_vec(), 4),
			],
			next_asset_id: 5,
		},
		pallet_transaction_multi_payment: MultiTransactionPaymentConfig {
			currencies: vec![],
			authorities: vec![],
			fallback_account: tx_fee_payment_account,
		},
		orml_tokens: TokensConfig {
			endowed_accounts: endowed_accounts
				.iter()
				.flat_map(|x| {
					vec![
						(x.clone(), 1, 1_000_000_000_000_000_000_000u128),
						(x.clone(), 2, 1_000_000_000_000_000_000_000u128),
						(x.clone(), 3, 1_000_000_000_000_000_000_000u128),
						(x.clone(), 4, 1_000_000_000_000_000_000_000u128),
					]
				})
				.collect(),
		},
		pallet_treasury: Default::default(),
		pallet_collective_Instance1: CouncilConfig {
			members: council_members,
			phantom: Default::default(),
		},
		pallet_collective_Instance2: TechnicalCommitteeConfig {
			members: tech_committee_members,
			phantom: Default::default(),
		},
		orml_nft: OrmlNftConfig {
			tokens: Default::default(),
		},
		pallet_vesting: VestingConfig { vesting: vec![] },
		parachain_info: ParachainInfoConfig { parachain_id },
		cumulus_pallet_aura_ext: Default::default(),
	}
}
