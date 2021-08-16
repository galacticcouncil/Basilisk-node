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

use super::*;
pub use crate::mock::{
	Event as TestEvent, ExtBuilder, Origin, PriceOracle, System, Test, ASSET_PAIR_A, ASSET_PAIR_B, PRICE_ENTRY_1,
	PRICE_ENTRY_2,
};
use frame_support::{assert_storage_noop, traits::OnInitialize};

pub fn new_test_ext() -> sp_io::TestExternalities {
	let ext = ExtBuilder.build();
	ext
}

#[test]
fn add_new_asset_pair_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(PriceOracle::num_of_assets(), 0);
		assert_eq!(
			<PriceDataTen<Test>>::get().contains(&(ASSET_PAIR_A.name(), BucketQueue::default())),
			false
		);
		PriceOracle::on_create_pool(ASSET_PAIR_A);

		assert_eq!(PriceOracle::num_of_assets(), 1);
		assert_eq!(
			<PriceDataTen<Test>>::get().contains(&(ASSET_PAIR_A.name(), BucketQueue::default())),
			true
		);
	});
}

#[test]
fn add_existing_asset_pair_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			<PriceDataTen<Test>>::get().contains(&(ASSET_PAIR_A.name(), BucketQueue::default())),
			false
		);
		PriceOracle::on_create_pool(ASSET_PAIR_A);
		assert_storage_noop!(PriceOracle::on_create_pool(ASSET_PAIR_A));
	});
}

#[test]
fn on_trade_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(<PriceDataAccumulator<Test>>::try_get(ASSET_PAIR_A.name()), Err(()));
		PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);
		PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_2);
		let price_entry = PRICE_ENTRY_1.calculate_new_price_entry(&PRICE_ENTRY_2);
		assert_eq!(<PriceDataAccumulator<Test>>::try_get(ASSET_PAIR_A.name()).ok(), price_entry);
	});
}

#[test]
fn on_trade_handler_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(<PriceDataAccumulator<Test>>::try_get(ASSET_PAIR_A.name()), Err(()));
		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: 1_000,
			amount_out: 500,
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};

		PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000);
		assert_eq!(<PriceDataAccumulator<Test>>::try_get(ASSET_PAIR_A.name()), Ok(PRICE_ENTRY_1));
	});
}

#[test]
fn price_normalization_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(<PriceDataAccumulator<Test>>::try_get(ASSET_PAIR_A.name()), Err(()));

		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: Balance::MAX,
			amount_out: 1,
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};
		assert_storage_noop!(PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000));

		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: 1,
			amount_out: Balance::MAX,
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};
		assert_storage_noop!(PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000));

		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: Balance::zero(),
			amount_out: 1_000,
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};
		assert_storage_noop!(PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000));

		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: 1_000,
			amount_out: Balance::zero(),
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};
		assert_storage_noop!(PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000));

		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: 340282366920938463463,
			amount_out: 1,
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};
		PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000);

		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: 1,
			amount_out: 340282366920938463463,
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};
		assert_storage_noop!(PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000));

		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: 2_000_000,
			amount_out: 1_000,
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};
		PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000);

		let amm_transfer = AMMTransfer {
			origin: 1,
			assets: ASSET_PAIR_A,
			amount: 1_000,
			amount_out: 2_000_000,
			discount: false,
			discount_amount: 0,
			fee: (1, 0),
		};
		PriceOracleHandler::<Test>::on_trade(&amm_transfer, 2_000);

		let price_entry = PriceDataAccumulator::<Test>::get(ASSET_PAIR_A.name());
		let first_entry = PriceEntry {
			price: Price::from(340282366920938463463),
			trade_amount: 340282366920938463463,
			liquidity_amount: 2_000,
		};

		let second_entry = PriceEntry {
			price: Price::from(2_000),
			trade_amount: 2_000_000,
			liquidity_amount: 2_000,
		};

		let third_entry = PriceEntry {
			price: Price::from_float(0.0005),
			trade_amount: 1_000,
			liquidity_amount: 2_000,
		};

		let result = PriceEntry::default().calculate_new_price_entry(&first_entry).unwrap().calculate_new_price_entry(&second_entry).unwrap().calculate_new_price_entry(&third_entry).unwrap();
		assert_eq!(price_entry, result);
	});
}

