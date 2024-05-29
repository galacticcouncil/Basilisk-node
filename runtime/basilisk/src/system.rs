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
use crate::governance::{
	old::{MajorityCouncilOrRoot, MajorityTechCommitteeOrRoot, SuperMajorityTechCommitteeOrRoot},
	origins::GeneralAdmin,
	TreasuryAccount,
};

use pallet_transaction_multi_payment::{DepositAll, TransferFees};
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use primitives::constants::{
	chain::{CORE_ASSET_ID, MAXIMUM_BLOCK_WEIGHT},
	currency::{deposit, CENTS, DOLLARS, MILLICENTS},
	time::{HOURS, SLOT_DURATION},
};


use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::DispatchClass,
	pallet_prelude::Get,
	parameter_types,
	sp_runtime::{traits::IdentityLookup, FixedPointNumber, Perbill, Perquintill, RuntimeDebug},
	traits::{
		fungible::HoldConsideration, ConstBool, Contains, Defensive, EitherOf, EqualPrivilegeOnly, InstanceFilter, LinearStoragePrice,
		SortedMembers,
	},
	weights::{
		constants::{BlockExecutionWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_MICROS},
		ConstantMultiplier, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use hydradx_adapters::RelayChainBlockNumberProvider;
use hydradx_traits::evm::InspectEvmAccounts;
use primitives::constants::time::DAYS;
use scale_info::TypeInfo;

/// We assume that an on-initialize consumes 2.5% of the weight on average, hence a single extrinsic
/// will not be allowed to consume more than `AvailableBlockRatio - 2.5%`.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_perthousand(25);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
	fn contains(call: &RuntimeCall) -> bool {
		if matches!(
			call,
			RuntimeCall::System(_) | RuntimeCall::Timestamp(_) | RuntimeCall::ParachainSystem(_)
		) {
			// always allow
			// Note: this is done to avoid unnecessary check of paused storage.
			return true;
		}

		if pallet_transaction_pause::PausedTransactionFilter::<Runtime>::contains(call) {
			// if paused, dont allow!
			return false;
		}

		match call {
			RuntimeCall::Uniques(_) => false,
			RuntimeCall::PolkadotXcm(_) => false,
			RuntimeCall::OrmlXcm(_) => false,
			_ => true,
		}
	}
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	/// Maximum length of block. Up to 5MB.
	pub BlockLength: frame_system::limits::BlockLength =
		frame_system::limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u16 = 10041;
	/// Basilisk base weight of an extrinsic
	/// This includes weight for payment in non-native currency.
	// Default substrate base weight is 125 * WEIGHT_PER_MICROS
	pub const BasiliskExtrinsicBaseWeight: Weight = Weight::from_parts(200 * WEIGHT_REF_TIME_PER_MICROS, 0);
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
	pub ExtrinsicBaseWeight: Weight = BasiliskExtrinsicBaseWeight::get();
}

impl frame_system::Config for Runtime {
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = BaseFilter;
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = RuntimeTask;
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Index;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = IdentityLookup<AccountId>;
	/// The index type for blocks.
	type Block = Block;
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
	type SystemWeightInfo = weights::frame_system::BasiliskWeight<Runtime>;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
	pub const NativeAssetId : AssetId = CORE_ASSET_ID;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::BasiliskWeight<Runtime>;
}

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
		let p = 11 * CENTS;
		let q = Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec::smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/overview/2-token-economics.html?highlight=token%20economics#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier, MaximumMultiplier>;

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(6, 100_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);
	/// Maximum amount of the multiplier.
	pub MaximumMultiplier: Multiplier = Multiplier::saturating_from_integer(4);
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = TransferFees<Currencies, DepositAll<Runtime>, TreasuryAccount>;
	type OperationalFeeMultiplier = ();
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

pub struct WethAssetId;
impl Get<AssetId> for WethAssetId {
	fn get() -> AssetId {
		pallet_asset_registry::Pallet::<crate::Runtime>::next_asset_id().defensive_unwrap_or(AssetId::MAX)
	}
}

