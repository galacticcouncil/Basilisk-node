#![allow(clippy::type_complexity)]
#![cfg(test)]

pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];
pub const CHARLIE: [u8; 32] = [6u8; 32];
pub const DAVE: [u8; 32] = [7u8; 32];

pub const UNITS: Balance = 1_000_000_000_000;

pub const OTHER_PARA_ID: u32 = 2000;
pub const BASILISK_PARA_ID: u32 = 2090;

pub const BSX: AssetId = CORE_ASSET_ID;
pub const AUSD: AssetId = 1;
pub const MOVR: AssetId = 2;
pub const KSM: AssetId = 3;
pub const NEW_BOOTSTRAPPED_TOKEN: AssetId = 4;
pub const KAR: AssetId = 5;

pub const ALICE_INITIAL_BSX_BALANCE: u128 = 1_000 * UNITS;
pub const BOB_INITIAL_BSX_BALANCE: u128 = 1000 * UNITS;
pub const CHARLIE_INITIAL_BSX_BALANCE: u128 = 1000 * UNITS;
pub const DAVE_INITIAL_BSX_BALANCE: u128 = 1000 * UNITS;
pub const VESTING_ACCOUNT_INITIAL_BSX_BALANCE: u128 = 1_000_000 * UNITS;

pub const BOB_INITIAL_AUSD_BALANCE: u128 = 1000 * UNITS;
pub const BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE: u128 = 1000 * UNITS;
pub const CHARLIE_INITIAL_AUSD_BALANCE: u128 = 1000 * UNITS;
pub const DAVE_INITIAL_AUSD_BALANCE: u128 = 1000 * UNITS;
pub const ALICE_INITIAL_AUSD_BALANCE: u128 = 400 * UNITS;
pub const ALICE_INITIAL_MOVR_BALANCE: u128 = 200 * UNITS;
pub const ALICE_INITIAL_KSM_BALANCE: u128 = 400 * UNITS;
pub const ALICE_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE: u128 = 400 * UNITS;

pub const ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN: u128 = 200 * UNITS;
pub const ALICE_INITIAL_AUSD_BALANCE_ON_OTHER_PARACHAIN: u128 = 200 * UNITS;
pub const BOB_INITIAL_AUSD_BALANCE_ON_OTHER_PARACHAIN: u128 = 1000 * UNITS;

pub fn parachain_reserve_account() -> AccountId {
	polkadot_parachain::primitives::Sibling::from(OTHER_PARA_ID).into_account_truncating()
}

pub use basilisk_runtime::{AccountId, VestingPalletId};
use cumulus_primitives_core::ParaId;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::assert_ok;
use frame_support::traits::OnInitialize;
use pallet_transaction_multi_payment::Price;
pub use pallet_xyk::types::AssetPair;
use polkadot_primitives::v6::{BlockNumber, MAX_CODE_SIZE, MAX_POV_SIZE};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use pretty_assertions::assert_eq;
use primitives::{AssetId, Balance};
use sp_core::storage::Storage;
use sp_runtime::{traits::AccountIdConversion, BuildStorage};

use primitives::constants::chain::CORE_ASSET_ID;
pub use xcm_emulator::Network;
use xcm_emulator::{decl_test_networks, decl_test_parachains, decl_test_relay_chains};

pub type Rococo = RococoRelayChain<TestNet>;
pub type Basilisk = BasiliskParachain<TestNet>;
pub type OtherParachain = OtherPara<TestNet>;

decl_test_networks! {
	pub struct TestNet {
		relay_chain = RococoRelayChain,
		parachains = vec![
			OtherPara,
			BasiliskParachain,
		],
		bridge = ()
	},
}

