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
pub const NUM_OF_NESTED_ITERS: u32 = 100;

benchmarks! {
	on_initialize_no_entry {
		let block_num: u32 = 5;
		PriceOracle::<T>::on_create_pool(ASSET_PAIR_A);

		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(block_num.into()); }
	verify {
		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});
	}

	on_initialize_one_entry {
		let block_num: u32 = 5;
		PriceOracle::<T>::on_create_pool(ASSET_PAIR_A);
		PriceOracle::<T>::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);

		let mut vec = Vec::new();
		vec.push(PRICE_ENTRY_1);
		assert_eq!(<PriceBuffer<T>>::try_get(ASSET_PAIR_A.name()), Ok(vec));

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(block_num.into()); }
	verify {
		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: 1_000});
	}

	on_initialize_multiple_entries_one_token {
		let block_num: u32 = 5;
		let a in 1 .. NUM_OF_ITERS; // trade_num
		PriceOracle::<T>::on_create_pool(ASSET_PAIR_A);
		let mut vec = Vec::new();

		for i in 0 .. a {
			PriceOracle::<T>::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);
			vec.push(PRICE_ENTRY_1);
		}

		assert_eq!(PriceBuffer::<T>::try_get(ASSET_PAIR_A.name()), Ok(vec));

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(block_num.into()); }
	verify {
		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: a as u128 * 1_000});
	}

	on_initialize_multiple_entries_one_token_all_bucket_levels {
		let block_num: u32 = BUCKET_SIZE.pow(3);
		let a in 1 .. NUM_OF_ITERS; // trade_num
		PriceOracle::<T>::on_create_pool(ASSET_PAIR_A);
		frame_system::Pallet::<T>::set_block_number(Zero::zero());

		let mut vec = Vec::new();
		for i in 0 .. a {
			PriceOracle::<T>::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);
			vec.push(PRICE_ENTRY_1);
		}

		assert_eq!(PriceBuffer::<T>::try_get(ASSET_PAIR_A.name()), Ok(vec));
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

		for round in 1.. block_num {
			frame_system::Pallet::<T>::set_block_number((round - 1) .into());
			PriceOracle::<T>::on_initialize((round - 1).into());

			let mut vec = Vec::new();
			for i in 0 .. a {
				PriceOracle::<T>::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);
				vec.push(PRICE_ENTRY_1);
			}

			assert_eq!(PriceBuffer::<T>::try_get(ASSET_PAIR_A.name()), Ok(vec));

			let price_data = PriceOracle::<T>::price_data_ten();
			let bucket_queue = price_data.iter().find(|&x| x.0 == ASSET_PAIR_A.name()).unwrap().1;
			assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: a as u128 * 1_000});
		}

		frame_system::Pallet::<T>::set_block_number(block_num.into());

	}: { PriceOracle::<T>::on_initialize(block_num.into()); }
	verify {
		assert_eq!(PriceBuffer::<T>::contains_key(ASSET_PAIR_A.name()), false);
		let bucket_queue = PriceOracle::<T>::price_data_hundred(ASSET_PAIR_A.name());
		for i in 0 .. BucketQueue::BUCKET_SIZE {
			assert_eq!(bucket_queue[i as usize].volume, a as u128 * 1_000);
		}

		let bucket_queue = PriceOracle::<T>::price_data_thousand(ASSET_PAIR_A.name());
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: a as u128 * 1_000});
	}

	on_initialize_one_entry_multiple_tokens {
		let block_num: u32 = 5;
		let b in 1 .. NUM_OF_ITERS; // token num
		let mut vec = Vec::new();
		let asset_pair = AssetPair::default();

		for i in 0 .. b {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			PriceOracle::<T>::on_create_pool(asset_pair);
			PriceOracle::<T>::on_trade(asset_pair, PRICE_ENTRY_1);
			vec.push(PRICE_ENTRY_1);
		}

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(block_num.into()); }
	verify {
		for i in 0 .. b {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			assert_eq!(PriceBuffer::<T>::contains_key(asset_pair.name()), false);
			let price_data = PriceOracle::<T>::price_data_ten();
			let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
			assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: 1_000});
		}
	}

	on_initialize_multiple_entries_multiple_tokens {
		let block_num: u32 = 5;
		let b in 1 .. NUM_OF_ITERS;
		let a in 1 .. NUM_OF_NESTED_ITERS;
		let mut vec = Vec::new();
		let asset_pair = AssetPair::default();

		for i in 0 .. b {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			for j in 0 .. a {
				PriceOracle::<T>::on_create_pool(asset_pair);
				PriceOracle::<T>::on_trade(asset_pair, PRICE_ENTRY_1);
			}
			vec.push(PRICE_ENTRY_1);
		}

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(block_num.into()); }
	verify {
		for i in 0 .. b {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			assert_eq!(PriceBuffer::<T>::contains_key(asset_pair.name()), false);
			let price_data = PriceOracle::<T>::price_data_ten();
			let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
			assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: a as u128 * 1_000});
		}

	}

	on_initialize_uniform_distribution {
		let block_num: u32 = 5;
		let mut vec = Vec::new();
		let asset_pair = AssetPair::default();

		for i in 0 .. NUM_OF_ITERS {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			PriceOracle::<T>::on_create_pool(asset_pair);
			// 2 trades
			PriceOracle::<T>::on_trade(asset_pair, PRICE_ENTRY_1);
			PriceOracle::<T>::on_trade(asset_pair, PRICE_ENTRY_1);
			vec.push(PRICE_ENTRY_1);
		}

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(block_num.into()); }
	verify {
		for i in 0 .. NUM_OF_ITERS {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			assert_eq!(PriceBuffer::<T>::contains_key(asset_pair.name()), false);
			let price_data = PriceOracle::<T>::price_data_ten();
			let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
			assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: 2_000});
		}

	}

	// we expect this test to have similar results as the previous test
	on_initialize_nonuniform_distribution {
		let block_num: u32 = 5;
		let mut vec = Vec::new();

		let asset_pair = AssetPair {asset_in: 100, asset_out: 200};
		for i in 0 .. NUM_OF_ITERS + 1{
			PriceOracle::<T>::on_create_pool(asset_pair);
			PriceOracle::<T>::on_trade(asset_pair, PRICE_ENTRY_1);
			vec.push(PRICE_ENTRY_1);
		}

		for i in 1 .. NUM_OF_ITERS {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			PriceOracle::<T>::on_create_pool(asset_pair);
			PriceOracle::<T>::on_trade(asset_pair, PRICE_ENTRY_1);
			vec.push(PRICE_ENTRY_1);
		}

		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::zero(), volume: 0});

	}: { PriceOracle::<T>::on_initialize(block_num.into()); }
	verify {
		let asset_pair = AssetPair {asset_in: 100, asset_out: 200};
		assert_eq!(PriceBuffer::<T>::contains_key(asset_pair.name()), false);
		let price_data = PriceOracle::<T>::price_data_ten();
		let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
		assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: (NUM_OF_ITERS + 1) as u128 * 1_000});

		for i in 1 .. NUM_OF_ITERS {
			let asset_pair = AssetPair {asset_in: i * 1_000, asset_out: i * 2_000};
			assert_eq!(PriceBuffer::<T>::contains_key(asset_pair.name()), false);
			let price_data = PriceOracle::<T>::price_data_ten();
			let bucket_queue = price_data.iter().find(|&x| x.0 == asset_pair.name()).unwrap().1;
			assert_eq!(bucket_queue.get_last(), PriceInfo{ avg_price: Price::from(2), volume: 1_000});
		}

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
			assert_ok!(test_benchmark_on_initialize_multiple_entries_one_token_all_bucket_levels::<Test>());
			assert_ok!(test_benchmark_on_initialize_one_entry_multiple_tokens::<Test>());
			assert_ok!(test_benchmark_on_initialize_multiple_entries_multiple_tokens::<Test>());
			assert_ok!(test_benchmark_on_initialize_uniform_distribution::<Test>());
			assert_ok!(test_benchmark_on_initialize_nonuniform_distribution::<Test>());
		});
	}
}
