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

#[test]
fn redeposit_lp_shares_should_work_when_deposit_already_exists() {
	//Arrange
	predefined_test_ext_with_deposits().execute_with(|| {
		set_block_number(50_000);

		//Act
		assert_ok!(LiquidityMining::redeposit_lp_shares(
			Origin::signed(ALICE),
			EVE_FARM,
			EVE_BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR,
			PREDEFINED_DEPOSIT_IDS[0],
		));

		//Assert
		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, EVE_FARM, EVE_BSX_TKN1_YIELD_FARM_ID))
				.unwrap()
				.entries_count,
			1
		);

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesRedeposited {
			global_farm_id: EVE_FARM,
			yield_farm_id: EVE_BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: 50
		})]);

		set_block_number(800_000);
		//Dave's farm incentivize TKN1 - some balance must be set so `valued_shares` will not be `0`.
		let bsx_tkn1_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(BSX_TKN1_ASSET_PAIR)).unwrap().0);
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, TKN1, 100, 0).unwrap();
		assert_ok!(LiquidityMining::redeposit_lp_shares(
			Origin::signed(ALICE),
			DAVE_FARM,
			DAVE_BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR,
			PREDEFINED_DEPOSIT_IDS[0]
		));

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, DAVE_FARM, DAVE_BSX_TKN1_YIELD_FARM_ID))
				.unwrap()
				.entries_count,
			1
		);

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesRedeposited {
			global_farm_id: DAVE_FARM,
			yield_farm_id: DAVE_BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: 50
		})]);

		let deposit = WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap();

		assert_eq!(deposit.yield_farm_entries.len(), 3);
	})
}

#[test]
fn redeposit_lp_shares_deposit_should_fail_when_asset_pair_has_invalid_asset() {
	let invalid_asset = 9999;
	let bsx_with_invalid_assets = AssetPair {
		asset_in: BSX,
		asset_out: invalid_asset,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::redeposit_lp_shares(
				Origin::signed(ALICE),
				EVE_FARM,
				EVE_BSX_TKN1_YIELD_FARM_ID,
				bsx_with_invalid_assets,
				PREDEFINED_DEPOSIT_IDS[0],
			),
			Error::<Test>::AmmPoolDoesNotExist
		);
	});
}

#[test]
fn redeposit_lp_shares_deposit_should_fail_when_called_by_not_the_deposit_owner() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::redeposit_lp_shares(
				Origin::signed(BOB),
				EVE_FARM,
				EVE_BSX_TKN1_YIELD_FARM_ID,
				BSX_TKN1_ASSET_PAIR,
				PREDEFINED_DEPOSIT_IDS[0],
			),
			Error::<Test>::NotDepositOwner
		);
	});
}

#[test]
fn redeposit_lp_shares_deposit_should_fail_when_called_with_non_known_deposit() {
	let not_known_deposit = 9999;
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::redeposit_lp_shares(
				Origin::signed(BOB),
				EVE_FARM,
				EVE_BSX_TKN1_YIELD_FARM_ID,
				BSX_TKN1_ASSET_PAIR,
				not_known_deposit,
			),
			Error::<Test>::CantFindDepositOwner
		);
	});
}

#[test]
fn redeposit_lp_shares_deposit_should_fail_when_called_by_not_signed_user() {
	let not_known_deposit = 9999;
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::redeposit_lp_shares(
				Origin::none(),
				EVE_FARM,
				EVE_BSX_TKN1_YIELD_FARM_ID,
				BSX_TKN1_ASSET_PAIR,
				not_known_deposit,
			),
			BadOrigin
		);
	});
}