decl_test_relay_chains! {
	#[api_version(10)]
	pub struct RococoRelayChain {
		genesis = rococo::genesis(),
		on_init = (),
		runtime = rococo_runtime,
		core = {
			SovereignAccountOf: rococo_runtime::xcm_config::LocationConverter,
		},
		pallets = {
			XcmPallet: rococo_runtime::XcmPallet,
			Balances: rococo_runtime::Balances,
			Hrmp: rococo_runtime::Hrmp,
		}
	}
}

decl_test_parachains! {
	pub struct BasiliskParachain {
		genesis = basilisk::genesis(),
		on_init = {
			basilisk_runtime::System::set_block_number(1);
			// Make sure the prices are up-to-date.
			basilisk_runtime::MultiTransactionPayment::on_initialize(1);
		},
		runtime = basilisk_runtime,
		core = {
			XcmpMessageHandler: basilisk_runtime::XcmpQueue,
			LocationToAccountId: basilisk_runtime::xcm::LocationToAccountId,
			ParachainInfo: basilisk_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: basilisk_runtime::PolkadotXcm,
			Balances: basilisk_runtime::Balances,
		}
	},
	pub struct OtherPara {
		genesis = other_parachain::genesis(),
		on_init = {
			basilisk_runtime::System::set_block_number(1);
			// Make sure the prices are up-to-date.
			basilisk_runtime::MultiTransactionPayment::on_initialize(1);
		},
		runtime = basilisk_runtime,
		core = {
			XcmpMessageHandler: basilisk_runtime::XcmpQueue,
			LocationToAccountId: basilisk_runtime::LocationToAccountId,
			ParachainInfo: basilisk_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: basilisk_runtime::PolkadotXcm,
			Balances: basilisk_runtime::Balances,
		}
	}
}

pub mod rococo {
	use super::*;

	fn get_host_configuration() -> HostConfiguration<BlockNumber> {
		HostConfiguration {
			minimum_validation_upgrade_delay: 5,
			validation_upgrade_cooldown: 5u32,
			validation_upgrade_delay: 5,
			code_retention_period: 1200,
			max_code_size: MAX_CODE_SIZE,
			max_pov_size: MAX_POV_SIZE,
			max_head_data_size: 32 * 1024,
			group_rotation_frequency: 20,
			paras_availability_period: 4,
			max_upward_queue_count: 8,
			max_upward_queue_size: 1024 * 1024,
			max_downward_message_size: 1024,
			max_upward_message_size: 50 * 1024,
			max_upward_message_num_per_candidate: 5,
			hrmp_sender_deposit: 0,
			hrmp_recipient_deposit: 0,
			hrmp_channel_max_capacity: 8,
			hrmp_channel_max_total_size: 8 * 1024,
			hrmp_max_parachain_inbound_channels: 4,
			hrmp_channel_max_message_size: 1024 * 1024,
			hrmp_max_parachain_outbound_channels: 4,
			hrmp_max_message_num_per_candidate: 5,
			dispute_period: 6,
			no_show_slots: 2,
			n_delay_tranches: 25,
			needed_approvals: 2,
			relay_vrf_modulo_samples: 2,
			zeroth_delay_tranche_width: 0,
			..Default::default()
		}
	}

	use polkadot_primitives::{AssignmentId, ValidatorId};
	use polkadot_service::chain_spec::get_authority_keys_from_seed_no_beefy;
	use sc_consensus_grandpa::AuthorityId as GrandpaId;
	use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
	use sp_consensus_babe::AuthorityId as BabeId;
	use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;

	use sp_core::{Pair, Public};

