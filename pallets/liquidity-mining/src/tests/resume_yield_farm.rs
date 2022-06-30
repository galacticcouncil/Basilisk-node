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

#[test]
fn resume_yield_farm_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arrange
		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR
		));

		let yield_farm = WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap();
		let global_farm = WarehouseLM::global_farm(GC_FARM).unwrap();

		assert_eq!(yield_farm.state, FarmState::Stopped);
		assert!(yield_farm.multiplier.is_zero());

		set_block_number(13_420_000);

		//Act
		let new_multiplier = FixedU128::from(7_490_000);
		assert_ok!(LiquidityMining::resume_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR,
			new_multiplier
		));

		//Assert
		let yield_farm_stake_in_global_farm = new_multiplier.checked_mul_int(45_540).unwrap();

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				state: FarmState::Active,
				accumulated_rpz: 62_996,
				multiplier: new_multiplier,
				updated_at: 134_200,
				..yield_farm
			}
		);

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
				total_shares_z: global_farm.total_shares_z + yield_farm_stake_in_global_farm,
				updated_at: 134_200,
				accumulated_rpz: 62_996,
				accumulated_rewards: 29_999_067_250,
				..global_farm
			}
		);

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityMiningResumed {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: GC,
			asset_pair: BSX_TKN1_ASSET_PAIR,
			multiplier: new_multiplier,
		})]);
	});
}

#[test]
fn resume_yield_farm_should_fail_with_propagated_error_when_farm_does_not_exist() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7_490_000);

		assert_noop!(
			LiquidityMining::resume_yield_farm(
				Origin::signed(GC),
				GC_FARM,
				BSX_TKN1_YIELD_FARM_ID,
				BSX_KSM_ASSET_PAIR,
				new_multiplier
			),
			warehouse_liquidity_mining::Error::<Test, Instance1>::YieldFarmNotFound
		);
	});
}

#[test]
fn resume_yield_farm_should_fail_when_caller_is_not_signed() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7_490_000);

		assert_noop!(
			LiquidityMining::resume_yield_farm(
				Origin::none(),
				GC_FARM,
				BSX_TKN1_YIELD_FARM_ID,
				BSX_KSM_ASSET_PAIR,
				new_multiplier
			),
			BadOrigin
		);
	});
}