pub struct EvmAccounts;
impl InspectEvmAccounts<AccountId, sp_core::H160> for EvmAccounts {
	fn is_evm_account(_account_id: AccountId) -> bool {
		false
	}

	fn evm_address(_account_id: &impl AsRef<[u8; 32]>) -> sp_core::H160 {
		sp_core::H160::default()
	}

	fn truncated_account_id(_evm_address: sp_core::H160) -> AccountId {
		AccountId::new([0u8; 32])
	}

	fn bound_account_id(_evm_address: sp_core::H160) -> Option<AccountId> {
		None
	}

	fn account_id(_evm_address: sp_core::H160) -> AccountId {
		AccountId::new([0u8; 32])
	}

	fn can_deploy_contracts(_evm_address: sp_core::H160) -> bool {
		false
	}
}

impl pallet_transaction_multi_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AcceptedCurrencyOrigin = EitherOf<EnsureRoot<Self::AccountId>, GeneralAdmin>;
	type Currencies = Currencies;
	type RouteProvider = Router;
	type OraclePriceProvider = adapter::OraclePriceProvider<AssetId, EmaOracle>;
	type WeightInfo = weights::pallet_transaction_multi_payment::BasiliskWeight<Runtime>;
	type WeightToFee = WeightToFee;
	type NativeAssetId = NativeAssetId;
	type EvmAssetId = WethAssetId;
	type InspectEvmAccounts = EvmAccounts;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ProxyType {
	Any,
	CancelProxy,
	Governance,
	Exchange,
	Transfer,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::CancelProxy => matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Democracy(..)
					| RuntimeCall::Council(..)
					| RuntimeCall::TechnicalCommittee(..)
					| RuntimeCall::Elections(..)
					| RuntimeCall::Treasury(..)
					| RuntimeCall::Tips(..)
					| RuntimeCall::Utility(..)
			),
			ProxyType::Exchange => matches!(c, RuntimeCall::XYK(..) | RuntimeCall::LBP(..) | RuntimeCall::NFT(..)),
			// Transfer group doesn't include cross-chain transfers
			ProxyType::Transfer => matches!(
				c,
				RuntimeCall::Balances(..) | RuntimeCall::Currencies(..) | RuntimeCall::Tokens(..)
			),
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

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub PreimageBaseDeposit: Balance = deposit(2, 64);
	pub PreimageByteDeposit: Balance = deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_preimage::BasiliskWeight<Runtime>;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::BasiliskWeight<Runtime>;
	type Preimages = Preimage;
}

parameter_types! {
	pub ProxyDepositBase: Balance = 4 * DOLLARS + 480 * MILLICENTS;
	pub ProxyDepositFactor: Balance = 1_980 * MILLICENTS;
	pub const MaxProxies: u16 = 32;
	pub AnnouncementDepositBase: Balance = 4 * DOLLARS + 480 * MILLICENTS;
	pub AnnouncementDepositFactor: Balance = 3_960 * MILLICENTS;
	pub const MaxPending: u16 = 32;
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = weights::pallet_proxy::BasiliskWeight<Runtime>;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
	pub ReservedXcmpWeight: Weight = BlockWeights::get().max_block / 4;
	pub ReservedDmpWeight: Weight = BlockWeights::get().max_block / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = pallet_relaychain_info::OnValidationDataHandler<Runtime>;
	type SelfParaId = ParachainInfo;
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type WeightInfo = weights::cumulus_pallet_parachain_system::BasiliskWeight<Runtime>;
}

parameter_types! {
	pub const MaxAuthorities: u32 = 50;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type MaxAuthorities = MaxAuthorities;
	type DisabledValidators = ();
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::BasiliskWeight<Runtime>;
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 20;
	pub const MaxInvulnerables: u32 = 50;
}

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = EitherOf<EnsureRoot<Self::AccountId>, GeneralAdmin>;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	#[cfg(feature = "runtime-benchmarks")]
	type WeightInfo = ();
	#[cfg(not(feature = "runtime-benchmarks"))]
	type WeightInfo = weights::pallet_collator_selection::BasiliskWeight<Runtime>;
	type MinEligibleCollators = ConstU32<4>;
}

