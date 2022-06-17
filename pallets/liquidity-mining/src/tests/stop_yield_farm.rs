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
use pretty_assertions::assert_eq;
use sp_runtime::FixedPointNumber;
use test_ext::*;
use warehouse_liquidity_mining::GlobalFarmData;
use warehouse_liquidity_mining::YieldFarmData;
use warehouse_liquidity_mining::YieldFarmState;

#[test]
fn stop_yield_farm_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();
		let global_farm_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();
		let yield_farm_bsx_balance = Tokens::free_balance(BSX, &yield_farm_account);
		let global_farm_bsx_balance = Tokens::free_balance(BSX, &global_farm_account);
		let yield_farm = WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap();
		let global_farm = WarehouseLM::global_farm(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityMiningCanceled {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: GC,
			asset_pair: BSX_TKN1_ASSET_PAIR,
		})]);

		let stake_in_global_farm = yield_farm
			.multiplier
			.checked_mul_int(yield_farm.total_valued_shares)
			.unwrap();

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				state: YieldFarmState::Stopped,
				multiplier: 0.into(),
				..yield_farm
			}
		);

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
				total_shares_z: global_farm.total_shares_z.checked_sub(stake_in_global_farm).unwrap(),
				..global_farm
			}
		);

		assert_eq!(Tokens::free_balance(BSX, &yield_farm_account), yield_farm_bsx_balance);
		assert_eq!(Tokens::free_balance(BSX, &global_farm_account), global_farm_bsx_balance);
	});
}

#[test]
fn stop_yield_farm_should_fail_when_caller_is_not_signed() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::stop_yield_farm(Origin::none(), GC_FARM, BSX_DOT_ASSET_PAIR),
			BadOrigin
		);
	});
}

#[test]
fn stop_yield_farm_should_fail_with_propagated_error_when_yield_farm_is_already_stopped() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR
		));

		assert_noop!(
			LiquidityMining::stop_yield_farm(Origin::signed(GC), GC_FARM, BSX_TKN1_ASSET_PAIR),
			warehouse_liquidity_mining::Error::<Test>::YieldFarmNotFound
		);
	});
}

#[test]
fn stop_yield_farm_not_owner_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_LIQ_POOL_OWNER: u128 = ALICE;

		assert_noop!(
			LiquidityMining::stop_yield_farm(Origin::signed(NOT_LIQ_POOL_OWNER), GC_FARM, BSX_TKN1_ASSET_PAIR),
			warehouse_liquidity_mining::Error::<Test>::Forbidden
		);
	});
}
