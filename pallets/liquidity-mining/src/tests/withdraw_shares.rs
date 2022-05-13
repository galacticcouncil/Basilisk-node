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
use pallet_liquidity_mining::GlobalPool;
use pallet_liquidity_mining::LiquidityPoolYieldFarm;
use pallet_liquidity_mining::LoyaltyCurve;
use test_ext::*;

#[test]
fn withdraw_shares_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	let bsx_tkn2_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN2,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		const REWARD_CURRENCY: u32 = BSX;

		let pallet_account = LiquidityMining::account_id();
		let bsx_tkn1_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn2_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN2_LIQ_POOL_ID).unwrap();
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();

		// withdraw 1A
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let bsx_tkn2_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account);
		let global_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &global_pool_account);
		let bsx_tkn1_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account);
		let bsx_tkn2_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account);

		let expected_claimed_rewards = 79_906;
		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				claimed: expected_claimed_rewards,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: 50,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: ALICE,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[0],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[0],
			}),
		]);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 25,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 12,
				liq_pools_count: 2,
				total_shares_z: 691_490,
				accumulated_rewards: 231_650,
				paid_accumulated_rewards: 1_164_400,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 566,
				total_valued_shares: 43_040,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 215_200,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + 50
		);

		//pallet amm shares balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - 50
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account),
			bsx_tkn2_pallet_amm_shares_balance
		);

		//liq pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_bsx_balance - (expected_claimed_rewards + 70_094) //70_094 unclaimable rewards after withdrawn
		);
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account),
			bsx_tkn2_liq_pool_bsx_balance
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &global_pool_account),
			global_pool_bsx_balance + 70_094 //70_094 unclaimable rewards after withdrawn
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[0]), None);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 2, GC_FARM)
		);

		set_block_number(12_800);

		// withdraw 3B
		let bsx_tkn2_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let bsx_tkn2_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account);
		let global_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &global_pool_account);
		let bsx_tkn1_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account);
		let bsx_tkn2_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account);

		let expected_claimed_rewards = 100_324;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[4]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::FarmAccRPZUpdated {
				farm_id: GC_FARM,
				accumulated_rpz: 63,
				total_shares_z: 691_490,
			}),
			mock::Event::LiquidityMining(Event::LiquidityPoolAccRPVSUpdated {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				accumulated_rpvs: 630,
				total_valued_shares: 47_629,
			}),
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: ALICE,
				claimed: expected_claimed_rewards,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN2_SHARE_ID,
				amount: 87,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: ALICE,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[4],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[4],
			}),
		]);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 688_880,
				accumulated_rewards: 11_552_595,
				paid_accumulated_rewards: 25_455_190,
			}
		);

		// this pool should not change
		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 566,
				total_valued_shares: 43_040,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 215_200,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 128,
				accumulated_rpvs: 630,
				accumulated_rpz: 63,
				total_shares: 873,
				total_valued_shares: 47_368,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 473_680,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE),
			bsx_tkn2_alice_amm_shares_balance + 87
		);

		//pallet amm shares balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account),
			bsx_tkn2_pallet_amm_shares_balance - 87
		);

		//liq pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_bsx_balance
		);

		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account),
			(bsx_tkn2_liq_pool_bsx_balance + 24_290_790 - (expected_claimed_rewards + 32_786)) //24_290_790 - liq. pool claim from global pool, 32_786 unclaimable rewards after withdrawn
		);

		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &global_pool_account),
			global_pool_bsx_balance + 32_786 - 24_290_790 //24_290_790 - liq. pool claim from global pool, 32_786 unclaimable rewards after withdrawn
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[4]), None);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 3, GC_FARM)
		);

		// withdraw 3A
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let bsx_tkn2_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account);
		let global_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &global_pool_account);
		let bsx_tkn1_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account);
		let bsx_tkn2_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account);

		let expected_claimed_rewards = 7_472_429;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[6]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::LiquidityPoolAccRPVSUpdated {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				accumulated_rpvs: 315,
				total_valued_shares: 43040,
			}),
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				claimed: expected_claimed_rewards,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: 486,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: ALICE,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[6],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[6],
			}),
		]);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 494_480,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 128,
				accumulated_rpvs: 315,
				accumulated_rpz: 63,
				total_shares: 80,
				total_valued_shares: 4_160,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 20_800,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + 486
		);

		//pallet amm shares balance checks
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - 486
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account),
			bsx_tkn2_pallet_amm_shares_balance
		);

		//liq pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_bsx_balance + 10_975_200 - (expected_claimed_rewards + 2_441_971) //10_975_200 - liq. pool claim from global pool, 2_441_971 unclaimable rewards after withdrawn
		);

		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account),
			bsx_tkn2_liq_pool_bsx_balance
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &global_pool_account),
			global_pool_bsx_balance + 2_441_971 - 10_975_200 //10_975_200 - liq. pool claim from global pool, 2_441_971 unclaimable rewards after withdrawn
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[6]), None);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 1, GC_FARM)
		);

		// withdraw 2A
		let bsx_tkn1_bob_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB);
		let bob_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let bsx_tkn2_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account);
		let global_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &global_pool_account);
		let bsx_tkn1_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account);
		let bsx_tkn2_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account);

		let expected_claimed_rewards = 855_771;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(BOB),
			PREDEFINED_NFT_IDS[1]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: BOB,
				claimed: expected_claimed_rewards,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: BOB,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: 80,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: BOB,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[1],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: BOB,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[1],
			}),
		]);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 473_680,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 128,
				accumulated_rpvs: 315,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_bsx_balance + expected_claimed_rewards
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB),
			bsx_tkn1_bob_amm_shares_balance + 80
		);

		//pallet amm shares balance checks
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 0);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account),
			bsx_tkn2_pallet_amm_shares_balance
		);

		//liq pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_bsx_balance - (expected_claimed_rewards + 267_429) //267_429 unclaimable rewards after withdrawn
		);

		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account),
			bsx_tkn2_liq_pool_bsx_balance
		);

		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &global_pool_account),
			global_pool_bsx_balance + 267_429 //267_429 unclaimable rewards after withdrawn
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[1]), None);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 0, GC_FARM)
		);

		// withdraw 1B
		let bsx_tkn2_bob_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB);
		let bob_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let bsx_tkn2_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account);
		let global_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &global_pool_account);
		let bsx_tkn1_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account);
		let bsx_tkn2_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account);

		let expected_claimed_rewards = 95_999;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(BOB),
			PREDEFINED_NFT_IDS[2]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: BOB,
				claimed: expected_claimed_rewards,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: BOB,
				lp_token: BSX_TKN2_SHARE_ID,
				amount: 25,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: BOB,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[2],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: BOB,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[2],
			}),
		]);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 471_680,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 128,
				accumulated_rpvs: 315,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 128,
				accumulated_rpvs: 630,
				accumulated_rpz: 63,
				total_shares: 848,
				total_valued_shares: 47_168,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 471_680,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_bsx_balance + expected_claimed_rewards
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB),
			bsx_tkn2_bob_amm_shares_balance + 25
		);

		//pallet amm shares balances checks
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 0);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account),
			bsx_tkn2_pallet_amm_shares_balance - 25
		);

		//liq pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_bsx_balance
		);

		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account),
			bsx_tkn2_liq_pool_bsx_balance - (expected_claimed_rewards + 30_001) //30_001 unclaimable rewards after withdrawn
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &global_pool_account),
			global_pool_bsx_balance + 30_001 //30_001 unclaimable rewards after withdrawn
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[2]), None);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 2, GC_FARM)
		);

		// withdraw 4B
		let bsx_tkn2_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let bsx_tkn2_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account);
		let global_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &global_pool_account);
		let bsx_tkn1_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account);
		let bsx_tkn2_liq_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account);

		let expected_claimed_rewards = 295_207;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[5]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: ALICE,
				claimed: expected_claimed_rewards,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN2_SHARE_ID,
				amount: 48,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: ALICE,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[5],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[5],
			}),
		]);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 464_000,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 128,
				accumulated_rpvs: 630,
				accumulated_rpz: 63,
				total_shares: 800,
				total_valued_shares: 46_400,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 464_000,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_bsx_balance + expected_claimed_rewards
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE),
			bsx_tkn2_alice_amm_shares_balance + 48
		);

		//pallet amm shares balances checks
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 0);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account),
			bsx_tkn2_pallet_amm_shares_balance - 48
		);

		//liq pool balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_bsx_balance
		);
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account),
			bsx_tkn2_liq_pool_bsx_balance - (expected_claimed_rewards + 96_473) //96_473 unclaimable rewards after withdrawn
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &global_pool_account),
			global_pool_bsx_balance + 96_473 //96_473 unclaimable rewards after withdrawn
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[5]), None);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 1, GC_FARM)
		);

		// withdraw 2B
		let bsx_tkn2_bob_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB);
		let bob_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let global_pool_bsx_balance = Tokens::free_balance(REWARD_CURRENCY, &global_pool_account);
		let bsx_tkn1_liq_pool_amm_shares_balance = Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account);

		let expected_claimed_rewards = 18_680_461;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(BOB),
			PREDEFINED_NFT_IDS[3]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: BOB,
				claimed: expected_claimed_rewards,
				reward_currency: BSX,
			}),
			mock::Event::System(frame_system::Event::KilledAccount {
				account: 29533360621462889584138678125,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: BOB,
				lp_token: BSX_TKN2_SHARE_ID,
				amount: 800,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: BOB,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[3],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: BOB,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[3],
			}),
		]);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 0,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 128,
				accumulated_rpvs: 630,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_bsx_balance + expected_claimed_rewards
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB),
			bsx_tkn2_bob_amm_shares_balance + 800
		);

		//pallet balances checks - everything should be withdrawn
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 0);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account), 0);

		//liq pool balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_amm_shares_balance
		);
		assert_eq!(Tokens::free_balance(REWARD_CURRENCY, &bsx_tkn2_liq_pool_account), 0);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &global_pool_account),
			global_pool_bsx_balance + 5_911_539 //5_911_539 unclaimable rewards after withdrawn
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[2]), None);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 0, GC_FARM)
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
			LiquidityMining::deposit(4294967303).unwrap(),
			Deposit {
				shares: deposited_amount,
				valued_shares: 2500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 0,
				entered_at: 18,
				updated_at: 18,
			}
		);

		set_block_number(2_596); //period 25

		let aca_ksm_alice_amm_shares_balance = Tokens::free_balance(ACA_KSM_SHARE_ID, &ALICE);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 4294967303));

		//alice had 0 ACA before claim
		assert_eq!(Tokens::free_balance(ACA, &ALICE), expected_claimed_rewards);
		assert_eq!(
			Tokens::free_balance(ACA_KSM_SHARE_ID, &ALICE),
			aca_ksm_alice_amm_shares_balance + deposited_amount
		);
	});
}

