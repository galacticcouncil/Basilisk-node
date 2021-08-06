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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::benchmarks;
use frame_support::traits::OnInitialize;

use crate::Pallet as PriceOracle;

pub const ASSET_PAIR_A: AssetPair = AssetPair {
	asset_in: 1_000,
	asset_out: 2_000,
};

pub const PRICE_ENTRY_1: PriceEntry = PriceEntry {
	price: Price::from_inner(2000000000000000000),
	amount: 1_000,
	liq_amount: 2_000,
};

pub const NUM_OF_ITERS: u32 = 100;

benchmarks! {
	on_initialize_no_entry {
		let t: u32 = 5;
		PriceOracle::<T>::on_create_pool(ASSET_PAIR_A);

		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(t.into()); }
	verify {
		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});
	}

	on_initialize_one_entry {
		let t: u32 = 5;
		PriceOracle::<T>::on_create_pool(ASSET_PAIR_A);
		PriceOracle::<T>::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);

		let mut vec = Vec::new();
		vec.push(PRICE_ENTRY_1);
		assert_eq!(<PriceBuffer<T>>::try_get(ASSET_PAIR_A.name()), Ok(vec));

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(t.into()); }
	verify {
		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: 1_000});
	}

	on_initialize_multiple_entries_one_token {
		let t in 1 .. NUM_OF_ITERS;
		PriceOracle::<T>::on_create_pool(ASSET_PAIR_A);
		let mut vec = Vec::new();

		for i in 0 .. t {
			PriceOracle::<T>::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);
			vec.push(PRICE_ENTRY_1);
		}

		// PriceOracle::<T>::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);// ??????????????
		// vec.push(PRICE_ENTRY_1);

		assert_eq!(PriceBuffer::<T>::try_get(ASSET_PAIR_A.name()), Ok(vec));

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(t.into()); }
	verify {
		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: t as u128 * 1_000});
	}

	on_initialize_one_entry_multiple_tokens {
		let t in 1 .. NUM_OF_ITERS;
		let mut vec = Vec::new();
		let mut asset_pair = AssetPair::default();

		for i in 0 .. t {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			PriceOracle::<T>::on_create_pool(asset_pair);
			PriceOracle::<T>::on_trade(asset_pair, PRICE_ENTRY_1);
			vec.push(PRICE_ENTRY_1);
		}

		// assert_eq!(PriceBuffer::<T>::try_get(asset_pair.name()), Ok(vec));

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(t.into()); }
	verify {
		assert_eq!(PriceBuffer::<T>::contains_key(asset_pair.name()), false);
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: 1_000});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_on_initialize_no_entry::<Test>());
			assert_ok!(test_benchmark_on_initialize_one_entry::<Test>());
			assert_ok!(test_benchmark_on_initialize_multiple_entries_one_token::<Test>());
			assert_ok!(test_benchmark_on_initialize_one_entry_multiple_tokens::<Test>());
		});
	}
}
