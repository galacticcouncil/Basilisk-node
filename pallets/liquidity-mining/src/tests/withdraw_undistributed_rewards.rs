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
fn withdraw_undistributed_rewards_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	let bsx_tkn2_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN2,
	};

	predefined_test_ext().execute_with(|| {
		//farm have to empty to be able to withdraw undistributed rewards
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn2_assets
		));

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));
		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn2_assets
		));

		let farm_owner_bsx_balance = Tokens::total_balance(BSX, &GC);

		assert_ok!(LiquidityMining::withdraw_undistributed_rewards(
			Origin::signed(GC),
			GC_FARM
		));

		assert_eq!(Tokens::total_balance(BSX, &GC), farm_owner_bsx_balance + 30_000_000_000);
	});
}

#[test]
fn withdraw_undistributed_rewards_non_existing_farm_should_not_work() {
	const NON_EXISTING_FARM: PoolId = 879_798;

	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::withdraw_undistributed_rewards(Origin::signed(GC), NON_EXISTING_FARM),
			pallet_liquidity_mining::Error::<Test>::FarmNotFound
		);
	});
}

#[test]
fn withdraw_undistributed_rewards_not_owner_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	let bsx_tkn2_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN2,
	};

	predefined_test_ext().execute_with(|| {
		//farm have to empty to be able to withdraw undistributed rewards
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn2_assets
		));

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));
		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn2_assets
		));

		const NOT_OWNER: u128 = ALICE;
		assert_noop!(
			LiquidityMining::withdraw_undistributed_rewards(Origin::signed(NOT_OWNER), GC_FARM),
			pallet_liquidity_mining::Error::<Test>::Forbidden
		);
	});
}

#[test]
fn withdraw_undistributed_rewards_not_empty_farm_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	let bsx_tkn2_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN2,
	};

	predefined_test_ext().execute_with(|| {
		//only cancel liq. pools, DON'T remove (farm is not empty)
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn2_assets
		));

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn2_assets
		));

		assert_noop!(
			LiquidityMining::withdraw_undistributed_rewards(Origin::signed(GC), GC_FARM),
			pallet_liquidity_mining::Error::<Test>::FarmIsNotEmpty
		);
	});

	predefined_test_ext().execute_with(|| {
		//not all liq. pools are canceled
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn2_assets
		));

		assert_noop!(
			LiquidityMining::withdraw_undistributed_rewards(Origin::signed(GC), GC_FARM),
			pallet_liquidity_mining::Error::<Test>::FarmIsNotEmpty
		);
	});
}
