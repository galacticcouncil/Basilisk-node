// This file is part of Basilisk-node.

// Copyright (C) 2020-2023  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use crate::governance::{MajorityTechCommitteeOrRoot, SuperMajorityCouncilOrRoot, TreasuryAccount};
use crate::system::WeightToFee;

use codec::{Decode, Encode, MaxEncodedLen};
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::traits::TransformOrigin;
use frame_support::{
	parameter_types,
	sp_runtime::traits::Convert,
	traits::{Contains, Everything, Get, Nothing},
	PalletId,
};
use frame_system::EnsureRoot;
use hydradx_adapters::xcm_exchange::XcmAssetExchanger;
use hydradx_adapters::xcm_execute_filter::AllowTransferAndSwap;
use hydradx_adapters::{MultiCurrencyTrader, ToFeeReceiver};
use hydradx_traits::router::PoolType;
use orml_traits::{location::AbsoluteReserveProvider, parameter_type_with_key};
pub use orml_xcm_support::{DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
use pallet_transaction_multi_payment::DepositAll;
use pallet_xcm::XcmPassthrough;
use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};
use polkadot_parachain::primitives::{RelayChainBlockNumber, Sibling};
use polkadot_xcm::latest::{Asset, Junctions, Location};
use polkadot_xcm::prelude::InteriorLocation;
use polkadot_xcm::v3::{
	prelude::{AccountId32, Concrete, GeneralIndex, GlobalConsensus, Here, NetworkId, Parachain, X1, X2, X3},
	MultiAsset, MultiLocation, Weight as XcmWeight,
};
use primitives::AssetId;
use scale_info::TypeInfo;
use sp_runtime::traits::MaybeEquivalence;
use sp_runtime::Perbill;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom,
	DescribeAllTerminal, DescribeFamily, EnsureXcmOrigin, FixedWeightBounds, HashedDescription, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, WithComputedOrigin,
};
use xcm_executor::{Config, XcmExecutor};

#[derive(Debug, Default, Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct AssetLocation(pub MultiLocation);

impl Into<Option<Location>> for AssetLocation {
	fn into(self) -> Option<Location> {
		xcm_builder::V4V3LocationConverter::convert_back(&self.0)
	}
}

impl TryFrom<Location> for AssetLocation {
	type Error = ();

	fn try_from(value: Location) -> Result<Self, Self::Error> {
		let loc: MultiLocation = value.try_into()?;
		Ok(AssetLocation(loc.into()))
	}
}

pub const RELAY_CHAIN_ASSET_LOCATION: AssetLocation = AssetLocation(MultiLocation {
	parents: 1,
	interior: Here,
});

parameter_types! {
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

pub type Barrier = (
	TakeWeightCredit,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	WithComputedOrigin<
		(
			AllowTopLevelPaidExecutionFrom<Everything>,
			// Subscriptions for version tracking are OK.
			AllowSubscriptionsFrom<Everything>,
		),
		UniversalLocation,
		ConstU32<8>,
	>,
);

use sp_std::sync::Arc;
parameter_types! {
	pub SelfLocation: Location = Location::new(1, cumulus_primitives_core::Junctions::X1(Arc::new([cumulus_primitives_core::Junction::Parachain(ParachainInfo::get().into());1])));
}

parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;

	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();

	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToCallOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	/// The amount of weight an XCM operation takes. This is a safe overestimate.
	pub const BaseXcmWeight: XcmWeight = XcmWeight::from_parts(100_000_000, 0);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsForTransfer: usize = 2;
	pub UniversalLocation: InteriorLocation = [polkadot_xcm::v4::prelude::GlobalConsensus(RelayNetwork::get().into()), polkadot_xcm::v4::prelude::Parachain(ParachainInfo::parachain_id().into())].into();
}

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;

	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToCallOrigin;
	type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;

	type IsTeleporter = (); // disabled
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
		ToFeeReceiver<AccountId, AssetId, Balance, Price, CurrencyIdConvert, DepositAll<Runtime>, TreasuryAccount>,
	>;

	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetLocker = ();
	type AssetExchanger = XcmAssetExchanger<Runtime, TempAccount, CurrencyIdConvert, Currencies>;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = ConstU32<64>;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = SafeCallFilter;
	type Aliasers = Nothing;
	type TransactionalProcessor = xcm_builder::FrameTransactionalProcessor;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

