#![cfg(test)]

pub use basilisk_runtime::{AccountId, VestingPalletId};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, Nothing},
	weights::{constants::WEIGHT_PER_SECOND, Pays, Weight},
	PalletId,
};
use hydradx_adapters::{MultiCurrencyTrader, ToFeeReceiver};

use orml_xcm_support::{DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
use sp_runtime::traits::Convert;
use sp_runtime::Perbill;

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use orml_traits::parameter_type_with_key;

use pallet_transaction_multi_payment::{DepositAll, Price, TransferFees};
use polkadot_xcm::latest::prelude::*;
use primitives::Balance;

use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use primitives::{constants::currency::*, AssetId};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use xcm_builder::{
	AccountId32Aliases, AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds, LocationInverter, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation,
};
use xcm_executor::{Config, XcmExecutor};
pub type Amount = i128;

pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];
pub const CHARLIE: [u8; 32] = [6u8; 32];
pub const DAVE: [u8; 32] = [7u8; 32];

pub const UNITS: Balance = 1_000_000_000_000;

pub const KARURA_PARA_ID: u32 = 2000;
pub const BASILISK_PARA_ID: u32 = 2090;
pub type BlockNumberKarura = u64;

use cumulus_primitives_core::ParaId;
use frame_support::traits::{GenesisBuild};
use hydradx_traits::pools::SpotPriceProvider;
use orml_currencies::BasicCurrencyAdapter;
use pallet_transaction_payment::TargetedFeeAdjustment;
use polkadot_primitives::v1::{BlockNumber, MAX_CODE_SIZE, MAX_POV_SIZE};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use polkadot_xcm::prelude::MultiLocation;
use pretty_assertions::assert_eq;
use sp_arithmetic::FixedU128;
use sp_runtime::traits::AccountIdConversion;

use basilisk_runtime::{AdjustmentVariable, MinimumMultiplier, TargetBlockFullness, WeightToFee};
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
	pub struct Karura{
		Runtime = karura_runtime_mock::KaruraRuntime,
		Origin = karura_runtime_mock::Origin,
		XcmpMessageHandler = karura_runtime_mock::XcmpQueue,
		DmpMessageHandler = karura_runtime_mock::DmpQueue,
		new_ext = karura_ext(),
	}
}

decl_test_network! {
	pub struct TestNet {
		relay_chain = KusamaRelay,
		parachains = vec![
			(2000, Karura),
			(2090, Basilisk),
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
			(ParaId::from(BASILISK_PARA_ID).into_account(), 10 * UNITS),
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

pub fn basilisk_ext() -> sp_io::TestExternalities {
	use basilisk_runtime::{MultiTransactionPayment, NativeExistentialDeposit, Runtime, System};
	use frame_support::traits::OnInitialize;

	let existential_deposit = NativeExistentialDeposit::get();

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from(ALICE), 200 * UNITS),
			(AccountId::from(BOB), 1000 * UNITS),
			(AccountId::from(CHARLIE), 1000 * UNITS),
			(AccountId::from(DAVE), 1000 * UNITS),
			(vesting_account(), 1_000_000 * UNITS),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_asset_registry::GenesisConfig::<Runtime> {
		asset_names: vec![(b"KSM".to_vec(), 1_000_000u128), (b"aUSD".to_vec(), 1_000u128)],
		native_asset_name: b"BSX".to_vec(),
		native_existential_deposit: existential_deposit,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	<parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
		&parachain_info::GenesisConfig {
			parachain_id: BASILISK_PARA_ID.into(),
		},
		&mut t,
	)
	.unwrap();
	orml_tokens::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from(ALICE), 1, 200 * UNITS),
			(AccountId::from(ALICE), 2, 200 * UNITS),
			(AccountId::from(BOB), 1, 1_000 * UNITS),
			(AccountId::from(CHARLIE), 1, 1000 * UNITS),
			(AccountId::from(DAVE), 1, 1_000 * UNITS),
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

pub fn karura_ext() -> sp_io::TestExternalities {
	use karura_runtime_mock::{MultiTransactionPayment, NativeExistentialDeposit, KaruraRuntime, System};
	use frame_support::traits::OnInitialize;

	let existential_deposit = NativeExistentialDeposit::get();

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<KaruraRuntime>()
		.unwrap();

	pallet_balances::GenesisConfig::<KaruraRuntime> {
		balances: vec![
			(AccountId::from(ALICE), 200 * UNITS),
			(AccountId::from(BOB), 1000 * UNITS),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	orml_tokens::GenesisConfig::<KaruraRuntime> {
		balances: vec![
			(AccountId::from(ALICE), 1, 200 * UNITS),
			(AccountId::from(BOB), 1, 1_000 * UNITS),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	<parachain_info::GenesisConfig as GenesisBuild<KaruraRuntime>>::assimilate_storage(
		&parachain_info::GenesisConfig {
			parachain_id: KARURA_PARA_ID.into(),
		},
		&mut t,
	)
	.unwrap();

	<pallet_xcm::GenesisConfig as GenesisBuild<KaruraRuntime>>::assimilate_storage(
		&pallet_xcm::GenesisConfig {
			safe_xcm_version: Some(2),
		},
		&mut t,
	)
	.unwrap();

	pallet_asset_registry::GenesisConfig::<KaruraRuntime> {
		asset_names: vec![(b"KSM".to_vec(), 1_000_000u128), (b"aUSD".to_vec(), 1_000u128)],
		native_asset_name: b"KAR".to_vec(),
		native_existential_deposit: existential_deposit,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_transaction_multi_payment::GenesisConfig::<KaruraRuntime> {
		currencies: vec![(1, Price::from_inner(462_962_963_000_u128))], //0.000_000_462_962_963
		account_currencies: vec![],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
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