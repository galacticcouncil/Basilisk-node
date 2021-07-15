#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::from_over_into)]

#[cfg(test)]
mod tests;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use frame_system::{EnsureOneOf, EnsureRoot, RawOrigin};
use sp_api::impl_runtime_apis;
use sp_core::{
	u32_trait::{_1, _2, _3},
	OpaqueMetadata,
};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, BlakeTwo256, Block as BlockT, IdentifyAccount, IdentityLookup, Verify, Zero},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};
use sp_std::convert::From;
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{EnsureOrigin, Filter, Get, KeyOwnerProofSystem, LockIdentifier, Randomness, U128CurrencyToVote},
	weights::{
		constants::{BlockExecutionWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Pays, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
	PalletId, StorageValue,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{FixedPointNumber, Perbill, Permill, Perquintill, Percent};

use primitives::fee;

mod currency;

mod weights;

use pallet_xyk_rpc_runtime_api as xyk_rpc;

use orml_currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;

pub use primitives::{Amount, AssetId, Balance, Moment, CORE_ASSET_ID};

pub use pallet_asset_registry;

use pallet_transaction_multi_payment::{weights::WeightInfo, MultiCurrencyAdapter};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

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

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("basilisk"),
	impl_name: create_runtime_str!("basilisk"),
	authoring_version: 1,
	spec_version: 12,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};
pub const MILLISECS_PER_BLOCK: u64 = 12000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const FORTUNE: Balance = u128::MAX;
pub const BSX: Balance = 1_000_000_000_000;
pub const DOLLARS: Balance = BSX * 100; // 100 BSX ~= 1 $
pub const CENTS: Balance = DOLLARS / 100; // 1 BSX ~= 1 cent
pub const MILLICENTS: Balance = CENTS / 1_000;

/// We assume that an on-initialize consumes 2.5% of the weight on average, hence a single extrinsic
/// will not be allowed to consume more than `AvailableBlockRatio - 2.5%`.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_perthousand(25);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

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

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

pub struct BaseFilter;
impl Filter<Call> for BaseFilter {
	fn filter(call: &Call) -> bool {
		match call {
			Call::System(_)
			| Call::Timestamp(_)
			| Call::ParachainSystem(_)
			| Call::Elections(_)
			| Call::Democracy(_)
			| Call::Scheduler(_)
			| Call::Council(_)
			| Call::TechnicalCommittee(_)
			| Call::Treasury(_)
			| Call::Authorship(_)
			| Call::CollatorSelection(_)
			| Call::Session(_)
			| Call::Balances(_)
			| Call::AssetRegistry(_)
			| Call::Currencies(_)
			| Call::Exchange(_)
			| Call::MultiTransactionPayment(_)
			| Call::Tokens(_)
			| Call::Tips(_)
			| Call::LBP(_)
			| Call::Utility(_)
			| Call::Vesting(_)
			| Call::NFT(_)
			| Call::Sudo(_) => true,

			Call::XYK(_) => false,
		}
	}
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	/// Maximum length of block. Up to 5MB.
	pub BlockLength: frame_system::limits::BlockLength =
		frame_system::limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
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
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub ExtrinsicPaymentExtraWeight: Weight =  <Runtime as pallet_transaction_multi_payment::Config>::WeightInfo::swap_currency();
	pub ExtrinsicBaseWeight: Weight = frame_support::weights::constants::ExtrinsicBaseWeight::get() + ExtrinsicPaymentExtraWeight::get();
	pub const SS58Prefix: u16 = 10041;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = BaseFilter;
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
	type SystemWeightInfo = weights::system::HydraWeight<Runtime>;
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
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
	type WeightInfo = weights::timestamp::HydraWeight<Runtime>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

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
	pub const MultiPaymentCurrencySetFee: Pays = Pays::Yes;
}

/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/Token%20Economics.html#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = MultiCurrencyAdapter<Balances, (), MultiTransactionPayment>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

impl pallet_transaction_multi_payment::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type AMMPool = XYK;
	type WeightInfo = weights::payment::HydraWeight<Runtime>;
	type WithdrawFeeForSetCurrency = MultiPaymentCurrencySetFee;
	type WeightToFee = WeightToFee;
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		Zero::zero()
	};
}

