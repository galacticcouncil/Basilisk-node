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
use polkadot_primitives::v7::{BlockNumber, MAX_CODE_SIZE, MAX_POV_SIZE};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use pretty_assertions::assert_eq;
use primitives::{constants::time::SLOT_DURATION, AssetId, Balance};
use sp_consensus_aura::AURA_ENGINE_ID;
use sp_consensus_babe::digests::SecondaryPlainPreDigest;
use sp_consensus_babe::BABE_ENGINE_ID;
use sp_consensus_slots::{Slot, SlotDuration};
use sp_core::{storage::Storage, Encode};
use sp_runtime::{traits::AccountIdConversion, BuildStorage, Digest, DigestItem};

use primitives::constants::chain::CORE_ASSET_ID;
pub use xcm_emulator::Network;
use xcm_emulator::{decl_test_networks, decl_test_parachains, decl_test_relay_chains};

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
	#[api_version(11)]
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
			set_para_slot_info(0);
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
			set_para_slot_info(0);
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
				epoch_config: rococo_runtime::BABE_GENESIS_EPOCH_CONFIG,
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

pub fn go_to_block(target_block: BlockNumber) {
	while rococo_runtime::System::block_number() < target_block {
		let stop = rococo_runtime::System::block_number() == target_block - 1;
		go_to_next_block(true, !stop);
	}
}

pub fn initialize_rococo_block(target_block: BlockNumber, target_slot: Slot) {
	use sp_consensus_babe::digests::PreDigest;

	let authority_index: u32 = 0;

	rococo_runtime::System::set_block_number(target_block);

	rococo_runtime::System::initialize(
		&target_block,
		&Default::default(),
		&Digest {
			logs: vec![DigestItem::PreRuntime(
				BABE_ENGINE_ID,
				PreDigest::SecondaryPlain(SecondaryPlainPreDigest {
					authority_index,
					slot: target_slot,
				})
				.encode(),
			)],
		},
	);
	rococo_runtime::System::on_initialize(target_block);
	rococo_runtime::Scheduler::on_initialize(target_block);
	rococo_runtime::Preimage::on_initialize(target_block);
	rococo_runtime::Babe::on_initialize(target_block);
	rococo_runtime::Timestamp::on_initialize(target_block);
	rococo_runtime::Session::on_initialize(target_block);
	rococo_runtime::Grandpa::on_initialize(target_block);
	rococo_runtime::ParachainsOrigin::on_initialize(target_block);
	rococo_runtime::ParasShared::on_initialize(target_block);
	rococo_runtime::ParaInclusion::on_initialize(target_block);
	// rococo_runtime::ParaInherent::on_initialize(target_block);
	rococo_runtime::ParaScheduler::on_initialize(target_block);
	rococo_runtime::Paras::on_initialize(target_block);
	rococo_runtime::Initializer::on_initialize(target_block);
	rococo_runtime::Dmp::on_initialize(target_block);
	rococo_runtime::Hrmp::on_initialize(target_block);
	rococo_runtime::ParaSessionInfo::on_initialize(target_block);
	rococo_runtime::Slots::on_initialize(target_block);
	rococo_runtime::XcmPallet::on_initialize(target_block);
	rococo_runtime::MessageQueue::on_initialize(target_block);
	rococo_runtime::Beefy::on_initialize(target_block);
	assert_ok!(rococo_runtime::Timestamp::set(
		rococo_runtime::RuntimeOrigin::none(),
		SLOT_DURATION * *target_slot
	));
	// rococo_runtime::AllPalletsWithSystem::on_initialize(target_block);
}

pub fn initialize_basilisk_block(target_block: BlockNumber, target_slot: Slot) {
	// Force a new Basilisk block to be created

	basilisk_runtime::System::set_block_number(target_block);
	basilisk_runtime::System::initialize(
		&target_block,
		&Default::default(),
		&Digest {
			logs: vec![DigestItem::PreRuntime(AURA_ENGINE_ID, target_slot.encode())],
		},
	);

	basilisk_runtime::System::on_initialize(target_block);
	// basilisk_runtime::Timestamp::on_initialize(target_block);
	basilisk_runtime::Session::on_initialize(target_block);
	basilisk_runtime::Aura::on_initialize(target_block);
	basilisk_runtime::AuraExt::on_initialize(target_block);
	basilisk_runtime::RelayChainInfo::on_initialize(target_block);
	basilisk_runtime::Scheduler::on_initialize(target_block);
	basilisk_runtime::ParachainSystem::on_initialize(target_block);
	basilisk_runtime::ParachainInfo::on_initialize(target_block);
	basilisk_runtime::PolkadotXcm::on_initialize(target_block);
	basilisk_runtime::CumulusXcm::on_initialize(target_block);
	basilisk_runtime::XcmpQueue::on_initialize(target_block);
	basilisk_runtime::MessageQueue::on_initialize(target_block);
	basilisk_runtime::MultiTransactionPayment::on_initialize(target_block);
	basilisk_runtime::EmaOracle::on_initialize(target_block);
	// assert_ok!(basilisk_runtime::Timestamp::set(
	// 	basilisk_runtime::RuntimeOrigin::none(),
	// 	SLOT_DURATION * *target_slot
	// ));

	// basilisk_runtime::AllPalletsWithSystem::on_initialize(target_block);
	set_validation_data(target_block, target_slot);
}

