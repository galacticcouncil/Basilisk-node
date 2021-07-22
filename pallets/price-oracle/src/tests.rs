// This file is part of HydraDX.

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

use super::*;
pub use crate::mock::{
	Event as TestEvent, ExtBuilder, Origin, PriceOracle, System, Test, ASSET_PAIR_A, ASSET_PAIR_B, PRICE_ENTRY_1,
	PRICE_ENTRY_2,
};
use frame_support::{assert_noop, assert_ok, traits::OnInitialize};

pub fn new_test_ext() -> sp_io::TestExternalities {
	let ext = ExtBuilder.build();
	ext
}

fn last_events(n: usize) -> Vec<TestEvent> {
	frame_system::Pallet::<Test>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

fn expect_events(e: Vec<TestEvent>) {
	assert_eq!(last_events(e.len()), e);
}

#[test]
fn add_new_asset_pair_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(PriceOracle::asset_count(), 0);
		assert_eq!(
			<PriceDataTen<Test>>::get().contains(&(ASSET_PAIR_A, BucketQueue::default())),
			false
		);
		assert_ok!(PriceOracle::new_entry(ASSET_PAIR_A));
		assert_eq!(
			<PriceDataTen<Test>>::get().contains(&(ASSET_PAIR_A, BucketQueue::default())),
			true
		);
		assert_eq!(PriceOracle::asset_count(), 1);
	});
}

#[test]
fn on_trade_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(<PriceBuffer<Test>>::try_get(ASSET_PAIR_A), Err(()));
		assert_ok!(PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1));
		assert_ok!(PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_2));
		let mut vec = Vec::new();
		vec.push(PRICE_ENTRY_1);
		vec.push(PRICE_ENTRY_2);
		assert_eq!(<PriceBuffer<Test>>::try_get(ASSET_PAIR_A), Ok(vec));
	});
}

#[test]
fn update_data_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(3);

		assert_ok!(PriceOracle::new_entry(ASSET_PAIR_B));
		assert_ok!(PriceOracle::new_entry(ASSET_PAIR_A));

		assert_ok!(PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1));
		assert_ok!(PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_2));
		assert_ok!(PriceOracle::on_trade(ASSET_PAIR_B, PRICE_ENTRY_1));

		assert_ok!(PriceOracle::update_data());

		let data_ten_a = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_A)
			.unwrap()
			.1;
		let data_ten_b = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_B)
			.unwrap()
			.1;

		assert_eq!(
			data_ten_a.get_last(),
			PriceInfo {
				avg_price: 4.into(),
				volume: 4_000
			}
		);
		assert_eq!(
			data_ten_b.get_last(),
			PriceInfo {
				avg_price: 2.into(),
				volume: 1_000
			}
		);
	});
}

#[test]
fn update_empty_data_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(3);

		assert_ok!(PriceOracle::new_entry(ASSET_PAIR_B));
		assert_ok!(PriceOracle::new_entry(ASSET_PAIR_A));

		assert_ok!(PriceOracle::update_data());

		let data_ten = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_A)
			.unwrap()
			.1;

		assert_eq!(
			data_ten.get_last(),
			PriceInfo {
				avg_price: Zero::zero(),
				volume: Zero::zero()
			}
		);
	});
}
#[test]
fn bucket_queue_should_work() {
	let mut queue = BucketQueue::default();
	for i in 0..BucketQueue::BUCKET_SIZE {
		assert_eq!(queue[i as usize], PriceInfo::default());
	}
	assert_eq!(queue.get_last(), PriceInfo::default());

	for i in 0..BucketQueue::BUCKET_SIZE {
		let new_price = Price::from(i as u128);
		queue.update_last(PriceInfo {
			avg_price: new_price,
			volume: 0,
		});
		assert_eq!(
			queue.get_last(),
			PriceInfo {
				avg_price: new_price,
				volume: 0
			}
		);
		// for k in 0..BucketQueue::BUCKET_SIZE {
		//     print!(" {}", queue.bucket[k as usize].avg_price.to_float());
		// }
		// println!();

		for j in 0..BucketQueue::BUCKET_SIZE {
			if i < j {
				assert_eq!(queue[j as usize], PriceInfo::default());
			} else {
				assert_eq!(
					queue[j as usize],
					PriceInfo {
						avg_price: Price::from(j as u128),
						volume: 0
					}
				);
			}
		}
	}

	for i in BucketQueue::BUCKET_SIZE..BucketQueue::BUCKET_SIZE * 3 {
		let new_price = Price::from(i as u128);
		queue.update_last(PriceInfo {
			avg_price: new_price,
			volume: 0,
		});
		// for k in 0..BucketQueue::BUCKET_SIZE {
		// 	print!(" {}", queue.bucket[k as usize].avg_price.to_float());
		// }
		// println!();

		for j in 0..BucketQueue::BUCKET_SIZE {
			if (i % BucketQueue::BUCKET_SIZE) < j {
				assert_eq!(
					queue[j as usize],
					PriceInfo {
						avg_price: Price::from((10 * (i / BucketQueue::BUCKET_SIZE).saturating_sub(1) + j) as u128),
						volume: 0
					}
				);
			} else {
				assert_eq!(
					queue[j as usize],
					PriceInfo {
						avg_price: Price::from((j as u128) + 10u128 * (i / BucketQueue::BUCKET_SIZE) as u128),
						volume: 0
					}
				);
			}
		}
	}
}

#[test]
fn continuous_trades_should_work() {
	ExtBuilder.build().execute_with(|| {
		assert_ok!(PriceOracle::new_entry(ASSET_PAIR_A));

		for i in 0..210 {
			System::set_block_number(i);
			PriceOracle::on_initialize(System::block_number());

			assert_ok!(PriceOracle::on_trade(
				ASSET_PAIR_A,
				PriceEntry {
					price: Price::from((i + 1) as u128),
					amount: (i * 1_000).into(),
					liq_amount: 1u128
				}
			));

			// let ten = PriceOracle::price_data_ten().iter().find(|&x| x.0 == ASSET_PAIR_A).unwrap().1;
			// let hundred = PriceOracle::price_data_hundred(ASSET_PAIR_A);
			// let thousand = PriceOracle::price_data_thousand(ASSET_PAIR_A);
			//
			// for i in 0..BUCKET_SIZE {
			// 	print!(" {}", ten[i as usize].avg_price.to_float());
			// }
			// println!();
			//
			// for i in 0..BUCKET_SIZE {
			// 	print!(" {}", hundred[i as usize].avg_price.to_float());
			// }
			// println!();
			//
			// for i in 0..BUCKET_SIZE {
			// 	print!(" {}", thousand[i as usize].avg_price.to_float());
			// }
			// println!("\n");
		}
	})
}
