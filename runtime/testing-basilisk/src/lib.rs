// This file is part of Basilisk-node.

// Copyright (C) 2020-2021  Intergalactic, Limited (GIB).
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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::from_over_into)]

#[cfg(test)]
mod tests;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use frame_system::{EnsureRoot, EnsureSigned, RawOrigin};
use orml_tokens::CurrencyAdapter;
use sp_api::impl_runtime_apis;
use sp_core::{
	u32_trait::{_1, _2, _3},
	OpaqueMetadata,
};
use sp_runtime::{
	app_crypto::sp_core::crypto::UncheckedFrom,
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, BlakeTwo256, Block as BlockT, IdentityLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, Perbill,
};
use sp_std::convert::From;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "runtime-benchmarks")]
use codec::Decode;

// A few exports that help ease life for downstream crates.
use frame_support::traits::Contains;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{EnsureOneOf, EnsureOrigin, EqualPrivilegeOnly, Everything, Get, InstanceFilter, U128CurrencyToVote},
	weights::{
		constants::{BlockExecutionWeight, RocksDbWeight},
		DispatchClass, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
};
use hydradx_traits::AssetPairAccountIdFor;
use pallet_transaction_payment::TargetedFeeAdjustment;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

mod xcm;

use pallet_xyk_rpc_runtime_api as xyk_rpc;

use orml_currencies::BasicCurrencyAdapter;

pub use common_runtime::*;
use pallet_transaction_multi_payment::{AddTxAssetOnAccount, DepositAll, RemoveTxAssetOnKilled, TransferFees};

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
		}
	}
}

mod testing {
	use super::{parameter_types, BlockNumber, Everything, MINUTES};
	pub type BaseFilter = Everything;

	parameter_types! {
		pub const LaunchPeriod: BlockNumber = MINUTES;
		pub const VotingPeriod: BlockNumber = MINUTES;
		pub const Period: u32 = 10 * MINUTES;
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("testing-basilisk"),
	impl_name: create_runtime_str!("testing-basilisk"),
	authoring_version: 1,
	spec_version: 73,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

use common_runtime::adapter::OrmlTokensAdapter;
use common_runtime::locked_balance::MultiCurrencyLockedBalance;
use primitives::{ClassId, InstanceId};
use smallvec::smallvec;

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT
		let p = CENTS; // 1_000_000_000_000
		let q = 10 * Balance::from(ExtrinsicBaseWeight::get()); // 7_919_840_000
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q, // 124
		}]
	}
}

pub struct AssetPairAccountId<T: frame_system::Config>(PhantomData<T>);
impl<T: frame_system::Config> AssetPairAccountIdFor<AssetId, T::AccountId> for AssetPairAccountId<T>
where
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn from_assets(asset_a: AssetId, asset_b: AssetId, identifier: &str) -> T::AccountId {
		let mut buf: Vec<u8> = identifier.as_bytes().to_vec();

		if asset_a < asset_b {
			buf.extend_from_slice(&asset_a.to_le_bytes());
			buf.extend_from_slice(&asset_b.to_le_bytes());
		} else {
			buf.extend_from_slice(&asset_b.to_le_bytes());
			buf.extend_from_slice(&asset_a.to_le_bytes());
		}
		T::AccountId::unchecked_from(<T::Hashing as frame_support::sp_runtime::traits::Hash>::hash(&buf[..]))
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	/// Block weights base values and limits.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have an extra reserved space, so that they
			// are included even if block reachd `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();

	pub ExtrinsicBaseWeight: Weight = common_runtime::BasiliskExtrinsicBaseWeight::get();
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = testing::BaseFilter;
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = IdentityLookup<AccountId>;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The weight of the overhead invoked on the block import process, independent of the
	/// extrinsics included in that block.
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = common_runtime::weights::system::BasiliskWeight<Runtime>;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = common_runtime::weights::timestamp::BasiliskWeight<Runtime>;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = Treasury;
	type ExistentialDeposit = NativeExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
}

/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/overview/2-token-economics.html?highlight=token%20economics#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = TransferFees<Currencies, MultiTransactionPayment, DepositAll<Runtime>>;
	type TransactionByteFee = TransactionByteFee;
	type OperationalFeeMultiplier = ();
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

