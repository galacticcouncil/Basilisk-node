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

use crate::governance::TechnicalCollective;
use crate::*;

use primitives::constants::{
	currency::DOLLARS,
	time::{DAYS, HOURS},
};

use frame_support::parameter_types;
use frame_system::{EnsureRoot, EnsureSigned};

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
	type WeightInfo = weights::pallet_democracy::BasiliskWeight<Runtime>;
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
	type ExternalOrigin = EnsureRoot<AccountId>;
	type ExternalMajorityOrigin = EnsureRoot<AccountId>;
	type ExternalDefaultOrigin = EnsureRoot<AccountId>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureRoot<AccountId>;
	type InstantOrigin = EnsureRoot<AccountId>;
	type CancellationOrigin = EnsureRoot<AccountId>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type PalletsOrigin = OriginCaller;
	type Slash = Treasury;
	type SubmitOrigin = EnsureSigned<AccountId>;
}
