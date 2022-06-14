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
use test_ext::*;

#[test]
fn create_global_farm_should_work() {
	new_test_ext().execute_with(|| {
		let id = 1;
		let total_rewards: Balance = 50_000_000_000;
		let reward_currency = BSX;
		let planned_yielding_periods: BlockNumber = 1_000_000_000_u64;
		let blocks_per_period = 20_000;
		let incentivized_asset = BSX;
		let owner = ALICE;
		let yield_per_period = Permill::from_percent(20);
		let max_reward_per_period: Balance = total_rewards.checked_div(planned_yielding_periods.into()).unwrap();

		let created_at_block = 15_896;

		set_block_number(created_at_block);

		let pool_account = WarehouseLM::farm_account_id(id).unwrap();
		assert_eq!(Tokens::free_balance(reward_currency, &pool_account), 0);

		assert_ok!(LiquidityMining::create_global_farm(
			Origin::root(),
			total_rewards,
			planned_yielding_periods,
			blocks_per_period,
			incentivized_asset,
			reward_currency,
			owner,
			yield_per_period
		));

		//check if total_rewards was transferd to pool account
		assert_eq!(Tokens::free_balance(reward_currency, &pool_account), total_rewards);
		assert_eq!(
			Tokens::free_balance(reward_currency, &ALICE),
			(INITIAL_BALANCE - total_rewards)
		);

		let updated_at = created_at_block / blocks_per_period;

		expect_events(vec![mock::Event::LiquidityMining(Event::GlobalFarmCreated {
			id,
			owner,
			reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			incentivized_asset,
			max_reward_per_period,
		})]);

		assert_eq!(
			WarehouseLM::global_farm(id).unwrap(),
			GlobalFarmData::new(
				id,
				updated_at,
				reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_asset,
				max_reward_per_period,
			)
		);
	});
}

#[test]
fn create_global_farm_from_basic_origin_should_not_work() {
	new_test_ext().execute_with(|| {
		let created_at_block = 15_896;

		set_block_number(created_at_block);

		assert_noop!(
			LiquidityMining::create_global_farm(
				Origin::signed(ALICE),
				1_000_000,
				1_000,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			BadOrigin
		);
	});
}

#[test]
fn create_global_farm_invalid_data_should_not_work() {
	new_test_ext().execute_with(|| {
		let created_at_block = 15_896;

		set_block_number(created_at_block);

		//total_rewards bellow min. limit
		assert_noop!(
			LiquidityMining::create_global_farm(
				Origin::root(),
				100,
				1_000,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			warehouse_liquidity_mining::Error::<Test>::InvalidTotalRewards
		);

		//planned_yielding_periods bellow min. limit
		assert_noop!(
			LiquidityMining::create_global_farm(
				Origin::root(),
				1_000_000,
				10,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			warehouse_liquidity_mining::Error::<Test>::InvalidPlannedYieldingPeriods
		);

		//blocks_per_period is 0.
		assert_noop!(
			LiquidityMining::create_global_farm(
				Origin::root(),
				1_000_000,
				1_000,
				0,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			warehouse_liquidity_mining::Error::<Test>::InvalidBlocksPerPeriod
		);

		//yield_per_period is 0.
		assert_noop!(
			LiquidityMining::create_global_farm(
				Origin::root(),
				1_000_000,
				1_000,
				1,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(0)
			),
			warehouse_liquidity_mining::Error::<Test>::InvalidYieldPerPeriod
		);
	});
}

#[test]
fn create_global_farm_with_insufficient_balance_should_not_work() {
	//owner account balance is 1M BSX
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::create_global_farm(
				Origin::root(),
				1_000_001,
				1_000,
				1,
				BSX,
				BSX,
				ACCOUNT_WITH_1M,
				Permill::from_percent(20)
			),
			//This error is from warehouse liq. mining pallet
			warehouse_liquidity_mining::Error::<Test>::InsufficientRewardCurrencyBalance
		);
	});
}