parameter_types! {
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

impl pallet_transaction_multi_payment::Config for Runtime {
	type Event = Event;
	type AcceptedCurrencyOrigin = EnsureSuperMajorityTechCommitteeOrRoot;
	type Currencies = Currencies;
	type SpotPriceProvider = pallet_xyk::XYKSpotPrice<Runtime>;
	type WeightInfo = common_runtime::weights::payment::BasiliskWeight<Runtime>;
	type WithdrawFeeForSetCurrency = MultiPaymentCurrencySetFee;
	type WeightToFee = WeightToFee;
	type NativeAssetId = NativeAssetId;
	type FeeReceiver = TreasuryAccount;
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::CancelProxy => matches!(c, Call::Proxy(pallet_proxy::Call::reject_announcement { .. })),
			ProxyType::Governance => matches!(
				c,
				Call::Democracy(..)
					| Call::Council(..) | Call::TechnicalCommittee(..)
					| Call::Elections(..)
					| Call::Treasury(..) | Call::Tips(..)
					| Call::Utility(..)
			),
			ProxyType::Exchange => matches!(c, Call::XYK(..) | Call::Exchange(..) | Call::LBP(..) | Call::NFT(..)),
			// Transfer group doesn't include cross-chain transfers
			ProxyType::Transfer => matches!(c, Call::Balances(..) | Call::Currencies(..) | Call::Tokens(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		// Always whitelists treasury account
		if *a == TreasuryAccount::get() {
			return true;
		}
		// Check duster whitelist
		pallet_duster::DusterWhitelist::<Runtime>::contains(a)
	}
}

/// Tokens Configurations
impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = AssetRegistry;
	type OnDust = Duster;
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type OnNewTokenAccount = AddTxAssetOnAccount<Runtime>;
	type OnKilledTokenAccount = RemoveTxAssetOnKilled<Runtime>;
}

impl orml_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = OrmlTokensAdapter<Runtime>;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = NativeAssetId;
	type WeightInfo = ();
}

impl pallet_duster::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type MultiCurrency = Currencies;
	type MinCurrencyDeposits = AssetRegistry;
	type Reward = DustingReward;
	type NativeCurrencyId = NativeAssetId;
	type BlacklistUpdateOrigin = EnsureSigned<AccountId>;
	type WeightInfo = common_runtime::weights::duster::BasiliskWeight<Runtime>;
}

/// Basilisk Pallets configurations
impl pallet_asset_registry::Config for Runtime {
	type Event = Event;
	type RegistryOrigin = EnsureSuperMajorityTechCommitteeOrRoot;
	type AssetId = AssetId;
	type Balance = Balance;
	type AssetNativeLocation = AssetLocation;
	type StringLimit = RegistryStrLimit;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = common_runtime::weights::asset_registry::BasiliskWeight<Runtime>;
}

impl pallet_xyk::Config for Runtime {
	type Event = Event;
	type AssetRegistry = AssetRegistry;
	type AssetPairAccountId = AssetPairAccountId<Self>;
	type Currency = Currencies;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = common_runtime::weights::xyk::BasiliskWeight<Runtime>;
	type GetExchangeFee = ExchangeFee;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type CanCreatePool = pallet_lbp::DisallowWhenLBPPoolRunning<Runtime>;
	type AMMHandler = ();
	type DiscountedFee = DiscountedFee;
	type NonDustableWhitelistHandler = Duster;
}

impl pallet_exchange::Config for Runtime {
	type Event = Event;
	type AMMPool = XYK;
	type Resolver = Exchange;
	type Currency = Currencies;
	type WeightInfo = common_runtime::weights::exchange::BasiliskWeight<Runtime>;
}

impl pallet_lbp::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Currencies;
	type LockedBalance = MultiCurrencyLockedBalance<Runtime>;
	type CreatePoolOrigin = EnsureSuperMajorityTechCommitteeOrRoot;
	type LBPWeightFunction = pallet_lbp::LBPWeightFunction;
	type AssetPairAccountId = AssetPairAccountId<Self>;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type WeightInfo = common_runtime::weights::lbp::BasiliskWeight<Runtime>;
	type BlockNumberProvider = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Runtime>;
}

