// This file is part of Basilisk-node

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
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

use crate::{AccountId, AssetId, Balance, Runtime, Tokens, TreasuryAccount};

use super::*;

use frame_benchmarking::account;
use frame_benchmarking::BenchmarkError;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_runtime::traits::SaturatedConversion;

use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};

const SEED: u32 = 1;

pub fn update_balance(currency_id: AssetId, who: &AccountId, balance: Balance) {
	assert_ok!(<Tokens as MultiCurrencyExtended<_>>::update_balance(
		currency_id,
		who,
		balance.saturated_into()
	));
}

runtime_benchmarks! {
	{ Runtime, pallet_duster }

	dust_account{
		let caller: AccountId = account("caller", 0, SEED);
		let to_dust_account: AccountId = account("dust", 0, SEED);
		let dest_account = TreasuryAccount::get();

		let asset_id = register_asset(b"TST".to_vec(), 100u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let min_deposit = AssetRegistry::get(&asset_id);

		update_balance(asset_id, &dest_account, min_deposit);

		let dust_amount = min_deposit;

		update_balance(asset_id, &to_dust_account, min_deposit);

		update_asset(asset_id, b"TST".to_vec(), 110u128).map_err(|_| BenchmarkError::Stop("Failed to update asset"))?;
		assert_eq!(Tokens::free_balance(asset_id, &to_dust_account), dust_amount);

		let current_balance = Tokens::free_balance(asset_id, &dest_account);

	}: { pallet_duster::Pallet::<Runtime>::dust_account(RawOrigin::Signed(caller.clone()).into(), to_dust_account.clone(), asset_id)? }
	verify {
		assert_eq!(Tokens::free_balance(asset_id, &to_dust_account), 0u128);
		assert_eq!(Tokens::free_balance(asset_id, &dest_account), current_balance + dust_amount);
	}

	whitelist_account{
		let nondustable_account: AccountId = account("dust", 0, SEED);
	}: { pallet_duster::Pallet::<Runtime>::whitelist_account(RawOrigin::Root.into(), nondustable_account.clone())? }
	verify {
		assert!(pallet_duster::Pallet::<Runtime>::whitelisted(&nondustable_account).is_some());
	}

	remove_from_whitelist{
		let nondustable_account: AccountId = account("dust", 0, SEED);
		pallet_duster::Pallet::<Runtime>::whitelist_account(RawOrigin::Root.into(), nondustable_account.clone())?;

	}: { pallet_duster::Pallet::<Runtime>::remove_from_whitelist(RawOrigin::Root.into(), nondustable_account.clone())? }
	verify {
		assert!(pallet_duster::Pallet::<Runtime>::whitelisted(&nondustable_account).is_none());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use orml_benchmarking::impl_benchmark_test_suite;
	use sp_runtime::BuildStorage;

	fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::<crate::Runtime>::default()
			.build_storage()
			.unwrap()
			.into()
	}

	impl_benchmark_test_suite!(new_test_ext(),);
}
