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
	run_to_block, Currency, Event as TestEvent, ExtBuilder, LBPPallet, Origin, System, Test, ACA, ALICE, BOB, CHARLIE,
	DOT, ETH, HDX,
};
use crate::mock::{ACA_DOT_POOL_ID, HDX_DOT_POOL_ID, INITIAL_BALANCE};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::convert::TryInto;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| run_to_block::<Test>(1));
	ext
}

use hydradx_traits::AMMTransfer;
use primitives::{
	asset::AssetPair,
	constants::chain::{MAX_IN_RATIO, MAX_OUT_RATIO},
	fee::Fee,
};

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			ACA,
			1_000_000_000,
			DOT,
			2_000_000_000,
			20_000_000,
			80_000_000,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(10),
			Some(20),
			None,
			None,
			None,
			None
		));

		let pool_data2 = Pool {
			owner: ALICE,
			start: 10u64,
			end: 20u64,
			assets: (ACA, DOT),
			initial_weight: 20_000_000,
			final_weight: 80_000_000,
			weight_curve: WeightCurveType::Linear,
			fee: Fee::default(),
			fee_collector: CHARLIE,
		};

		assert_eq!(<PoolData<Test>>::get(ACA_DOT_POOL_ID), pool_data2);

		expect_events(vec![
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::PoolUpdated(ACA_DOT_POOL_ID, pool_data2).into(),
		]);
	});
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
fn validate_pool_data_should_work() {
	new_test_ext().execute_with(|| {
		let pool_data = Pool {
			owner: ALICE,
			start: 10u64,
			end: 20u64,
			assets: (ACA, DOT),
			initial_weight: 20_000_000,
			final_weight: 90_000_000,
			weight_curve: WeightCurveType::Linear,
			fee: Fee::default(),
			fee_collector: CHARLIE,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		// null interval
		let pool_data = Pool {
			owner: ALICE,
			start: 0u64,
			end: 0u64,
			assets: (ACA, DOT),
			initial_weight: 20_000_000,
			final_weight: 90_000_000,
			weight_curve: WeightCurveType::Linear,
			fee: Fee::default(),
			fee_collector: CHARLIE,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		let pool_data = Pool {
			owner: ALICE,
			start: 10u64,
			end: 2u64,
			assets: (ACA, DOT),
			initial_weight: 20_000_000,
			final_weight: 90_000_000,
			weight_curve: WeightCurveType::Linear,
			fee: Fee::default(),
			fee_collector: CHARLIE,
		};
		assert_noop!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::InvalidBlockRange
		);

		let pool_data = Pool {
			owner: ALICE,
			start: 10u64,
			end: 11u64 + u32::MAX as u64,
			assets: (ACA, DOT),
			initial_weight: 20_000_000,
			final_weight: 90_000_000,
			weight_curve: WeightCurveType::Linear,
			fee: Fee::default(),
			fee_collector: CHARLIE,
		};
		assert_noop!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::MaxSaleDurationExceeded
		);
	});
}

#[test]
fn calculate_weights_should_work() {
	new_test_ext().execute_with(|| {
		let mut pool_data = Pool {
			owner: ALICE,
			start: 100,
			end: 200,
			assets: (ACA, DOT),
			initial_weight: 50_000_000,
			final_weight: 33_333_333,
			weight_curve: WeightCurveType::Linear,
			fee: Fee::default(),
			fee_collector: CHARLIE,
		};
		assert_eq!(LBPPallet::calculate_weights(&pool_data, 170), Ok((38333333, 61666667)));

		pool_data.initial_weight = 33_333_333;
		pool_data.final_weight = 66_666_666;
		assert_eq!(LBPPallet::calculate_weights(&pool_data, 100), Ok((33333333, 66666667)));

		pool_data.initial_weight = 33_333_333;
		pool_data.final_weight = 33_333_333;
		assert_eq!(LBPPallet::calculate_weights(&pool_data, 100), Ok((33333333, 66666667)));

		pool_data.initial_weight = 50_000_000;
		pool_data.final_weight = 33_333_333;
		assert_eq!(LBPPallet::calculate_weights(&pool_data, 200), Ok((33333333, 66666667)));

		// invalid interval
		pool_data.start = 200;
		pool_data.end = 100;
		assert_eq!(
			LBPPallet::calculate_weights(&pool_data, 200),
			Err(Error::<Test>::WeightCalculationError.into())
		);

		// invalid interval
		pool_data.start = 100;
		pool_data.end = 100;
		assert_eq!(
			LBPPallet::calculate_weights(&pool_data, 200),
			Err(Error::<Test>::WeightCalculationError.into())
		);

		// out of bound
		pool_data.start = 100;
		pool_data.end = 200;
		assert_eq!(
			LBPPallet::calculate_weights(&pool_data, 10),
			Err(Error::<Test>::WeightCalculationError.into())
		);
		assert_eq!(
			LBPPallet::calculate_weights(&pool_data, 210),
			Err(Error::<Test>::WeightCalculationError.into())
		);
	});
}

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			ACA,
			1_000_000_000,
			DOT,
			2_000_000_000,
			20_000_000u32,
			90_000_000u32,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));

		assert_eq!(Currency::free_balance(ACA, &ACA_DOT_POOL_ID), 1_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &ACA_DOT_POOL_ID), 2_000_000_000);
		assert_eq!(
			Currency::free_balance(ACA, &ALICE),
			INITIAL_BALANCE.saturating_sub(1_000_000_000)
		);
		assert_eq!(
			Currency::free_balance(DOT, &ALICE),
			INITIAL_BALANCE.saturating_sub(2_000_000_000)
		);

		let pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(pool_data.owner, ALICE);
		assert_eq!(pool_data.start, 0u64);
		assert_eq!(pool_data.end, 0u64);
		assert_eq!(pool_data.assets, (ACA, DOT));
		assert_eq!(pool_data.initial_weight, 20_000_000);
		assert_eq!(pool_data.final_weight, 90_000_000);
		assert_eq!(pool_data.weight_curve, WeightCurveType::Linear);
		assert_eq!(pool_data.fee, Fee::default());
		assert_eq!(pool_data.fee_collector, CHARLIE);

		expect_events(vec![Event::LiquidityAdded(
			ACA_DOT_POOL_ID,
			ACA,
			DOT,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn create_pool_from_basic_origin_should_not_work() {
	new_test_ext().execute_with(|| {
		// only CreatePoolOrigin is allowed to create new pools
		assert_noop!(
			LBPPallet::create_pool(
				Origin::signed(ALICE),
				ALICE,
				HDX,
				1_000_000_000,
				DOT,
				2_000_000_000,
				80_000_000u32,
				10_000_000u32,
				WeightCurveType::Linear,
				Fee::default(),
				CHARLIE,
			),
			BadOrigin
		);
	});
}

#[test]
fn create_same_pool_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			ACA,
			1_000_000_000,
			DOT,
			2_000_000_000,
			80_000_000u32,
			10_000_000u32,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));

		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				ACA,
				10_000_000_000,
				DOT,
				20_000_000_000,
				80_000_000u32,
				10_000_000u32,
				WeightCurveType::Linear,
				Fee::default(),
				CHARLIE,
			),
			Error::<Test>::PoolAlreadyExists
		);

		expect_events(vec![Event::LiquidityAdded(
			ACA_DOT_POOL_ID,
			ACA,
			DOT,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn create_pool_with_same_assets_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				ACA,
				1_000_000_000,
				ACA,
				2_000_000_000,
				80_000_000u32,
				10_000_000u32,
				WeightCurveType::Linear,
				Fee::default(),
				CHARLIE,
			),
			Error::<Test>::CannotCreatePoolWithSameAssets
		);
	});
}