	/// Helper function to generate a crypto pair from seed
	fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
		TPublic::Pair::from_string(&format!("//{}", seed), None)
			.expect("static values are valid; qed")
			.public()
	}

	fn session_keys(
		babe: BabeId,
		grandpa: GrandpaId,
		para_validator: ValidatorId,
		para_assignment: AssignmentId,
		authority_discovery: AuthorityDiscoveryId,
		beefy: BeefyId,
	) -> rococo_runtime::SessionKeys {
		rococo_runtime::SessionKeys {
			babe,
			grandpa,
			para_validator,
			para_assignment,
			authority_discovery,
			beefy,
		}
	}

	pub fn initial_authorities() -> Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
		BeefyId,
	)> {
		let no_beefy = get_authority_keys_from_seed_no_beefy("Alice");
		let with_beefy = (
			no_beefy.0,
			no_beefy.1,
			no_beefy.2,
			no_beefy.3,
			no_beefy.4,
			no_beefy.5,
			no_beefy.6,
			get_from_seed::<BeefyId>("Alice"),
		);
		vec![with_beefy]
	}

	pub fn genesis() -> Storage {
		let genesis_config = rococo_runtime::RuntimeGenesisConfig {
			balances: rococo_runtime::BalancesConfig {
				balances: vec![
					(AccountId::from(ALICE), 2002 * UNITS),
					(ParaId::from(BASILISK_PARA_ID).into_account_truncating(), 10 * UNITS),
				],
			},
			session: rococo_runtime::SessionConfig {
				keys: initial_authorities()
					.iter()
					.map(|x| {
						(
							x.0.clone(),
							x.0.clone(),
							session_keys(
								x.2.clone(),
								x.3.clone(),
								x.4.clone(),
								x.5.clone(),
								x.6.clone(),
								x.7.clone(),
							),
						)
					})
					.collect::<Vec<_>>(),
			},
			babe: rococo_runtime::BabeConfig {
				authorities: Default::default(),
				epoch_config: Some(rococo_runtime::BABE_GENESIS_EPOCH_CONFIG),
				..Default::default()
			},
			configuration: rococo_runtime::ConfigurationConfig {
				config: get_host_configuration(),
			},

			xcm_pallet: rococo_runtime::XcmPalletConfig {
				safe_xcm_version: Some(3),
				..Default::default()
			},
			..Default::default()
		};

		genesis_config.build_storage().unwrap()
	}
}

pub mod basilisk {
	use super::*;
	use sp_core::{sr25519, Pair, Public};
	use sp_runtime::{
		traits::{IdentifyAccount, Verify},
		MultiSignature,
	};
	type AccountPublic = <MultiSignature as Verify>::Signer;

