#![cfg(test)]
pub use basilisk_runtime::{AccountId, VestingPalletId};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, Nothing},
	weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use sp_runtime::traits::Convert;

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use orml_traits::parameter_type_with_key;
use frame_system::EnsureSigned;

use pallet_transaction_multi_payment::Price;
use polkadot_xcm::{latest::prelude::*};
use primitives::Balance;

use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use primitives::{AssetId};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use xcm_executor::{Config, XcmExecutor};
use xcm_builder::{
	AccountId32Aliases, AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds, LocationInverter, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation,
};
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
use frame_support::traits::GenesisBuild;
use polkadot_primitives::v1::{BlockNumber, MAX_CODE_SIZE, MAX_POV_SIZE};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use polkadot_xcm::prelude::MultiLocation;
use pretty_assertions::assert_eq;
use sp_runtime::traits::AccountIdConversion;

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
		Runtime = KaruraRuntime,
		Origin = Origin,
		XcmpMessageHandler = XcmpQueue,
		DmpMessageHandler = DmpQueue,
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

pub fn karura_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<KaruraRuntime>()
		.unwrap();

	pallet_balances::GenesisConfig::<KaruraRuntime> {
		balances: vec![(AccountId::from(ALICE), 200 * UNITS)],
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


//Setting up mock for Karura runtime
//TODO: once it works properly, extract it to dedicated file/project
parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 63;
	pub static MockBlockNumberProvider: u64 = 0;
	pub const MaxLocks: u32 = 50;
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxReserves: u32 = 50;
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub const UnitWeightCost: Weight = 10;
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub const MaxInstructions: u32 = 100;
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}
parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		1u128
	};
}

impl frame_system::Config for KaruraRuntime {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumberKarura;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);
pub type LocalAssetTransactor = ();

pub type XcmOriginToCallOrigin = (
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	RelayChainAsNative<RelayChainOrigin, Origin>,
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	SignedAccountId32AsNative<RelayNetwork, Origin>,
	XcmPassthrough<Origin>,
);
pub type LocationToAccountId = (
	ParentIsPreset<AccountId>,
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<RelayNetwork, AccountId>,
);
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToCallOrigin;
	type IsReserve = ();
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type Trader = ();
	type ResponseHandler = ();
	type AssetTrap = ();
	type AssetClaims = ();
	type SubscriptionService = ();
}

pub const CORE_ASSET_ID: AssetId = 0;

parameter_types! {
	pub const ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
	pub const ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
	pub RegistryStringLimit: u32 = 100;
	pub const NativeAssetId : AssetId = CORE_ASSET_ID;

}

impl cumulus_pallet_parachain_system::Config for KaruraRuntime {
	type Event = Event;
	type OnSystemEvent = ();
	type SelfParaId = ParachainInfo;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl cumulus_pallet_xcmp_queue::Config for KaruraRuntime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToCallOrigin;
}



impl pallet_xcm::Config for KaruraRuntime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything	;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type Origin = Origin;
	type Call = Call;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_dmp_queue::Config for KaruraRuntime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}


impl parachain_info::Config for KaruraRuntime {}


impl orml_tokens::Config for KaruraRuntime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = Nothing;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

impl pallet_balances::Config for KaruraRuntime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<KaruraRuntime>;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
}
impl cumulus_pallet_xcm::Config for KaruraRuntime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl pallet_asset_registry::Config for KaruraRuntime {
	type Event = Event;
	type RegistryOrigin = EnsureSigned<AccountId>;
	type AssetId = AssetId;
	type Balance = Balance;
	type AssetNativeLocation = AssetLocation;
	type StringLimit = RegistryStringLimit;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<KaruraRuntime>;
type Block = frame_system::mocking::MockBlock<KaruraRuntime>;

#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct AssetLocation(pub MultiLocation);

impl Default for AssetLocation {
	fn default() -> Self {
		AssetLocation(MultiLocation::here())
	}
}

pub struct CurrencyIdConvert;


impl Convert<AssetId, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(id: AssetId) -> Option<MultiLocation> {
		match id {
			CORE_ASSET_ID => Some(MultiLocation::new(
				1,
				X2(Parachain(ParachainInfo::parachain_id().into()), GeneralIndex(id.into())),
			)),
			_ => {
				if let Some(loc) = AssetRegistry::asset_to_location(id) {
					Some(loc.0)
				} else {
					None
				}
			}
		}
	}
}

impl Convert<MultiLocation, Option<AssetId>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<AssetId> {
		match location {
			MultiLocation {
				parents,
				interior: X2(Parachain(id), GeneralIndex(index)),
			} if parents == 1 && ParaId::from(id) == ParachainInfo::parachain_id() && (index as u32) == CORE_ASSET_ID => {
				// Handling native asset for this parachain
				Some(CORE_ASSET_ID)
			}
			// handle reanchor canonical location: https://github.com/paritytech/polkadot/pull/4470
			MultiLocation {
				parents: 0,
				interior: X1(GeneralIndex(index)),
			} if (index as u32) == CORE_ASSET_ID => Some(CORE_ASSET_ID),
			// delegate to asset-registry
			_ => AssetRegistry::location_to_asset(AssetLocation(location)),
		}
	}
}

impl Convert<MultiAsset, Option<AssetId>> for CurrencyIdConvert {
	fn convert(asset: MultiAsset) -> Option<AssetId> {
		if let MultiAsset {
			id: Concrete(location), ..
		} = asset
		{
			Self::convert(location)
		} else {
			None
		}
	}
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		X1(AccountId32 {
			network: NetworkId::Any,
			id: account.into(),
		})
			.into()
	}
}

parameter_types! {
	pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
	pub const BaseXcmWeight: Weight = 100_000_000;
	pub const MaxAssetsForTransfer: usize = 2;
}



impl orml_xtokens::Config for KaruraRuntime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = AssetId;
	type CurrencyIdConvert = CurrencyIdConvert;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type Weigher = FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
}


construct_runtime!(
	pub enum KaruraRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Config, Event<T>},
			ParachainInfo: parachain_info::{Pallet, Storage, Config},
			XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
			DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},
		    CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
			PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
			AssetRegistry: pallet_asset_registry::{Pallet, Storage, Event<T>},
			XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 154,
		}
);