#[test]
fn withdraw_shares_from_destroyed_farm_should_work() {
	//this is the case when liq. pools was removed and global pool was destroyed. Only deposits stayed in
	//the storage. In this case only amm shares should be withdrawn

	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	let bsx_tkn2_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN2,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(bsx_tkn1_assets)).unwrap().0);
		let bsx_tkn2_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(bsx_tkn2_assets)).unwrap().0);

		//check if farm and pools exist
		assert!(WarehouseLM::liquidity_pool(GC_FARM, bsx_tkn1_amm_account).is_some());
		assert!(WarehouseLM::liquidity_pool(GC_FARM, bsx_tkn2_amm_account).is_some());
		assert!(LiquidityMining::global_pool(GC_FARM).is_some());

		//cancel all liq. pools in the farm
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

		//remove all liq. pools from farm
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

		//withdraw all undistributed rewards form global pool before destroying
		assert_ok!(LiquidityMining::withdraw_undistributed_rewards(
			Origin::signed(GC),
			GC_FARM
		));

		//destroy farm
		assert_ok!(LiquidityMining::destroy_farm(Origin::signed(GC), GC_FARM));

		//check if farm and pools was removed from storage
		assert!(WarehouseLM::liquidity_pool(GC_FARM, bsx_tkn1_amm_account).is_none());
		assert!(WarehouseLM::liquidity_pool(GC_FARM, bsx_tkn2_amm_account).is_none());
		assert!(LiquidityMining::global_pool(GC_FARM).is_none());

		let pallet_account = LiquidityMining::account_id();

		let test_data = vec![
			(ALICE, 0, 50, 2, BSX_TKN1_LIQ_POOL_ID, BSX_TKN1_SHARE_ID),
			(BOB, 1, 80, 1, BSX_TKN1_LIQ_POOL_ID, BSX_TKN1_SHARE_ID),
			(BOB, 2, 25, 3, BSX_TKN2_LIQ_POOL_ID, BSX_TKN2_SHARE_ID),
			(BOB, 3, 800, 2, BSX_TKN2_LIQ_POOL_ID, BSX_TKN2_SHARE_ID),
			(ALICE, 4, 87, 1, BSX_TKN2_LIQ_POOL_ID, BSX_TKN2_SHARE_ID),
			(ALICE, 5, 48, 0, BSX_TKN2_LIQ_POOL_ID, BSX_TKN2_SHARE_ID),
			(ALICE, 6, 486, 0, BSX_TKN1_LIQ_POOL_ID, BSX_TKN1_SHARE_ID),
		];

		for (caller, nft_id_index, withdrawn_shares, deposits_left, liq_pool_farm_id, lp_token) in test_data {
			let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
			let bsx_tkn2_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account);
			let bsx_tkn1_caller_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &caller);
			let bsx_tkn2_caller_shares_balance = Tokens::free_balance(BSX_TKN2_SHARE_ID, &caller);

			//withdraw
			assert_ok!(LiquidityMining::withdraw_shares(
				Origin::signed(caller),
				PREDEFINED_NFT_IDS[nft_id_index]
			));

			expect_events(vec![
				mock::Event::LiquidityMining(Event::SharesWithdrawn {
					farm_id: GC_FARM,
					who: caller,
					amount: withdrawn_shares,
					liq_pool_farm_id,
					lp_token,
				}),
				mock::Event::Uniques(pallet_uniques::Event::Burned {
					owner: caller,
					class: LIQ_MINING_NFT_CLASS,
					instance: PREDEFINED_NFT_IDS[nft_id_index],
				}),
				mock::Event::NFT(pallet_nft::Event::InstanceBurned {
					owner: caller,
					class_id: LIQ_MINING_NFT_CLASS,
					instance_id: PREDEFINED_NFT_IDS[nft_id_index],
				}),
			]);

			let mut bsx_tkn1_shares_withdrawn = 0;
			let mut bsx_tkn2_shares_withdrawn = 0;

			if liq_pool_farm_id == BSX_TKN1_LIQ_POOL_ID {
				bsx_tkn1_shares_withdrawn = withdrawn_shares;
			} else {
				bsx_tkn2_shares_withdrawn = withdrawn_shares;
			}

			//check pool account shares balance
			assert_eq!(
				Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
				bsx_tkn1_pallet_amm_shares_balance - bsx_tkn1_shares_withdrawn
			);
			assert_eq!(
				Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account),
				bsx_tkn2_pallet_amm_shares_balance - bsx_tkn2_shares_withdrawn
			);

			//check user balances
			assert_eq!(
				Tokens::free_balance(BSX_TKN1_SHARE_ID, &caller),
				bsx_tkn1_caller_amm_shares_balance + bsx_tkn1_shares_withdrawn
			);
			assert_eq!(
				Tokens::free_balance(BSX_TKN2_SHARE_ID, &caller),
				bsx_tkn2_caller_shares_balance + bsx_tkn2_shares_withdrawn
			);

			//check if deposit was removed
			assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[nft_id_index]), None);

			//check if liq. pool meta was updated
			if deposits_left.is_zero() {
				// last deposit should remove liq. pool metadata
				assert!(LiquidityMining::liq_pool_meta(liq_pool_farm_id).is_none());
			} else {
				let assets = if liq_pool_farm_id == BSX_TKN1_LIQ_POOL_ID {
					bsx_tkn1_assets
				} else {
					bsx_tkn2_assets
				};

				assert_eq!(
					LiquidityMining::liq_pool_meta(liq_pool_farm_id).unwrap(),
					(assets, deposits_left, GC_FARM)
				);
			}
		}
	});
}