#[test]
fn create_pool_with_insufficient_liquidity_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				HDX,
				0,
				DOT,
				0,
				80_000_000u32,
				10_000_000u32,
				WeightCurveType::Linear,
				Fee::default(),
				CHARLIE,
			),
			Error::<Test>::InsufficientLiquidity
		);

		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				HDX,
				0,
				DOT,
				2_000_000_000,
				80_000_000u32,
				10_000_000u32,
				WeightCurveType::Linear,
				Fee::default(),
				CHARLIE,
			),
			Error::<Test>::InsufficientLiquidity
		);

		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				HDX,
				100,
				DOT,
				100,
				80_000_000u32,
				10_000_000u32,
				WeightCurveType::Linear,
				Fee::default(),
				CHARLIE,
			),
			Error::<Test>::InsufficientLiquidity
		);
	});
}

#[test]
fn create_pool_with_insufficient_balance_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				ACA,
				2_000_000_000_000_000,
				DOT,
				2_000_000_000_000_000,
				80_000_000u32,
				10_000_000u32,
				WeightCurveType::Linear,
				Fee::default(),
				CHARLIE,
			),
			Error::<Test>::InsufficientAssetBalance
		);
	});
}

#[test]
fn update_pool_data_should_work() {
	predefined_test_ext().execute_with(|| {
		// update all parameters
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(15),
			Some(18),
			Some(10_000_000),
			Some(80_000_000),
			Some(Fee {
				numerator: 5,
				denominator: 100,
			}),
			Some(BOB),
		));

		// verify changes
		let updated_pool_data_1 = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data_1.start, 15);
		assert_eq!(updated_pool_data_1.end, 18);
		assert_eq!(updated_pool_data_1.initial_weight, 10_000_000);
		assert_eq!(updated_pool_data_1.final_weight, 80_000_000);
		assert_eq!(
			updated_pool_data_1.fee,
			Fee {
				numerator: 5,
				denominator: 100
			}
		);
		assert_eq!(updated_pool_data_1.fee_collector, BOB);

		// update only one parameter
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			None,
			Some(30),
			None,
			None,
			None,
			None,
		));

		// verify changes
		let updated_pool_data_2 = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data_2.start, 15);
		assert_eq!(updated_pool_data_2.end, 30);
		assert_eq!(updated_pool_data_2.initial_weight, 10_000_000);
		assert_eq!(updated_pool_data_2.final_weight, 80_000_000);
		assert_eq!(
			updated_pool_data_2.fee,
			Fee {
				numerator: 5,
				denominator: 100
			}
		);
		assert_eq!(updated_pool_data_2.fee_collector, BOB);

		// update only one parameter
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			None,
			None,
			Some(12_500_000),
			None,
			None,
			None,
		));

		// verify changes
		let updated_pool_data_3 = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data_3.start, 15);
		assert_eq!(updated_pool_data_3.end, 30);
		assert_eq!(updated_pool_data_3.initial_weight, 12_500_000);
		assert_eq!(updated_pool_data_3.final_weight, 80_000_000);
		assert_eq!(
			updated_pool_data_3.fee,
			Fee {
				numerator: 5,
				denominator: 100
			}
		);
		assert_eq!(updated_pool_data_3.fee_collector, BOB);

		// update only one parameter
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			None,
			None,
			None,
			None,
			None,
			Some(ALICE),
		));

		// verify changes
		let updated_pool_data_4 = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data_4.start, 15);
		assert_eq!(updated_pool_data_4.end, 30);
		assert_eq!(updated_pool_data_4.initial_weight, 12_500_000);
		assert_eq!(updated_pool_data_4.final_weight, 80_000_000);
		assert_eq!(
			updated_pool_data_4.fee,
			Fee {
				numerator: 5,
				denominator: 100
			}
		);
		assert_eq!(updated_pool_data_4.fee_collector, ALICE);

		// mix
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			None,
			Some(18),
			Some(10_000_000),
			Some(80_000_000),
			Some(Fee {
				numerator: 6,
				denominator: 1_000
			}),
			None,
		));

		// verify changes
		let updated_pool_data_5 = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data_5.start, 15);
		assert_eq!(updated_pool_data_5.end, 18);
		assert_eq!(updated_pool_data_5.initial_weight, 10_000_000);
		assert_eq!(updated_pool_data_5.final_weight, 80_000_000);
		assert_eq!(
			updated_pool_data_5.fee,
			Fee {
				numerator: 6,
				denominator: 1_000
			}
		);
		assert_eq!(updated_pool_data_5.fee_collector, ALICE);

		expect_events(vec![
			Event::PoolUpdated(ACA_DOT_POOL_ID, updated_pool_data_1).into(),
			Event::PoolUpdated(ACA_DOT_POOL_ID, updated_pool_data_2).into(),
			Event::PoolUpdated(ACA_DOT_POOL_ID, updated_pool_data_3).into(),
			Event::PoolUpdated(ACA_DOT_POOL_ID, updated_pool_data_4).into(),
			Event::PoolUpdated(ACA_DOT_POOL_ID, updated_pool_data_5).into(),
		]);
	});
}

#[test]
fn update_non_existing_pool_data_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				Some(15),
				Some(18),
				Some(10_000_000),
				Some(80_000_000),
				Some(Fee {
					numerator: 5,
					denominator: 100,
				}),
				None,
			),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn update_pool_with_invalid_data_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				// reversed interval, the end precedes the beginning
				Some(20),
				Some(10),
				Some(10_000_000),
				Some(80_000_000),
				Some(Fee {
					numerator: 5,
					denominator: 100,
				}),
				None,
			),
			Error::<Test>::InvalidBlockRange
		);

		run_to_block::<Test>(6);

		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				Some(5),
				Some(20),
				Some(10_000_000),
				Some(80_000_000),
				Some(Fee {
					numerator: 5,
					denominator: 100,
				}),
				None,
			),
			Error::<Test>::InvalidBlockRange
		);

		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				Some(0),
				Some(20),
				Some(10_000_000),
				Some(80_000_000),
				Some(Fee {
					numerator: 5,
					denominator: 100,
				}),
				None,
			),
			Error::<Test>::InvalidBlockRange
		);

		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				Some(5),
				Some(0),
				Some(10_000_000),
				Some(80_000_000),
				Some(Fee {
					numerator: 5,
					denominator: 100,
				}),
				None,
			),
			Error::<Test>::InvalidBlockRange
		);
	});
}

