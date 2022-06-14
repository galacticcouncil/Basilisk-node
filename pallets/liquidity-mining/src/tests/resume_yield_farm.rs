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
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		//cancel liq. pool before resuming
		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let liq_pool = WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_LIQ_POOL_ID)).unwrap();
		let global_pool = WarehouseLM::global_farm(GC_FARM).unwrap();

		let new_multiplier = FixedU128::from(7_490_000);

		assert_eq!(liq_pool.state, YieldFarmState::Stopped);
		assert!(liq_pool.multiplier.is_zero());

		set_block_number(13_420_000);

		assert_ok!(LiquidityMining::resume_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_LIQ_POOL_ID,
			bsx_tkn1_assets,
			new_multiplier
		));

		let liq_pool_stake_in_global_pool = new_multiplier.checked_mul_int(45_540).unwrap();

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_LIQ_POOL_ID)).unwrap(),
			YieldFarmData {
				state: YieldFarmState::Active,
				accumulated_rpz: 62_996,
				multiplier: new_multiplier,
				updated_at: 134_200,
				..liq_pool
			}
		);

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
				total_shares_z: global_pool.total_shares_z + liq_pool_stake_in_global_pool,
				updated_at: 134_200,
				accumulated_rpz: 62_996,
				accumulated_rewards: 29_999_067_250,
				..global_pool
			}
		);
	});
}

#[test]
fn resume_yield_farm_non_existing_pool_should_not_work() {
	let bsx_ksm_assets = AssetPair {
		asset_in: BSX,
		asset_out: KSM,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7_490_000);

		assert_noop!(
			LiquidityMining::resume_yield_farm(
				Origin::signed(GC),
				GC_FARM,
				BSX_TKN1_LIQ_POOL_ID,
				bsx_ksm_assets,
				new_multiplier
			),
			warehouse_liquidity_mining::Error::<Test>::YieldFarmNotFound
		);
	});
}

#[test]
fn resume_yield_farm_non_canceled_pool_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7_490_000);

		assert_noop!(
			LiquidityMining::resume_yield_farm(
				Origin::signed(GC),
				GC_FARM,
				BSX_TKN1_LIQ_POOL_ID,
				bsx_tkn1_assets,
				new_multiplier
			),
			warehouse_liquidity_mining::Error::<Test>::YieldFarmAlreadyExists
		);
	});
}

#[test]
fn resume_yield_farm_not_owner_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7_490_000);

		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		assert_noop!(
			LiquidityMining::resume_yield_farm(
				Origin::signed(ALICE),
				GC_FARM,
				BSX_TKN1_LIQ_POOL_ID,
				bsx_tkn1_assets,
				new_multiplier
			),
			warehouse_liquidity_mining::Error::<Test>::Forbidden
		);
	});
}