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
use primitives::constants::{
	currency::{deposit, CENTS, DOLLARS},
	time::{DAYS, HOURS},
};

use frame_support::{
	parameter_types,
	sp_runtime::{Perbill, Percent, Permill},
	traits::{EitherOfDiverse, EqualPrivilegeOnly, LockIdentifier, NeverEnsureOrigin},
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_collective::EnsureProportionAtLeast;
use sp_staking::currency_to_vote::U128CurrencyToVote;

pub type MajorityCouncilOrRoot =
	EitherOfDiverse<EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>, EnsureRoot<AccountId>>;
pub type UnanimousCouncilOrRoot =
	EitherOfDiverse<EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>, EnsureRoot<AccountId>>;
pub type SuperMajorityCouncilOrRoot =
	EitherOfDiverse<EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>, EnsureRoot<AccountId>>;
pub type SuperMajorityTechCommitteeOrRoot =
	EitherOfDiverse<EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>, EnsureRoot<AccountId>>;
pub type UnanimousTechCommitteeOrRoot =
	EitherOfDiverse<EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>, EnsureRoot<AccountId>>;
pub type MajorityTechCommitteeOrRoot =
	EitherOfDiverse<EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 2>, EnsureRoot<AccountId>>;

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 3 * DAYS;
	pub const VotingPeriod: BlockNumber = 3 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
	pub const MinimumDeposit: Balance = 1000 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 12 * HOURS;
	// Make sure VoteLockingPeriod > EnactmentPeriod
	pub const VoteLockingPeriod: BlockNumber = 6 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type WeightInfo = weights::democracy::BasiliskWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Preimages = Preimage;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = VoteLockingPeriod;
	type MinimumDeposit = MinimumDeposit;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type CooloffPeriod = CooloffPeriod;
	type MaxVotes = MaxVotes;
	type MaxProposals = MaxProposals;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = MajorityCouncilOrRoot;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote
	type ExternalMajorityOrigin = MajorityCouncilOrRoot;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = UnanimousCouncilOrRoot;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = MajorityTechCommitteeOrRoot;
	type InstantOrigin = UnanimousTechCommitteeOrRoot;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = SuperMajorityCouncilOrRoot;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = UnanimousTechCommitteeOrRoot;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type PalletsOrigin = OriginCaller;
	type Slash = Treasury;
	type SubmitOrigin = EnsureSigned<AccountId>;
}

parameter_types! {
	// Bond for candidacy into governance
	pub const CandidacyBond: Balance = 5 * DOLLARS;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = CENTS;
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = CENTS;
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 7;
	pub const DesiredRunnersUp: u32 = 9;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
	pub const MaxElectionCandidates: u32 = 1_000;
	pub const MaxElectionVoters: u32 = 10_000;
	pub const MaxVotesPerVoter: u32 = 5;
}

impl pallet_elections_phragmen::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type MaxCandidates = MaxElectionCandidates;
	type MaxVoters = MaxElectionVoters;
	type WeightInfo = ();
	type MaxVotesPerVoter = MaxVotesPerVoter;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 13;
	pub const CouncilMaxMembers: u32 = 7;
	pub MaxProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = (); // TODO
	type MaxProposalWeight = MaxProposalWeight;
	type SetMembersOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
	pub const TechnicalMaxProposals: u32 = 20;
	pub const TechnicalMaxMembers: u32 = 10;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = (); // TODO
	type MaxProposalWeight = MaxProposalWeight;
	type SetMembersOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const DataDepositPerByte: Balance = CENTS;
	pub const TipCountdown: BlockNumber = 2 * HOURS;
	pub const TipFindersFee: Percent = Percent::from_percent(1);
	pub const TipReportDepositBase: Balance = 10 * DOLLARS;
	pub const TipReportDepositPerByte: Balance = CENTS;
	pub const MaximumReasonLength: u32 = 1024;
}

impl pallet_tips::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaximumReasonLength = MaximumReasonLength;
	type DataDepositPerByte = DataDepositPerByte;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type Tippers = Elections;
	type WeightInfo = ();
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub PreimageBaseDeposit: Balance = deposit(2, 64);
	pub PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
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
	type WeightInfo = weights::scheduler::BasiliskWeight<Runtime>;
	type Preimages = Preimage;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(3);
	pub const ProposalBondMinimum: Balance = 100 * DOLLARS;
	pub const ProposalBondMaximum: Balance = 500 * DOLLARS;
	pub const SpendPeriod: BlockNumber = 3 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaxApprovals: u32 =  100;
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

impl pallet_treasury::Config for Runtime {
	type Currency = Balances;
	type ApproveOrigin = SuperMajorityCouncilOrRoot;
	type RejectOrigin = MajorityCouncilOrRoot;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ProposalBondMaximum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type PalletId = TreasuryPalletId;
	type BurnDestination = ();
	type WeightInfo = weights::treasury::BasiliskWeight<Runtime>;
	type SpendFunds = ();
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = NeverEnsureOrigin<Balance>;
}
