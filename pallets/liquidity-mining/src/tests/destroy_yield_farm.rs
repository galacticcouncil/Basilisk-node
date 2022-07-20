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
use test_ext::*;
use warehouse_liquidity_mining::GlobalFarmData;

#[test]
fn destroy_yield_farm_should_work_when_it_has_deposit() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arrange
		let global_farm_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();
		let yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();

		let yield_farm_bsx_balance = Tokens::free_balance(BSX, &yield_farm_account);
		let global_farm_bsx_balance = Tokens::free_balance(BSX, &global_farm_account);

		// cancel liq. pool before removing
		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR
		));

		let global_farm = WarehouseLM::global_farm(GC_FARM).unwrap();
		let yield_farm = WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap();

		//Act
		assert_ok!(LiquidityMining::destroy_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR
		));

		//Assert
		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmDestroyed {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: GC,
			asset_pair: BSX_TKN1_ASSET_PAIR,
		})]);

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
				yield_farms_count: (
					global_farm.yield_farms_count.0.checked_sub(1).unwrap(),
					global_farm.yield_farms_count.1
				),
				..global_farm
			}
		);

		//Yield farm is removed from storage only if there are no more farm entries.
		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				state: FarmState::Deleted,
				..yield_farm
			}
		);

		assert_eq!(Tokens::free_balance(BSX, &yield_farm_account), 0);

		//unpaid rewards from yield farm account should be transferred back to global farm account
		assert_eq!(
			Tokens::free_balance(BSX, &global_farm_account),
			global_farm_bsx_balance.checked_add(yield_farm_bsx_balance).unwrap()
		);
	});
}

#[test]
fn destroy_yield_farm_should_fail_when_caller_is_not_signed() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR
		));

		assert_noop!(
			LiquidityMining::destroy_yield_farm(Origin::none(), GC_FARM, BSX_TKN1_YIELD_FARM_ID, BSX_TKN1_ASSET_PAIR),
			BadOrigin
		);
	});
}

#[test]
fn destroy_yield_farm_should_fail_with_propagated_error_when_called_by_not_owner() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arrange
		const NOT_OWNER: u128 = ALICE;

		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR
		));

		//Act and assert
		assert_noop!(
			LiquidityMining::destroy_yield_farm(
				Origin::signed(NOT_OWNER),
				GC_FARM,
				BSX_TKN1_YIELD_FARM_ID,
				BSX_TKN1_ASSET_PAIR
			),
			warehouse_liquidity_mining::Error::<Test, Instance1>::Forbidden
		);
	});
}
