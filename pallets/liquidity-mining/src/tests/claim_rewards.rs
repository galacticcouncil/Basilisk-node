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
use crate::mock::AccountId;
use pretty_assertions::assert_eq;
use test_ext::*;
use warehouse_liquidity_mining::{DepositData, YieldFarmEntry};

#[test]
fn claim_rewards_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arrange
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();
		let bsx_tkn1_yield_farm_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account);

		let expected_claimed_rewards = 79_906;

		//Act
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID
		));

		//Assert
		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_claim_rewards_of_deposit_yield_farm_entry(PREDEFINED_DEPOSIT_IDS[0], expected_claimed_rewards);

		//check if claimed rewards was transferred
		assert_eq!(
			Tokens::free_balance(BSX, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		//check balance on liq. pool account
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account),
			bsx_tkn1_yield_farm_reward_balance - expected_claimed_rewards
		);
	});
}

#[test]
fn claim_rewards_should_propagate_error_when_warehouse_claims_rewards_fails_due_to_double_claim() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//Arrange
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID
		));

		//Act and assert
		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), PREDEFINED_DEPOSIT_IDS[0], BSX_TKN1_YIELD_FARM_ID),
			warehouse_liquidity_mining::Error::<Test, Instance1>::DoubleClaimInPeriod
		);
	});
}

#[test]
fn claim_rewards_should_fail_when_cannot_find_deposit_owner() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let invalid_deposit_id = 9999u128;
		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), invalid_deposit_id, BSX_TKN1_YIELD_FARM_ID),
			Error::<Test>::CantFindDepositOwner
		);
	});
}

#[test]
fn claim_rewards_should_fail_when_called_by_not_signed_owner() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::claim_rewards(Origin::none(), PREDEFINED_DEPOSIT_IDS[0], BSX_TKN1_YIELD_FARM_ID),
			BadOrigin
		);
	});
}

#[test]
fn claim_rewards_should_fail_when_claimed_by_non_deposit_owner() {
	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_OWNER: u128 = BOB;

		assert_noop!(
			LiquidityMining::claim_rewards(
				Origin::signed(NOT_OWNER),
				PREDEFINED_DEPOSIT_IDS[0],
				BSX_TKN1_YIELD_FARM_ID
			),
			Error::<Test>::NotDepositOwner
		);
	});
}

#[test]
fn claim_rewards_should_fail_when_double_claim_happens() {
	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			claimed: 79_906,
			reward_currency: BSX,
		})]);

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), PREDEFINED_DEPOSIT_IDS[0], BSX_TKN1_YIELD_FARM_ID),
			warehouse_liquidity_mining::Error::<Test, Instance1>::DoubleClaimInPeriod
		);
	});
}

fn create_deposit_data(
	shares: Balance,
	amm_pool_id: AccountId,
	yield_farms: Vec<YieldFarmEntry<Test, Instance1>>,
) -> DepositData<Test, Instance1> {
	let mut deposit_data = DepositData::<Test, Instance1>::new(shares, amm_pool_id);
	for farm in yield_farms {
		assert_ok!(deposit_data.add_yield_farm_entry(farm));
	}
	deposit_data
}

fn assert_claim_rewards_of_deposit_yield_farm_entry(deposit_id: u128, expected_claimed_rewards: Balance) {
	let yield_farm_entry: &YieldFarmEntry<Test, Instance1> =
		&WarehouseLM::deposit(deposit_id).unwrap().yield_farm_entries[0];

	assert_eq!(yield_farm_entry.accumulated_claimed_rewards, expected_claimed_rewards);
}