#[test]
fn update_data_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(3);

		PriceOracle::on_create_pool(ASSET_PAIR_B);
		PriceOracle::on_create_pool(ASSET_PAIR_A);

		PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);
		PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_2);
		PriceOracle::on_trade(ASSET_PAIR_B, PRICE_ENTRY_1);

		PriceOracle::update_data();

		let data_ten_a = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_A.name())
			.unwrap()
			.1;
		let data_ten_b = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_B.name())
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
fn update_data_with_incorrect_input_should_not_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(3);

		PriceOracle::on_create_pool(ASSET_PAIR_A);

		PriceOracle::on_trade(ASSET_PAIR_A, PriceEntry {
			price: Price::from(1),
			trade_amount: Zero::zero(),
			liquidity_amount: Zero::zero(),
		});

		PriceOracle::update_data();

		let data_ten = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_A.name())
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
fn update_empty_data_should_work() {
	new_test_ext().execute_with(|| {

		PriceOracle::on_create_pool(ASSET_PAIR_A);

        for i in 0..1002 {
			System::set_block_number(i);
			PriceOracle::update_data();
		}

		let data_ten = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_A.name())
			.unwrap()
			.1;
		assert_eq!(
			data_ten.get_last(),
			PriceInfo {
				avg_price: Zero::zero(),
				volume: Zero::zero()
			}
		);

		let data_hundred = PriceOracle::price_data_hundred(ASSET_PAIR_A.name());
		assert_eq!(
			data_hundred.get_last(),
			PriceInfo {
				avg_price: Zero::zero(),
				volume: Zero::zero()
			}
		);

		let data_thousand = PriceOracle::price_data_thousand(ASSET_PAIR_A.name());
		assert_eq!(
			data_thousand.get_last(),
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
		PriceOracle::on_create_pool(ASSET_PAIR_A);

		for i in 0..210 {
			System::set_block_number(i);
			PriceOracle::on_initialize(System::block_number());

			PriceOracle::on_trade(
				ASSET_PAIR_A,
				PriceEntry {
					price: Price::from((i + 1) as u128),
					trade_amount: (i * 1_000).into(),
					liquidity_amount: 1u128,
				},
			);

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


#[test]
fn stable_price_should_work() {
	new_test_ext().execute_with(|| {
		let num_of_iters = BucketQueue::BUCKET_SIZE.pow(3);
		PriceOracle::on_create_pool(ASSET_PAIR_A);

		for i in num_of_iters - 2 .. 2 * num_of_iters + 2{
			System::set_block_number(i.into());
			PriceOracle::on_trade(ASSET_PAIR_A, PRICE_ENTRY_1);
			PriceOracle::on_initialize(i.into());
		}

		let data_ten = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_A.name())
			.unwrap()
			.1;
		let data_hundred = PriceOracle::price_data_hundred(ASSET_PAIR_A.name());
		let data_thousand = PriceOracle::price_data_thousand(ASSET_PAIR_A.name());

		assert_eq!(
			data_ten.get_last(),
			PriceInfo {
				avg_price: 2.into(),
				volume: 1_000
			}
		);
		assert_eq!(
			data_hundred.get_last(),
			PriceInfo {
				avg_price: 2.into(),
				volume: 1_000
			}
		);
		assert_eq!(
			data_thousand.get_last(),
			PriceInfo {
				avg_price: 2.into(),
				volume: 1_000
			}
		);

		for i in num_of_iters .. 2 * num_of_iters {
			System::set_block_number(i.into());
			PriceOracle::on_initialize(i.into());
		}

		let data_ten = PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == ASSET_PAIR_A.name())
			.unwrap()
			.1;
		let data_hundred = PriceOracle::price_data_hundred(ASSET_PAIR_A.name());
		let data_thousand = PriceOracle::price_data_thousand(ASSET_PAIR_A.name());

		assert_eq!(
			data_ten.get_last(),
			PriceInfo {
				avg_price: 2.into(),
				volume: 1_000
			}
		);
		assert_eq!(
			data_hundred.get_last(),
			PriceInfo {
				avg_price: 2.into(),
				volume: 1_000
			}
		);
		assert_eq!(
			data_thousand.get_last(),
			PriceInfo {
				avg_price: 2.into(),
				volume: 1_000
			}
		);
	});
}