#[test]
fn withdraw_shares_from_canceled_pool_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		set_block_number(10_000);

		// cancel liq. pool before withdraw test
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let pallet_account = LiquidityMining::account_id();
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		//1-th withdraw
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();
		let liq_pool = WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		let user_reward = 444_230;
		expect_events(vec![
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				claimed: user_reward,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: 50,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[0],
				owner: ALICE,
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[0],
				owner: ALICE,
			}),
		]);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), global_pool);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				total_shares: liq_pool.total_shares - 50,
				total_valued_shares: liq_pool.total_valued_shares - 2500,
				..liq_pool
			}
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[0]), None);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - 50
		);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + 50
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + user_reward);

		let unclaimable_rewards = 168_270;
		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance + unclaimable_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_account),
			liq_pool_bsx_balance - user_reward - unclaimable_rewards
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 2, GC_FARM)
		);

		//2-nd withdraw
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();
		let liq_pool = WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();

		let user_reward = 5_137_714;
		let unclaimable_rewards = 2_055_086;
		let shares_amount = 486;
		let valued_shares_amount = 38_880;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[6]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				claimed: user_reward,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: shares_amount,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[6],
				owner: ALICE,
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[6],
				owner: ALICE,
			}),
		]);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), global_pool);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				total_shares: liq_pool.total_shares - shares_amount,
				total_valued_shares: liq_pool.total_valued_shares - valued_shares_amount,
				..liq_pool
			}
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[6]), None);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - shares_amount
		);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + shares_amount
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + user_reward);

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance + unclaimable_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_account),
			liq_pool_bsx_balance - user_reward - unclaimable_rewards
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 1, GC_FARM)
		);

		//3-th withdraw
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let bsx_tkn1_bob_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB);
		let bob_bsx_balance = Tokens::free_balance(BSX, &BOB);

		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();
		let liq_pool = WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();

		let user_reward = 603_428;
		let unclaimable_rewards = 228_572;
		let shares_amount = 80;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(BOB),
			PREDEFINED_NFT_IDS[1]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::RewardClaimed {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: BOB,
				claimed: user_reward,
				reward_currency: BSX,
			}),
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: BOB,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: shares_amount,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[1],
				owner: BOB,
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[1],
				owner: BOB,
			}),
		]);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), global_pool);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				total_shares: 0,
				total_valued_shares: 0,
				..liq_pool
			}
		);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[1]), None);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - shares_amount
		);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB),
			bsx_tkn1_bob_amm_shares_balance + shares_amount
		);

		assert_eq!(Tokens::free_balance(BSX, &BOB), bob_bsx_balance + user_reward);

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance + unclaimable_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_account),
			liq_pool_bsx_balance - user_reward - unclaimable_rewards
		);

		//Last withdraw should NOT remove pool_metadata because liq. pool can be
		//resumed in the future
		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 0, GC_FARM)
		);
	});
}