parameter_types! {
	pub const MaxDeferredMessages: u32 = 20;
	pub const MaxDeferredBuckets: u32 = 1_000;
	pub const MaxBucketsProcessed: u32 = 3;
	pub const MaxInboundSuspended: u32 = 1_000;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ControllerOrigin = MajorityTechCommitteeOrRoot;
	type ControllerOriginConverter = XcmOriginToCallOrigin;
	type PriceForSiblingDelivery = polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery<ParaId>;
	type WeightInfo = weights::xcmp_queue::BasiliskWeight<Runtime>;
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = MaxInboundSuspended;
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: Location| -> Option<u128> {
		None
	};
}

impl orml_xtokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = AssetId;
	type CurrencyIdConvert = CurrencyIdConvert;
	type AccountIdToLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type MinXcmFee = ParachainMinFee;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type LocationsFilter = Everything;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type UniversalLocation = UniversalLocation;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type ReserveProvider = AbsoluteReserveProvider;
	type RateLimiter = (); //TODO: what do ?
	type RateLimiterId = (); //TODO: what do ?
}

impl orml_unknown_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

impl orml_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SovereignOrigin = SuperMajorityCouncilOrRoot;
}

parameter_types! {
	//Xcm asset exchange
	pub DefaultPoolType: PoolType<AssetId>  = PoolType::XYK;
	pub TempAccount: AccountId = [42; 32].into();

	//Xcm executor filter
	pub const MaxXcmDepth: u16 = 5;
	pub const MaxNumberOfInstructions: u16 = 100;
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = AllowTransferAndSwap<MaxXcmDepth, MaxNumberOfInstructions, RuntimeCall>;
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
	type WeightInfo = weights::xcm::BasiliskWeight<Runtime>;
	type AdminOrigin = SuperMajorityTechCommitteeOrRoot;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
}

#[test]
fn defer_duration_configuration() {
	use sp_runtime::{traits::One, FixedPointNumber, FixedU128};
	/// Calculate the configuration value for the defer duration based on the desired defer duration and
	/// the threshold percentage when to start deferring.
	/// - `defer_by`: the desired defer duration when reaching the rate limit
	/// - `a``: the fraction of the rate limit where we start deferring, e.g. 0.9
	fn defer_duration(defer_by: u32, a: FixedU128) -> u32 {
		assert!(a < FixedU128::one());
		// defer_by * a / (1 - a)
		(FixedU128::one() / (FixedU128::one() - a)).saturating_mul_int(a.saturating_mul_int(defer_by))
	}
	assert_eq!(
		defer_duration(600 * 4, FixedU128::from_rational(9, 10)),
		DeferDuration::get()
	);
}

parameter_types! {
	pub DeferDuration: RelayChainBlockNumber = 600 * 36; // 36 hours
	pub MaxDeferDuration: RelayChainBlockNumber = 600 * 24 * 10; // 10 days

	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(25) * BlockWeights::get().max_block;
	pub const MessageQueueMaxStale: u32 = 8;
	pub const MessageQueueHeapSize: u32 = 128 * 1048;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MessageProcessor = xcm_builder::ProcessXcmMessage<AggregateMessageOrigin, XcmExecutor<XcmConfig>, RuntimeCall>;
	type Size = u32;
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = MessageQueueHeapSize;
	type MaxStale = MessageQueueMaxStale;
	type ServiceWeight = MessageQueueServiceWeight;
}

pub struct CurrencyIdConvert;
use primitives::constants::chain::CORE_ASSET_ID;

impl Convert<AssetId, Option<Location>> for CurrencyIdConvert {
	fn convert(id: AssetId) -> Option<Location> {
		match id {
			CORE_ASSET_ID => Some(Location {
				parents: 1,
				interior: [
					polkadot_xcm::prelude::Parachain(ParachainInfo::get().into()),
					polkadot_xcm::prelude::GeneralIndex(id.into()),
				]
				.into(),
			}),
			_ => {
				let loc = AssetRegistry::asset_to_location(id);
				if let Some(location) = loc {
					location.into()
				} else {
					None
				}
			}
		}
	}
}

