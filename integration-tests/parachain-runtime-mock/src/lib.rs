pub use basilisk_runtime::{AccountId, VestingPalletId};
use frame_support::{
	construct_runtime,
	dispatch::Pays,
	pallet_prelude::ConstU32,
	parameter_types,
	traits::{Everything, Nothing},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
	PalletId,
};
use hydradx_adapters::RelayChainBlockNumberProvider;
use hydradx_adapters::{MultiCurrencyTrader, ToFeeReceiver};

use orml_xcm_support::{DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
use sp_runtime::traits::Convert;
use sp_runtime::Perbill;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use orml_traits::parameter_type_with_key;

use pallet_transaction_multi_payment::{DepositAll, Price, TransferFees};
use polkadot_xcm::v3::prelude::*;
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
	AccountId32Aliases, AllowUnpaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation,
};
use xcm_executor::{Config, XcmExecutor};
pub type Amount = i128;
pub type BlockNumber = u64;

use cumulus_primitives_core::ParaId;
use frame_support::weights::ConstantMultiplier;
use hydradx_traits::pools::SpotPriceProvider;
use orml_traits::location::AbsoluteReserveProvider;
use pallet_currencies::BasicCurrencyAdapter;
use pallet_transaction_payment::TargetedFeeAdjustment;
use polkadot_xcm::prelude::MultiLocation;
use sp_arithmetic::FixedU128;
use sp_runtime::traits::AccountIdConversion;

use polkadot_xcm::v3::Weight as XcmWeight;

use basilisk_runtime::{AdjustmentVariable, MaximumMultiplier, MinimumMultiplier, TargetBlockFullness, WeightToFee};

pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

pub const CORE_ASSET_ID: AssetId = 0;

parameter_types! {
	pub const ParachainNativeCurrencyId: AssetId = 0;
	pub const MultiPaymentCurrencySetFee: Pays = Pays::Yes;
	pub const NativeExistentialDeposit: u128 = NATIVE_EXISTENTIAL_DEPOSIT;
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 63;
	pub static MockBlockNumberProvider: u64 = 0;
	pub const MaxLocks: u32 = 50;
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxReserves: u32 = 50;
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub const MaxInstructions: u32 = 100;
	pub BlockLength: frame_system::limits::BlockLength =
		frame_system::limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const ReservedXcmpWeight: Weight = Weight::from_ref_time(WEIGHT_REF_TIME_PER_SECOND / 4);
	pub const ReservedDmpWeight: Weight =Weight::from_ref_time(WEIGHT_REF_TIME_PER_SECOND / 4);
	pub RegistryStringLimit: u32 = 100;
	pub const TreasuryPalletId: PalletId = PalletId(*b"aca/trsy");
	pub ParachainTreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub KsmPerSecond: (AssetId, u128) = (0, 10);
	pub BaseRate: u128 = 100;
	pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
	pub const BaseXcmWeight: XcmWeight = XcmWeight::from_ref_time(100_000_000);
	pub const MaxAssetsForTransfer: usize = 2;
	pub const SequentialIdOffset: u32 = 1_000_000;
	pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
}
parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		1u128
	};
}

impl frame_system::Config for ParachainRuntime {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = BlockLength;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
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

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

pub type LocalAssetTransactor = MultiCurrencyAdapter<
	Currencies,
	UnknownTokens,
	IsNativeConcrete<AssetId, CurrencyIdConvert>,
	AccountId,
	LocationToAccountId,
	AssetId,
	CurrencyIdConvert,
	DepositToAlternative<ParachainTreasuryAccount, Currencies, AssetId, AccountId, Balance>,
>;
pub type XcmOriginToCallOrigin = (
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	XcmPassthrough<RuntimeOrigin>,
);
pub type LocationToAccountId = (
	ParentIsPreset<AccountId>,
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<RelayNetwork, AccountId>,
);
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;

	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToCallOrigin;
	type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;

	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;

	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;

	// We calculate weight fees the same way as for regular extrinsics and use the prices and choice
	// of accepted currencies of the transaction payment pallet. Fees go to the same fee receiver as
	// configured in `MultiTransactionPayment`.
	type Trader = MultiCurrencyTrader<
		AssetId,
		Balance,
		Price,
		WeightToFee,
		MultiTransactionPayment,
		CurrencyIdConvert,
		ToFeeReceiver<
			AccountId,
			AssetId,
			Balance,
			Price,
			CurrencyIdConvert,
			DepositAll<ParachainRuntime>,
			ParachainTreasuryAccount,
		>,
	>;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetLocker = ();
	type AssetExchanger = ();
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = ConstU32<64>;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
}