/// ORML Configurations
impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
}

impl orml_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = NativeAssetId;
	type WeightInfo = ();
}

/// Basilisk Pallets configurations

impl pallet_asset_registry::Config for Runtime {
	type AssetId = AssetId;
}

parameter_types! {
	pub ExchangeFee: fee::Fee = fee::Fee::default();
}

impl pallet_xyk::Config for Runtime {
	type Event = Event;
	type AssetPairAccountId = pallet_xyk::AssetPairAccountId<Self>;
	type Currency = Currencies;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = weights::xyk::HydraWeight<Runtime>;
	type GetExchangeFee = ExchangeFee;
}

impl pallet_exchange::Config for Runtime {
	type Event = Event;
	type AMMPool = XYK;
	type Resolver = Exchange;
	type Currency = Currencies;
	type WeightInfo = weights::exchange::HydraWeight<Runtime>;
}

parameter_types! {
	pub LBPExchangeFee: fee::Fee  = fee::Fee::default();
}

impl pallet_lbp::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Currencies;
	type NativeAssetId = NativeAssetId;
	type CreatePoolOrigin = EnsureRoot<AccountId>; // TODO: change to governance membership
	type LBPWeightFunction = pallet_lbp::LBPWeightFunction;
	type AssetPairPoolId = pallet_lbp::AssetPairPoolId<Self>;
	type WeightInfo = pallet_lbp::weights::HydraWeight<Runtime>;
}

/// Parachain Config

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = ParachainInfo;
	type OutboundXcmpMessageSource = ();
	type DmpMessageHandler = ();
	type ReservedDmpWeight = ();
	type XcmpMessageHandler = ();
	type ReservedXcmpWeight = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl pallet_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	pub ClassBondAmount: Balance = 10_000 * BSX;
	pub ClassBondDuration: u32 = 7 * DAYS;
}

impl pallet_nft::Config for Runtime {
	type Currency = Balances;
	type Event = Event;
	type WeightInfo = pallet_nft::weights::BasiliskWeight<Runtime>;
	type ClassBondAmount = ClassBondAmount;
	type ClassBondDuration = ClassBondDuration;
}

impl orml_nft::Config for Runtime {
	type ClassId = u64;
	type TokenId = u64;
	type ClassData = pallet_nft::ClassData;
	type TokenData = pallet_nft::TokenData;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 7 * DAYS;
	pub const VotingPeriod: BlockNumber = 7 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
	pub const MinimumDeposit: Balance = 1000 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 6 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub const PreimageByteDeposit: Balance = CENTS;
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 30;
	pub const MaxProposals: u32 = 30;
}

type EnsureMajorityCouncilOrRoot = frame_system::EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
type EnsureUnanimousCouncilOrRoot = frame_system::EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
type EnsureSuperMajorityCouncilOrRoot = frame_system::EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
type EnsureSuperMajorityTechCommitteeOrRoot = frame_system::EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
type EnsureUnanimousTechCommitteOrRoot = frame_system::EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	frame_system::EnsureRoot<AccountId>,
>;
impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
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
	type InstantOrigin = EnsureUnanimousTechCommitteOrRoot;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureSuperMajorityCouncilOrRoot;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureUnanimousTechCommitteOrRoot;
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
	type WeightInfo = ();
	type MaxProposals = MaxProposals;
}

parameter_types! {
	// Bond for candidacy into governance
	pub const CandidacyBond: Balance = 100_000_000_000 * BSX;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = CENTS;
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = CENTS;
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 1;
	pub const DesiredRunnersUp: u32 = 0;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
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

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 20;
	pub const ProposalVotesRequired: u32 = 1;
	pub const CouncilMaxMembers: u32 = 1;
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

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
	pub const TechnicalMaxProposals: u32 = 20;
	pub const TechnicalMaxMembers: u32 = 10;
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

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 100 * DOLLARS;
	pub const SpendPeriod: BlockNumber = DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaxApprovals: u32 =  100;
}

