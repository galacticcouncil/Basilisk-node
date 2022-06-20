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

use super::{register_asset, update_balance, AccountId, AssetId, Balance};
use crate::{NativeAssetId, Runtime, Tokens, LBP};

use pallet_lbp::{LBPWeight, WeightCurveType};

use frame_benchmarking::{account, BenchmarkError};
use frame_support::ensure;
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;

const SEED: u32 = 1;

const ASSET_A_AMOUNT: Balance = 1_000_000_000;
const ASSET_B_AMOUNT: Balance = 2_000_000_000;
const INITIAL_WEIGHT: LBPWeight = 20_000_000;
const FINAL_WEIGHT: LBPWeight = 90_000_000;

const ENDOWMENT: Balance = 1_000_000_000_000_000;

const DEFAULT_FEE: (u32, u32) = (2, 1_000);

fn create_funded_account(name: &'static str, index: u32, assets: &[AssetId]) -> AccountId {
	let account_id: AccountId = account(name, index, SEED);

	for asset_id in assets {
		update_balance(*asset_id, &account_id, ENDOWMENT);
	}

	account_id
}

runtime_benchmarks! {
	{ Runtime, pallet_lbp }

	create_pool {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[NativeAssetId::get(), asset_a, asset_b]);

		let pool_id = LBP::pair_account_from_assets(asset_a, asset_b);

	}: _(RawOrigin::Root, caller.clone(), asset_a, ASSET_A_AMOUNT, asset_b, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, DEFAULT_FEE, caller, 0)
	verify {
		assert!(pallet_lbp::PoolData::<Runtime>::contains_key(&pool_id));
	}

	update_pool_data {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[NativeAssetId::get(), asset_a, asset_b]);
		let fee_collector = create_funded_account("fee_collector", 0, &[NativeAssetId::get(), asset_a, asset_b]);

		let pool_id = LBP::pair_account_from_assets(asset_a, asset_b);
		let new_start = Some(50_u32);
		let new_end = Some(100_u32);
		let new_initial_weight = 45_250_600;
		let new_final_weight = 55_250_600;
		let fee = (5, 1_000);

		LBP::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, ASSET_A_AMOUNT, asset_b, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, fee, caller.clone(), 0)?;
		ensure!(pallet_lbp::PoolData::<Runtime>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller.clone()), pool_id.clone(), Some(caller.clone()), new_start, new_end, Some(new_initial_weight), Some(new_final_weight), Some(DEFAULT_FEE), Some(fee_collector), Some(1))
	verify {
		let pool_data = LBP::pool_data(pool_id).unwrap();
		assert_eq!(pool_data.start, new_start);
		assert_eq!(pool_data.end, new_end);
		assert_eq!(pool_data.initial_weight, new_initial_weight);
		assert_eq!(pool_data.final_weight, new_final_weight);
	}

	add_liquidity {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[NativeAssetId::get(), asset_a, asset_b]);

		let pool_id = LBP::pair_account_from_assets(asset_a, asset_b);

		LBP::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, ASSET_A_AMOUNT, asset_b, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, DEFAULT_FEE, caller.clone(), 0)?;
		ensure!(pallet_lbp::PoolData::<Runtime>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller), (asset_a, 1_000_000_000_u128), (asset_b, 2_000_000_000_u128))
	verify {
		assert_eq!(Tokens::free_balance(asset_a, &pool_id), 2_000_000_000_u128);
		assert_eq!(Tokens::free_balance(asset_b, &pool_id), 4_000_000_000_u128);
	}

	remove_liquidity {
		let asset_a = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_b = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[NativeAssetId::get(), asset_a, asset_b]);

		let pool_id = LBP::pair_account_from_assets(asset_a, asset_b);

		LBP::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, ASSET_A_AMOUNT, asset_b, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, DEFAULT_FEE, caller.clone(), 0)?;
		ensure!(pallet_lbp::PoolData::<Runtime>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller.clone()), pool_id.clone())
	verify {
		assert!(!pallet_lbp::PoolData::<Runtime>::contains_key(&pool_id));
		assert_eq!(Tokens::free_balance(asset_a, &caller), 1_000_000_000_000_000);
		assert_eq!(Tokens::free_balance(asset_b, &caller), 1_000_000_000_000_000);
	}

	sell {
		let asset_in = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_out = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[NativeAssetId::get(), asset_in, asset_out]);
		let fee_collector = create_funded_account("fee_collector", 0, &[NativeAssetId::get(), asset_in, asset_out]);

		let pool_id = LBP::pair_account_from_assets(asset_in, asset_out);

		let amount : Balance = 100_000_000;
		let max_limit: Balance = 10_000_000;

		LBP::create_pool(RawOrigin::Root.into(), caller.clone(), asset_in, ASSET_A_AMOUNT, asset_out, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, DEFAULT_FEE, fee_collector.clone(), 0)?;
		ensure!(pallet_lbp::PoolData::<Runtime>::contains_key(&pool_id), "Pool does not exist.");

		let start = 1u32;
		let end = 11u32;

		LBP::update_pool_data(RawOrigin::Signed(caller.clone()).into(), pool_id, None, Some(start), Some(end), None, None, None, None, None)?;

	}: _(RawOrigin::Signed(caller.clone()), asset_in, asset_out, amount, max_limit)
	verify{
		assert_eq!(Tokens::free_balance(asset_in, &caller), 999_998_900_000_000);
		assert_eq!(Tokens::free_balance(asset_out, &caller), 999_998_047_091_820);
		assert_eq!(Tokens::free_balance(asset_in, &fee_collector), 1_000_000_000_200_000);
	}

	buy {
		let asset_in = register_asset(b"ASSETA".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_out = register_asset(b"ASSETB".to_vec(), 1_000_000).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller = create_funded_account("caller", 0, &[NativeAssetId::get(), asset_in, asset_out]);
		let fee_collector = create_funded_account("fee_collector", 0, &[NativeAssetId::get(), asset_in, asset_out]);

		let pool_id = LBP::pair_account_from_assets(asset_in, asset_out);
		let amount : Balance = 100_000_000;
		let max_limit: Balance = 1_000_000_000;

		LBP::create_pool(RawOrigin::Root.into(), caller.clone(), asset_in, ASSET_A_AMOUNT, asset_out, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, DEFAULT_FEE, fee_collector.clone(), 0)?;
		ensure!(pallet_lbp::PoolData::<Runtime>::contains_key(&pool_id), "Pool does not exist.");

		let start = 1u32;
		let end = 11u32;

		LBP::update_pool_data(RawOrigin::Signed(caller.clone()).into(), pool_id, None, Some(start), Some(end), None, None, None, None, None)?;

	}: _(RawOrigin::Signed(caller.clone()), asset_out, asset_in, amount, max_limit)
	verify{
		assert_eq!(Tokens::free_balance(asset_out, &caller), 999_998_100_000_000);
		assert_eq!(Tokens::free_balance(asset_in, &caller), 999_998_772_262_327);
		assert_eq!(Tokens::free_balance(asset_in, &fee_collector), 1_000_000_000_455_474);
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
