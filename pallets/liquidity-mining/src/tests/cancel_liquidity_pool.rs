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
fn cancel_liquidity_pool_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	//same period
	predefined_test_ext_with_deposits().execute_with(|| {
		let liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let global_pool_account = WarehouseLM::pool_account_id(GC_FARM).unwrap();
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let liq_pool = WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = WarehouseLM::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityMiningCanceled {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				stake_in_global_pool: 0,
				canceled: true,
				multiplier: 0.into(),
				..liq_pool
			}
		);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				total_shares_z: global_pool
					.total_shares_z
					.checked_sub(liq_pool.stake_in_global_pool)
					.unwrap(),
				..global_pool
			}
		);

		assert_eq!(WarehouseLM::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(), (3, GC_FARM));

		assert_eq!(Tokens::free_balance(BSX, &liq_pool_account), liq_pool_bsx_balance);
		assert_eq!(Tokens::free_balance(BSX, &global_pool_account), global_pool_bsx_balance);
	});

	//canelc liq. pool with pools update
	predefined_test_ext_with_deposits().execute_with(|| {
		let liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let global_pool_account = WarehouseLM::pool_account_id(GC_FARM).unwrap();
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let liq_pool = WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = WarehouseLM::global_pool(GC_FARM).unwrap();

		set_block_number(10_000);

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityMiningCanceled {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				updated_at: 100,
				accumulated_rpvs: 245,
				accumulated_rpz: 49,
				stake_in_global_pool: 0,
				canceled: true,
				multiplier: 0.into(),
				..liq_pool
			}
		);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				updated_at: 100,
				accumulated_rpz: 49,
				total_shares_z: global_pool
					.total_shares_z
					.checked_sub(liq_pool.stake_in_global_pool)
					.unwrap(),
				accumulated_rewards: 18_206_375,
				paid_accumulated_rewards: 9_589_300,
				..global_pool
			}
		);

		assert_eq!(WarehouseLM::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(), (3, GC_FARM));

		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_account),
			liq_pool_bsx_balance + 8_424_900 //8_424_900 - liq. pool last claim from global pool
		);

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance - 8_424_900 //8_424_900 - liq. pool last claim from global pool
		);
	});
}

#[test]
fn cancel_liquidity_pool_invalid_liq_pool_should_not_work() {
	let bsx_dot_assets = AssetPair {
		asset_in: BSX,
		asset_out: DOT,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_dot_assets),
			warehouse_liquidity_mining::Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn cancel_liquidity_pool_liq_pool_already_canceled() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		//1-th cancel should pass ok
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_tkn1_assets),
			warehouse_liquidity_mining::Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn cancel_liquidity_pool_not_owner_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_LIQ_POOL_OWNER: u128 = ALICE;

		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(NOT_LIQ_POOL_OWNER), GC_FARM, bsx_tkn1_assets),
			warehouse_liquidity_mining::Error::<Test>::Forbidden
		);
	});
}