type AllCouncilMembers = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
type ManageOrigin = EnsureOneOf<AccountId, EnsureRoot<AccountId>, AllCouncilMembers>;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = ManageOrigin;
	type RejectOrigin = ManageOrigin;
	type Event = Event;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type WeightInfo = ();
	type SpendFunds = ();
	type MaxApprovals = MaxApprovals;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 20;             // only for benchmarking
	pub const MinCandidates: u32 = 0;
	pub const MaxInvulnerables: u32 = 10;
}

parameter_types! {
	pub const DataDepositPerByte: Balance = CENTS;
	pub const TipCountdown: BlockNumber = 4 * HOURS;
	pub const TipFindersFee: Percent = Percent::from_percent(1);
	pub const TipReportDepositBase: Balance = 10 * DOLLARS;
	pub const TipReportDepositPerByte: Balance = CENTS;
	pub const MaximumReasonLength: u32 = 1024;
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
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
	pub const Period: u32 = 4 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <opaque::SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

pub struct EnsureRootOrTreasury;
impl EnsureOrigin<Origin> for EnsureRootOrTreasury {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(TreasuryPalletId::get().into_account()),
			RawOrigin::Signed(caller) => {
				if caller == TreasuryPalletId::get().into_account() {
					Ok(caller)
				} else {
					Err(Origin::from(Some(caller)))
				}
			}
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		Origin::from(RawOrigin::Signed(Default::default()))
	}
}

parameter_types! {
	pub MinVestedTransfer: Balance = 100_000 * BSX;
	pub const MaxVestingSchedules: u32 = 100;
}

impl orml_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = EnsureRootOrTreasury;
	type WeightInfo = ();
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},

		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Event<T>},
		Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Vesting: orml_vesting::{Pallet, Call, Storage, Event<T>, Config<T>},

		// Parachain
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
		ParachainInfo: parachain_info::{Pallet, Storage, Config},

		// Collator support
		Authorship: pallet_authorship::{Pallet, Call, Storage},
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
		Aura: pallet_aura::{Pallet, Config<T>},
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Config},

		// ORML related modules
		Currencies: orml_currencies::{Pallet, Call, Event<T>},
		OrmlNft: orml_nft::{Pallet, Storage, Config<T>},
		Tips: pallet_tips::{Pallet, Call, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>, Config<T>},

		// Basilisk related modules
		AssetRegistry: pallet_asset_registry::{Pallet, Call, Storage, Config<T>},
		XYK: pallet_xyk::{Pallet, Call, Storage, Event<T>},
		Exchange: pallet_exchange::{Pallet, Call, Storage, Event<T>},
		LBP: pallet_lbp::{Pallet, Call, Storage, Event<T>},
		MultiTransactionPayment: pallet_transaction_multi_payment::{Pallet, Call, Config<T>, Storage, Event<T>},
		NFT: pallet_nft::{Pallet, Call, Event<T>, Storage},
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
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPallets>;

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
			Runtime::metadata().into()
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
			Aura::authorities()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info()
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

			let pool_balances = XYK::get_pool_balances(pool_address).unwrap();

			for b in pool_balances {
				let item  = xyk_rpc::BalanceInfo{
				 asset: Some(b.0),
					amount: b.1
				};

				vec.push(item);
			}

			vec
		}

	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use pallet_exchange_benchmarking::Pallet as ExchangeBench;
			use frame_system_benchmarking::Pallet as SystemBench;
			use pallet_multi_payment_benchmarking::Pallet as MultiBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl pallet_exchange_benchmarking::Config for Runtime {}
			impl pallet_multi_payment_benchmarking::Config for Runtime {}

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

			add_benchmark!(params, batches, xyk, XYK);
			add_benchmark!(params, batches, lbp, LBP);
			add_benchmark!(params, batches, transaction_multi_payment, MultiBench::<Runtime>);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, exchange, ExchangeBench::<Runtime>);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_nft, NFT);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, transaction_multi_payment, MultiBench::<Runtime>);

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

		inherent_data.check_extrinsics(&block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