// Parachain Config

parameter_types! {
	pub ReservedXcmpWeight: Weight = BlockWeights::get().max_block / 4;
	pub ReservedDmpWeight: Weight = BlockWeights::get().max_block / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnSystemEvent = pallet_relaychain_info::OnValidationDataHandler<Runtime>;
	type SelfParaId = ParachainInfo;

	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type MaxAuthorities = MaxAuthorities;
	type DisabledValidators = ();
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub ReserveClassIdUpTo: u128 = 999;
}

impl pallet_nft::Config for Runtime {
	type Event = Event;
	type WeightInfo = pallet_nft::weights::BasiliskWeight<Runtime>;
	type NftClassId = ClassId;
	type NftInstanceId = InstanceId;
	type ProtocolOrigin = EnsureRoot<AccountId>;
	type ClassType = pallet_nft::ClassType;
	type Permissions = pallet_nft::NftPermissions;
	type ReserveClassIdUpTo = ReserveClassIdUpTo;
}

parameter_types! {
	pub const ClassDeposit: Balance = 10_000 * UNITS; // 10 000 UNITS deposit to create asset class
	pub const InstanceDeposit: Balance = 100 * UNITS; // 100 UNITS deposit to create asset instance
	pub const KeyLimit: u32 = 256;	// Max 256 bytes per key
	pub const ValueLimit: u32 = 1024;	// Max 1024 bytes per value
	pub const UniquesMetadataDepositBase: Balance = 100 * UNITS;
	pub const AttributeDepositBase: Balance = 10 * UNITS;
	pub const DepositPerByte: Balance = UNITS;
	pub const UniquesStringLimit: u32 = 72;
}

impl pallet_uniques::Config for Runtime {
	type Event = Event;
	type ClassId = ClassId;
	type InstanceId = InstanceId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type ClassDeposit = ClassDeposit;
	type InstanceDeposit = InstanceDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = ();
}

type EnsureMajorityCouncilOrRoot = EnsureOneOf<
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
type EnsureUnanimousCouncilOrRoot = EnsureOneOf<
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
type EnsureSuperMajorityCouncilOrRoot = EnsureOneOf<
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
type EnsureSuperMajorityTechCommitteeOrRoot = EnsureOneOf<
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
type EnsureUnanimousTechCommitteeOrRoot = EnsureOneOf<
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = testing::LaunchPeriod;
	type VotingPeriod = testing::VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureMajorityCouncilOrRoot;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote
	type ExternalMajorityOrigin = EnsureMajorityCouncilOrRoot;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureUnanimousCouncilOrRoot;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureSuperMajorityTechCommitteeOrRoot;
	type InstantOrigin = EnsureUnanimousTechCommitteeOrRoot;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureSuperMajorityCouncilOrRoot;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureUnanimousTechCommitteeOrRoot;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = common_runtime::weights::democracy::BasiliskWeight<Runtime>;
	type MaxProposals = MaxProposals;
	type VoteLockingPeriod = EnactmentPeriod;
}

impl pallet_elections_phragmen::Config for Runtime {
	type Event = Event;
	type PalletId = ElectionsPhragmenPalletId;
	type Currency = Balances;
	type ChangeMembers = Council;
	type InitializeMembers = (); // Set to () if defined in chain spec
	type CurrencyToVote = U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = Treasury;
	type KickedMember = Treasury;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = ();
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type AllCouncilMembers = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
type ManageOrigin = EnsureOneOf<EnsureRoot<AccountId>, AllCouncilMembers>;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = ManageOrigin;
	type RejectOrigin = ManageOrigin;
	type Event = Event;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type WeightInfo = common_runtime::weights::treasury::BasiliskWeight<Runtime>;
	type SpendFunds = ();
	type MaxApprovals = MaxApprovals;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
	pub const NoPreimagePostponement: Option<u32> = Some(5 * MINUTES);
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = common_runtime::weights::scheduler::BasiliskWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = Preimage;
	type NoPreimagePostponement = NoPreimagePostponement;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = common_runtime::weights::utility::BasiliskWeight<Runtime>;
	type PalletsOrigin = OriginCaller;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (CollatorSelection,);
}

