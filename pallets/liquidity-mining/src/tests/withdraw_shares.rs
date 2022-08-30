// This file is part of Basilisk-node

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
use sp_runtime::traits::One;
use test_ext::*;
use warehouse_liquidity_mining::YieldFarmEntry;
use warehouse_liquidity_mining::{GlobalFarmData, YieldFarmId};

#[test]
fn withdraw_shares_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arrange
		const REWARD_CURRENCY: u32 = BSX;

		let pallet_account = LiquidityMining::account_id_for_all_lp_shares();
		let bsx_tkn1_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();
		let bsx_tkn2_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN2_YIELD_FARM_ID).unwrap();
		let global_farm_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();

		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let bsx_tkn2_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account);
		let global_farm_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &global_farm_account);
		let bsx_tkn2_yield_farm_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_yield_farm_account);
		let bsx_tkn1_yield_farm_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_yield_farm_account);
		let alice_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);

		let expected_claimed_rewards = 23_306;
		let withdrawn_amount = 50;

		let deposit_valued_shares = WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0])
			.unwrap()
			.yield_farm_entries[0]
			.valued_shares;

		let yield_farm_before_withdraw =
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap();

		//Act
		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR
		));

		//Assert
		has_event(
			crate::Event::RewardClaimed {
				global_farm_id: GC_FARM,
				yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
				who: ALICE,
				claimed: expected_claimed_rewards,
				reward_currency: BSX,
				deposit_id: PREDEFINED_DEPOSIT_IDS[0],
			}
			.into(),
		);
		has_event(
			crate::Event::SharesWithdrawn {
				global_farm_id: GC_FARM,
				yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: withdrawn_amount,
				deposit_id: PREDEFINED_DEPOSIT_IDS[0],
			}
			.into(),
		);
		has_event(
			crate::Event::DepositDestroyed {
				who: ALICE,
				deposit_id: PREDEFINED_DEPOSIT_IDS[0],
			}
			.into(),
		);

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
				id: GC_FARM,
				updated_at: 25,
				reward_currency: BSX,
				yield_per_period: Perquintill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: FixedU128::from_inner(3_500_000_000_000_000_000_u128),
				yield_farms_count: (2, 2),
				total_shares_z: 691_490,
				accumulated_rewards: 0,
				paid_accumulated_rewards: 1_283_550,
				state: FarmState::Active,
				min_deposit: 1,
				price_adjustment: One::one()
			}
		);

		let yield_farm = WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap();
		assert_eq!(
			yield_farm.total_shares,
			yield_farm_before_withdraw.total_shares - withdrawn_amount
		);
		assert_eq!(yield_farm.total_valued_shares, 45540 - deposit_valued_shares);
		assert_eq!(yield_farm.entries_count, yield_farm_before_withdraw.entries_count - 1);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + withdrawn_amount
		);

		//Stash shares account balances checks.
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - withdrawn_amount
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account),
			bsx_tkn2_pallet_amm_shares_balance
		);

		//yield farm balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_yield_farm_account),
			bsx_tkn1_yield_farm_bsx_balance - (expected_claimed_rewards + 20_444)
		);
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_yield_farm_account),
			bsx_tkn2_yield_farm_bsx_balance
		);

		//global farm balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &global_farm_account),
			global_farm_bsx_balance + 20_444 //20_444 unclaimable rewards after withdrawn
		);

		assert_eq!(WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]), None);
	});
}

#[test]
fn withdraw_should_work_when_it_is_in_same_period_as_claim() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arrange
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();
		let bsx_tkn1_yield_farm_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account);
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);
		let global_farm_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();
		let global_farm_bsx_balance = Tokens::free_balance(BSX, &global_farm_account);

		let claimed_rewards = 23_306;
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID
		));

		assert_last_event!(crate::Event::RewardClaimed {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			claimed: claimed_rewards,
			reward_currency: BSX,
			deposit_id: PREDEFINED_DEPOSIT_IDS[0],
		}
		.into());

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0])
				.unwrap()
				.yield_farm_entries[0],
			YieldFarmEntry {
				global_farm_id: GC_FARM,
				yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
				valued_shares: 2_500,
				accumulated_rpvs: Zero::zero(),
				accumulated_claimed_rewards: claimed_rewards, //1-th claim for this deposit so accumulated claimed == claimed rewards
				entered_at: 18,
				updated_at: 25,
				_phantom: PhantomData
			}
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + claimed_rewards);
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account),
			bsx_tkn1_yield_farm_reward_balance - claimed_rewards
		);

		//withdraw should pass without claiming additional rewards
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_yield_farm_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account);

		//Act
		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR
		));

		//Assert
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + 50
		);

		has_event(
			crate::Event::SharesWithdrawn {
				global_farm_id: GC_FARM,
				yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: 50,
				deposit_id: PREDEFINED_DEPOSIT_IDS[0],
			}
			.into(),
		);

		has_event(
			crate::Event::DepositDestroyed {
				who: ALICE,
				deposit_id: PREDEFINED_DEPOSIT_IDS[0],
			}
			.into(),
		);

		//check if balances didn't change after withdraw which should not claim
		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance);
		//unclaimable rewards are transferred from liq. pool account to global pool account
		let unclaimable_rewards = 20_444;
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account),
			bsx_tkn1_yield_farm_reward_balance - unclaimable_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &global_farm_account),
			global_farm_bsx_balance + unclaimable_rewards
		);
	});
}

#[test]
fn withdraw_shares_from_removed_pool_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arrange
		set_block_number(10_000);

		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR
		));

		assert_ok!(LiquidityMining::destroy_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR
		));

		assert!(WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID))
			.unwrap()
			.is_deleted());

		let global_farm = WarehouseLM::global_farm(GC_FARM).unwrap();

		let yield_farm_id_removed: YieldFarmId = BSX_TKN1_YIELD_FARM_ID;
		let pallet_account = LiquidityMining::account_id_for_all_lp_shares();
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);

		//Act
		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			yield_farm_id_removed,
			BSX_TKN1_ASSET_PAIR
		));

		//Assert
		let shares_amount = 50;

		has_event(
			crate::Event::SharesWithdrawn {
				global_farm_id: GC_FARM,
				yield_farm_id: yield_farm_id_removed,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: shares_amount,
				deposit_id: PREDEFINED_DEPOSIT_IDS[0],
			}
			.into(),
		);

		has_event(
			crate::Event::DepositDestroyed {
				who: ALICE,
				deposit_id: PREDEFINED_DEPOSIT_IDS[0],
			}
			.into(),
		);

		assert_eq!(WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]), None);

		assert_eq!(WarehouseLM::global_farm(GC_FARM).unwrap(), global_farm);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - shares_amount
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + shares_amount
		);
	});
}

#[test]
fn withdraw_shares_not_owner_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_FNT_OWNER: u128 = BOB;

		assert_noop!(
			LiquidityMining::withdraw_shares(
				Origin::signed(NOT_FNT_OWNER),
				PREDEFINED_DEPOSIT_IDS[0],
				BSX_TKN1_YIELD_FARM_ID,
				BSX_TKN1_ASSET_PAIR
			),
			Error::<Test>::NotDepositOwner
		);
	});
}

#[test]
fn withdraw_shares_should_not_work_when_global_farm_is_not_found() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let non_known_farm: u32 = 99999;

		assert_noop!(
			LiquidityMining::withdraw_shares(
				Origin::signed(ALICE),
				PREDEFINED_DEPOSIT_IDS[0],
				non_known_farm,
				BSX_TKN1_ASSET_PAIR
			),
			Error::<Test>::DepositDataNotFound
		);
	});
}