#[test]
fn claim_and_withdraw_in_same_period_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn1_liq_pool_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account);
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);

		let claimed_rewards = 79_906;
		//1-th claim should pass ok
		assert_ok!(LiquidityMining::claim_rewards(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RewardClaimed {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			claimed: claimed_rewards,
			reward_currency: BSX,
		})]);

		assert_eq!(
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: claimed_rewards, //1-th claim for this deposit so accumulated claimed == claimed rewards
				entered_at: 18,
				updated_at: 25,
			}
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + claimed_rewards);
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_reward_balance - claimed_rewards
		);

		//withdraw should pass without claiming additional rewards
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_liq_pool_reward_balance = Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account);

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + 50
		);

		expect_events(vec![
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: 50,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: ALICE,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[0],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[0],
			}),
		]);

		//check if balances didn't change after withdraw which should not claim
		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance);
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account),
			bsx_tkn1_liq_pool_reward_balance
		);
	});
}

#[test]
fn withdraw_shares_from_removed_pool_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		set_block_number(10_000);

		//cancel liq. pool before removing
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		//remove liq. pool before test
		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		assert_eq!(WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM), None);

		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		let liq_pool_id_removed: PoolId = BSX_TKN1_LIQ_POOL_ID;
		let pallet_account = LiquidityMining::account_id();
		let globa_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &globa_pool_account);
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		//1-th withdraw
		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[0]
		));

		let shares_amount = 50;

		expect_events(vec![
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: liq_pool_id_removed,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: shares_amount,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: ALICE,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[0],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[0],
			}),
		]);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[0]), None);

		assert_eq!(
			LiquidityMining::liq_pool_meta(liq_pool_id_removed).unwrap(),
			(bsx_tkn1_assets, 2, GC_FARM)
		);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), global_pool);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - shares_amount
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + shares_amount
		);

		//removed liq. pool don't pay rewards, only transfer amm shares
		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance);
		assert_eq!(Tokens::free_balance(BSX, &globa_pool_account), global_pool_bsx_balance);

		//2-nd withdraw
		let bsx_tkn1_alice_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_pallet_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account);
		let shares_amount = 486;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(ALICE),
			PREDEFINED_NFT_IDS[6]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: liq_pool_id_removed,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: shares_amount,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: ALICE,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[6],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[6],
			}),
		]);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[6]), None);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), global_pool);

		assert_eq!(
			LiquidityMining::liq_pool_meta(liq_pool_id_removed).unwrap(),
			(bsx_tkn1_assets, 1, GC_FARM)
		);

		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			bsx_tkn1_pallet_amm_shares_balance - shares_amount
		);
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_amm_shares_balance + shares_amount
		);

		//removed liq. pool don't pay rewards, only transfer amm shares
		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance);
		assert_eq!(Tokens::free_balance(BSX, &globa_pool_account), global_pool_bsx_balance);

		//3-th withdraw
		let bsx_tkn1_bob_amm_shares_balance = Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB);
		let bob_bsx_balance = Tokens::free_balance(BSX, &BOB);
		let shares_amount = 80;

		assert_ok!(LiquidityMining::withdraw_shares(
			Origin::signed(BOB),
			PREDEFINED_NFT_IDS[1]
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::SharesWithdrawn {
				farm_id: GC_FARM,
				liq_pool_farm_id: liq_pool_id_removed,
				who: BOB,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: shares_amount,
			}),
			mock::Event::Uniques(pallet_uniques::Event::Burned {
				owner: BOB,
				class: LIQ_MINING_NFT_CLASS,
				instance: PREDEFINED_NFT_IDS[1],
			}),
			mock::Event::NFT(pallet_nft::Event::InstanceBurned {
				owner: BOB,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[1],
			}),
		]);

		assert_eq!(LiquidityMining::deposit(PREDEFINED_NFT_IDS[1]), None);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), global_pool);

		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 0);
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB),
			bsx_tkn1_bob_amm_shares_balance + shares_amount
		);

		//removed liq. pool don't pay rewards, only transfer amm shares
		assert_eq!(Tokens::free_balance(BSX, &BOB), bob_bsx_balance);
		assert_eq!(Tokens::free_balance(BSX, &globa_pool_account), global_pool_bsx_balance);

		//last withdrawn from removed pool should remove liq. pool metadata
		assert_eq!(LiquidityMining::liq_pool_meta(liq_pool_id_removed), None);
	});
}

#[test]
fn withdraw_shares_pool_metadata_not_found_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		// liq. pool or liq. pool metadata don't exists for this nft id
		// 723_500_752_978_313_215 -> liq. pool id: u32::max(), nft sequence: 168_453_145
		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), 723_500_752_978_313_215_u128),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn withdraw_shares_invalid_nft_id_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let invalid_nft_id = 684;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), invalid_nft_id),
			Error::<Test>::InvalidNftId
		);
	});
}

#[test]
fn withdraw_shares_nft_not_found_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		//72_334_321_125_861_359_621 -> liq. pool id: 5, nft sequence: 16_841_646_546
		//deposit and nft with this id don't exist
		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), 72_334_321_125_861_359_621),
			Error::<Test>::NftDoesNotExist
		);
	});
}

#[test]
fn withdraw_shares_not_owner_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_FNT_OWNER: u128 = BOB;

		assert_noop!(
			LiquidityMining::withdraw_shares(Origin::signed(NOT_FNT_OWNER), PREDEFINED_NFT_IDS[0]),
			Error::<Test>::NotDepositOwner
		);
	});
}
