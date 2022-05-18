// This file is part of Basilisk-node.

// Copyright (C) 2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{Runtime, XYK, Tokens};
use super::{register_asset, update_balance, AccountId, AssetId, Balance, Price};

use frame_benchmarking::{account};
use frame_benchmarking::BenchmarkError;
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_std::prelude::*;
use orml_traits::MultiCurrency;

const SEED: u32 = 1;
const ENDOWMENT: Balance = 1_000_000_000_000_000;

fn create_funded_account(name: &'static str, index: u32, assets: &[AssetId]) -> AccountId {
	let account_id: AccountId = account(name, index, SEED);

	for asset_id in assets {
		update_balance(*asset_id, &account_id, ENDOWMENT);
	}

	account_id
}

runtime_benchmarks! {
	{ Runtime, pallet_xyk }

	create_pool {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[asset_a, asset_b]);

		let amount : Balance = 10 * 1_000_000_000;
		let initial_price : Price = Price::from(2);

	}: _(RawOrigin::Signed(caller.clone()), asset_a, asset_b, amount, initial_price)
	verify {
		assert_eq!(Tokens::free_balance(asset_a, &caller), 999_990_000_000_000);
	}

	add_liquidity {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[asset_a, asset_b]);
		let maker = create_funded_account("maker", 0, &[asset_a, asset_b]);

		let amount : Balance = 10 * 1_000_000_000;
		let max_limit : Balance = 10 * 1_000_000_000_000;

		XYK::create_pool(RawOrigin::Signed(maker).into(), asset_a, asset_b, 1_000_000_000, Price::from(1))?;

	}: _(RawOrigin::Signed(caller.clone()), asset_a, asset_b, amount, max_limit)
	verify {
		assert_eq!(Tokens::free_balance(asset_a, &caller), 999_990_000_000_000);
		assert_eq!(Tokens::free_balance(asset_b, &caller), 999_990_000_000_000);
	}

	remove_liquidity {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[asset_a, asset_b]);
		let maker = create_funded_account("maker", 0, &[asset_a, asset_b]);

		let amount : Balance = 1_000_000_000;

		XYK::create_pool(RawOrigin::Signed(maker).into(), 1, 2, 10_000_000_000, Price::from(2))?;
		XYK::add_liquidity(RawOrigin::Signed(caller.clone()).into(), 1, 2, 5_000_000_000, 10_000_000_000)?;

		assert_eq!(Tokens::free_balance(asset_a, &caller), 999_995_000_000_000);
		assert_eq!(Tokens::free_balance(asset_b, &caller), 999_990_000_000_000);

	}: _(RawOrigin::Signed(caller.clone()), asset_a, asset_b, amount)
	verify {
		assert_eq!(Tokens::free_balance(asset_a, &caller), 999_996_000_000_000);
		assert_eq!(Tokens::free_balance(asset_b, &caller), 999_992_000_000_000);
	}

	sell {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[asset_a, asset_b]);
		let maker = create_funded_account("maker", 0, &[asset_a, asset_b]);

		let amount : Balance = 1_000_000_000;
		let discount = false;

		let min_bought: Balance = 10 * 1_000;

		XYK::create_pool(RawOrigin::Signed(maker).into(), asset_a, asset_b, 1_000_000_000_000, Price::from(3))?;

	}: _(RawOrigin::Signed(caller.clone()), asset_a, asset_b, amount, min_bought, discount)
	verify{
		assert_eq!(Tokens::free_balance(asset_a, &caller), 999_999_000_000_000);
		assert_eq!(Tokens::free_balance(asset_b, &caller), 1_000_002_991_008_993);
	}

	buy {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[asset_a, asset_b]);
		let maker = create_funded_account("maker", 0, &[asset_a, asset_b]);

		let amount : Balance = 1_000_000_000;
		let discount = false;

		let max_sold: Balance = 6_000_000_000;

		XYK::create_pool(RawOrigin::Signed(maker).into(), asset_a, asset_b, 1_000_000_000_000, Price::from(3))?;

	}: _(RawOrigin::Signed(caller.clone()), asset_a, asset_b, amount, max_sold, discount)
	verify{
		assert_eq!(Tokens::free_balance(asset_a, &caller), 1_000_001_000_000_000);
		assert_eq!(Tokens::free_balance(asset_b, &caller), 999_996_990_990_990);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use orml_benchmarking::impl_benchmark_test_suite;

	fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<crate::Runtime>()
			.unwrap()
			.into()
	}

	impl_benchmark_test_suite!(new_test_ext(),);
}