impl Convert<Location, Option<AssetId>> for CurrencyIdConvert {
	fn convert(location: Location) -> Option<AssetId> {
		let Location { parents, interior } = location.clone();

		match interior {
			Junctions::X2(a)
				if parents == 1
					&& a.contains(&polkadot_xcm::prelude::GeneralIndex(CORE_ASSET_ID.into()))
					&& a.contains(&polkadot_xcm::prelude::Parachain(ParachainInfo::get().into())) =>
			{
				Some(CORE_ASSET_ID)
			}
			Junctions::X1(a)
				if parents == 0 && a.contains(&polkadot_xcm::prelude::GeneralIndex(CORE_ASSET_ID.into())) =>
			{
				Some(CORE_ASSET_ID)
			}
			_ => {
				let location: Option<AssetLocation> = location.try_into().ok();
				if let Some(location) = location {
					AssetRegistry::location_to_asset(location)
				} else {
					None
				}
			}
		}

		// Note: keeping the original code for reference until tests are successful
		/*
		match location {
			Location {
				parents: p,
				interior: [Parachain(id), GeneralIndex(index)].into(),
			} if p == 1 && ParaId::from(id) == ParachainInfo::get() && (index as u32) == CORE_ASSET_ID => {
				// Handling native asset for this parachain
				Some(CORE_ASSET_ID)
			}
			// handle reanchor canonical location: https://github.com/paritytech/polkadot/pull/4470
			Location {
				parents: 0,
				interior: [GeneralIndex(index)].into(),
			} if (index as u32) == CORE_ASSET_ID => Some(CORE_ASSET_ID),
			// delegate to asset-registry
			_ => AssetRegistry::location_to_asset(AssetLocation(location)),
		}

		 */
	}
}

impl Convert<Asset, Option<AssetId>> for CurrencyIdConvert {
	fn convert(asset: Asset) -> Option<AssetId> {
		if let Asset { id: asset_id, .. } = asset {
			Self::convert(asset_id.0)
		} else {
			None
		}
	}
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, Location> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> Location {
		[polkadot_xcm::prelude::AccountId32 {
			network: None,
			id: account.into(),
		}]
		.into()
	}
}

/*
impl Convert<polkadot_xcm::latest::Location, Option<AssetId>> for CurrencyIdConvert {
	fn convert(location: polkadot_xcm::latest::Location) -> Option<AssetId> {
		let polkadot_xcm::latest::Location { parents, interior } = location.clone();

		match interior {
			polkadot_xcm::latest::Junctions::X2(a)
			if parents == 1
				&& a.contains(&polkadot_xcm::prelude::GeneralIndex(CORE_ASSET_ID.into()))
				&& a.contains(&polkadot_xcm::prelude::Parachain(ParachainInfo::get().into())) =>
				{
					Some(CORE_ASSET_ID)
				}
			polkadot_xcm::latest::Junctions::X1(a) if parents == 0 && a.contains(&polkadot_xcm::prelude::GeneralIndex(CORE_ASSET_ID.into())) => Some(CORE_ASSET_ID),
			_ => AssetRegistry::location_to_asset(AssetLocation(location.into())),
		}

		// Note: keeping the original code for reference until tests are successful
		/*
		match location {
			Location {
				parents: p,
				interior: [Parachain(id), GeneralIndex(index)].into(),
			} if p == 1 && ParaId::from(id) == ParachainInfo::get() && (index as u32) == CORE_ASSET_ID => {
				// Handling native asset for this parachain
				Some(CORE_ASSET_ID)
			}
			// handle reanchor canonical location: https://github.com/paritytech/polkadot/pull/4470
			Location {
				parents: 0,
				interior: [GeneralIndex(index)].into(),
			} if (index as u32) == CORE_ASSET_ID => Some(CORE_ASSET_ID),
			// delegate to asset-registry
			_ => AssetRegistry::location_to_asset(AssetLocation(location)),
		}

		 */
	}
}

 */

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
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
	// Foreign locations alias into accounts according to a hash of their standard description.
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
);

parameter_types! {
	// The account which receives multi-currency tokens from failed attempts to deposit them
	pub Alternative: AccountId = PalletId(*b"xcm/alte").into_account_truncating();
}

pub type LocalAssetTransactor = MultiCurrencyAdapter<
	Currencies,
	UnknownTokens,
	IsNativeConcrete<AssetId, CurrencyIdConvert>,
	AccountId,
	LocationToAccountId,
	AssetId,
	CurrencyIdConvert,
	DepositToAlternative<Alternative, Currencies, AssetId, AccountId, Balance>,
