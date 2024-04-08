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
use crate::old::{MajorityCouncilOrRoot, SuperMajorityCouncilOrRoot};
use frame_support::{parameter_types, sp_runtime::Permill, traits::NeverEnsureOrigin, PalletId};
use primitives::constants::{currency::DOLLARS, time::DAYS};

// Old governance configurations.
pub mod old;

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
