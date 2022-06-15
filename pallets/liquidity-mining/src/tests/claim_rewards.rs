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
use warehouse_liquidity_mining::LoyaltyCurve;
use warehouse_liquidity_mining::YieldFarmData;
use warehouse_liquidity_mining::{DepositData, YieldFarmEntry};

#[test]
fn claim_rewards_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();
		let bsx_tkn2_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN2_YIELD_FARM_ID).unwrap();
		let bsx_tkn1_yield_farm_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account);

		let expected_claimed_rewards = 79_906;

		//claim A1.1  (dep. A1 1-th time)
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap(),
			DepositData {
				shares: 50,
				amm_pool_id: BSX_TKN1_AMM,
				yield_farm_entries: vec![YieldFarmEntry {
					global_farm_id: GC_FARM,
					yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
					valued_shares: 2_500,
					accumulated_rpvs: 0,
					accumulated_claimed_rewards: expected_claimed_rewards,
					entered_at: 18,
					updated_at: 25,
				}],
			}
		);

		//check if claimed rewards was transfered
		assert_eq!(
			Tokens::free_balance(BSX, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		//check balance on liq. pool account
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account),
			bsx_tkn1_yield_farm_reward_balance - expected_claimed_rewards
		);

		// claim B3.1
		set_block_number(3_056);
		let bsx_tkn2_yield_farm_reward_balance = Tokens::free_balance(BSX, &bsx_tkn2_yield_farm_account);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		let expected_claimed_rewards = 2_734;

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[4],
			BSX_TKN2_YIELD_FARM_ID
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN2_YIELD_FARM_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[4]).unwrap(),
			DepositData {
				shares: 87,
				amm_pool_id: BSX_TKN2_AMM,
				yield_farm_entries: vec![YieldFarmEntry {
					global_farm_id: GC_FARM,
					yield_farm_id: BSX_TKN2_YIELD_FARM_ID,
					valued_shares: 261,
					accumulated_rpvs: 120,
					accumulated_claimed_rewards: expected_claimed_rewards,
					entered_at: 25,
					updated_at: 30,
				}],
			}
		);

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
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
				yield_farms_count: (2, 2),
				total_shares_z: 703_990,
				accumulated_rewards: 1_039_045,
				paid_accumulated_rewards: 2_116_980,
				state: GlobalFarmState::Active
			}
		);

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN2_AMM, GC_FARM, BSX_TKN2_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				id: BSX_TKN2_YIELD_FARM_ID,
				updated_at: 30,
				accumulated_rpvs: 140,
				accumulated_rpz: 14,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				multiplier: FixedU128::from(10_u128),
				state: YieldFarmState::Active,
				entries_count: 4
			},
		);

		//check if claimed rewards was transferred
		assert_eq!(
			Tokens::free_balance(BSX, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn2_yield_farm_account),
			bsx_tkn2_yield_farm_reward_balance + 952_580 - expected_claimed_rewards //952_580 liq. claim from global pool
		);

		//run for log time(longer than planned_yielding_periods) without interaction or claim.
		//planned_yielding_periods = 500; 100 blocks per period
		//claim A1.2
		set_block_number(125_879);
		let bsx_tkn1_yield_farm_reward_banance = Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		let expected_claimed_rewards = 7_477_183;

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap(),
			DepositData {
				shares: 50,
				amm_pool_id: BSX_TKN1_AMM,
				yield_farm_entries: vec![YieldFarmEntry {
					global_farm_id: GC_FARM,
					yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
					valued_shares: 2_500,
					accumulated_rpvs: 0,
					accumulated_claimed_rewards: 7_557_089,
					entered_at: 18,
					updated_at: 1_258,
				}],
			}
		);

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
				id: GC_FARM,
				updated_at: 1_258,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				yield_farms_count: (2, 2),
				accumulated_rpz: 628,
				total_shares_z: 703_990,
				accumulated_rewards: 293_025_705,
				paid_accumulated_rewards: 142_380_180,
				state: GlobalFarmState::Active
			}
		);

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				id: BSX_TKN1_YIELD_FARM_ID,
				updated_at: 1_258,
				accumulated_rpvs: 3_140,
				accumulated_rpz: 628,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				multiplier: FixedU128::from(5_u128),
				state: YieldFarmState::Active,
				entries_count: 3
			},
		);

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN2_AMM, GC_FARM, BSX_TKN2_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				id: BSX_TKN2_YIELD_FARM_ID,
				updated_at: 30,
				accumulated_rpvs: 140,
				accumulated_rpz: 14,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				multiplier: FixedU128::from(10_u128),
				state: YieldFarmState::Active,
				entries_count: 4
			},
		);

		//check if claimed rewards was transfered
		assert_eq!(
			Tokens::free_balance(BSX, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account),
			bsx_tkn1_yield_farm_reward_banance + 140_263_200 - expected_claimed_rewards //140_263_200 liq. claim from global pool
		);
	});

	//charlie's farm incetive KSM and reward currency is ACA
	//This test check if correct currency is transferred if rewards and incentivized
	//assets are different, otherwise pool behaviour is the same as in test above.
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
			ACA_KSM_YIELD_FARM_ID,
			aca_ksm_assets,
			deposited_amount
		));

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap(),
			DepositData {
				shares: 50,
				amm_pool_id: ACA_KSM_AMM,
				yield_farm_entries: vec![YieldFarmEntry {
					global_farm_id: CHARLIE_FARM,
					yield_farm_id: ACA_KSM_YIELD_FARM_ID,
					valued_shares: 2500,
					accumulated_rpvs: 0,
					accumulated_claimed_rewards: 0,
					entered_at: 18,
					updated_at: 18,
				}],
			}
		);

		set_block_number(2_596); //period 25

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			ACA_KSM_YIELD_FARM_ID
		));

		//alice had 0 ACA before claim
		assert_eq!(Tokens::free_balance(ACA, &ALICE), expected_claimed_rewards);
	});
}

