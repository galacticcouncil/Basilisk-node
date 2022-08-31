// This file is part of Basilisk-node

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

use crate::{AccountId, AssetId, Balance, Currencies, Runtime};
use primitives::Price;

use super::*;

use frame_benchmarking::account;
use frame_benchmarking::BenchmarkError;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_runtime::traits::SaturatedConversion;

use hydradx_traits::pools::SpotPriceProvider;
use orml_traits::MultiCurrencyExtended;

type RouteExecutor<T> = pallet_route_executor::Pallet<T>;

use hydradx_traits::router::{ExecutorError, PoolType};
use pallet_route_executor::types::Trade;


const SEED: u32 = 1;

pub fn update_balance(currency_id: AssetId, who: &AccountId, balance: Balance) {
	assert_ok!(<Currencies as MultiCurrencyExtended<_>>::update_balance(
		currency_id,
		who,
		balance.saturated_into()
	));
}

runtime_benchmarks! {
	{ Runtime, pallet_route_executor}

	execute_sell {
		let caller: AccountId = account("caller", 0, SEED);

		//let asset_in = 0u32;

		let asset_in = register_asset(b"TST".to_vec(), 0u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_out = register_asset(b"TST2".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		//let asset_out = 0u32;
		update_balance(asset_in, &caller, 2_000_000_000_000_000);
		//update_balance(asset_out, &caller, 2_000_000_000_000_000);

		create_pool(caller.clone(), asset_in, asset_out, 1_000_000_000_000_000, Price::from_inner(500_000_000_000_000_000));

		let routes = sp_std::vec![
            Trade {
                pool: PoolType::XYK,
                asset_in: asset_in.into(),
                asset_out: asset_out.into(),
            }
        ];

	}: {

		RouteExecutor::<Runtime>::execute_sell(RawOrigin::Signed(caller.clone()).into(), asset_in, asset_out, 1u128, 0u128, routes)?
	}
	verify{

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