#[test]
fn update_pool_data_without_changes_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				None,
				None,
				None,
				None,
				None,
				None,
			),
			Error::<Test>::NothingToUpdate
		);
	});
}

#[test]
fn update_pool_data_by_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(BOB),
				ACA_DOT_POOL_ID,
				None,
				Some(15),
				Some(20),
				Some(10_000_000),
				Some(80_000_000),
				None,
				None,
			),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn update_pool_owner_by_new_owner_should_work() {
	predefined_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			Some(BOB),
			Some(15),
			Some(20),
			Some(10_000_000),
			Some(80_000_000),
			None,
			None,
		));

		let pool_data1 = LBPPallet::pool_data(ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(BOB),
			ACA_DOT_POOL_ID,
			Some(ALICE),
			Some(15),
			Some(20),
			Some(10_000_000),
			Some(80_000_000),
			None,
			None,
		));

		let pool_data2 = LBPPallet::pool_data(ACA_DOT_POOL_ID);

		expect_events(vec![
			Event::PoolUpdated(ACA_DOT_POOL_ID, pool_data1).into(),
			Event::PoolUpdated(ACA_DOT_POOL_ID, pool_data2).into(),
		]);
	});
}

#[test]
fn update_pool_data_for_running_lbp_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(15),
			Some(20),
			None,
			None,
			None,
			None,
		));

		run_to_block::<Test>(16);

		// update starting block and final weights
		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				Some(15),
				Some(30),
				Some(10_000_000),
				Some(80_000_000),
				Some(Fee {
					numerator: 5,
					denominator: 100
				}),
				Some(BOB),
			),
			Error::<Test>::SaleStarted
		);

		let pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);

		expect_events(vec![Event::PoolUpdated(ACA_DOT_POOL_ID, pool_data).into()]);
	});
}

#[test]
fn update_pool_interval_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			ACA,
			1_000_000_000,
			DOT,
			2_000_000_000,
			10_000_000,
			10_000_000,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));

		run_to_block::<Test>(15);

		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				Some(16),
				Some(0),
				None,
				None,
				None,
				None,
			),
			Error::<Test>::InvalidBlockRange
		);

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(16),
			Some(20),
			None,
			None,
			None,
			None,
		));

		// verify changes
		let updated_pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data.start, 16);
		assert_eq!(updated_pool_data.end, 20);

		expect_events(vec![
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::PoolUpdated(ACA_DOT_POOL_ID, updated_pool_data).into(),
		]);
	});
}

#[test]
fn add_liquidity_should_work() {
	predefined_test_ext().execute_with(|| {
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);
		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		let added_a = 10_000_000_000;
		let added_b = 20_000_000_000;

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(ALICE),
			(ACA, added_a),
			(DOT, added_b),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);
		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(added_a));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_add(added_b));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(added_a));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(added_b));

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);
		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(ALICE),
			(ACA, added_a),
			(DOT, 0),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);
		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(added_a));
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(added_a));
		assert_eq!(user_balance_b_after, user_balance_b_before);

		// change asset order
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);
		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(ALICE),
			(DOT, added_b),
			(ACA, added_a),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);
		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(added_a));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_add(added_b));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(added_a));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(added_b));

		expect_events(vec![
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, added_a, added_b).into(),
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, added_a, 0).into(),
			Event::LiquidityAdded(ACA_DOT_POOL_ID, DOT, ACA, added_b, added_a).into(),
		]);
	});
}

#[test]
fn add_liquidity_by_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_eq!(Currency::free_balance(ACA, &BOB), 1000000000000000);
		assert_eq!(Currency::free_balance(DOT, &BOB), 1000000000000000);

		assert_eq!(Currency::free_balance(ACA, &ACA_DOT_POOL_ID), 1_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &ACA_DOT_POOL_ID), 2_000_000_000);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(BOB), (ACA, 10_000_000_000), (DOT, 20_000_000_000),),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn add_zero_liquidity_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(ALICE), (ACA, 0), (DOT, 0),),
			Error::<Test>::CannotAddZeroLiquidity
		);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before);
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		// No new events expected
		expect_events(vec![]);
	});
}

#[test]
fn add_liquidity_with_insufficient_balance_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(ALICE), (ACA, u128::MAX), (DOT, 0),),
			Error::<Test>::InsufficientAssetBalance
		);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before);
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before);
	});
}

#[test]
fn add_liquidity_after_sale_started_should_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block::<Test>(15);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(ALICE),
			(ACA, 1_000),
			(DOT, 1_000),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(1_000));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_add(1_000));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);

		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(1_000));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(1_000));

		// sale ended at the block number 20
		run_to_block::<Test>(30);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(ALICE),
			(ACA, 1_000),
			(DOT, 1_000),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(1_000));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_add(1_000));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);

		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(1_000));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(1_000));

		expect_events(vec![
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, 1_000, 1_000).into(),
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, 1_000, 1_000).into(),
		]);
	});
}

#[test]
fn add_liquidity_to_non_existing_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(ALICE), (ACA, 1_000), (HDX, 1_000),),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn remove_liquidity_should_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block::<Test>(21);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID,));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, 0);
		assert_eq!(pool_balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(
			user_balance_a_after,
			user_balance_a_before.saturating_add(pool_balance_a_before)
		);

		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(
			user_balance_b_after,
			user_balance_b_before.saturating_add(pool_balance_b_before)
		);

		assert!(!<PoolData<Test>>::contains_key(ACA_DOT_POOL_ID));

		expect_events(vec![
			frame_system::Event::KilledAccount(ACA_DOT_POOL_ID).into(),
			Event::LiquidityRemoved(ACA_DOT_POOL_ID, ACA, DOT, pool_balance_a_before, pool_balance_b_before).into(),
		]);
	});
}

#[test]
fn remove_liquidity_from_not_started_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID,));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, 0);
		assert_eq!(pool_balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(
			user_balance_a_after,
			user_balance_a_before.saturating_add(pool_balance_a_before)
		);

		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(
			user_balance_b_after,
			user_balance_b_before.saturating_add(pool_balance_b_before)
		);

		assert!(!<PoolData<Test>>::contains_key(ACA_DOT_POOL_ID));

		expect_events(vec![
			frame_system::Event::KilledAccount(ACA_DOT_POOL_ID).into(),
			Event::LiquidityRemoved(ACA_DOT_POOL_ID, ACA, DOT, pool_balance_a_before, pool_balance_b_before).into(),
		]);

		// sale duration is not specified
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			HDX,
			1_000_000_000,
			DOT,
			2_000_000_000,
			10_000_000,
			90_000_000,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));

		let user_balance_a_before = Currency::free_balance(HDX, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(HDX, &HDX_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &HDX_DOT_POOL_ID);

		assert_ok!(LBPPallet::remove_liquidity(Origin::signed(ALICE), HDX_DOT_POOL_ID,));

		let pool_balance_a_after = Currency::free_balance(HDX, &HDX_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &HDX_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, 0);
		assert_eq!(pool_balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(HDX, &ALICE);
		assert_eq!(
			user_balance_a_after,
			user_balance_a_before.saturating_add(pool_balance_a_before)
		);

		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(
			user_balance_b_after,
			user_balance_b_before.saturating_add(pool_balance_b_before)
		);

		assert!(!<PoolData<Test>>::contains_key(HDX_DOT_POOL_ID));

		expect_events(vec![
			frame_system::Event::KilledAccount(HDX_DOT_POOL_ID).into(),
			Event::LiquidityRemoved(HDX_DOT_POOL_ID, HDX, DOT, pool_balance_a_before, pool_balance_b_before).into(),
		]);
	});
}