	/// Helper function to generate a crypto pair from seed
	fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
		TPublic::Pair::from_string(&format!("//{}", seed), None)
			.expect("static values are valid; qed")
			.public()
	}

	/// Helper function to generate an account ID from seed.
	fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
	where
		AccountPublic: From<<TPublic::Pair as Pair>::Public>,
	{
		AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
	}

	pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

	pub fn invulnerables() -> Vec<(AccountId, AuraId)> {
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_from_seed::<AuraId>("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_from_seed::<AuraId>("Bob"),
			),
		]
	}

	pub fn genesis() -> Storage {
		use basilisk_runtime::NativeExistentialDeposit;

		let existential_deposit = NativeExistentialDeposit::get();

		let genesis_config = basilisk_runtime::RuntimeGenesisConfig {
			balances: basilisk_runtime::BalancesConfig {
				balances: vec![
					(AccountId::from(ALICE), ALICE_INITIAL_BSX_BALANCE),
					(AccountId::from(BOB), BOB_INITIAL_BSX_BALANCE),
					(AccountId::from(CHARLIE), CHARLIE_INITIAL_BSX_BALANCE),
					(AccountId::from(DAVE), DAVE_INITIAL_BSX_BALANCE),
					(vesting_account(), VESTING_ACCOUNT_INITIAL_BSX_BALANCE),
				],
			},
			collator_selection: basilisk_runtime::CollatorSelectionConfig {
				invulnerables: invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: 2 * UNITS,
				..Default::default()
			},
			session: basilisk_runtime::SessionConfig {
				keys: invulnerables()
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),                                    // account id
							acc,                                            // validator id
							basilisk_runtime::opaque::SessionKeys { aura }, // session keys
						)
					})
					.collect(),
			},
			asset_registry: basilisk_runtime::AssetRegistryConfig {
				registered_assets: vec![
					(b"aUSD".to_vec(), 1_000_000u128, Some(AUSD)),
					(b"MOVR".to_vec(), 1_000u128, Some(MOVR)),
					(b"KSMN".to_vec(), 1_000u128, Some(KSM)),
					(
						b"NEW_BOOTSRAPPED_TOKEN".to_vec(),
						1_000u128,
						Some(NEW_BOOTSTRAPPED_TOKEN),
					),
				],
				native_asset_name: b"BSX".to_vec(),
				native_existential_deposit: existential_deposit,
			},
			parachain_info: basilisk_runtime::ParachainInfoConfig {
				parachain_id: BASILISK_PARA_ID.into(),
				..Default::default()
			},
			tokens: basilisk_runtime::TokensConfig {
				balances: vec![
					(AccountId::from(ALICE), AUSD, ALICE_INITIAL_AUSD_BALANCE),
					(AccountId::from(ALICE), MOVR, ALICE_INITIAL_MOVR_BALANCE),
					(AccountId::from(ALICE), KSM, ALICE_INITIAL_KSM_BALANCE),
					(
						AccountId::from(ALICE),
						NEW_BOOTSTRAPPED_TOKEN,
						ALICE_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE,
					),
					(AccountId::from(BOB), AUSD, BOB_INITIAL_AUSD_BALANCE),
					(
						AccountId::from(BOB),
						NEW_BOOTSTRAPPED_TOKEN,
						BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE,
					),
					(AccountId::from(CHARLIE), AUSD, CHARLIE_INITIAL_AUSD_BALANCE),
					(AccountId::from(DAVE), AUSD, DAVE_INITIAL_AUSD_BALANCE),
				],
			},
			polkadot_xcm: basilisk_runtime::PolkadotXcmConfig {
				safe_xcm_version: Some(3),
				..Default::default()
			},
			multi_transaction_payment: basilisk_runtime::MultiTransactionPaymentConfig {
				currencies: vec![(AUSD, Price::from_inner(462_962_963_000_u128))], //0.000_000_462_962_963
				account_currencies: vec![],
			},
			duster: basilisk_runtime::DusterConfig {
				account_blacklist: vec![basilisk_runtime::Treasury::account_id()],
				reward_account: Some(basilisk_runtime::Treasury::account_id()),
				dust_account: Some(basilisk_runtime::Treasury::account_id()),
			},
			..Default::default()
		};

		genesis_config.build_storage().unwrap()
	}
}

pub mod other_parachain {
	use super::*;

	pub fn genesis() -> Storage {
		use basilisk_runtime::NativeExistentialDeposit;

		let existential_deposit = NativeExistentialDeposit::get();

		let genesis_config = basilisk_runtime::RuntimeGenesisConfig {
			balances: basilisk_runtime::BalancesConfig {
				balances: vec![(AccountId::from(ALICE), ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN)],
			},
			collator_selection: basilisk_runtime::CollatorSelectionConfig {
				invulnerables: basilisk::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: 2 * UNITS,
				..Default::default()
			},
			session: basilisk_runtime::SessionConfig {
				keys: basilisk::invulnerables()
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),                                    // account id
							acc,                                            // validator id
							basilisk_runtime::opaque::SessionKeys { aura }, // session keys
						)
					})
					.collect(),
			},
			asset_registry: basilisk_runtime::AssetRegistryConfig {
				registered_assets: vec![(b"AUSD".to_vec(), 1_000_000u128, Some(AUSD))],
				native_asset_name: b"KAR".to_vec(),
				native_existential_deposit: existential_deposit,
			},
			parachain_info: basilisk_runtime::ParachainInfoConfig {
				parachain_id: OTHER_PARA_ID.into(),
				..Default::default()
			},
			tokens: basilisk_runtime::TokensConfig {
				balances: vec![
					(
						AccountId::from(ALICE),
						AUSD,
						ALICE_INITIAL_AUSD_BALANCE_ON_OTHER_PARACHAIN,
					),
					(AccountId::from(BOB), AUSD, BOB_INITIAL_AUSD_BALANCE_ON_OTHER_PARACHAIN),
				],
			},
			polkadot_xcm: basilisk_runtime::PolkadotXcmConfig {
				safe_xcm_version: Some(3),
				..Default::default()
			},
			multi_transaction_payment: basilisk_runtime::MultiTransactionPaymentConfig {
				currencies: vec![(AUSD, Price::from_inner(462_962_963_000_u128))], //0.000_000_462_962_963
				account_currencies: vec![],
			},
			duster: basilisk_runtime::DusterConfig {
				account_blacklist: vec![basilisk_runtime::Treasury::account_id()],
				reward_account: Some(basilisk_runtime::Treasury::account_id()),
				dust_account: Some(basilisk_runtime::Treasury::account_id()),
			},
			..Default::default()
		};
		genesis_config.build_storage().unwrap()
	}
}

