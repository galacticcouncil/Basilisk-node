use super::{AssetId, *};

use codec::{Decode, Encode};
use cumulus_primitives_core::ParaId;
use frame_support::traits::{Everything, Nothing};
pub use orml_xcm_support::{IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_xcm::latest::prelude::*;
use polkadot_xcm::latest::Error;
use primitives::Price;
use sp_runtime::{traits::{Convert, Saturating, Zero}, FixedPointNumber, SaturatedConversion};
use sp_std::collections::btree_map::BTreeMap;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom,
	EnsureXcmOrigin, FixedWeightBounds, LocationInverter, ParentIsPreset, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::traits::WeightTrader;
use xcm_executor::{Assets, Config, XcmExecutor};

pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
);

parameter_types! {
	pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::get().into())));
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

parameter_types! {
	/// The amount of weight an XCM operation takes. This is a safe overestimate.
	pub const BaseXcmWeight: Weight = 100_000_000;
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsForTransfer: usize = 2;
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

/// Very simple price oracle trait.
/// TODO: Move to correct location and properly define the price oracle interface.
pub trait PriceOracle {
	fn price(currency: AssetId) -> Option<Price>;
}

impl PriceOracle for MultiTransactionPayment {
	fn price(currency: AssetId) -> Option<Price> {
		MultiTransactionPayment::currency_price(currency)
	}
}

/// Weight trader which uses `WeightToFee` in combination with a `PriceOracle` to set the right
/// price for weight. Keeps track of the assets used used to pay for weight and can refund them one
/// by one (interface only allows returning one asset per refund).
pub struct MultiCurrencyTrader<
	WeightToFee: WeightToFeePolynomial<Balance = Balance>,
	AcceptedCurrencyPrices: PriceOracle,
	ConvertCurrency: Convert<MultiAsset, Option<AssetId>>,
> {
	weight: Weight,
	paid_assets: BTreeMap<(MultiLocation, Price), u128>,
	_phantom: PhantomData<(WeightToFee, AcceptedCurrencyPrices, ConvertCurrency)>,
}

impl<
	WeightToFee: WeightToFeePolynomial<Balance = Balance>,
	AcceptedCurrencyPrices: PriceOracle,
	ConvertCurrency: Convert<MultiAsset, Option<AssetId>>,
	>
	MultiCurrencyTrader<WeightToFee, AcceptedCurrencyPrices, ConvertCurrency>
{
	/// Get the asset id of the first asset in `payment` and try to determine its price via the
	/// price oracle.
	fn get_asset_and_price(&mut self, payment: &Assets) -> Option<(MultiLocation, Price)> {
		if let Some(asset) = payment.fungible_assets_iter().next() {
			// TODO: consider optimizing out the clone
			ConvertCurrency::convert(asset.clone())
				.and_then(|currency| {
					AcceptedCurrencyPrices::price(currency)
				}).and_then(|price| match asset.id.clone() {
					Concrete(location) => Some((location, price)),
					_ => None,
				})
		} else {
			None
		}
	}
}

impl<
		WeightToFee: WeightToFeePolynomial<Balance = Balance>,
		AcceptedCurrencyPrices: PriceOracle,
		ConvertCurrency: Convert<MultiAsset, Option<AssetId>>,
	> WeightTrader for MultiCurrencyTrader<WeightToFee, AcceptedCurrencyPrices, ConvertCurrency>
{
	fn new() -> Self {
		Self { weight: 0, paid_assets: Default::default(), _phantom: PhantomData }
	}

	/// Will try to buy weight with the first asset in `payment`.
	/// The fee is determined by `WeightToFee` in combination with the determined price.
	fn buy_weight(&mut self, weight: Weight, payment: Assets) -> Result<Assets, XcmError> {
		log::trace!(target: "xcm::weight", "MultiCurrencyTrader::buy_weight weight: {:?}, payment: {:?}", weight, payment);
		let (asset_loc, price) = self.get_asset_and_price(&payment).ok_or(XcmError::TooExpensive)?;
		let fee = WeightToFee::calc(&weight);
		let converted_fee = price.checked_mul_int(fee).ok_or(XcmError::Overflow)?;
		let amount: u128 = converted_fee.try_into().map_err(|_| XcmError::Overflow)?;
		let required = (Concrete(asset_loc.clone()), amount).into();
		let unused = payment.checked_sub(required).map_err(|_| XcmError::TooExpensive)?;
		self.weight = self.weight.saturating_add(weight);
		let key = (asset_loc, price);
		match self.paid_assets.get_mut(&key) {
			Some(v) => v.saturating_accrue(amount),
			None => { self.paid_assets.insert(key, amount); },
		}
		Ok(unused)
	}

	/// Will refund up to `weight` from the first asset tracked by the trader.
	fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
		log::trace!(target: "xcm::weight", "MultiCurrencyTrader::refund_weight weight: {:?}, paid_assets: {:?}", weight, self.paid_assets);
		let weight = weight.min(self.weight);
		self.weight -= weight; // Will not underflow because of `min()` above.
		let fee = WeightToFee::calc(&weight);
		if let Some(((asset_loc, price), amount)) = self.paid_assets.iter_mut().next() {
			let converted_fee: u128 = price.saturating_mul_int(fee).saturated_into();
			let refund = converted_fee.min(*amount);
			*amount -= refund; // Will not underflow because of `min()` above.

			let refund_asset = asset_loc.clone();
			if amount.is_zero() {
				let key = (asset_loc.clone(), price.clone());
				self.paid_assets.remove(&key);
			}
			Some((Concrete(refund_asset), refund).into())
		} else {
			None
		}
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
	type Weigher = FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
	type Trader = MultiCurrencyTrader<WeightToFee, MultiTransactionPayment, CurrencyIdConvert>;

	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = crate::EnsureMajorityCouncilOrRoot;
	type ControllerOriginConverter = XcmOriginToCallOrigin;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

impl orml_xtokens::Config for Runtime {
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

impl orml_unknown_tokens::Config for Runtime {
	type Event = Event;
}

impl orml_xcm::Config for Runtime {
	type Event = Event;
	type SovereignOrigin = crate::EnsureMajorityCouncilOrRoot;
}

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type Origin = Origin;
	type Call = Call;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

pub struct CurrencyIdConvert;
use primitives::constants::chain::CORE_ASSET_ID;

impl Convert<AssetId, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(id: AssetId) -> Option<MultiLocation> {
		match id {
			CORE_ASSET_ID => Some(MultiLocation::new(
				1,
				X2(Parachain(ParachainInfo::get().into()), GeneralKey(id.encode())),
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
				interior: X2(Parachain(id), GeneralKey(key)),
			} if parents == 1 && ParaId::from(id) == ParachainInfo::get() => {
				// Handling native asset for this parachain
				if let Ok(currency_id) = AssetId::decode(&mut &key[..]) {
					// we currently have only one native asset
					match currency_id {
						CORE_ASSET_ID => Some(currency_id),
						_ => None,
					}
				} else {
					None
				}
			}
			// handle reanchor canonical location: https://github.com/paritytech/polkadot/pull/4470
			MultiLocation {
				parents: 0,
				interior: X1(GeneralKey(key)),
			} => {
				if let Ok(currency_id) = AssetId::decode(&mut &key[..]) {
					// we currently have only one native asset
					match currency_id {
						CORE_ASSET_ID => Some(currency_id),
						_ => None,
					}
				} else {
					None
				}
			}
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

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
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
	(),
>;

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::weights::IdentityFee;
	use sp_runtime::traits::One;

	const TEST_ASSET_ID: AssetId = 123;

	struct MockOracle;
	impl PriceOracle for MockOracle {
		fn price(currency: AssetId) -> Option<Price> {
			match currency {
				CORE_ASSET_ID => Some(Price::one()),
				TEST_ASSET_ID => Some(Price::from_float(0.5)),
				_ => None,
			}
		}
	}

	struct MockConvert;
	impl Convert<AssetId, Option<MultiLocation>> for MockConvert {
		fn convert(id: AssetId) -> Option<MultiLocation> {
			match id {
				CORE_ASSET_ID | TEST_ASSET_ID => Some(MultiLocation::new(
					0,
					X1(GeneralKey(id.encode())),
				)),
				_ => None
			}
		}
	}

	impl Convert<MultiLocation, Option<AssetId>> for MockConvert {
		fn convert(location: MultiLocation) -> Option<AssetId> {
			match location {
				MultiLocation {
					parents: 0,
					interior: X1(GeneralKey(key)),
				} => {
					if let Ok(currency_id) = AssetId::decode(&mut &key[..]) {
						// we currently have only one native asset
						match currency_id {
							CORE_ASSET_ID | TEST_ASSET_ID => Some(currency_id),
							_ => None,
						}
					} else {
						None
					}
				}
				_ => None,
			}
		}
	}
	
	impl Convert<MultiAsset, Option<AssetId>> for MockConvert {
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

	#[test]
	fn multi_currency_trader() {
		type Trader = MultiCurrencyTrader<IdentityFee<Balance>, MockOracle, MockConvert>;

		let mut trader = Trader::new();
		let payment: MultiAsset = (Concrete(MockConvert::convert(CORE_ASSET_ID).unwrap()), 1_000_000).into();

		let res = dbg!(trader.buy_weight(1_000_000, payment.into()));
		assert!(res.unwrap().is_empty());
	}
}