#[test]
fn remove_liquidity_from_non_existing_pool_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn remove_liquidity_from_not_finalized_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block::<Test>(15);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID,),
			Error::<Test>::SaleNotEnded
		);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_before, pool_balance_a_after);
		assert_eq!(pool_balance_b_before, pool_balance_b_after);
		assert_eq!(user_balance_a_before, user_balance_a_after);
		assert_eq!(user_balance_b_before, user_balance_b_after);
	});
}

#[test]
fn remove_liquidity_from_finalized_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block::<Test>(21);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID,));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, 0);
		assert_eq!(pool_balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(
			user_balance_a_after,
			user_balance_a_before.saturating_add(pool_balance_a_before)
		);

		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(
			user_balance_b_after,
			user_balance_b_before.saturating_add(pool_balance_b_before)
		);

		assert!(!<PoolData<Test>>::contains_key(ACA_DOT_POOL_ID));

		expect_events(vec![
			frame_system::Event::KilledAccount(ACA_DOT_POOL_ID).into(),
			Event::LiquidityRemoved(ACA_DOT_POOL_ID, ACA, DOT, pool_balance_a_before, pool_balance_b_before).into(),
		]);
	});
}

#[test]
fn remove_liquidity_by_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(BOB), ACA_DOT_POOL_ID),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn execute_trade_should_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = ACA_DOT_POOL_ID;

		let amount_in = 5_000_000_u128;
		let amount_out = 10_000_000_u128;
		let t_sell = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
			fee: (asset_out, 1_000),
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(asset_in, &CHARLIE), 0);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_ok!(LBPPallet::execute_trade(&t_sell));

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_998_995_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_010_000_000);
		assert_eq!(Currency::free_balance(asset_in, &CHARLIE), 0);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 1_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_005_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_989_999_000);

		let t_buy = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
			fee: (asset_in, 1_000),
		};

		assert_ok!(LBPPallet::execute_trade(&t_buy));

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_998_989_999_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_020_000_000);
		assert_eq!(Currency::free_balance(asset_in, &CHARLIE), 1_000);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 1_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_010_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_979_999_000);
	});
}

// // This test ensure storage was not modified on error
#[test]
fn execute_trade_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 5_000_000_u128;
		let amount_out = 10_000_000_000_000_000u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
			fee: (asset_in, 1_000),
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_noop!(LBPPallet::execute_trade(&t), orml_tokens::Error::<Test>::BalanceTooLow);

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);
	});
}

#[test]
fn execute_sell_should_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 8_000_000_u128;
		let amount_out = 20_000_000_u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
			fee: (asset_out, 1_000),
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_ok!(LBPPallet::execute_sell(&t));

		expect_events(vec![Event::SellExecuted(
			ALICE, asset_in, asset_out, amount_in, amount_out, asset_out, 1_000,
		)
		.into()]);

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_998_992_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_020_000_000);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 1_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_008_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_979_999_000);

		expect_events(vec![Event::SellExecuted(
			ALICE, asset_in, asset_out, 8_000_000, 20_000_000, asset_out, 1_000,
		)
		.into()]);
	});
}

// // This test ensure storage was not modified on error
#[test]
fn execute_sell_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair {
				asset_in: ACA,
				asset_out: DOT,
			},
			amount: 8_000_000_000_u128,
			amount_out: 200_000_000_000_000_u128,
			discount: false,
			discount_amount: 0_u128,
			fee: (DOT, 1_000),
		};

		assert_eq!(Currency::free_balance(ACA, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(ACA, &ACA_DOT_POOL_ID), 1_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &ACA_DOT_POOL_ID), 2_000_000_000);

		assert_noop!(LBPPallet::execute_sell(&t), orml_tokens::Error::<Test>::BalanceTooLow);

		assert_eq!(Currency::free_balance(ACA, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(ACA, &ACA_DOT_POOL_ID), 1_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &ACA_DOT_POOL_ID), 2_000_000_000);
	});
}

#[test]
fn zero_weight_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				ACA,
				1_000_000_000,
				ETH,
				2_000_000_000,
				0u32,
				20u32,
				WeightCurveType::Linear,
				Fee::default(),
				CHARLIE,
			),
			Error::<Test>::InvalidWeight
		);

		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				Some(15),
				Some(18),
				Some(0),
				Some(80),
				Some(Fee {
					numerator: 5,
					denominator: 100,
				}),
				Some(BOB),
			),
			Error::<Test>::InvalidWeight
		);
	});
}

#[test]
fn execute_buy_should_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 8_000_000_u128;
		let amount_out = 20_000_000_u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
			fee: (asset_in, 1_000),
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(asset_in, &CHARLIE), 0);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_ok!(LBPPallet::execute_buy(&t));

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_998_991_999_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_020_000_000);
		assert_eq!(Currency::free_balance(asset_in, &CHARLIE), 1_000);
		assert_eq!(Currency::free_balance(asset_out, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_008_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_980_000_000);

		expect_events(vec![Event::BuyExecuted(
			ALICE, asset_out, asset_in, 8_000_000, 20_000_000, asset_in, 1_000,
		)
		.into()]);
	});
}

// This test ensures storage was not modified on error
#[test]
fn execute_buy_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 8_000_000_000_u128;
		let amount_out = 200_000_000_000_000_u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
			fee: (asset_in, 1_000),
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(asset_in, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_noop!(LBPPallet::execute_buy(&t), orml_tokens::Error::<Test>::BalanceTooLow);

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);
		assert_eq!(Currency::free_balance(asset_in, &CHARLIE), 0);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);
	});
}

#[test]
fn sell_zero_amount_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::sell(Origin::signed(BOB), ACA, DOT, 0_u128, 200_000_u128),
			Error::<Test>::ZeroAmount
		);
	});
}

#[test]
fn buy_zero_amount_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::buy(Origin::signed(BOB), ACA, DOT, 0_u128, 200_000_u128),
			Error::<Test>::ZeroAmount
		);
	});
}

#[test]
fn sell_to_non_existing_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::sell(Origin::signed(BOB), ACA, ETH, 800_000_u128, 200_000_u128),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn buy_from_non_existing_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::buy(Origin::signed(BOB), ACA, ETH, 800_000_u128, 200_000_u128),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn exceed_max_in_ratio_should_not_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block::<Test>(11); //start sale
		assert_noop!(
			LBPPallet::sell(
				Origin::signed(BOB),
				ACA,
				DOT,
				1_000_000_000 / MAX_IN_RATIO + 1,
				200_000_u128
			),
			Error::<Test>::MaxInRatioExceeded
		);

		// 1/2 should not work
		assert_noop!(
			LBPPallet::sell(Origin::signed(BOB), ACA, DOT, 1_000_000_000 / 2, 200_000_u128),
			Error::<Test>::MaxInRatioExceeded
		);

		// max ratio should work
		assert_ok!(LBPPallet::sell(
			Origin::signed(BOB),
			ACA,
			DOT,
			1_000_000_000 / MAX_IN_RATIO,
			2_000_u128
		));
	});
}