#[test]
fn claim_rewards_deposit_with_multiple_entries_should_work() {
	let bsx_tkn1_assets: AssetPair = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		//predefined_deposit[0] - GC_FARM, BSX_TKN1_AMM
		set_block_number(50_000);
		assert_ok!(LiquidityMining::redeposit_lp_shares(
			Origin::signed(ALICE),
			EVE_FARM,
			EVE_BSX_TKN1_YIELD_FARM_ID,
			bsx_tkn1_assets,
			PREDEFINED_DEPOSIT_IDS[0]
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesRedeposited {
			farm_id: EVE_FARM,
			yield_farm_id: EVE_BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: 50,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_DEPOSIT_IDS[0],
		})]);

		set_block_number(800_000);
		//Dave's farm incentivize TKN1 - some balance must be set so `valued_shares` will not be `0`.
		let bsx_tkn1_amm_account = AMM_POOLS.with(|v| {
			v.borrow()
				.get(&asset_pair_to_map_key(AssetPair {
					asset_in: BSX,
					asset_out: TKN1,
				}))
				.unwrap()
				.0
		});
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, TKN1, 100, 0).unwrap();
		assert_ok!(LiquidityMining::redeposit_lp_shares(
			Origin::signed(ALICE),
			DAVE_FARM,
			DAVE_BSX_TKN1_YIELD_FARM_ID,
			bsx_tkn1_assets,
			PREDEFINED_DEPOSIT_IDS[0]
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesRedeposited {
			farm_id: DAVE_FARM,
			yield_farm_id: DAVE_BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: 50,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_DEPOSIT_IDS[0],
		})]);

		let deposit = WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap();

		assert_eq!(
			deposit.yield_farm_entries,
			vec![
				YieldFarmEntry {
					global_farm_id: GC_FARM,
					valued_shares: 2_500,
					yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 0,
					accumulated_rpvs: 0,
					entered_at: 18,
					updated_at: 18
				},
				YieldFarmEntry {
					global_farm_id: DAVE_FARM,
					valued_shares: 5_000,
					yield_farm_id: DAVE_BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 0,
					accumulated_rpvs: 0,
					entered_at: 800,
					updated_at: 800
				},
				YieldFarmEntry {
					global_farm_id: EVE_FARM,
					valued_shares: 4_000,
					yield_farm_id: EVE_BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 0,
					accumulated_rpvs: 0,
					entered_at: 50,
					updated_at: 50
				},
			]
		);

		set_block_number(1_000_000);

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			EVE_BSX_TKN1_YIELD_FARM_ID
		));

		assert_noop!(
			LiquidityMining::claim_rewards(
				Origin::signed(ALICE),
				PREDEFINED_DEPOSIT_IDS[0],
				EVE_BSX_TKN1_YIELD_FARM_ID
			),
			warehouse_liquidity_mining::Error::<Test>::DoubleClaimInThePeriod
		);

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID
		));

		let deposit = WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap();
		assert_eq!(
			deposit.yield_farm_entries,
			vec![
				YieldFarmEntry {
					global_farm_id: GC_FARM,
					valued_shares: 2_500,
					yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 62_177_603,
					accumulated_rpvs: 0,
					entered_at: 18,
					updated_at: 10_000
				},
				YieldFarmEntry {
					global_farm_id: DAVE_FARM,
					valued_shares: 5_000,
					yield_farm_id: DAVE_BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 0,
					accumulated_rpvs: 0,
					entered_at: 800,
					updated_at: 800
				},
				YieldFarmEntry {
					global_farm_id: EVE_FARM,
					valued_shares: 4_000,
					yield_farm_id: EVE_BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 7_619_047,
					accumulated_rpvs: 0,
					entered_at: 50,
					updated_at: 1_000
				},
			]
		);

		//Same period different block.
		set_block_number(1_000_050);
		assert_noop!(
			LiquidityMining::claim_rewards(
				Origin::signed(ALICE),
				PREDEFINED_DEPOSIT_IDS[0],
				EVE_BSX_TKN1_YIELD_FARM_ID
			),
			warehouse_liquidity_mining::Error::<Test>::DoubleClaimInThePeriod
		);

		assert_noop!(
			LiquidityMining::claim_rewards(
				Origin::signed(ALICE),
				PREDEFINED_DEPOSIT_IDS[0],
				EVE_BSX_TKN1_YIELD_FARM_ID
			),
			warehouse_liquidity_mining::Error::<Test>::DoubleClaimInThePeriod
		);

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			DAVE_BSX_TKN1_YIELD_FARM_ID
		));

		let deposit = WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap();
		assert_eq!(
			deposit.yield_farm_entries,
			vec![
				YieldFarmEntry {
					global_farm_id: GC_FARM,
					valued_shares: 2_500,
					yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 62_177_603,
					accumulated_rpvs: 0,
					entered_at: 18,
					updated_at: 10_000
				},
				YieldFarmEntry {
					global_farm_id: DAVE_FARM,
					valued_shares: 5_000,
					yield_farm_id: DAVE_BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 8_333_333,
					accumulated_rpvs: 0,
					entered_at: 800,
					updated_at: 1_000
				},
				YieldFarmEntry {
					global_farm_id: EVE_FARM,
					valued_shares: 4_000,
					yield_farm_id: EVE_BSX_TKN1_YIELD_FARM_ID,
					accumulated_claimed_rewards: 7_619_047,
					accumulated_rpvs: 0,
					entered_at: 50,
					updated_at: 1_000
				},
			]
		);
	});
}

