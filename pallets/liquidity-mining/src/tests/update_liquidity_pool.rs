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
use warehouse_liquidity_mining::GlobalPool;
use warehouse_liquidity_mining::LiquidityPoolYieldFarm;

#[test]
fn update_liquidity_pool_should_work() {
	//liq. pool without deposits
	predefined_test_ext().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		let new_multiplier: PoolMultiplier = FixedU128::from(5_000_u128);
		let liq_pool = WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = WarehouseLM::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets,
			new_multiplier
		));

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				multiplier: new_multiplier,
				..liq_pool
			}
		);

		assert_eq!(WarehouseLM::global_pool(GC_FARM).unwrap(), global_pool);
	});

	//liq. pool with deposits
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		//same period as last pool update so no pool(global or liq. pool) updated
		let new_multiplier: PoolMultiplier = FixedU128::from(10_000_u128);
		let liq_pool = WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = WarehouseLM::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets,
			new_multiplier
		));

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				multiplier: new_multiplier,
				..liq_pool
			}
		);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				total_shares_z: 455_876_290,
				..global_pool
			}
		);

		//different period so pool update should happen
		set_block_number(5_000);
		let new_multiplier: PoolMultiplier = FixedU128::from(5_000_u128);
		let liq_pool = WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = WarehouseLM::global_pool(GC_FARM).unwrap();

		let global_pool_account = WarehouseLM::pool_account_id(GC_FARM).unwrap();
		let liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets,
			new_multiplier
		));

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				updated_at: 50,
				accumulated_rpvs: 30_060,
				accumulated_rpz: 15,
				multiplier: new_multiplier,
				..liq_pool
			}
		);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				updated_at: 50,
				accumulated_rpz: 15,
				total_shares_z: 228_176_290,
				accumulated_rewards: global_pool.accumulated_rewards + 133_800_000,
				paid_accumulated_rewards: global_pool.paid_accumulated_rewards + 1_366_200_000,
				..global_pool
			}
		);

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance - 1_366_200_000 //1_366_200_000 - liq. pool claim from global pool
		);
		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_account),
			liq_pool_bsx_balance + 1_366_200_000 //1_366_200_000 - liq. pool claim from global pool
		);
	});
}

#[test]
fn update_liquidity_pool_zero_multiplier_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::update_liquidity_pool(
				Origin::signed(GC),
				GC_FARM,
				bsx_tkn1_assets,
				FixedU128::from(0_u128)
			),
			warehouse_liquidity_mining::Error::<Test>::InvalidMultiplier
		);
	});
}

#[test]
fn update_liquidity_pool_canceled_pool_should_not_work() {
	let bsx_tkn1_liq_pool = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_liq_pool
		));

		assert_noop!(
			LiquidityMining::update_liquidity_pool(
				Origin::signed(GC),
				GC_FARM,
				bsx_tkn1_liq_pool,
				FixedU128::from(10_001)
			),
			warehouse_liquidity_mining::Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn update_liquidity_pool_not_owner_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let not_owner = ALICE;
		assert_noop!(
			LiquidityMining::update_liquidity_pool(
				Origin::signed(not_owner),
				GC_FARM,
				bsx_tkn1_assets,
				FixedU128::from(10_001_u128)
			),
			warehouse_liquidity_mining::Error::<Test>::LiquidityMiningCanceled
		);
	});
}