>;

/// A call filter for the XCM Transact instruction. This is a temporary measure until we properly
/// account for proof size weights.
///
/// Calls that are allowed through this filter must:
/// 1. Have a fixed weight;
/// 2. Cannot lead to another call being made;
/// 3. Have a defined proof size weight, e.g. no unbounded vecs in call parameters.
pub struct SafeCallFilter;
impl Contains<RuntimeCall> for SafeCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		#[cfg(feature = "runtime-benchmarks")]
		{
			if matches!(call, RuntimeCall::System(frame_system::Call::remark_with_event { .. })) {
				return true;
			}
		}

		// check the runtime call filter
		if !BaseFilter::contains(call) {
			return false;
		}

		match call {
			RuntimeCall::System(frame_system::Call::kill_prefix { .. } | frame_system::Call::set_heap_pages { .. })
			| RuntimeCall::Timestamp(..)
			| RuntimeCall::Balances(..)
			| RuntimeCall::Treasury(..)
			| RuntimeCall::Utility(pallet_utility::Call::as_derivative { .. })
			| RuntimeCall::Vesting(..)
			| RuntimeCall::Proxy(..)
			| RuntimeCall::CollatorSelection(
				pallet_collator_selection::Call::set_desired_candidates { .. }
				| pallet_collator_selection::Call::set_candidacy_bond { .. }
				| pallet_collator_selection::Call::register_as_candidate { .. }
				| pallet_collator_selection::Call::leave_intent { .. },
			)
			| RuntimeCall::Session(pallet_session::Call::purge_keys { .. })
			| RuntimeCall::Uniques(
				pallet_uniques::Call::create { .. }
				| pallet_uniques::Call::force_create { .. }
				| pallet_uniques::Call::mint { .. }
				| pallet_uniques::Call::burn { .. }
				| pallet_uniques::Call::transfer { .. }
				| pallet_uniques::Call::freeze { .. }
				| pallet_uniques::Call::thaw { .. }
				| pallet_uniques::Call::freeze_collection { .. }
				| pallet_uniques::Call::thaw_collection { .. }
				| pallet_uniques::Call::transfer_ownership { .. }
				| pallet_uniques::Call::set_team { .. }
				| pallet_uniques::Call::approve_transfer { .. }
				| pallet_uniques::Call::cancel_approval { .. }
				| pallet_uniques::Call::force_item_status { .. }
				| pallet_uniques::Call::set_attribute { .. }
				| pallet_uniques::Call::clear_attribute { .. }
				| pallet_uniques::Call::set_metadata { .. }
				| pallet_uniques::Call::clear_metadata { .. }
				| pallet_uniques::Call::set_collection_metadata { .. }
				| pallet_uniques::Call::clear_collection_metadata { .. }
				| pallet_uniques::Call::set_accept_ownership { .. }
				| pallet_uniques::Call::set_price { .. }
				| pallet_uniques::Call::buy_item { .. },
			)
			| RuntimeCall::Identity(
				pallet_identity::Call::add_registrar { .. }
				| pallet_identity::Call::set_identity { .. }
				| pallet_identity::Call::clear_identity { .. }
				| pallet_identity::Call::request_judgement { .. }
				| pallet_identity::Call::cancel_request { .. }
				| pallet_identity::Call::set_fee { .. }
				| pallet_identity::Call::set_account_id { .. }
				| pallet_identity::Call::set_fields { .. }
				| pallet_identity::Call::provide_judgement { .. }
				| pallet_identity::Call::kill_identity { .. }
				| pallet_identity::Call::add_sub { .. }
				| pallet_identity::Call::rename_sub { .. }
				| pallet_identity::Call::remove_sub { .. }
				| pallet_identity::Call::quit_sub { .. },
			)
			| RuntimeCall::XYK(..)
			| RuntimeCall::NFT(..)
			| RuntimeCall::MultiTransactionPayment(..)
			| RuntimeCall::XYKLiquidityMining(..)
			| RuntimeCall::Currencies(..)
			| RuntimeCall::Tokens(..)
			| RuntimeCall::OrmlXcm(..) => true,
			_ => false,
		}
	}
}
