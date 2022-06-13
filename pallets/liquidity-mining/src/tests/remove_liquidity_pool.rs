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
fn remove_liquidity_pool_with_deposits_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let global_pool_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();
		let liq_pool_account = WarehouseLM::farm_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);

		// cancel liq. pool before removing
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let global_farm = WarehouseLM::global_farm(GC_FARM).unwrap();
		let yield_farm = WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_LIQ_POOL_ID)).unwrap();

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_LIQ_POOL_ID,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmRemoved {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		//TODO: Dani - chheck - otherwise Martin
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
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_LIQ_POOL_ID)).unwrap(),
			YieldFarmData {
				state: YieldFarmState::Deleted,
				..yield_farm
			}
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
		let global_pool_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();
		let liq_pool_account = WarehouseLM::farm_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);

		//cancel pool before removing
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let global_pool = WarehouseLM::global_farm(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_LIQ_POOL_ID,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmRemoved {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		let live_farms_count = global_pool.yield_farms_count.0.checked_sub(1).unwrap();

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
				yield_farms_count: (live_farms_count, live_farms_count),
				..global_pool
			}
		);

		//liq. pool should be removed from storage
		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_LIQ_POOL_ID)),
			None
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
fn remove_liquidity_pool_non_canceled_liq_pool_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::remove_liquidity_pool(Origin::signed(GC), GC_FARM, BSX_TKN1_LIQ_POOL_ID, bsx_tkn1_assets),
			warehouse_liquidity_mining::Error::<Test>::LiquidityMiningIsNotCanceled
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
			LiquidityMining::remove_liquidity_pool(Origin::signed(NOT_OWNER), GC_FARM, BSX_TKN1_LIQ_POOL_ID, bsx_tkn1),
			warehouse_liquidity_mining::Error::<Test>::Forbidden
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
			LiquidityMining::remove_liquidity_pool(Origin::signed(GC), GC_FARM, BSX_TKN1_LIQ_POOL_ID, bsx_dot_assets),
			warehouse_liquidity_mining::Error::<Test>::YieldFarmNotFound
		);
	});
}