impl pallet_tips::Config for Runtime {
	type Event = Event;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = Elections;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type WeightInfo = ();
}

impl pallet_collator_selection::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	//allow 1/2 of council to execute privileged collator selection operations. (require code from: feat/initial_chain_setup)
	type UpdateOrigin = EnsureMajorityCouncilOrRoot;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinCandidates = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = testing::Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<testing::Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<testing::Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <opaque::SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type WeightInfo = ();
}

pub struct RootAsVestingPallet;
impl EnsureOrigin<Origin> for RootAsVestingPallet {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(VestingPalletId::get().into_account()),
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
			.expect("infinite length input; no invalid inputs for type; qed");
		Origin::from(RawOrigin::Signed(zero_account_id))
	}
}

impl orml_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = RootAsVestingPallet;
	type WeightInfo = ();
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Runtime>;
}

parameter_types! {
	pub const MinimumOfferAmount: Balance = 10000 * UNITS;
	pub const RoyaltyBondAmount: Balance = 2000 * UNITS;
}

pub struct RelayChainAssetId;
impl Get<AssetId> for RelayChainAssetId {
	fn get() -> AssetId {
		let invalid_id = pallet_asset_registry::Pallet::<Runtime>::next_asset_id();

		match pallet_asset_registry::Pallet::<Runtime>::location_to_asset(RELAY_CHAIN_ASSET_LOCATION) {
			Some(asset_id) => asset_id,
			None => invalid_id,
		}
	}
}

impl pallet_marketplace::Config for Runtime {
	type Event = Event;
	type Currency = CurrencyAdapter<Runtime, RelayChainAssetId>;
	type WeightInfo = pallet_marketplace::weights::BasiliskWeight<Runtime>;
	type MinimumOfferAmount = MinimumOfferAmount;
	type RoyaltyBondAmount = RoyaltyBondAmount;
}

impl pallet_relaychain_info::Config for Runtime {
	type Event = Event;
	type RelaychainBlockNumberProvider = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Runtime>;
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub PreimageBaseDeposit: Balance = deposit(2, 64);
	pub PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = ();
	type Event = Event;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type MaxSize = PreimageMaxSize;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

impl pallet_identity::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureMajorityCouncilOrRoot;
	type RegistrarOrigin = EnsureMajorityCouncilOrRoot;
	type WeightInfo = ();
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Substrate
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 2,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 3,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 4,
		Utility: pallet_utility::{Pallet, Call, Event} = 5,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 6,
		Democracy: pallet_democracy::{Pallet, Call, Storage, Event<T>} = 7,
		Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 8,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 9,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 10,
		Vesting: orml_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 11,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 12,
		Tips: pallet_tips::{Pallet, Call, Storage, Event<T>} = 13,

