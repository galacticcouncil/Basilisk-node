#![cfg(test)]
pub use basilisk_runtime::{AccountId, VestingPalletId};
use pallet_transaction_multi_payment::Price;
use primitives::{AssetId, Balance};

pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];
pub const CHARLIE: [u8; 32] = [6u8; 32];
pub const DAVE: [u8; 32] = [7u8; 32];

pub const UNITS: Balance = 1_000_000_000_000;

pub const BSX: AssetId = CORE_ASSET_ID;
pub const AUSD: AssetId = 1;
pub const MOVR: AssetId = 2;
pub const KSM: AssetId = 3;
pub const NEW_BOOTSTRAPPED_TOKEN: AssetId = 4;

pub const ALICE_INITIAL_BSX_BALANCE: u128 = 200 * UNITS;
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

use cumulus_primitives_core::ParaId;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::assert_ok;
use frame_support::traits::GenesisBuild;
use polkadot_primitives::v1::{BlockNumber, MAX_CODE_SIZE, MAX_POV_SIZE};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use pretty_assertions::assert_eq;
use sp_runtime::traits::AccountIdConversion;

use primitives::constants::chain::CORE_ASSET_ID;
use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

decl_test_relay_chain! {
	pub struct KusamaRelay {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		new_ext = kusama_ext(),
	}
}

decl_test_parachain! {
	pub struct Basilisk{
		Runtime = basilisk_runtime::Runtime,
		Origin = basilisk_runtime::Origin,
		XcmpMessageHandler = basilisk_runtime::XcmpQueue,
		DmpMessageHandler = basilisk_runtime::DmpQueue,
		new_ext = basilisk_ext(),
	}
}

decl_test_parachain! {
	pub struct Hydra{
		Runtime = basilisk_runtime::Runtime,
		Origin = basilisk_runtime::Origin,
		XcmpMessageHandler = basilisk_runtime::XcmpQueue,
		DmpMessageHandler = basilisk_runtime::DmpQueue,
		new_ext = hydra_ext(),
	}
}

decl_test_network! {
	pub struct TestNet {
		relay_chain = KusamaRelay,
		parachains = vec![
			(2000, Basilisk),
			(3000, Hydra),
		],
	}
}

fn default_parachains_host_configuration() -> HostConfiguration<BlockNumber> {
	HostConfiguration {
		minimum_validation_upgrade_delay: 5,
		validation_upgrade_cooldown: 5u32,
		validation_upgrade_delay: 5,
		code_retention_period: 1200,
		max_code_size: MAX_CODE_SIZE,
		max_pov_size: MAX_POV_SIZE,
		max_head_data_size: 32 * 1024,
		group_rotation_frequency: 20,
		chain_availability_period: 4,
		thread_availability_period: 4,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024 * 1024,
		max_downward_message_size: 1024,
		ump_service_total_weight: 4 * 1_000_000_000,
		max_upward_message_size: 1024 * 1024,
		max_upward_message_num_per_candidate: 5,
		hrmp_sender_deposit: 0,
		hrmp_recipient_deposit: 0,
		hrmp_channel_max_capacity: 8,
		hrmp_channel_max_total_size: 8 * 1024,
		hrmp_max_parachain_inbound_channels: 4,
		hrmp_max_parathread_inbound_channels: 4,
		hrmp_channel_max_message_size: 1024 * 1024,
		hrmp_max_parachain_outbound_channels: 4,
		hrmp_max_parathread_outbound_channels: 4,
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

pub fn kusama_ext() -> sp_io::TestExternalities {
	use kusama_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from(ALICE), 2002 * UNITS),
			(ParaId::from(2000).into_account(), 10 * UNITS),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
		config: default_parachains_host_configuration(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&pallet_xcm::GenesisConfig {
			safe_xcm_version: Some(2),
		},
		&mut t,
	)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn hydra_ext() -> sp_io::TestExternalities {
	use basilisk_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(AccountId::from(ALICE), 200 * UNITS)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	<parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&parachain_info::GenesisConfig {
			parachain_id: 3000.into(),
		},
		&mut t,
	)
	.unwrap();

	<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&pallet_xcm::GenesisConfig {
			safe_xcm_version: Some(2),
		},
		&mut t,
	)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn basilisk_ext() -> sp_io::TestExternalities {
	use basilisk_runtime::{MultiTransactionPayment, NativeExistentialDeposit, Runtime, System};
	use frame_support::traits::OnInitialize;

	let existential_deposit = NativeExistentialDeposit::get();

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from(ALICE), ALICE_INITIAL_BSX_BALANCE),
			(AccountId::from(BOB), BOB_INITIAL_BSX_BALANCE),
			(AccountId::from(CHARLIE), CHARLIE_INITIAL_BSX_BALANCE),
			(AccountId::from(DAVE), DAVE_INITIAL_BSX_BALANCE),
			(vesting_account(), VESTING_ACCOUNT_INITIAL_BSX_BALANCE),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_asset_registry::GenesisConfig::<Runtime> {
		asset_names: vec![
			(b"AUSD".to_vec(), 1_000_000u128),
			(b"MOVR".to_vec(), 1_000u128),
			(b"KSM".to_vec(), 1_000u128),
			(b"NEW_BOOTSRAPPED_TOKEN".to_vec(), 1_000u128),
		],
		native_asset_name: b"BSX".to_vec(),
		native_existential_deposit: existential_deposit,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	<parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&parachain_info::GenesisConfig {
			parachain_id: 2000u32.into(),
		},
		&mut t,
	)
	.unwrap();
	orml_tokens::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from(ALICE), AUSD, ALICE_INITIAL_AUSD_BALANCE * UNITS),
			(AccountId::from(ALICE), MOVR, ALICE_INITIAL_MOVR_BALANCE * UNITS),
			(AccountId::from(ALICE), KSM, ALICE_INITIAL_KSM_BALANCE * UNITS),
			(
				AccountId::from(ALICE),
				NEW_BOOTSTRAPPED_TOKEN,
				ALICE_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE * UNITS,
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
	}
	.assimilate_storage(&mut t)
	.unwrap();

	<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&pallet_xcm::GenesisConfig {
			safe_xcm_version: Some(2),
		},
		&mut t,
	)
	.unwrap();

	pallet_transaction_multi_payment::GenesisConfig::<Runtime> {
		currencies: vec![(1, Price::from_inner(462_962_963_000_u128))], //0.000_000_462_962_963
		account_currencies: vec![],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		// Make sure the prices are up-to-date.
		MultiTransactionPayment::on_initialize(1);
	});
	ext
}

fn last_basilisk_events(n: usize) -> Vec<basilisk_runtime::Event> {
	frame_system::Pallet::<basilisk_runtime::Runtime>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

pub fn expect_basilisk_events(e: Vec<basilisk_runtime::Event>) {
	assert_eq!(last_basilisk_events(e.len()), e);
}

pub fn vesting_account() -> AccountId {
	VestingPalletId::get().into_account()
}

pub fn set_relaychain_block_number(number: BlockNumber) {
	use basilisk_runtime::Origin;
	use basilisk_runtime::ParachainSystem;
	use frame_support::traits::OnInitialize;

	ParachainSystem::on_initialize(number);

	let (relay_storage_root, proof) = RelayStateSproofBuilder::default().into_state_root_and_proof();

	assert_ok!(ParachainSystem::set_validation_data(
		Origin::none(),
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