#[test]
fn exceed_max_out_ratio_should_not_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block::<Test>(11); //start sale

		// max_ratio_out + 1 should not work
		assert_noop!(
			LBPPallet::buy(
				Origin::signed(BOB),
				ACA,
				DOT,
				1_000_000_000 / MAX_OUT_RATIO + 1,
				200_000_u128
			),
			Error::<Test>::MaxOutRatioExceeded
		);

		// 1/2 should not work
		assert_noop!(
			LBPPallet::buy(Origin::signed(BOB), ACA, DOT, 1_000_000_000 / 2, 200_000_u128),
			Error::<Test>::MaxOutRatioExceeded
		);

		// max ratio should work
		assert_ok!(LBPPallet::buy(
			Origin::signed(BOB),
			ACA,
			DOT,
			1_000_000_000 / MAX_OUT_RATIO,
			2_000_000_000_u128
		));
	});
}

#[test]
fn trade_in_non_running_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let amount = 800_000_u128;
		let limit = 200_000_u128;

		//sale not started
		run_to_block::<Test>(9);
		assert_noop!(
			LBPPallet::sell(Origin::signed(who), asset_in, asset_out, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);
		assert_noop!(
			LBPPallet::buy(Origin::signed(who), asset_in, asset_out, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);

		//sale ended
		run_to_block::<Test>(21);
		assert_noop!(
			LBPPallet::sell(Origin::signed(who), asset_in, asset_out, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);
		assert_noop!(
			LBPPallet::buy(Origin::signed(who), asset_in, asset_out, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);
	});
}

#[test]
fn exceed_trader_limit_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let amount = 800_000_u128;
		let sell_limit = 800_000_u128;
		let buy_limit = 1_000_u128;

		//start sale
		run_to_block::<Test>(11);
		assert_noop!(
			LBPPallet::sell(Origin::signed(who), asset_in, asset_out, amount, sell_limit),
			Error::<Test>::AssetBalanceLimitExceeded
		);

		assert_noop!(
			LBPPallet::buy(Origin::signed(who), asset_in, asset_out, amount, buy_limit),
			Error::<Test>::AssetBalanceLimitExceeded
		);
	});
}

#[test]
fn sell_with_insufficient_balance_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let amount = 1_000_000_u128;

		assert_ok!(Currency::withdraw(asset_in, &who, 999_999_999_900_000));
		assert_eq!(Currency::free_balance(asset_in, &who), 100_000);

		//start sale
		run_to_block::<Test>(11);
		assert_noop!(
			LBPPallet::sell(Origin::signed(who), asset_in, asset_out, amount, 800_000_u128),
			Error::<Test>::InsufficientAssetBalance
		);
	});
}

#[test]
fn buy_with_insufficient_balance_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let amount = 1_000_000_u128;

		assert_ok!(Currency::withdraw(asset_in, &who, 999_999_999_900_000));
		assert_eq!(Currency::free_balance(asset_in, &who), 100_000);

		//start sale
		run_to_block::<Test>(11);
		assert_noop!(
			LBPPallet::buy(Origin::signed(who), asset_out, asset_in, amount, 2_000_000_u128),
			Error::<Test>::InsufficientAssetBalance
		);
	});
}

#[test]
fn buy_should_work() {
	predefined_test_ext().execute_with(|| {
		let buyer = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		//start sale
		run_to_block::<Test>(11);
		assert_ok!(LBPPallet::buy(
			Origin::signed(buyer),
			asset_out,
			asset_in,
			10_000_000_u128,
			2_000_000_000_u128
		));

		assert_eq!(Currency::free_balance(asset_in, &buyer), 999_999_985_602_548);
		assert_eq!(Currency::free_balance(asset_out, &buyer), 1_000_000_010_000_000);
		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_014_368_715);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_990_000_000);

		// test buy where the amount_in is less than the amount_out
		let asset_in = HDX;
		let asset_out = DOT;
		let pool_id2 = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			HDX,
			1_000_000_000,
			DOT,
			2_000_000_000,
			80_000_000u32,
			10_000_000u32,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));

		let pool_data1 = LBPPallet::pool_data(pool_id2);

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			HDX_DOT_POOL_ID,
			None,
			Some(20),
			Some(30),
			None,
			None,
			None,
			None
		));

		let pool_data2 = LBPPallet::pool_data(pool_id2);

		//start sale
		run_to_block::<Test>(21);
		assert_ok!(LBPPallet::buy(
			Origin::signed(buyer),
			asset_out,
			asset_in,
			10_000_000_u128,
			2_000_000_000_u128
		));

		assert_eq!(Currency::free_balance(asset_in, &buyer), 999_999_998_140_617);
		assert_eq!(Currency::free_balance(asset_out, &buyer), 1_000_000_020_000_000);
		assert_eq!(Currency::free_balance(asset_in, &pool_id2), 1_001_855_672);
		assert_eq!(Currency::free_balance(asset_out, &pool_id2), 1_990_000_000);

		expect_events(vec![
			orml_tokens::Event::Endowed(ACA, CHARLIE, 28_737).into(),
			Event::BuyExecuted(buyer, DOT, ACA, 14_368_715, 10_000_000, ACA, 28_737).into(),
			Event::PoolCreated(pool_id2, pool_data1).into(),
			frame_system::Event::NewAccount(pool_id2).into(),
			orml_tokens::Event::Endowed(HDX, pool_id2, 1_000_000_000).into(),
			orml_tokens::Event::Endowed(DOT, pool_id2, 2_000_000_000).into(),
			Event::LiquidityAdded(pool_id2, HDX, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::PoolUpdated(pool_id2, pool_data2).into(),
			orml_tokens::Event::Endowed(asset_in, CHARLIE, 3711).into(),
			Event::BuyExecuted(buyer, asset_out, asset_in, 1_855_672, 10_000_000, 0, 3711).into(),
		]);
	});
}

#[test]
fn update_pool_data_after_sale_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let buyer = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		//start sale
		run_to_block::<Test>(11);
		assert_ok!(LBPPallet::buy(
			Origin::signed(buyer),
			asset_out,
			asset_in,
			10_000_000_u128,
			2_000_000_000_u128
		));

		assert_eq!(Currency::free_balance(asset_in, &buyer), 999_999_985_602_548);
		assert_eq!(Currency::free_balance(asset_out, &buyer), 1_000_000_010_000_000);
		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_014_368_715);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_990_000_000);
		assert_eq!(Currency::free_balance(asset_in, &CHARLIE), 28_737);

		run_to_block::<Test>(30);

		expect_events(vec![Event::BuyExecuted(
			buyer, DOT, ACA, 14_368_715, 10_000_000, ACA, 28_737,
		)
		.into()]);

		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				Some(50),
				Some(60),
				None,
				None,
				None,
				None
			),
			Error::<Test>::SaleStarted
		);
	});
}