		Authorship: pallet_authorship::{Pallet, Call, Storage} = 14,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 15,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 16, // Session must be after collator and before aura
		Aura: pallet_aura::{Pallet, Config<T>} = 17,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Config} = 18,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 19,
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>} = 20,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 21,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 22,

		// Parachain and XCM - starts at index 50
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>, ValidateUnsigned} = 50,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 51,

		PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config} = 52,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin} = 53,
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Storage, Event<T>} = 54,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 55,

		// Basilisk - runtime module index for basilisk's pallets starts at 100
		AssetRegistry: pallet_asset_registry::{Pallet, Call, Config<T>, Storage, Event<T>} = 100,
		XYK: pallet_xyk::{Pallet, Call, Storage, Event<T>} = 101,
		Duster: pallet_duster::{Pallet, Call, Config<T>, Storage, Event<T>} = 102,
		Exchange: pallet_exchange::{Pallet, Call, Storage, Event<T>} = 103,
		LBP: pallet_lbp::{Pallet, Call, Storage, Event<T>} = 104,
		NFT: pallet_nft::{Pallet, Call, Event<T>, Storage} = 105,

		MultiTransactionPayment: pallet_transaction_multi_payment::{Pallet, Call, Config<T>, Storage, Event<T>} = 106,
		RelayChainInfo: pallet_relaychain_info::{Pallet, Event<T>} = 108,
		Marketplace: pallet_marketplace::{Pallet, Call, Event<T>, Storage} = 109,

		// ORML related modules - starts at 150
		Currencies: orml_currencies::{Pallet, Call, Event<T>} = 150,
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>} = 151,

		// ORML XCM
		OrmlXcm: orml_xcm::{Pallet, Call, Event<T>} = 153,
		XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 154,
		UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 155,

		// TEMPORARY - always last. Sudo will be removed at some point.
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 255,
	}
);

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	pallet_transaction_multi_payment::CurrencyBalanceCheck<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsReversedWithSystemFirst,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

		#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			//log::info!("try-runtime::on_runtime_upgrade.");
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block_no_check(block: Block) -> Weight {
			Executive::execute_block_no_check(block)
		}
	}


	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl xyk_rpc::XYKApi<
		Block,
		AccountId,
		AssetId,
		Balance,
	> for Runtime {
		fn get_pool_balances(
			pool_address: AccountId,
		) -> Vec<xyk_rpc::BalanceInfo<AssetId, Balance>> {
			let mut vec = Vec::new();

			if let Some(pool_balances) = XYK::get_pool_balances(pool_address){
				for b in pool_balances {
					let item  = xyk_rpc::BalanceInfo{
					 asset: Some(b.0),
						amount: b.1
					};

					vec.push(item);
				}
			}

			vec
		}

		fn get_pool_id(asset_a: AssetId, asset_b: AssetId) -> AccountId{
			XYK::pair_account_from_assets(asset_a, asset_b)
		}

	}

	impl pallet_lbp_rpc_runtime_api::LBPApi<
		Block,
		AccountId,
		AssetId,
	> for Runtime {
		fn get_pool_id(asset_a: AssetId, asset_b: AssetId) -> AccountId{
			LBP::pair_account_from_assets(asset_a, asset_b)
		}
	}


	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			use pallet_exchange_benchmarking::Pallet as ExchangeBench;
			use frame_system_benchmarking::Pallet as SystemBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, pallet_xyk, XYK);
			list_benchmark!(list, extra, pallet_lbp, LBP);
			list_benchmark!(list, extra, pallet_exchange, ExchangeBench::<Runtime>);
			list_benchmark!(list, extra, pallet_nft, NFT);
			list_benchmark!(list, extra, pallet_marketplace, Marketplace);
			list_benchmark!(list, extra, pallet_asset_registry, AssetRegistry);

			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_exchange, ExchangeBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_democracy, Democracy);
			list_benchmark!(list, extra, pallet_treasury, Treasury);
			list_benchmark!(list, extra, pallet_scheduler, Scheduler);
			list_benchmark!(list, extra, pallet_utility, Utility);
			list_benchmark!(list, extra, pallet_tips, Tips);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use pallet_exchange_benchmarking::Pallet as ExchangeBench;
			use frame_system_benchmarking::Pallet as SystemBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl pallet_exchange_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			// Basilisk pallets
			add_benchmark!(params, batches, pallet_xyk, XYK);
			add_benchmark!(params, batches, pallet_lbp, LBP);
			add_benchmark!(params, batches, pallet_exchange, ExchangeBench::<Runtime>);
			add_benchmark!(params, batches, pallet_nft, NFT);
			add_benchmark!(params, batches, pallet_asset_registry, AssetRegistry);
			add_benchmark!(params, batches, pallet_marketplace, Marketplace);

			// Substrate pallets
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_exchange, ExchangeBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_democracy, Democracy);
			add_benchmark!(params, batches, pallet_treasury, Treasury);
			add_benchmark!(params, batches, pallet_scheduler, Scheduler);
			add_benchmark!(params, batches, pallet_utility, Utility);
			add_benchmark!(params, batches, pallet_tips, Tips);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
			relay_chain_slot,
			sp_std::time::Duration::from_secs(6),
		)
		.create_inherent_data()
		.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
