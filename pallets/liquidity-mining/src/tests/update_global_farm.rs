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
use warehouse_liquidity_mining::GlobalFarmData;

#[test]
fn update_global_farm_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arange
		let new_price_adjustment = FixedU128::from_float(234.1234_f64);
		let global_farm_0 = WarehouseLM::global_farm(GC_FARM).unwrap();

		set_block_number(100_000);

		//Act
		assert_ok!(LiquidityMining::update_global_farm(
			Origin::signed(GC),
			GC_FARM,
			new_price_adjustment
		));

		//Assert
		assert_last_event!(crate::Event::GlobalFarmUpdated {
			id: GC_FARM,
			price_adjustment: new_price_adjustment,
		}
		.into());

		pretty_assertions::assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
				updated_at: 1_000,
				accumulated_rpz: FixedU128::from_inner(491_000_000_000_000_000_000_u128),
				accumulated_rewards: 343_195_125_u128,
				price_adjustment: new_price_adjustment,
				..global_farm_0
			}
		)
	});
}

#[test]
fn udpate_global_farm_should_fail_with_propagated_error_when_price_adjustment_is_zero() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arange
		set_block_number(100_000);

		let new_price_adjustment = FixedU128::zero();

		//Act & assert
		assert_noop!(
			LiquidityMining::update_global_farm(Origin::signed(GC), GC_FARM, new_price_adjustment),
			warehouse_liquidity_mining::Error::<Test, Instance1>::InvalidPriceAdjustment
		);
	});
}

#[test]
fn udpate_global_farm_should_fail_with_propagated_error_when_origin_is_not_farm_owner() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arange
		set_block_number(100_000);

		let new_price_adjustment = FixedU128::from_float(0.5_f64);
		let not_owner = ALICE;

		//Act & assert
		assert_noop!(
			LiquidityMining::update_global_farm(Origin::signed(not_owner), GC_FARM, new_price_adjustment),
			warehouse_liquidity_mining::Error::<Test, Instance1>::Forbidden
		);
	});
}