#[test]
fn sell_should_work() {
	predefined_test_ext().execute_with(|| {
		let buyer = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		//start sale
		run_to_block::<Test>(11);

		assert_ok!(LBPPallet::sell(
			Origin::signed(buyer),
			asset_in,
			asset_out,
			10_000_000_u128,
			2_000_u128
		));

		assert_eq!(Currency::free_balance(asset_in, &buyer), 999_999_990_000_000);
		assert_eq!(Currency::free_balance(asset_out, &buyer), 1_000_000_006_965_956);
		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_010_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_993_020_085);

		// test buy where the amount_in is less than the amount_out
		let asset_in = HDX;
		let asset_out = DOT;
		let pool_id2 = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			HDX,
			1_000_000_000,
			DOT,
			2_000_000_000,
			80_000_000u32,
			10_000_000u32,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));

		let pool_data1 = LBPPallet::pool_data(pool_id2);

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			HDX_DOT_POOL_ID,
			None,
			Some(20),
			Some(30),
			None,
			None,
			None,
			None
		));

		let pool_data2 = LBPPallet::pool_data(pool_id2);

		//start sale
		run_to_block::<Test>(21);
		assert_ok!(LBPPallet::sell(
			Origin::signed(buyer),
			asset_out,
			asset_in,
			10_000_000_u128,
			2_000_u128
		));

		assert_eq!(Currency::free_balance(asset_in, &buyer), 1_000_000_001_839_319);
		assert_eq!(Currency::free_balance(asset_out, &buyer), 999_999_996_965_956);
		assert_eq!(Currency::free_balance(asset_in, &pool_id2), 998_156_995);
		assert_eq!(Currency::free_balance(asset_out, &pool_id2), 2_010_000_000);

		expect_events(vec![
			orml_tokens::Event::Endowed(DOT, CHARLIE, 13_959).into(),
			Event::SellExecuted(buyer, ACA, DOT, 10_000_000, 6_965_956, DOT, 13_959).into(),
			Event::PoolCreated(pool_id2, pool_data1).into(),
			frame_system::Event::NewAccount(pool_id2).into(),
			orml_tokens::Event::Endowed(asset_in, pool_id2, 1_000_000_000).into(),
			orml_tokens::Event::Endowed(asset_out, pool_id2, 2_000_000_000).into(),
			Event::LiquidityAdded(pool_id2, HDX, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::PoolUpdated(pool_id2, pool_data2).into(),
			orml_tokens::Event::Endowed(asset_in, CHARLIE, 3_686).into(),
			Event::SellExecuted(buyer, asset_out, asset_in, 10_000_000, 1_839_319, 0, 3_686).into(),
		]);
	});
}

#[test]
fn zero_fee_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			ACA,
			1_000_000_000,
			DOT,
			2_000_000_000,
			20_000_000,
			80_000_000,
			WeightCurveType::Linear,
			Fee {
				numerator: 0,
				denominator: 100,
			},
			CHARLIE,
		));

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(10),
			Some(20),
			None,
			None,
			None,
			None
		));

		//start sale
		run_to_block::<Test>(11);

		assert_ok!(LBPPallet::sell(Origin::signed(ALICE), ACA, DOT, 1_000, 1,));
	});
}

#[test]
fn invalid_fee_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				ACA,
				1_000_000_000,
				DOT,
				2_000_000_000,
				20_000_000,
				80_000_000,
				WeightCurveType::Linear,
				Fee {
					numerator: 10,
					denominator: 0,
				},
				CHARLIE,
			),
			Error::<Test>::FeeAmountInvalid
		);
	});
}

#[test]
fn amm_trait_should_work() {
	predefined_test_ext().execute_with(|| {
		let asset_pair = AssetPair {
			asset_in: ACA,
			asset_out: DOT,
		};
		let reversed_asset_pair = AssetPair {
			asset_in: DOT,
			asset_out: ACA,
		};
		let non_existing_asset_pair = AssetPair {
			asset_in: DOT,
			asset_out: HDX,
		};

		run_to_block::<Test>(11);

		assert!(LBPPallet::exists(asset_pair));
		assert!(LBPPallet::exists(reversed_asset_pair));
		assert!(!LBPPallet::exists(non_existing_asset_pair));

		assert_eq!(LBPPallet::get_pair_id(asset_pair), ACA_DOT_POOL_ID);
		assert_eq!(LBPPallet::get_pair_id(reversed_asset_pair), ACA_DOT_POOL_ID);

		assert_eq!(LBPPallet::get_pool_assets(&ACA_DOT_POOL_ID), Some(vec![ACA, DOT]));
		assert_eq!(LBPPallet::get_pool_assets(&HDX_DOT_POOL_ID), None);

		// calculate_spot_price is tested in get_spot_price_should_work
		// execute_sell and execute_buy is tested in execute_sell_should_work and execute_buy_should_work

		let who = BOB;
		let amount_in = 1_000_000;
		let sell_limit = 100_000;
		let t_sell = AMMTransfer {
			origin: who,
			assets: asset_pair,
			amount: amount_in,
			amount_out: 700_822,
			discount: false,
			discount_amount: 0_u128,
			fee: (asset_pair.asset_out, 1_404),
		};
		assert_eq!(
			LBPPallet::validate_sell(&who, asset_pair, amount_in, sell_limit, false).unwrap(),
			t_sell
		);

		let amount_out = 1_000_000;
		let buy_limit = 10_000_000;
		let t_buy = AMMTransfer {
			origin: who,
			assets: asset_pair,
			amount: 1_424_443,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
			fee: (asset_pair.asset_in, 2_848),
		};
		assert_eq!(
			LBPPallet::validate_buy(&who, asset_pair, amount_in, buy_limit, false).unwrap(),
			t_buy
		);

		assert_eq!(
			LBPPallet::get_min_trading_limit(),
			<Test as Config>::MinTradingLimit::get()
		);
		assert_eq!(
			LBPPallet::get_min_pool_liquidity(),
			<Test as Config>::MinPoolLiquidity::get()
		);
		assert_eq!(LBPPallet::get_max_in_ratio(), <Test as Config>::MaxInRatio::get());
		assert_eq!(LBPPallet::get_max_out_ratio(), <Test as Config>::MaxOutRatio::get());
	});
}