pub fn last_basilisk_events(n: usize) -> Vec<basilisk_runtime::RuntimeEvent> {
	frame_system::Pallet::<basilisk_runtime::Runtime>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

#[allow(dead_code)]
pub fn expect_basilisk_events(e: Vec<basilisk_runtime::RuntimeEvent>) {
	assert_eq!(last_basilisk_events(e.len()), e);
}

pub fn expect_basilisk_event(e: basilisk_runtime::RuntimeEvent) {
	let last_10_events = last_basilisk_events(10);
	let mut found = false;
	for event in last_10_events {
		if event == e {
			found = true;
			break;
		}
	}
	assert!(found, "Event not found in the last 10 basilisk events");
}

#[allow(dead_code)]
pub fn last_parachain_events(n: usize) -> Vec<basilisk_runtime::RuntimeEvent> {
	frame_system::Pallet::<basilisk_runtime::Runtime>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

pub fn vesting_account() -> AccountId {
	VestingPalletId::get().into_account_truncating()
}

pub fn set_relaychain_block_number(number: BlockNumber) {
	use basilisk_runtime::ParachainSystem;
	use basilisk_runtime::RuntimeOrigin;

	rococo_run_to_block(number); //We need to set block number this way as well because tarpaulin code coverage tool does not like the way how we set the block number with `cumulus-test-relay-sproof-builder` package

	ParachainSystem::on_initialize(number);

	let (relay_storage_root, proof) = RelayStateSproofBuilder::default().into_state_root_and_proof();

	assert_ok!(ParachainSystem::set_validation_data(
		RuntimeOrigin::none(),
		cumulus_primitives_parachain_inherent::ParachainInherentData {
			validation_data: cumulus_primitives_core::PersistedValidationData {
				parent_head: Default::default(),
				relay_parent_number: number,
				relay_parent_storage_root: relay_storage_root,
				max_pov_size: Default::default(),
			},
			relay_chain_state: proof,
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
		}
	));
}

pub fn rococo_run_to_block(to: BlockNumber) {
	use frame_support::traits::OnFinalize;

	while rococo_runtime::System::block_number() < to {
		let b = rococo_runtime::System::block_number();
		rococo_runtime::System::on_finalize(b);
		rococo_runtime::System::on_initialize(b + 1);
		rococo_runtime::System::set_block_number(b + 1);
	}
}

use xcm_emulator::pallet_message_queue;

pub fn assert_xcm_message_processing_failed() {
	assert!(basilisk_runtime::System::events().iter().any(|r| matches!(
		r.event,
		basilisk_runtime::RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: false, .. })
	)));
}

pub fn assert_xcm_message_processing_passed() {
	assert!(basilisk_runtime::System::events().iter().any(|r| matches!(
		r.event,
		basilisk_runtime::RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true, .. })
	)));
}
