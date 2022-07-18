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
use warehouse_liquidity_mining::{FarmMultiplier, YieldFarmData};

#[test]
fn update_yield_farm_should_() {
	predefined_test_ext().execute_with(|| {
		//Arrange
		let new_multiplier: FarmMultiplier = FixedU128::from(5_000_u128);
		let yield_farm = WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap();
		let global_farm = WarehouseLM::global_farm(GC_FARM).unwrap();

		//Act
		assert_ok!(LiquidityMining::update_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR,
			new_multiplier
		));

		//Assert
		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmUpdated {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: GC,
			asset_pair: BSX_TKN1_ASSET_PAIR,
			multiplier: new_multiplier,
		})]);

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				multiplier: new_multiplier,
				..yield_farm
			}
		);

		assert_eq!(WarehouseLM::global_farm(GC_FARM).unwrap(), global_farm);
	});
}

#[test]
fn update_yield_farm_should_fail_with_propagated_error_when_multiplier_is_zero() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::update_yield_farm(
				Origin::signed(GC),
				GC_FARM,
				BSX_TKN1_ASSET_PAIR,
				FixedU128::from(0_u128)
			),
			warehouse_liquidity_mining::Error::<Test, Instance1>::InvalidMultiplier
		);
	});
}

#[test]
fn update_yield_farm_should_fail_when_caller_is_not_signed() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::update_yield_farm(Origin::none(), GC_FARM, BSX_TKN1_ASSET_PAIR, FixedU128::from(10_001)),
			BadOrigin
		);
	});
}

#[test]
fn update_yield_farm_should_fail_when_amm_pool_does_not_exist() {
	let unknown_asset_pair: AssetPair = AssetPair {
		asset_in: 9999,
		asset_out: 19999,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::update_yield_farm(Origin::signed(GC), GC_FARM, unknown_asset_pair, FixedU128::from(10_001)),
			Error::<Test>::AmmPoolDoesNotExist
		);
	});
}