#[test]
fn get_spot_price_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			ACA,
			1_000_000_000,
			DOT,
			2_000_000_000,
			20_000_000,
			90_000_000,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(10),
			Some(20),
			None,
			None,
			None,
			None
		));

		run_to_block::<Test>(10);

		let price = hydra_dx_math::lbp::calculate_spot_price(
			1_000_000_000_u128,
			2_000_000_000_u128,
			20_u32,
			80_u32,
			1_000_000_u128,
		)
		.unwrap_or_else(|_| BalanceOf::<Test>::zero());

		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, DOT, 1_000_000_u128), price);

		// swap assets
		let price = hydra_dx_math::lbp::calculate_spot_price(
			2_000_000_000_u128,
			1_000_000_000_u128,
			80_u32,
			20_u32,
			1_000_000_u128,
		)
		.unwrap_or_else(|_| BalanceOf::<Test>::zero());

		assert_eq!(LBPPallet::get_spot_price_unchecked(DOT, ACA, 1_000_000_u128), price);

		// change weights
		run_to_block::<Test>(20);

		let price = hydra_dx_math::lbp::calculate_spot_price(
			1_000_000_000_u128,
			2_000_000_000_u128,
			90_u32,
			10_u32,
			1_000_000_u128,
		)
		.unwrap_or_else(|_| BalanceOf::<Test>::zero());

		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, DOT, 1_000_000), price);

		// pool does not exist
		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, HDX, 1_000_000), 0);

		// overflow
		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, DOT, u128::MAX), 0);

		// sale ended
		run_to_block::<Test>(21);
		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, DOT, 1_000_000), 0);
	});
}

#[test]
fn simulate_lbp_event_should_work() {
	new_test_ext().execute_with(|| {
		// setup
		let pool_owner = BOB;
		let lbp_participant = CHARLIE;

		let asset_in = DOT;
		let asset_in_pool_reserve: u128 = 1_000_000;
		let owner_initial_asset_in_balance: u128 = 1_000_000_000_000;
		let lbp_participant_initial_asset_in_balance: u128 = 1_000_000_000_000;

		let asset_in_initial_weight = 10_000_000; // 10%
		let asset_in_final_weight = 75_000_000; // 75%

		let asset_out = HDX;
		let asset_out_pool_reserve: u128 = 500_000_000;
		let owner_initial_asset_out_balance: u128 = 1_000_000_000_000;
		let lbp_participant_initial_asset_out_balance: u128 = 1_000_000_000_000;

		let sale_start: u64 = 1_000;
		let sale_end: u64 = 22_600; // in blocks; 3 days

		let mut trades = BTreeMap::new();
		let intervals: u64 = 72;

		let sale_rate = 200_000_000; // asset_out per day
		let buy_amount = sale_rate / 24;
		let sell_amount = 100_000_000 / 24;

		let skip = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
		let sells = vec![19, 20, 21, 33, 34, 35, 48, 49, 50, 62, 63, 64];
		for i in 0..=intervals {
			let block_num = sale_start + (i * ((sale_end - sale_start) / intervals));

			if skip.contains(&i) {
				continue;
			}

			let (is_buy, amount) = if sells.contains(&i) {
				(false, sell_amount)
			} else {
				(true, buy_amount)
			};

			trades.insert(block_num, (is_buy, amount));
		}

		let fee = Fee {
			numerator: 9,
			denominator: 1_000,
		};

		let fee_collector = ALICE;

		let trade_limit_factor: u128 = 1_000;

		// preparations
		let asset_pair = AssetPair { asset_in, asset_out };
		let pool_account = LBPPallet::get_pair_id(asset_pair);

		Currency::set_balance(Origin::root(), fee_collector, asset_in, 0, 0).unwrap();
		Currency::set_balance(Origin::root(), fee_collector, asset_out, 0, 0).unwrap();

		Currency::set_balance(Origin::root(), pool_owner, asset_in, 0, 0).unwrap();
		Currency::set_balance(Origin::root(), pool_owner, asset_out, 0, 0).unwrap();

		Currency::set_balance(
			Origin::root(),
			pool_owner,
			asset_in,
			owner_initial_asset_in_balance
				.checked_add(asset_in_pool_reserve)
				.unwrap(),
			0,
		)
		.unwrap();
		Currency::set_balance(
			Origin::root(),
			pool_owner,
			asset_out,
			owner_initial_asset_out_balance
				.checked_add(asset_out_pool_reserve)
				.unwrap(),
			0,
		)
		.unwrap();

		<Test as Config>::MultiCurrency::update_balance(
			asset_in,
			&lbp_participant,
			lbp_participant_initial_asset_in_balance.try_into().unwrap(),
		)
		.unwrap();
		<Test as Config>::MultiCurrency::update_balance(
			asset_out,
			&lbp_participant,
			lbp_participant_initial_asset_out_balance.try_into().unwrap(),
		)
		.unwrap();

		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			pool_owner,
			asset_in,
			asset_in_pool_reserve,
			asset_out,
			asset_out_pool_reserve,
			asset_in_initial_weight,
			asset_in_final_weight,
			WeightCurveType::Linear,
			fee,
			fee_collector,
		));

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(pool_owner),
			pool_account,
			None,
			Some(sale_start),
			Some(sale_end),
			None,
			None,
			None,
			None
		));

		run_to_block::<Test>(sale_start.checked_sub(1).unwrap());
		//frame_system::Pallet::<Test>::set_block_number(sale_start + 1);

		// start LBP
		for block_num in sale_start..=sale_end {
			run_to_block::<Test>(block_num);

			if let Some((is_buy, amount)) = trades.get(&block_num) {
				if *is_buy {
					assert_ok!(LBPPallet::buy(
						Origin::signed(lbp_participant),
						asset_out,
						asset_in,
						*amount,
						amount.saturating_mul(trade_limit_factor)
					));
				} else {
					assert_ok!(LBPPallet::sell(
						Origin::signed(lbp_participant),
						asset_out,
						asset_in,
						*amount,
						amount.checked_div(trade_limit_factor).unwrap()
					));
				}
			}
		}

		// end LBP and consolidate results
		run_to_block::<Test>(sale_end.checked_add(1).unwrap());

		let pool_account_result_asset_in = Currency::free_balance(asset_in, &pool_account);
		let pool_account_result_asset_out = Currency::free_balance(asset_out, &pool_account);

		assert_eq!(
			Currency::free_balance(asset_in, &pool_owner),
			owner_initial_asset_in_balance
		);
		assert_eq!(
			Currency::free_balance(asset_out, &pool_owner),
			owner_initial_asset_out_balance
		);

		assert_eq!(Currency::free_balance(asset_in, &pool_account), 4_970_435);
		assert_eq!(Currency::free_balance(asset_out, &pool_account), 125_000_009);

		assert_eq!(Currency::free_balance(asset_in, &lbp_participant), 999_995_984_267);
		assert_eq!(Currency::free_balance(asset_out, &lbp_participant), 1_000_374_999_991);

		// remove liquidity from the pool
		assert_ok!(LBPPallet::remove_liquidity(Origin::signed(pool_owner), pool_account));

		assert_eq!(Currency::free_balance(asset_in, &pool_account), 0);
		assert_eq!(Currency::free_balance(asset_out, &pool_account), 0);

		assert_eq!(
			Currency::free_balance(asset_in, &pool_owner),
			owner_initial_asset_in_balance
				.checked_add(pool_account_result_asset_in)
				.unwrap()
		);
		assert_eq!(
			Currency::free_balance(asset_out, &pool_owner),
			owner_initial_asset_out_balance
				.checked_add(pool_account_result_asset_out)
				.unwrap()
		);

		assert_eq!(Currency::free_balance(asset_in, &fee_collector), 45_298);
		assert_eq!(Currency::free_balance(asset_out, &fee_collector), 0);
	});
}