impl cumulus_pallet_parachain_system::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = ParachainInfo;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
}

impl cumulus_pallet_xcmp_queue::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToCallOrigin;
	type PriceForSiblingDelivery = ();
	type WeightInfo = ();
	type ExecuteDeferredOrigin = EnsureRoot<AccountId>;
	type MaxDeferredMessages = ConstU32<100>;
	type RelayChainBlockNumberProvider = RelayChainBlockNumberProvider<ParachainRuntime>;
	type XcmDeferFilter = ();
}

impl pallet_xcm::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = basilisk_runtime::weights::xcm::BasiliskWeight<ParachainRuntime>;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ();
}

impl cumulus_pallet_dmp_queue::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

impl parachain_info::Config for ParachainRuntime {}

impl orml_tokens::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = Nothing;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
	type CurrencyHooks = ();
}

impl pallet_balances::Config for ParachainRuntime {
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<ParachainRuntime>;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
}
impl cumulus_pallet_xcm::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl pallet_asset_registry::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type AssetId = AssetId;
	type RegistryOrigin = EnsureRoot<AccountId>;
	type Balance = Balance;
	type AssetNativeLocation = AssetLocation;
	type StringLimit = RegistryStringLimit;
	type SequentialIdStartAt = SequentialIdOffset;
	type NativeAssetId = ParachainNativeCurrencyId;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<ParachainRuntime>;
type Block = frame_system::mocking::MockBlock<ParachainRuntime>;

#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
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
			_ => AssetRegistry::asset_to_location(id).map(|loc| loc.0),
		}
	}
}

impl Convert<MultiLocation, Option<AssetId>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<AssetId> {
		match location {
			MultiLocation {
				parents,
				interior: X2(Parachain(id), GeneralIndex(index)),
			} if parents == 1
				&& ParaId::from(id) == ParachainInfo::parachain_id()
				&& (index as u32) == CORE_ASSET_ID =>
			{
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
			network: None,
			id: account.into(),
		})
		.into()
	}
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		None
	};
}

impl orml_xtokens::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = AssetId;
	type CurrencyIdConvert = CurrencyIdConvert;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type MinXcmFee = ParachainMinFee;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type MultiLocationsFilter = Everything;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type UniversalLocation = UniversalLocation;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type ReserveProvider = AbsoluteReserveProvider;
}

impl pallet_currencies::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<ParachainRuntime, Balances, Amount, u32>;
	type GetNativeCurrencyId = ParachainNativeCurrencyId;
	type WeightInfo = ();
}

impl orml_unknown_tokens::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
}
pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier, MaximumMultiplier>;

impl pallet_transaction_payment::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = TransferFees<Currencies, DepositAll<ParachainRuntime>, ParachainTreasuryAccount>;
	type OperationalFeeMultiplier = ();
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
}

pub struct ParachainSpotPriceProviderStub;

impl SpotPriceProvider<AssetId> for ParachainSpotPriceProviderStub {
	type Price = FixedU128;

	fn pair_exists(_asset_a: AssetId, _asset_b: AssetId) -> bool {
		true
	}

	fn spot_price(_asset_a: AssetId, _asset_b: AssetId) -> Option<Self::Price> {
		Some(FixedU128::from_inner(462_962_963_000_u128))
	}
}

impl pallet_transaction_multi_payment::Config for ParachainRuntime {
	type RuntimeEvent = RuntimeEvent;
	type AcceptedCurrencyOrigin = EnsureRoot<AccountId>;
	type Currencies = Currencies;
	type SpotPriceProvider = ParachainSpotPriceProviderStub;
	type WeightInfo = ();
	type WeightToFee = WeightToFee;
	type NativeAssetId = ParachainNativeCurrencyId;
}

construct_runtime!(
	pub enum ParachainRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system,
			Tokens: orml_tokens,
			Balances: pallet_balances,
			Currencies: pallet_currencies,
			ParachainSystem: cumulus_pallet_parachain_system,
			ParachainInfo: parachain_info,
			XcmpQueue: cumulus_pallet_xcmp_queue,
			DmpQueue: cumulus_pallet_dmp_queue,
			CumulusXcm: cumulus_pallet_xcm,
			PolkadotXcm: pallet_xcm,
			AssetRegistry: pallet_asset_registry,
			TransactionPayment: pallet_transaction_payment,
			MultiTransactionPayment: pallet_transaction_multi_payment,
			XTokens: orml_xtokens = 154,
			UnknownTokens: orml_unknown_tokens = 155,
		}
);
