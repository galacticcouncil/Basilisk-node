use super::*;

use cumulus_primitives_core::ParaId;
use polkadot_xcm::v0::{Junction::*, MultiAsset, MultiLocation, MultiLocation::*, NetworkId};
use sp_runtime::traits::Convert;
use xcm_builder::{
	AllowUnpaidExecutionFrom, FixedWeightBounds, LocationInverter, ParentAsSuperuser, ParentIsDefault,
	SovereignSignedViaLocation,
};
use xcm_executor::{Config, XcmExecutor};

parameter_types! {
		pub SelfLocation: MultiLocation = X2(Parent, Parachain(ParachainInfo::get().into()));
}

parameter_types! {
	pub const RococoLocation: MultiLocation = X1(Parent);
	pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<ParentIsDefault<AccountId>, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<Origin>,
);

match_type! {
	pub type JustTheParent: impl Contains<MultiLocation> = { X1(Parent) };
}

parameter_types! {
	// One XCM operation is 1_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = 1_000_000;
}

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = (); // sending XCM not supported
	type AssetTransactor = (); // balances not supported
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = (); // balances not supported
	type IsTeleporter = (); // balances not supported
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = AllowUnpaidExecutionFrom<JustTheParent>;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>; // balances not supported
	type Trader = (); // balances not supported
	type ResponseHandler = (); // Don't handle responses for now.
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
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

pub struct CurrencyIdConvert;

// Note: stub implementation
impl Convert<AssetId, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(id: AssetId) -> Option<MultiLocation> {
		match id {
			1 => Some(X1(Parent)),
			0 => Some(X1(Parent)),
			_ => None,
		}
	}
}

// Note: stub implementation
impl Convert<MultiLocation, Option<AssetId>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<AssetId> {
		match location {
			X1(Parent) => Some(1),
			X3(Parent, Parachain(id), GeneralKey(_key)) if ParaId::from(id) == ParachainInfo::get() => Some(1),
			_ => None,
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