#[test]
fn validate_trade_should_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block::<Test>(10);

		assert_eq!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				1_000_000_u128,
				2_157_153_u128,
				TradeType::Buy,
			)
			.unwrap(),
			AMMTransfer {
				origin: ALICE,
				assets: AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				amount: 2_002_498_u128,
				amount_out: 1_000_000_u128,
				discount: false,
				discount_amount: 0_u128,
				fee: (ACA, 4_004),
			}
		);

		assert_eq!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				1_000_000_u128,
				2_000_u128,
				TradeType::Sell,
			)
			.unwrap(),
			AMMTransfer {
				origin: ALICE,
				assets: AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				amount: 1_000_000_u128,
				amount_out: 498_686_u128,
				discount: false,
				discount_amount: 0_u128,
				fee: (DOT, 999),
			}
		);
	});
}

#[test]
fn validate_trade_should_not_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block::<Test>(9);

		assert_noop!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				1_000_000_u128,
				2_157_153_u128,
				TradeType::Buy,
			),
			Error::<Test>::SaleIsNotRunning
		);

		run_to_block::<Test>(10);

		assert_noop!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				0,
				2_157_153_u128,
				TradeType::Buy,
			),
			Error::<Test>::ZeroAmount
		);

		assert_ok!(Currency::transfer(Origin::signed(ALICE), BOB, DOT, 999997500000000));
		assert_ok!(Currency::transfer(Origin::signed(ALICE), BOB, ACA, 999998500000000));
		assert_eq!(Currency::free_balance(DOT, &ALICE), 500000000);
		assert_eq!(Currency::free_balance(ACA, &ALICE), 500000000);
		assert_err!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				500_000_001u128,
				3000000000_u128,
				TradeType::Buy,
			),
			Error::<Test>::InsufficientAssetBalance
		);

		assert_noop!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: HDX
				},
				1_000_000_u128,
				2_157_153_u128,
				TradeType::Buy,
			),
			Error::<Test>::PoolNotFound
		);

		assert_err!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				1_000_000_000_u128,
				2_157_153_u128,
				TradeType::Buy,
			),
			Error::<Test>::MaxOutRatioExceeded
		);

		assert_err!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				400_000_000_u128,
				2_157_153_u128,
				TradeType::Sell,
			),
			Error::<Test>::MaxInRatioExceeded
		);

		assert_err!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				1_000_u128,
				499_u128,
				TradeType::Sell,
			),
			Error::<Test>::AssetBalanceLimitExceeded
		);

		assert_err!(
			LBPPallet::validate_trade(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: DOT
				},
				1_000_u128,
				1_994_u128,
				TradeType::Buy,
			),
			Error::<Test>::AssetBalanceLimitExceeded
		);

		Currency::set_balance(Origin::root(), ALICE, ACA, INITIAL_BALANCE, 0).unwrap();
		Currency::set_balance(Origin::root(), ALICE, HDX, INITIAL_BALANCE, 0).unwrap();

		// transfer fee > token amount in
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			ACA,
			1_000_000_000,
			HDX,
			2_000_000_000,
			20_000_000,
			80_000_000,
			WeightCurveType::Linear,
			Fee {
				numerator: 10,
				denominator: 1
			},
			CHARLIE,
		));
		let pool_id2 = LBPPallet::get_pair_id(AssetPair {
			asset_in: ACA,
			asset_out: HDX,
		});
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			pool_id2,
			None,
			Some(12),
			Some(20),
			None,
			None,
			None,
			None
		));
		run_to_block::<Test>(15);
		assert_noop!(
			LBPPallet::validate_sell(
				&ALICE,
				AssetPair {
					asset_in: ACA,
					asset_out: HDX
				},
				1_000_000,
				100_000,
				false
			),
			Error::<Test>::Overflow
		);
	});
}

#[test]
fn get_sorted_weight_should_work() {
	predefined_test_ext().execute_with(|| {
		let pool = LBPPallet::pool_data(ACA_DOT_POOL_ID);

		assert_eq!(
			LBPPallet::get_sorted_weight(ACA, <Test as frame_system::Config>::BlockNumber::from(10u32), &pool).unwrap(),
			(20_000_000, 80_000_000),
		);

		assert_eq!(
			LBPPallet::get_sorted_weight(DOT, <Test as frame_system::Config>::BlockNumber::from(10u32), &pool).unwrap(),
			(80_000_000, 20_000_000),
		);

		assert_eq!(
			LBPPallet::get_sorted_weight(ACA, <Test as frame_system::Config>::BlockNumber::from(30u32), &pool)
				.err()
				.unwrap()
				.as_u8(),
			12_u8, // InvalidWeight
		);
	});
}

#[test]
fn calculate_fees_should_work() {
	predefined_test_ext().execute_with(|| {
		let pool = LBPPallet::pool_data(ACA_DOT_POOL_ID);

		assert_eq!(LBPPallet::calculate_fees(&pool, 1000_u128).unwrap(), 2,);

		assert_eq!(LBPPallet::calculate_fees(&pool, 500_u128).unwrap(), 1,);

		assert_eq!(LBPPallet::calculate_fees(&pool, 499_u128).unwrap(), 0,);

		assert_err!(
			LBPPallet::calculate_fees(&pool, u128::MAX / 2 + 1),
			Error::<Test>::FeeAmountInvalid,
		);
	});
}

#[test]
fn can_create_should_work() {
	new_test_ext().execute_with(|| {
		let asset_pair = AssetPair{ asset_in: ACA, asset_out: DOT };
		// pool doesn't exist
		assert!(DisallowLBPRunningPool::<Test>::can_create(asset_pair.asset_in, asset_pair.asset_out));

		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			ACA,
			1_000_000_000,
			DOT,
			2_000_000_000,
			20_000_000,
			80_000_000,
			WeightCurveType::Linear,
			Fee::default(),
			CHARLIE,
		));
		// pool is not initialized
		assert!(!DisallowLBPRunningPool::<Test>::can_create(asset_pair.asset_in, asset_pair.asset_out));

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(10),
			Some(20),
			None,
			None,
			None,
			None
		));
		// pool is initialized but is not running
		assert!(!DisallowLBPRunningPool::<Test>::can_create(asset_pair.asset_in, asset_pair.asset_out));

		run_to_block::<Test>(15);
		// pool is running
		assert!(!DisallowLBPRunningPool::<Test>::can_create(asset_pair.asset_in, asset_pair.asset_out));

		run_to_block::<Test>(30);
		// sale ended
		assert!(!DisallowLBPRunningPool::<Test>::can_create(asset_pair.asset_in, asset_pair.asset_out));

		assert_ok!(LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID,));
		// pool was destroyed
		assert!(DisallowLBPRunningPool::<Test>::can_create(asset_pair.asset_in, asset_pair.asset_out));
	});
}