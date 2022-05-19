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
use warehouse_liquidity_mining::Deposit;
use warehouse_liquidity_mining::GlobalPool;
use warehouse_liquidity_mining::LiquidityPoolYieldFarm;
use warehouse_liquidity_mining::LoyaltyCurve;

#[test]
fn claim_rewards_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn2_liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN2_LIQ_POOL_ID).unwrap();
		let bsx_tkn1_liq_pool_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account);

		let expected_claimed_rewards = 79_906;

		//claim A1.1  (dep. A1 1-th time)
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: expected_claimed_rewards,
				entered_at: 18,
				updated_at: 25,
			}
		);

		//check if claimed rewards was transfered
		assert_eq!(
			Tokens::free_balance(BSX, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		//check balance on liq. pool account
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_reward_balance - expected_claimed_rewards
		);

		// claim B3.1
		set_block_number(3_056);
		let bsx_tkn2_liq_pool_reward_balance = Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		let expected_claimed_rewards = 2_734;

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[4]
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_NFT_IDS[4]).unwrap(),
			Deposit {
				shares: 87,
				valued_shares: 261,
				accumulated_rpvs: 120,
				accumulated_claimed_rewards: expected_claimed_rewards,
				entered_at: 25,
				updated_at: 30,
			}
		);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 30,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 14,
				liq_pools_count: 2,
				total_shares_z: 703_990,
				accumulated_rewards: 1_039_045,
				paid_accumulated_rewards: 2_116_980,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 30,
				accumulated_rpvs: 140,
				accumulated_rpz: 14,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		//check if claimed rewards was transfered
		assert_eq!(
			Tokens::free_balance(BSX, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account),
			bsx_tkn2_liq_pool_reward_balance + 952_580 - expected_claimed_rewards //952_580 liq. claim from global pool
		);

		//run for log time(longer than planned_yielding_periods) without interaction or claim.
		//planned_yielding_periods = 500; 100 blocks per period
		//claim A1.2
		set_block_number(125_879);
		let bsx_tkn1_liq_pool_reward_banance = Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		let expected_claimed_rewards = 7_477_183;

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 7_557_089,
				entered_at: 18,
				updated_at: 1_258,
			}
		);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 1_258,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 628,
				liq_pools_count: 2,
				total_shares_z: 703_990,
				accumulated_rewards: 293_025_705,
				paid_accumulated_rewards: 142_380_180,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 1_258,
				accumulated_rpvs: 3_140,
				accumulated_rpz: 628,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 227_700,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 30,
				accumulated_rpvs: 140,
				accumulated_rpz: 14,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		//check if claimed rewards was transfered
		assert_eq!(
			Tokens::free_balance(BSX, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_reward_banance + 140_263_200 - expected_claimed_rewards //140_263_200 liq. claim from global pool
		);
	});

	//charlie's farm inncetivize KSM and reward currency is ACA
	//This test check if correct currency is tranfered if rewards and incetvized
	//assts are different, otherwise pool behaviour is the same as in test above.
	predefined_test_ext().execute_with(|| {
		let aca_ksm_assets = AssetPair {
			asset_in: ACA,
			asset_out: KSM,
		};

		let aca_ksm_amm_account = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(aca_ksm_assets)).unwrap().0);

		let ksm_balance_in_amm = 50;
		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), aca_ksm_amm_account, KSM, ksm_balance_in_amm, 0).unwrap();
		Tokens::set_balance(Origin::root(), aca_ksm_amm_account, ACA, 20, 0).unwrap();

		set_block_number(1_800); //period 18

		let expected_claimed_rewards = 159_813; //ACA
		let deposited_amount = 50;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			CHARLIE_FARM,
			aca_ksm_assets,
			deposited_amount
		));

		assert_eq!(
			WarehouseLM::deposit(4294967303).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 0,
				entered_at: 18,
				updated_at: 18,
			}
		);

		set_block_number(2_596); //period 25

		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 4294967303));

		//alice had 0 ACA before claim
		assert_eq!(Tokens::free_balance(ACA, &ALICE), expected_claimed_rewards);
	});
}

#[test]
fn claim_rewards_double_claim_in_the_same_period_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn1_liq_pool_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account);

		//1-th claim should work ok
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			claimed: 79_906,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 79_906,
				entered_at: 18,
				updated_at: 25,
			}
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + 79_906);
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_reward_balance - 79_906
		);

		//second claim should fail
		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), PREDEFINED_NFT_IDS[0]),
			warehouse_liquidity_mining::Error::<Test>::DoubleClaimInThePeriod
		);
	});
}

#[test]
fn claim_rewards_from_canceled_pool_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		//cancel liq. pool before claim test
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn1_liq_pool_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account);

		let expected_claimed_rewards = 79_906;

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: expected_claimed_rewards,
				entered_at: 18,
				updated_at: 25,
			}
		);

		//check if claimed rewards was transfered
		assert_eq!(
			Tokens::free_balance(BSX, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		//check balance on liq. pool account
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_reward_balance - expected_claimed_rewards
		);
	});
}

#[test]
fn claim_rewards_from_removed_pool_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		//cancel liq. pool before removing
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		//remove liq. pool before claim test
		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), PREDEFINED_NFT_IDS[0]),
			warehouse_liquidity_mining::Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn claim_rewards_not_deposit_owner_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_OWNER: u128 = BOB;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(NOT_OWNER), PREDEFINED_NFT_IDS[0]),
			Error::<Test>::NotDepositOwner
		);
	});
}
