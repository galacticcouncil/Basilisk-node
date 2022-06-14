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
fn destroy_global_farm_should_work() {
	predefined_test_ext().execute_with(|| {
		//transfer all rewards from farm account
		let farm_account = WarehouseLM::farm_account_id(BOB_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_FARMS[1].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_FARMS[1].reward_currency, &farm_account),
			0
		);

		assert_ok!(LiquidityMining::destroy_global_farm(Origin::signed(BOB), BOB_FARM));

		expect_events(vec![mock::Event::LiquidityMining(Event::GlobalFarmDestroyed {
			id: BOB_FARM,
			who: BOB,
		})]);

		assert!(WarehouseLM::global_farm(BOB_FARM).is_none());
	});
}

#[test]
fn destroy_global_farm_not_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		//transfer all rewards from farm account
		let farm_account = WarehouseLM::farm_account_id(BOB_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_FARMS[1].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_FARMS[1].reward_currency, &farm_account),
			0
		);

		assert_noop!(
			LiquidityMining::destroy_global_farm(Origin::signed(ALICE), BOB_FARM),
			warehouse_liquidity_mining::Error::<Test>::Forbidden
		);

		assert_eq!(WarehouseLM::global_farm(BOB_FARM).unwrap(), PREDEFINED_GLOBAL_FARMS[1]);
	});
}

#[test]
fn destroy_global_farm_farm_not_exists_should_not_work() {
	predefined_test_ext().execute_with(|| {
		const NON_EXISTING_FARM: u32 = 999_999_999;
		assert_noop!(
			LiquidityMining::destroy_global_farm(Origin::signed(ALICE), NON_EXISTING_FARM),
			warehouse_liquidity_mining::Error::<Test>::GlobalFarmNotFound
		);
	});
}

#[test]
fn destroy_global_farm_with_pools_should_not_work() {
	//all rewards was distributed but yield farm still exist in the farm
	predefined_test_ext().execute_with(|| {
		//transfer all rewards from farm account
		let farm_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_FARMS[2].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_FARMS[2].reward_currency, &farm_account),
			0
		);

		assert_noop!(
			LiquidityMining::destroy_global_farm(Origin::signed(GC), GC_FARM),
			warehouse_liquidity_mining::Error::<Test>::GlobalFarmIsNotEmpty
		);

		assert_eq!(WarehouseLM::global_farm(GC_FARM).unwrap(), PREDEFINED_GLOBAL_FARMS[2]);
	});
}

#[test]
fn destroy_global_farm_fail_when_farm_is_healthy() {
	//farm with undistributed rewards and liq. pools
	predefined_test_ext().execute_with(|| {
		let farm_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();
		assert!(!Tokens::free_balance(PREDEFINED_GLOBAL_FARMS[2].reward_currency, &farm_account).is_zero());

		assert_noop!(
			LiquidityMining::destroy_global_farm(Origin::signed(GC), GC_FARM),
			warehouse_liquidity_mining::Error::<Test>::GlobalFarmIsNotEmpty
		);

		assert_eq!(WarehouseLM::global_farm(GC_FARM).unwrap(), PREDEFINED_GLOBAL_FARMS[2]);
	});
}
