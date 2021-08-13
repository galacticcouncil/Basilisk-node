use super::*;

use codec::{Decode, Encode};
use cumulus_primitives_core::ParaId;
use frame_support::traits::All;
pub use orml_xcm_support::{IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_xcm::opaque::v0::Error;
use polkadot_xcm::v0::{Junction::*, MultiAsset, MultiLocation, MultiLocation::*, NetworkId, Xcm};
use sp_runtime::traits::Convert;
use xcm_builder::{
	AccountId32Aliases, AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds, LocationInverter,
	ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::traits::WeightTrader;
use xcm_executor::{Assets, Config, XcmExecutor};

pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

pub type Barrier = (TakeWeightCredit, AllowTopLevelPaidExecutionFrom<All<MultiLocation>>);

parameter_types! {
	pub SelfLocation: MultiLocation = X2(Parent, Parachain(ParachainInfo::get().into()));
}

parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;

	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();

	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<Origin>,
);

match_type! {
	pub type JustTheParent: impl Contains<MultiLocation> = { X1(Parent) };
}

//TODO: investigate this Trader part further
pub fn ksm_per_second() -> u128 {
	let base_weight = Balance::from(ExtrinsicBaseWeight::get());
	let base_tx_per_second = (WEIGHT_PER_SECOND as u128) / base_weight;
	//let kar_per_second = base_tx_per_second * base_tx_in_kar();
	//kar_per_second / 100
	base_tx_per_second / 100
}

parameter_types! {
	// One XCM operation is 1_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = 400_000_000;

	pub KsmPerSecond: (MultiLocation, u128) = (X1(Parent), ksm_per_second());
}

pub struct TradePassthrough();
impl WeightTrader for TradePassthrough {
	fn new() -> Self {
		Self()
	}

	fn buy_weight(&mut self, _weight: Weight, payment: Assets) -> Result<Assets, Error> {
		// Just let it through for now
		Ok(payment)
	}
}

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmRouter;

	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToCallOrigin;
	type IsReserve = MultiNativeAsset;

	type IsTeleporter = (); // disabled
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;

	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;

	//type Trader = FixedRateOfConcreteFungible<KsmPerSecond, ()>;
	type Trader = TradePassthrough;
	type ResponseHandler = (); // Don't handle responses for now.
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
		pub const BaseXcmWeight: Weight = 100_000_000;
}

impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = AssetId;
	type CurrencyIdConvert = CurrencyIdConvert;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
	type BaseXcmWeight = BaseXcmWeight;
}

impl orml_unknown_tokens::Config for Runtime {
	type Event = Event;
}

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = All<(MultiLocation, Xcm<Call>)>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = ();
	type XcmReserveTransferFilter = All<(MultiLocation, Vec<MultiAsset>)>;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
}

pub struct CurrencyIdConvert;

// Note: stub implementation
impl Convert<AssetId, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(id: AssetId) -> Option<MultiLocation> {
		match id {
			0 => Some(X3(
				Parent,
				Parachain(ParachainInfo::get().into()),
				GeneralKey(id.encode()),
			)),
			_ => {
				if let Some(loc) = Registry::asset_to_location(id) {
					Some(loc.0)
				} else {
					None
				}
			}
		}
	}
}

// Note: stub implementation
impl Convert<MultiLocation, Option<AssetId>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<AssetId> {
		match location {
			X3(Parent, Parachain(id), GeneralKey(key)) if ParaId::from(id) == ParachainInfo::get() => {
				// Handling native asset for this parachain
				if let Ok(currency_id) = AssetId::decode(&mut &key[..]) {
					// we currently have only one native asset
					match currency_id {
						0 => Some(currency_id),
						_ => None,
					}
				} else {
					None
				}
			}
			// delegate to registry
			_ => Registry::location_to_asset(AssetLocation(location)),
		}
	}
}

// Note: stub implementation
impl Convert<MultiAsset, Option<AssetId>> for CurrencyIdConvert {
	fn convert(asset: MultiAsset) -> Option<AssetId> {
		if let MultiAsset::ConcreteFungible { id, amount: _ } = asset {
			Self::convert(id)
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
	}
}

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type LocalAssetTransactor = MultiCurrencyAdapter<
	Currencies,
	UnknownTokens,
	IsNativeConcrete<AssetId, CurrencyIdConvert>,
	AccountId,
	LocationToAccountId,
	AssetId,
	CurrencyIdConvert,
>;
