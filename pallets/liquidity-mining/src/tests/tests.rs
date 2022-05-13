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
use pallet_liquidity_mining::LoyaltyCurve;
use test_ext::*;

#[test]
fn validate_create_farm_data_should_work() {
	assert_ok!(LiquidityMining::validate_create_farm_data(
		1_000_000,
		100,
		1,
		Permill::from_percent(50)
	));

	assert_ok!(LiquidityMining::validate_create_farm_data(
		9_999_000_000_000,
		2_000_000,
		500,
		Permill::from_percent(100)
	));

	assert_ok!(LiquidityMining::validate_create_farm_data(
		10_000_000,
		101,
		16_986_741,
		Permill::from_perthousand(1)
	));
}

#[test]
fn validate_create_farm_data_should_not_work() {
	assert_err!(
		LiquidityMining::validate_create_farm_data(999_999, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(9, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(0, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 99, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 0, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 87, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 0, Permill::from_percent(50)),
		Error::<Test>::InvalidBlocksPerPeriod
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 10, Permill::from_percent(0)),
		Error::<Test>::InvalidYieldPerPeriod
	);
}
#[test]
fn get_period_number_should_work() {
	let block_num: BlockNumber = 1_u64;
	let blocks_per_period = 1;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);

	let block_num: BlockNumber = 1_000_u64;
	let blocks_per_period = 1;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1_000
	);

	let block_num: BlockNumber = 23_u64;
	let blocks_per_period = 15;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);

	let block_num: BlockNumber = 843_712_398_u64;
	let blocks_per_period = 13_412_341;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		62
	);

	let block_num: BlockNumber = 843_u64;
	let blocks_per_period = 2_000;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		0
	);

	let block_num: BlockNumber = 10_u64;
	let blocks_per_period = 10;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);
}

#[test]
fn get_period_number_should_not_work() {
	let block_num: BlockNumber = 10_u64;
	assert_err!(
		LiquidityMining::get_period_number(block_num, 0),
		Error::<Test>::Overflow
	);
}

#[test]
fn get_next_pool_id_should_work() {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 1);
		assert_eq!(LiquidityMining::pool_id(), 1);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 2);
		assert_eq!(LiquidityMining::pool_id(), 2);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 3);
		assert_eq!(LiquidityMining::pool_id(), 3);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 4);
		assert_eq!(LiquidityMining::pool_id(), 4);
	});
}

#[test]
fn pool_account_id_should_work() {
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::max_value()];

	for id in ids {
		assert_ok!(LiquidityMining::pool_account_id(id));
	}
}

#[test]
fn pool_account_id_should_not_work() {
	let ids: Vec<PoolId> = vec![0];

	for id in ids {
		assert_err!(LiquidityMining::pool_account_id(id), Error::<Test>::InvalidPoolId);
	}
}

#[test]
fn validate_pool_id_should_work() {
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::max_value()];

	for id in ids {
		assert_ok!(LiquidityMining::validate_pool_id(id));
	}
}

#[test]
fn validate_pool_id_should_not_work() {
	assert_eq!(
		LiquidityMining::validate_pool_id(0).unwrap_err(),
		Error::<Test>::InvalidPoolId
	);
}

#[test]
fn get_next_nft_id_should_work() {
	new_test_ext().execute_with(|| {
		//(pool_id, result)
		let test_data = vec![
			(1, 4_294_967_297),
			(6_886, 8_589_941_478),
			(87_321, 12_884_989_209),
			(56, 17_179_869_240),
			(789, 21_474_837_269),
			(248, 25_769_804_024),
			(1_000_000_200, 31_064_771_272),
			(u32::max_value(), 38_654_705_663),
		];

		for (pool_id, expected_nft_id) in test_data {
			assert_eq!(LiquidityMining::get_next_nft_id(pool_id).unwrap(), expected_nft_id);
		}

		//This is last allowed sequencer number - 1, test with max pool id
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0])
				.checked_sub(1_u128)
				.unwrap();

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_334
		);

		assert_eq!(
			LiquidityMining::get_next_nft_id(u32::max_value()).unwrap(),
			u128::max_value()
		);

		//This is last allowed sequencer number - 1, test with min. pool id
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0])
				.checked_sub(1_u128)
				.unwrap();

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_334
		);

		assert_eq!(
			LiquidityMining::get_next_nft_id(1).unwrap(),
			340_282_366_920_938_463_463_374_607_427_473_244_161
		);
	});
}

#[test]
fn get_next_nft_id_should_not_work() {
	new_test_ext().execute_with(|| {
		//This is last allowed sequencer number, next should throw error
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0]);

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_335
		);

		assert_noop!(
			LiquidityMining::get_next_nft_id(u32::max_value()),
			Error::<Test>::NftIdOverflow
		);

		assert_noop!(LiquidityMining::get_next_nft_id(1), Error::<Test>::NftIdOverflow);
	});
}

#[test]
fn get_pool_id_from_nft_id_should_work() {
	new_test_ext().execute_with(|| {
		//(nft_id, liq. pool id)
		let test_data = vec![
			(4_294_967_297, 1),
			(8_589_941_478, 6_886),
			(12_884_989_209, 87_321),
			(17_179_869_240, 56),
			(21_474_837_269, 789),
			(25_769_804_024, 248),
			(31_064_771_272, 1_000_000_200),
			(38_654_705_663, u32::max_value()),
			(u128::max_value(), u32::max_value()),
			(340_282_366_920_938_463_463_374_607_427_473_244_161, 1),
			(340_282_366_920_938_463_463_374_607_427_473_244_161, 1),
		];

		for (nft_id, expected_pool_id) in test_data {
			assert_eq!(
				LiquidityMining::get_pool_id_from_nft_id(nft_id).unwrap(),
				expected_pool_id
			);
		}
	});
}

#[test]
fn get_pool_id_from_nft_id_should_not_work() {
	new_test_ext().execute_with(|| {
		let test_data = vec![0, 132_342_314, 4_294_967_296];

		for nft_id in test_data {
			assert_noop!(
				LiquidityMining::get_pool_id_from_nft_id(nft_id),
				Error::<Test>::InvalidNftId
			);
		}
	});
}