parameter_types! {
	pub const Period: u32 = 4 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorRewards;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <opaque::SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type WeightInfo = ();
}

impl staging_parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl pallet_relaychain_info::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RelaychainBlockNumberProvider = RelayChainBlockNumberProvider<Runtime>;
}

impl pallet_transaction_pause::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = MajorityTechCommitteeOrRoot;
	type WeightInfo = weights::pallet_transaction_pause::BasiliskWeight<Runtime>;
}

parameter_types! {
	pub const RewardPerCollator: Balance = 15_216_000_000_000_000; // 12.68[BSX/block] * 1200[block]
	//GalacticCouncil collators
	pub ExcludedCollators: Vec<AccountId> = vec![
		// bXn5CfJB2qHvqnuMqTpXn6un9Fjch8mwkb9i3JUsGVD4ChLoe
		hex_literal::hex!["f25e5d7b43266a5b4cca762c9be917f18852d7a5db85e734776206eeb539dd4f"].into(),
	];
}

impl pallet_collator_rewards::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = AssetId;
	type Currency = Currencies;
	type RewardPerCollator = RewardPerCollator;
	type RewardCurrencyId = NativeAssetId;
	type ExcludedCollators = ExcludedCollators;
	// We wrap the ` SessionManager` implementation of `CollatorSelection` to get the collatrs that
	// we hand out rewards to.
	type SessionManager = CollatorSelection;
	type MaxCandidates = MaxInvulnerables;
}

parameter_types! {
	pub const BasicDeposit: Balance = 5 * DOLLARS;
	pub const ByteDeposit: Balance = DOLLARS / 10;
	pub const SubAccountDeposit: Balance = 5 * DOLLARS;
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
	pub const PendingUserNameExpiration: u32 = 7 * DAYS;
	pub const MaxSuffixLength: u32 = 7;
	pub const MaxUsernameLength: u32 = 32;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type ByteDeposit = ByteDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EitherOf<EnsureRoot<Self::AccountId>, GeneralAdmin>;
	type RegistrarOrigin = EitherOf<EnsureRoot<Self::AccountId>, GeneralAdmin>;
	type OffchainSignature = Signature;
	type SigningPublicKey = <Signature as sp_runtime::traits::Verify>::Signer;
	type UsernameAuthorityOrigin = EnsureRoot<AccountId>;
	type PendingUsernameExpiration = PendingUserNameExpiration;
	type MaxSuffixLength = MaxSuffixLength;
	type MaxUsernameLength = MaxUsernameLength;
	type WeightInfo = weights::pallet_identity::BasiliskWeight<Runtime>;
}

parameter_types! {
	pub DepositBase: Balance = deposit(1, 88);
	pub DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = weights::pallet_multisig::BasiliskWeight<Runtime>;
}

pub struct TechCommAccounts;
impl SortedMembers<AccountId> for TechCommAccounts {
	fn sorted_members() -> Vec<AccountId> {
		TechnicalCommittee::members()
	}
}

parameter_types! {
	// The deposit configuration for the singed migration. Specially if you want to allow any signed account to do the migration (see `SignedFilter`, these deposits should be high)
	pub const MigrationSignedDepositPerItem: Balance = CENTS;
	pub const MigrationSignedDepositBase: Balance = 20 * DOLLARS;
	pub const MaxKeyLen: u32 = 512;	// 144, but use the default value
}

#[cfg(feature = "runtime-benchmarks")]
use frame_system::EnsureSigned;

impl pallet_state_trie_migration::Config for Runtime {
	type ControlOrigin = SuperMajorityTechCommitteeOrRoot;
	#[cfg(feature = "runtime-benchmarks")]
	type SignedFilter = EnsureSigned<AccountId>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type SignedFilter = EnsureSignedBy<TechCommAccounts, AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxKeyLen = MaxKeyLen;
	type SignedDepositPerItem = MigrationSignedDepositPerItem;
	type SignedDepositBase = MigrationSignedDepositBase;
	type WeightInfo = weights::pallet_state_trie_migration::BasiliskWeight<Runtime>;
}
