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
use frame_support::{
	parameter_types,
	sp_runtime::Permill,
	traits::{
		fungible,
		tokens::{Pay, PaymentStatus, Preservation, UnityAssetBalanceConversion},
		NeverEnsureOrigin,
	},
	PalletId,
};
use primitives::constants::{currency::DOLLARS, time::DAYS};
use sp_runtime::{traits::IdentityLookup, DispatchError};

mod origins;
mod tracks;
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
	pub const TreasuryPayoutPeriod: u32 = 30 * DAYS;
}

impl pallet_treasury::Config for Runtime {
	type Currency = Balances;
	// TODO origin
	type ApproveOrigin = SuperMajorityCouncilOrRoot;
	// TODO origin
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
	type WeightInfo = weights::pallet_treasury::BasiliskWeight<Runtime>;
	type SpendFunds = ();
	type MaxApprovals = MaxApprovals;
	// TODO origin TreasurySpender
	#[cfg(not(feature = "runtime-benchmarks"))]
	type SpendOrigin = NeverEnsureOrigin<Balance>; // Disabled, no spending
	#[cfg(feature = "runtime-benchmarks")]
	type SpendOrigin =
		frame_system::EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, crate::benches::BenchmarkMaxBalance>;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<AccountId>;
	type Paymaster = PayFromTreasuryAccount;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = TreasuryPayoutPeriod;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = benchmarking::BenchmarkHelper;
}

pub struct PayFromTreasuryAccount;

impl Pay for PayFromTreasuryAccount {
	type Balance = Balance;
	type Beneficiary = AccountId;
	type AssetKind = ();
	type Id = ();
	type Error = DispatchError;

	#[cfg(not(feature = "runtime-benchmarks"))]
	fn pay(
		who: &Self::Beneficiary,
		_asset_kind: Self::AssetKind,
		amount: Self::Balance,
	) -> Result<Self::Id, Self::Error> {
		let _ = <Balances as fungible::Mutate<_>>::transfer(
			&TreasuryAccount::get(),
			who,
			amount,
			Preservation::Expendable,
		)?;
		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn pay(
		who: &Self::Beneficiary,
		_asset_kind: Self::AssetKind,
		amount: Self::Balance,
	) -> Result<Self::Id, Self::Error> {
		// In case of benchmarks, we adjust the value by multiplying it by 1_000_000_000_000, otherwise it fails with BelowMinimum limit error, because
		// treasury benchmarks uses only 100 as the amount.
		let _ = <Balances as fungible::Mutate<_>>::transfer(
			&TreasuryAccount::get(),
			who,
			amount * 1_000_000_000_000,
			Preservation::Expendable,
		)?;
		Ok(())
	}

	fn check_payment(_id: Self::Id) -> PaymentStatus {
		PaymentStatus::Success
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(_: &Self::Beneficiary, _: Self::AssetKind, amount: Self::Balance) {
		<Balances as fungible::Mutate<_>>::mint_into(&TreasuryAccount::get(), amount * 1_000_000_000_000).unwrap();
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_concluded(_: Self::Id) {}
}

impl origins::pallet_custom_origins::Config for Runtime {}
