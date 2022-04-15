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
fn remove_liquidity_pool_with_deposits_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);

		// cancel liq. pool before removing
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolRemoved {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				liq_pools_count: global_pool.liq_pools_count.checked_sub(1).unwrap(),
				..global_pool
			}
		);

		//liq. pool should be removed from storage
		assert_eq!(LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM), None);

		//liq. pool meta should stay in storage until all deposits are withdrawn
		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 3, GC_FARM)
		);

		assert_eq!(Tokens::free_balance(BSX, &liq_pool_account), 0);

		//unpaid rewards from liq. pool account should be transfered back to global pool account
		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance.checked_add(liq_pool_bsx_balance).unwrap()
		);
	});
}

#[test]
fn remove_liquidity_pool_without_deposits_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext().execute_with(|| {
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);

		//cancel pool before removing
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolRemoved {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				liq_pools_count: global_pool.liq_pools_count.checked_sub(1).unwrap(),
				..global_pool
			}
		);

		//liq. pool should be removed from storage
		assert_eq!(LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM), None);

		//liq. pool metadata should be removed from storage if no deposits are left
		assert_eq!(LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID), None);

		assert_eq!(Tokens::free_balance(BSX, &liq_pool_account), 0);

		//unpaid rewards from liq. pool account should be transfered back to global pool account
		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance.checked_add(liq_pool_bsx_balance).unwrap()
		);
	});
}

#[test]
fn remove_liquidity_pool_non_canceled_liq_pool_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::remove_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_tkn1_assets),
			Error::<Test>::LiquidityMiningIsNotCanceled
		);
	});
}

#[test]
fn remove_liquidity_pool_not_owner_should_not_work() {
	let bsx_tkn1 = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_OWNER: u128 = ALICE;

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1
		));

		assert_noop!(
			LiquidityMining::remove_liquidity_pool(Origin::signed(NOT_OWNER), GC_FARM, bsx_tkn1),
			Error::<Test>::Forbidden
		);
	});
}

#[test]
fn remove_liquidity_pool_liq_pool_does_not_exists_should_not_work() {
	let bsx_dot_assets = AssetPair {
		asset_in: BSX,
		asset_out: DOT,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::remove_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_dot_assets),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}