pub fn finalize_basilisk_block(target_block: BlockNumber) {
	use frame_support::traits::OnFinalize;

	basilisk_runtime::System::on_finalize(target_block);
	// basilisk_runtime::Timestamp::on_finalize(target_block);
	basilisk_runtime::Session::on_finalize(target_block);
	basilisk_runtime::Aura::on_finalize(target_block);
	basilisk_runtime::AuraExt::on_finalize(target_block);
	basilisk_runtime::RelayChainInfo::on_finalize(target_block);
	basilisk_runtime::Scheduler::on_finalize(target_block);
	basilisk_runtime::ParachainSystem::on_finalize(target_block);
	basilisk_runtime::ParachainInfo::on_finalize(target_block);
	basilisk_runtime::PolkadotXcm::on_finalize(target_block);
	basilisk_runtime::CumulusXcm::on_finalize(target_block);
	basilisk_runtime::XcmpQueue::on_finalize(target_block);
	basilisk_runtime::MessageQueue::on_finalize(target_block);
	basilisk_runtime::MultiTransactionPayment::on_finalize(target_block);
	basilisk_runtime::EmaOracle::on_finalize(target_block);
	basilisk_runtime::System::finalize();
}

pub fn finalize_rococo_block(target_block: BlockNumber) {
	use frame_support::traits::OnFinalize;
	rococo_runtime::System::on_finalize(target_block);
	rococo_runtime::Scheduler::on_finalize(target_block);
	rococo_runtime::Preimage::on_finalize(target_block);
	rococo_runtime::Babe::on_finalize(target_block);
	rococo_runtime::Timestamp::on_finalize(target_block);
	rococo_runtime::Session::on_finalize(target_block);
	rococo_runtime::Grandpa::on_finalize(target_block);
	rococo_runtime::ParachainsOrigin::on_finalize(target_block);
	rococo_runtime::ParasShared::on_finalize(target_block);
	rococo_runtime::ParaInclusion::on_finalize(target_block);
	// rococo_runtime::ParaInherent::on_finalize(target_block);
	rococo_runtime::ParaScheduler::on_finalize(target_block);
	rococo_runtime::Paras::on_finalize(target_block);
	rococo_runtime::Initializer::on_finalize(target_block);
	rococo_runtime::Dmp::on_finalize(target_block);
	rococo_runtime::Hrmp::on_finalize(target_block);
	rococo_runtime::ParaSessionInfo::on_finalize(target_block);
	rococo_runtime::Slots::on_finalize(target_block);
	rococo_runtime::XcmPallet::on_finalize(target_block);
	rococo_runtime::MessageQueue::on_finalize(target_block);
	rococo_runtime::Beefy::on_finalize(target_block);
	// rococo_runtime::System::finalize();

	// rococo_runtime::AllPalletsWithSystem::on_finalize(target_block);
	//
}

pub fn go_to_next_block(initialize: bool, finalize: bool) {
	let current_block = rococo_runtime::System::block_number();
	let current_para_block = basilisk_runtime::System::block_number();
	let target_relay_block = current_block + 1;
	let target_para_block = current_para_block + 1;

	// Advance the relaychain block

	let slot = Slot::from_timestamp(
		(pallet_timestamp::Now::<rococo_runtime::Runtime>::get() + SLOT_DURATION).into(),
		SlotDuration::from_millis(SLOT_DURATION),
	);

	if initialize {
		initialize_rococo_block(target_relay_block, slot);
		initialize_basilisk_block(target_para_block, slot);
	}

	if finalize {
		finalize_basilisk_block(target_para_block);
		finalize_rococo_block(target_relay_block);
	}
}

pub fn set_validation_data(next_block: u32, slot: Slot) {
	use basilisk_runtime::RuntimeOrigin;
	use frame_support::storage::storage_prefix;
	use polkadot_primitives::HeadData;

	let parent_head = HeadData(b"deadbeef".into());
	let sproof_builder = RelayStateSproofBuilder {
		para_id: basilisk_runtime::ParachainInfo::parachain_id(),
		included_para_head: Some(parent_head.clone()),
		current_slot: slot,
		..Default::default()
	};

	let (relay_storage_root, proof) = sproof_builder.into_state_root_and_proof();

	assert_ok!(basilisk_runtime::ParachainSystem::set_validation_data(
		RuntimeOrigin::none(),
		cumulus_primitives_parachain_inherent::ParachainInherentData {
			validation_data: cumulus_primitives_core::PersistedValidationData {
				parent_head,
				relay_parent_number: next_block,
				relay_parent_storage_root: relay_storage_root,
				max_pov_size: Default::default(),
			},
			relay_chain_state: proof,
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
		}
	));

	sp_io::storage::clear(&storage_prefix(b"ParachainSystem", b"UnincludedSegment"));
}

pub fn set_para_slot_info(number: u64) {
	// sp_io::storage::clear(&frame_support::storage::storage_prefix(
	// 	b"ParachainSystem",
	// 	b"UnincludedSegment",
	// ));
	frame_support::storage::unhashed::put(
		&frame_support::storage::storage_prefix(b"AuraExt", b"SlotInfo"),
		&(Slot::from(number), 0),
	);
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