#[test]
fn claim_rewards_double_claim_in_the_same_period_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();
		let bsx_tkn1_yield_farm_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account);

		//1-th claim should work ok
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			claimed: 79_906,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap(),
			DepositData {
				shares: 50,
				amm_pool_id: BSX_TKN1_AMM,
				yield_farm_entries: vec![YieldFarmEntry {
					global_farm_id: GC_FARM,
					yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
					valued_shares: 2_500,
					accumulated_rpvs: 0,
					accumulated_claimed_rewards: 79_906,
					entered_at: 18,
					updated_at: 25,
				}],
			}
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + 79_906);
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account),
			bsx_tkn1_yield_farm_reward_balance - 79_906
		);

		//second claim should fail
		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), PREDEFINED_DEPOSIT_IDS[0], BSX_TKN1_YIELD_FARM_ID),
			warehouse_liquidity_mining::Error::<Test>::DoubleClaimInThePeriod
		);
	});
}

#[test]
fn claim_rewards_from_stopped_yield_farm_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		//cancel liq. pool before claim test
		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();
		let bsx_tkn1_yield_farm_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account);

		let expected_claimed_rewards = 79_906;

		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_DEPOSIT_IDS[0],
			BSX_TKN1_YIELD_FARM_ID
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			claimed: expected_claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			WarehouseLM::deposit(PREDEFINED_DEPOSIT_IDS[0]).unwrap(),
			DepositData {
				shares: 50,
				amm_pool_id: BSX_TKN1_AMM,
				yield_farm_entries: vec![YieldFarmEntry {
					global_farm_id: GC_FARM,
					yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
					valued_shares: 2_500,
					accumulated_rpvs: 0,
					accumulated_claimed_rewards: expected_claimed_rewards,
					entered_at: 18,
					updated_at: 25,
				}],
			}
		);

		//check if claimed rewards was transfered
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
fn claim_rewards_from_destroyed_yield_farm_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		//cancel liq. pool before removing
		assert_ok!(LiquidityMining::stop_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		//remove liq. pool before claim test
		assert_ok!(LiquidityMining::destroy_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_YIELD_FARM_ID,
			bsx_tkn1_assets
		));

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), PREDEFINED_DEPOSIT_IDS[0], BSX_TKN1_YIELD_FARM_ID),
			warehouse_liquidity_mining::Error::<Test>::YieldFarmNotFound
		);
	});
}

#[test]
fn claim_rewards_not_deposit_owner_should_not_work() {
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
fn claim_rewards_should_fail_on_double_claim() {
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
			warehouse_liquidity_mining::Error::<Test>::DoubleClaimInThePeriod
		);
	});
}
