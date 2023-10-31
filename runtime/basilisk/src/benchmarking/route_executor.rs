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
#![allow(clippy::result_large_err)]

use crate::{AccountId, AssetId, Balance, Currencies, Runtime};

use super::*;

use frame_benchmarking::account;
use frame_benchmarking::BenchmarkError;
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;

use orml_traits::MultiCurrency;

type RouteExecutor<T> = pallet_route_executor::Pallet<T>;

use codec::alloc::string::ToString;
use hydradx_traits::router::PoolType;
use pallet_route_executor::Trade;
use sp_std::vec;

const SEED: u32 = 1;
pub const UNITS: Balance = 100_000_000_000;
const MAX_NUMBER_OF_TRADES: u32 = 5;

pub fn register_asset_with_name(name_as_bye_string: &[u8]) -> Result<AssetId, BenchmarkError> {
	register_asset(name_as_bye_string.to_vec(), 0u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))
}

pub fn create_account(name: &'static str) -> AccountId {
	account(name, 0, SEED)
}

pub fn generate_trades(number_of_trades: u32) -> Result<(AssetId, AssetId, Vec<Trade<AssetId>>), BenchmarkError> {
	let pool_maker: AccountId = create_account("pool_maker");

	let balance = 2000 * UNITS;
	let main_asset_in = register_asset_with_name(b"TST")?;
	let main_asset_out = register_asset_with_name(b"TST2")?;
	update_balance(main_asset_in, &pool_maker, balance);
	update_balance(main_asset_out, &pool_maker, balance);

	let number_of_intermediate_assets = number_of_trades - 1;

	//Create intermediate assets
	let mut intermediate_assets: Vec<AssetId> = vec![];
	for n in 0..number_of_intermediate_assets {
		let intermediate_asset = register_asset_with_name(n.to_string().as_bytes())?;
		update_balance(intermediate_asset, &pool_maker, balance);
		intermediate_assets.push(intermediate_asset);
	}

	//Create pools and generate trades for intermediate assets
	let mut trades: Vec<Trade<AssetId>> = vec![];
	let mut asset_in = main_asset_in;
	for _ in 0..number_of_intermediate_assets {
		let asset_out = intermediate_assets.pop().unwrap();
		create_pool(pool_maker.clone(), asset_in, 1_000 * UNITS, asset_out, 500 * UNITS);
		let trade = Trade {
			pool: PoolType::XYK,
			asset_in,
			asset_out,
		};
		trades.push(trade);
		asset_in = asset_out;
	}

	//Create pool and trade for the last trade
	create_pool(pool_maker, asset_in, 1_000 * UNITS, main_asset_out, 500 * UNITS);
	let last_trade = Trade {
		pool: PoolType::XYK,
		asset_in,
		asset_out: main_asset_out,
	};
	trades.push(last_trade);

	Ok((main_asset_in, main_asset_out, trades))
}

runtime_benchmarks! {
	{ Runtime, pallet_route_executor}

	sell {
		let n in 1..MAX_NUMBER_OF_TRADES;
		let (asset_in, asset_out, trades) = generate_trades(n).unwrap();

		let caller: AccountId = create_account("caller");

		let amount_to_sell = 10 * UNITS;
		let caller_asset_in_balance = 2000 * UNITS;
		let caller_asset_out_balance = 2000 * UNITS;

		update_balance(asset_in, &caller, caller_asset_in_balance);
	}: {
		RouteExecutor::<Runtime>::sell(RawOrigin::Signed(caller.clone()).into(), asset_in, asset_out, amount_to_sell, 0u128, trades)?
	}
	verify{
		assert_eq!(<Currencies as MultiCurrency<_>>::total_balance(asset_in, &caller), caller_asset_in_balance -  amount_to_sell);
		assert!(<Currencies as MultiCurrency<_>>::total_balance(asset_out, &caller) > 0);
	}

	buy {
		let n in 1..MAX_NUMBER_OF_TRADES;
		let (asset_in, asset_out, trades) = generate_trades(n).unwrap();

		let caller: AccountId = create_account("caller");

		let amount_to_buy = UNITS;
		let caller_asset_in_balance = 2000 * UNITS;

		update_balance(asset_in, &caller, caller_asset_in_balance);
	}: {
		RouteExecutor::<Runtime>::buy(RawOrigin::Signed(caller.clone()).into(), asset_in, asset_out, amount_to_buy, 10000u128 * UNITS, trades)?
	}
	verify{
		assert!(<Currencies as MultiCurrency<_>>::total_balance(asset_in, &caller) < caller_asset_in_balance);
		assert_eq!(<Currencies as MultiCurrency<_>>::total_balance(asset_out, &caller), amount_to_buy);
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
