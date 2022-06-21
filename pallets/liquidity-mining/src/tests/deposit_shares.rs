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
fn deposit_shares_should_work() {
	predefined_test_ext().execute_with(|| {
		let farm_id = GC_FARM;

		let pallet_account = LiquidityMining::account_id();
		let bsx_tkn1_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(BSX_TKN1_ASSET_PAIR)).unwrap().0);

		set_block_number(1_800); //18-th period

		let bsx_tkn1_alice_shares = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 50, 0).unwrap();
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 0);

		//TEMP
		assert_eq!(WarehouseLM::global_farm(GC_FARM).unwrap().total_shares_z, 0);
		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID))
				.unwrap()
				.entries_count,
			0
		);

		let deposited_amount = 50;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_DEPOSIT_IDS[0],
		})]);

		assert_eq!(WarehouseLM::global_farm(GC_FARM).unwrap().total_shares_z, 12_500);

		let yield_farm = PREDEFINED_YIELD_FARMS.with(|v| v[0].clone());
		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				total_shares: 50,
				total_valued_shares: 2500,
				entries_count: 1,
				..yield_farm
			}
		);

		//TODO: Dani - check if the NFT is retrieved in instances of NFT pallet.

		//check if shares was transferred from extrinsic caller
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_shares - deposited_amount
		);

		//check if shares was transferred to liq. mining pallet account
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			deposited_amount
		);
	});
}

#[test]
fn deposit_shares_should_fail_when_amm_shares_balance_is_insufficient() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::deposit_shares(
				Origin::signed(ALICE),
				GC_FARM,
				BSX_TKN1_YIELD_FARM_ID,
				BSX_TKN1_ASSET_PAIR,
				4_000_000
			),
			Error::<Test>::InsufficientAmmSharesBalance
		);
	});
}

#[test]
fn deposit_shares_should_fail_when_called_by_noy_signed_user() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::deposit_shares(Origin::none(), GC_FARM, BSX_TKN1_YIELD_FARM_ID, BSX_TKN1_ASSET_PAIR, 50),
			BadOrigin
		);
	});
}
