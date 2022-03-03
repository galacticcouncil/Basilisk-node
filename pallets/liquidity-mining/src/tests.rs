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
use crate::mock::{
	asset_pair_to_map_key, set_block_number, BlockNumber, Event as TestEvent, ExtBuilder, LiquidityMining, Origin,
	Test, Tokens, ACA, ACA_FARM, ACA_KSM_AMM, ACA_KSM_SHARE_ID, ACCOUNT_WITH_1M, ALICE, AMM_POOLS, BOB, BSX,
	BSX_ACA_AMM, BSX_ACA_LM_POOL, BSX_ACA_SHARE_ID, BSX_DOT_AMM, BSX_DOT_LM_POOL, BSX_DOT_SHARE_ID, BSX_ETH_AMM,
	BSX_ETH_SHARE_ID, BSX_FARM, BSX_HDX_AMM, BSX_HDX_SHARE_ID, BSX_KSM_AMM, BSX_KSM_LM_POOL, BSX_KSM_SHARE_ID,
	BSX_TKN1_AMM, BSX_TKN1_SHARE_ID, BSX_TKN2_AMM, BSX_TKN2_SHARE_ID, CHARLIE, DOT, ETH, GC, GC_FARM, HDX,
	INITIAL_BALANCE, KSM, KSM_DOT_AMM, KSM_DOT_SHARE_ID, KSM_FARM, LIQ_MINING_NFT_CLASS, TKN1, TKN2, TREASURY,
};

use frame_support::{assert_err, assert_noop, assert_ok};
use primitives::Balance;

use sp_arithmetic::traits::CheckedSub;
use sp_runtime::traits::BadOrigin;

use std::cmp::Ordering;

const ALICE_FARM: u32 = BSX_FARM;
const BOB_FARM: u32 = KSM_FARM;
const CHARLIE_FARM: u32 = ACA_FARM;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| {
		migration::init_nft_class::<Test>();
		set_block_number(1)
	});
	ext
}

const PREDEFINED_GLOBAL_POOLS: [GlobalPool<Test>; 4] = [
	GlobalPool {
		id: ALICE_FARM,
		updated_at: 0,
		reward_currency: BSX,
		yield_per_period: Permill::from_percent(20),
		planned_yielding_periods: 300_u64,
		blocks_per_period: 1_000_u64,
		owner: ALICE,
		incentivized_asset: BSX,
		max_reward_per_period: 333_333_333,
		accumulated_rpz: 0,
		liq_pools_count: 0,
		paid_accumulated_rewards: 0,
		total_shares_z: 0,
		accumulated_rewards: 0,
	},
	GlobalPool {
		id: BOB_FARM,
		updated_at: 0,
		reward_currency: KSM,
		yield_per_period: Permill::from_percent(38),
		planned_yielding_periods: 5_000_u64,
		blocks_per_period: 10_000_u64,
		owner: BOB,
		incentivized_asset: BSX,
		max_reward_per_period: 200_000,
		accumulated_rpz: 0,
		liq_pools_count: 0,
		paid_accumulated_rewards: 0,
		total_shares_z: 0,
		accumulated_rewards: 0,
	},
	GlobalPool {
		id: GC_FARM,
		updated_at: 0,
		reward_currency: BSX,
		yield_per_period: Permill::from_percent(50),
		planned_yielding_periods: 500_u64,
		blocks_per_period: 100_u64,
		owner: GC,
		incentivized_asset: BSX,
		max_reward_per_period: 60_000_000,
		accumulated_rpz: 0,
		liq_pools_count: 2,
		paid_accumulated_rewards: 0,
		total_shares_z: 0,
		accumulated_rewards: 0,
	},
	GlobalPool {
		id: CHARLIE_FARM,
		updated_at: 0,
		reward_currency: ACA,
		yield_per_period: Permill::from_percent(50),
		planned_yielding_periods: 500_u64,
		blocks_per_period: 100_u64,
		owner: CHARLIE,
		incentivized_asset: KSM,
		max_reward_per_period: 60_000_000,
		accumulated_rpz: 0,
		liq_pools_count: 2,
		paid_accumulated_rewards: 0,
		total_shares_z: 0,
		accumulated_rewards: 0,
	},
];

const BSX_TKN1_LIQ_POOL_ID: u32 = 5;
const BSX_TKN2_LIQ_POOL_ID: u32 = 6;
const ACA_KSM_LIQ_POOL_ID: u32 = 7;

thread_local! {
	static PREDEFINED_LIQ_POOLS: [LiquidityPoolYieldFarm<Test>; 3] = [
		LiquidityPoolYieldFarm {
			id: BSX_TKN1_LIQ_POOL_ID,
			updated_at: 0,
			total_shares: 0,
			total_valued_shares: 0,
			accumulated_rpvs: 0,
			accumulated_rpz: 0,
			loyalty_curve: Some(LoyaltyCurve::default()),
			stake_in_global_pool: 0,
			multiplier: FixedU128::from(5),
			canceled: false,
		},
		LiquidityPoolYieldFarm {
			id: BSX_TKN2_LIQ_POOL_ID,
			updated_at: 0,
			total_shares: 0,
			total_valued_shares: 0,
			accumulated_rpvs: 0,
			accumulated_rpz: 0,
			loyalty_curve: Some(LoyaltyCurve::default()),
			stake_in_global_pool: 0,
			multiplier: FixedU128::from(10),
			canceled: false,
		},
		LiquidityPoolYieldFarm {
			id: ACA_KSM_LIQ_POOL_ID,
			updated_at: 0,
			total_shares: 0,
			total_valued_shares: 0,
			accumulated_rpvs: 0,
			accumulated_rpz: 0,
			loyalty_curve: Some(LoyaltyCurve::default()),
			stake_in_global_pool: 0,
			multiplier: FixedU128::from(10),
			canceled: false,
		},
	]
}

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			100_000_000_000,
			PREDEFINED_GLOBAL_POOLS[0].planned_yielding_periods,
			PREDEFINED_GLOBAL_POOLS[0].blocks_per_period,
			PREDEFINED_GLOBAL_POOLS[0].incentivized_asset,
			PREDEFINED_GLOBAL_POOLS[0].reward_currency,
			ALICE,
			PREDEFINED_GLOBAL_POOLS[0].yield_per_period,
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			1_000_000_000,
			PREDEFINED_GLOBAL_POOLS[1].planned_yielding_periods,
			PREDEFINED_GLOBAL_POOLS[1].blocks_per_period,
			PREDEFINED_GLOBAL_POOLS[1].incentivized_asset,
			PREDEFINED_GLOBAL_POOLS[1].reward_currency,
			BOB,
			PREDEFINED_GLOBAL_POOLS[1].yield_per_period,
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			30_000_000_000,
			PREDEFINED_GLOBAL_POOLS[2].planned_yielding_periods,
			PREDEFINED_GLOBAL_POOLS[2].blocks_per_period,
			PREDEFINED_GLOBAL_POOLS[2].incentivized_asset,
			PREDEFINED_GLOBAL_POOLS[2].reward_currency,
			GC,
			PREDEFINED_GLOBAL_POOLS[2].yield_per_period,
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			30_000_000_000,
			PREDEFINED_GLOBAL_POOLS[3].planned_yielding_periods,
			PREDEFINED_GLOBAL_POOLS[3].blocks_per_period,
			PREDEFINED_GLOBAL_POOLS[3].incentivized_asset,
			PREDEFINED_GLOBAL_POOLS[3].reward_currency,
			CHARLIE,
			PREDEFINED_GLOBAL_POOLS[3].yield_per_period,
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::FarmCreated {
				farm_id: PREDEFINED_GLOBAL_POOLS[0].id,
				owner: PREDEFINED_GLOBAL_POOLS[0].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[0].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[0].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[0].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[0].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[0].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[0].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 187989685649991564771226578797,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 4_000,
				who: 187989685649991564771226578797,
				amount: 1_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::FarmCreated {
				farm_id: PREDEFINED_GLOBAL_POOLS[1].id,
				owner: PREDEFINED_GLOBAL_POOLS[1].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[1].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[1].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[1].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[1].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[1].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[1].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 267217848164255902364770529133,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 1_000,
				who: 267217848164255902364770529133,
				amount: 30_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::FarmCreated {
				farm_id: PREDEFINED_GLOBAL_POOLS[2].id,
				owner: PREDEFINED_GLOBAL_POOLS[2].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[2].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[2].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[2].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[2].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[2].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[2].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 346446010678520239958314479469,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 3_000,
				who: 346446010678520239958314479469,
				amount: 30_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::FarmCreated {
				farm_id: PREDEFINED_GLOBAL_POOLS[3].id,
				owner: PREDEFINED_GLOBAL_POOLS[3].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[3].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[3].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[3].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[3].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[3].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[3].max_reward_per_period,
			}),
		]);

		let amm_mock_data = vec![
			(
				AssetPair {
					asset_in: BSX,
					asset_out: ACA,
				},
				(BSX_ACA_AMM, BSX_ACA_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: KSM,
					asset_out: BSX,
				},
				(BSX_KSM_AMM, BSX_KSM_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: DOT,
				},
				(BSX_DOT_AMM, BSX_DOT_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: ETH,
				},
				(BSX_ETH_AMM, BSX_ETH_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
				(BSX_HDX_AMM, BSX_HDX_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: TKN1,
				},
				(BSX_TKN1_AMM, BSX_TKN1_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: TKN2,
				},
				(BSX_TKN2_AMM, BSX_TKN2_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: KSM,
					asset_out: DOT,
				},
				(KSM_DOT_AMM, KSM_DOT_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: ACA,
					asset_out: KSM,
				},
				(ACA_KSM_AMM, ACA_KSM_SHARE_ID),
			),
		];

		AMM_POOLS.with(|h| {
			let mut hm = h.borrow_mut();
			for (k, v) in amm_mock_data {
				hm.insert(asset_pair_to_map_key(k), v);
			}
		});

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			AssetPair {
				asset_in: BSX,
				asset_out: TKN1,
			},
			PREDEFINED_LIQ_POOLS.with(|v| v[0].multiplier),
			PREDEFINED_LIQ_POOLS.with(|v| v[0].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolAdded {
			farm_id: GC_FARM,
			liq_pool_farm_id: PREDEFINED_LIQ_POOLS.with(|v| v[0].id),
			multiplier: PREDEFINED_LIQ_POOLS.with(|v| v[0].multiplier),
			nft_class: LIQ_MINING_NFT_CLASS,
			loyalty_curve: PREDEFINED_LIQ_POOLS.with(|v| v[0].loyalty_curve.clone()),
			asset_pair: AssetPair {
				asset_in: BSX,
				asset_out: TKN1,
			},
		})]);

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			AssetPair {
				asset_in: BSX,
				asset_out: TKN2,
			},
			PREDEFINED_LIQ_POOLS.with(|v| v[1].multiplier),
			PREDEFINED_LIQ_POOLS.with(|v| v[1].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolAdded {
			farm_id: GC_FARM,
			liq_pool_farm_id: PREDEFINED_LIQ_POOLS.with(|v| v[1].id),
			multiplier: PREDEFINED_LIQ_POOLS.with(|v| v[1].multiplier),
			nft_class: LIQ_MINING_NFT_CLASS,
			loyalty_curve: PREDEFINED_LIQ_POOLS.with(|v| v[1].loyalty_curve.clone()),
			asset_pair: AssetPair {
				asset_in: BSX,
				asset_out: TKN2,
			},
		})]);

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(CHARLIE),
			CHARLIE_FARM,
			AssetPair {
				asset_in: ACA,
				asset_out: KSM,
			},
			PREDEFINED_LIQ_POOLS.with(|v| v[2].multiplier),
			PREDEFINED_LIQ_POOLS.with(|v| v[2].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolAdded {
			farm_id: CHARLIE_FARM,
			liq_pool_farm_id: PREDEFINED_LIQ_POOLS.with(|v| v[2].id),
			multiplier: PREDEFINED_LIQ_POOLS.with(|v| v[2].multiplier),
			nft_class: LIQ_MINING_NFT_CLASS,
			loyalty_curve: PREDEFINED_LIQ_POOLS.with(|v| v[2].loyalty_curve.clone()),
			asset_pair: AssetPair {
				asset_in: ACA,
				asset_out: KSM,
			},
		})]);
	});

	ext
}

//nft_ids for deposits from "predefined_test_ext_with_deposits()"
const PREDEFINED_NFT_IDS: [u128; 7] = [
	4294967301,
	8589934597,
	12884901894,
	17179869190,
	21474836486,
	25769803782,
	30064771077,
];

pub fn predefined_test_ext_with_deposits() -> sp_io::TestExternalities {
	let mut ext = predefined_test_ext();

	ext.execute_with(|| {
		let farm_id = GC_FARM; //global pool

		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		let bsx_tkn2_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN2,
		};

		let pallet_account = LiquidityMining::account_id();
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let bsx_tkn1_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn2_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN2_LIQ_POOL_ID).unwrap();
		let bsx_tkn1_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(bsx_tkn1_assets)).unwrap().0);
		let bsx_tkn2_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(bsx_tkn2_assets)).unwrap().0);

		//DEPOSIT 1:
		set_block_number(1_800); //18-th period

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 50, 0).unwrap();

		let deposited_amount = 50;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn1_assets,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount,
			nft_class: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[0],
		})]);

		// DEPOSIT 2 (deposit in same period):

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 52, 0).unwrap();

		let deposited_amount = 80;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn1_assets,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: BOB,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount,
			nft_class: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[1],
		})]);

		// DEPOSIT 3 (same period, second liq pool yield farm):

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 8, 0).unwrap();

		let deposited_amount = 25;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: BOB,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount,
			nft_class: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[2],
		})]);

		// DEPOSIT 4 (new period):
		set_block_number(2051); //period 20

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 58, 0).unwrap();

		let deposited_amount = 800;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: BOB,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount,
			nft_class: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[3],
		})]);

		// DEPOSIT 5 (same period, second liq pool yield farm):
		set_block_number(2_586); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 3, 0).unwrap();

		let deposited_amount = 87;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: ALICE,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount,
			nft_class: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[4],
		})]);

		// DEPOSIT 6 (same period):
		set_block_number(2_596); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 16, 0).unwrap();

		let deposited_amount = 48;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: ALICE,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount,
			nft_class: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[5],
		})]);

		// DEPOSIT 7 : (same period differen liq poll farm)
		set_block_number(2_596); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 80, 0).unwrap();

		let deposited_amount = 486;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn1_assets,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount,
			nft_class: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[6],
		})]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
				total_shares_z: 703_990,
				accumulated_rewards: 231_650,
				paid_accumulated_rewards: 1_164_400,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 616,
				total_valued_shares: 45_540,
				stake_in_global_pool: 227_700,
				..PREDEFINED_LIQ_POOLS.with(|v| v[0].clone())
			},
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				updated_at: 25,
				accumulated_rpvs: 120,
				accumulated_rpz: 12,
				total_shares: 960,
				total_valued_shares: 47_629,
				stake_in_global_pool: 476_290,
				..PREDEFINED_LIQ_POOLS.with(|v| v[1].clone())
			},
		);

		//liq. pool meta check (nfts count)
		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 3, GC_FARM)
		);

		//liq. pool meta check (nfts count)
		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 4, GC_FARM)
		);

		//shares amount check on pallet account, sum of all deposits grouped by shares id
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 616);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account), 960);

		//reward currency balance check. total_rewards - sum(claimes from global pool)
		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			(30_000_000_000 - 1_164_400)
		);

		//check of claimed amount from global pool (sum of all claims)
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account), 212_400);
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account), 952_000);

		//balance check after transfer amm shares
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE), 3_000_000 - 536);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE), 3_000_000 - 135);

		//balance check after transfer amm shares
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB), 2_000_000 - 80);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB), 2_000_000 - 825);
	});

	ext
}

#[test]
fn get_period_number_should_work() {
	let block_num: BlockNumber = 1_u64;
	let blocks_per_period = 1;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);

	let block_num: BlockNumber = 1_000_u64;
	let blocks_per_period = 1;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1_000
	);

	let block_num: BlockNumber = 23_u64;
	let blocks_per_period = 15;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);

	let block_num: BlockNumber = 843_712_398_u64;
	let blocks_per_period = 13_412_341;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		62
	);

	let block_num: BlockNumber = 843_u64;
	let blocks_per_period = 2_000;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		0
	);

	let block_num: BlockNumber = 10_u64;
	let blocks_per_period = 10;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);
}

#[test]
fn get_period_number_should_not_work() {
	let block_num: BlockNumber = 10_u64;
	assert_err!(
		LiquidityMining::get_period_number(block_num, 0),
		Error::<Test>::Overflow
	);
}

#[test]
fn get_loyalty_multiplier_should_work() {
	let loyalty_curve_1 = LoyaltyCurve::default();
	let loyalty_curve_2 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from(1),
		scale_coef: 50,
	};
	let loyalty_curve_3 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from_inner(123_580_000_000_000_000), // 0.12358
		scale_coef: 23,
	};
	let loyalty_curve_4 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from_inner(0), // 0.12358
		scale_coef: 15,
	};

	let testing_values = vec![
		(
			0,
			FixedU128::from_float(0.5_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.12358_f64),
			FixedU128::from_float(0_f64),
		),
		(
			1,
			FixedU128::from_float(0.504950495_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.1600975_f64),
			FixedU128::from_float(0.0625_f64),
		),
		(
			4,
			FixedU128::from_float(0.5192307692_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.25342_f64),
			FixedU128::from_float(0.2105263158_f64),
		),
		(
			130,
			FixedU128::from_float(0.7826086957_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.8682505882_f64),
			FixedU128::from_float(0.8965517241_f64),
		),
		(
			150,
			FixedU128::from_float(0.8_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.8834817341_f64),
			FixedU128::from_float(0.9090909091_f64),
		),
		(
			180,
			FixedU128::from_float(0.8214285714_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9007011823_f64),
			FixedU128::from_float(0.9230769231_f64),
		),
		(
			240,
			FixedU128::from_float(0.8529411765_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9233549049_f64),
			FixedU128::from_float(0.9411764706_f64),
		),
		(
			270,
			FixedU128::from_float(0.8648648649_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9312025256_f64),
			FixedU128::from_float(0.9473684211_f64),
		),
		(
			280,
			FixedU128::from_float(0.8684210526_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9334730693_f64),
			FixedU128::from_float(0.9491525424_f64),
		),
		(
			320,
			FixedU128::from_float(0.880952381_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.941231312_f64),
			FixedU128::from_float(0.9552238806_f64),
		),
		(
			380,
			FixedU128::from_float(0.8958333333_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9499809926_f64),
			FixedU128::from_float(0.9620253165_f64),
		),
		(
			390,
			FixedU128::from_float(0.8979591837_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9511921065_f64),
			FixedU128::from_float(0.962962963_f64),
		),
		(
			4000,
			FixedU128::from_float(0.987804878_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.994989396_f64),
			FixedU128::from_float(0.99626401_f64),
		),
		(
			4400,
			FixedU128::from_float(0.9888888889_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9954425367_f64),
			FixedU128::from_float(0.9966024915_f64),
		),
		(
			4700,
			FixedU128::from_float(0.9895833333_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.995732022_f64),
			FixedU128::from_float(0.9968186638_f64),
		),
	];

	//Special case: loyalty curve is None
	assert_eq!(
		LiquidityMining::get_loyalty_multiplier(10, None).unwrap(),
		FixedU128::one()
	);

	let precission_delta = FixedU128::from_inner(100_000_000); //0.000_000_000_1
	for (periods, expected_multiplier_1, expected_multiplier_2, expected_multiplier_3, expected_multiplier_4) in
		testing_values.iter()
	{
		//1th curve test
		assert!(is_approx_eq_fixedu128(
			LiquidityMining::get_loyalty_multiplier(*periods, Some(loyalty_curve_1.clone())).unwrap(),
			*expected_multiplier_1,
			precission_delta
		));

		//2nd curve test
		assert!(is_approx_eq_fixedu128(
			LiquidityMining::get_loyalty_multiplier(*periods, Some(loyalty_curve_2.clone())).unwrap(),
			*expected_multiplier_2,
			precission_delta
		));

		//3rd curve test
		assert!(is_approx_eq_fixedu128(
			LiquidityMining::get_loyalty_multiplier(*periods, Some(loyalty_curve_3.clone())).unwrap(),
			*expected_multiplier_3,
			precission_delta
		));

		//4th curve test
		assert!(is_approx_eq_fixedu128(
			LiquidityMining::get_loyalty_multiplier(*periods, Some(loyalty_curve_4.clone())).unwrap(),
			*expected_multiplier_4,
			precission_delta
		));
	}
}

#[test]
fn get_reward_per_period_should_work() {
	let testing_values = vec![
		(
			FixedU128::from_float(0.0008333333333),
			12578954_u128,
			156789_u128,
			10482_u128,
		),
		(
			FixedU128::from_float(0.08333333333),
			1246578_u128,
			4684789_u128,
			103881_u128,
		),
		(FixedU128::from_float(0.03666666667), 3980_u128, 488_u128, 145_u128),
		(
			FixedU128::from_float(0.1666666667),
			9897454_u128,
			1684653_u128,
			1649575_u128,
		),
		(FixedU128::from_float(0.00625), 1687_u128, 28_u128, 10_u128),
		(FixedU128::from_float(0.0125), 3879_u128, 7_u128, 7_u128),
		(
			FixedU128::from_float(0.1333333333),
			35189_u128,
			468787897_u128,
			4691_u128,
		),
		(FixedU128::from_float(0.003111392405), 48954_u128, 161_u128, 152_u128),
		(FixedU128::from_float(0.000375), 54789782_u128, 3_u128, 3_u128),
		(
			FixedU128::from_float(0.1385714286),
			17989865464312_u128,
			59898_u128,
			59898_u128,
		),
		(FixedU128::from_float(0.0375), 2_u128, 7987_u128, 0_u128),
		(FixedU128::from_float(0.07875), 5_u128, 498741_u128, 0_u128),
		(FixedU128::from_float(0.04), 5468_u128, 8798_u128, 218_u128),
		(FixedU128::from_float(0.0), 68797_u128, 789846_u128, 0_u128),
	];

	for (yield_per_period, total_pool_shares_z, max_reward_per_period, expected_reward_per_period) in
		testing_values.iter()
	{
		assert_eq!(
			LiquidityMining::get_global_pool_reward_per_period(
				*yield_per_period,
				*total_pool_shares_z,
				*max_reward_per_period
			)
			.unwrap(),
			*expected_reward_per_period
		);
	}
}

#[test]
fn get_accumulated_rps_should_work() {
	let testing_values = vec![
		(596850065_u128, 107097_u128, 58245794_u128, 596850608_u128),
		(610642940_u128, 380089_u128, 72666449_u128, 610643131_u128),
		(342873091_u128, 328911_u128, 32953786_u128, 342873191_u128),
		(678009825_u128, 130956_u128, 49126054_u128, 678010200_u128),
		(579839575_u128, 349893_u128, 48822879_u128, 579839714_u128),
		(53648392_u128, 191826_u128, 5513773_u128, 53648420_u128),
		(474641194_u128, 224569_u128, 88288774_u128, 474641587_u128),
		(323929643_u128, 117672_u128, 43395220_u128, 323930011_u128),
		(18684290_u128, 293754_u128, 84347520_u128, 18684577_u128),
		(633517462_u128, 417543_u128, 43648027_u128, 633517566_u128),
		(899481210_u128, 217000_u128, 46063156_u128, 899481422_u128),
		(732260582_u128, 120313_u128, 91003576_u128, 732261338_u128),
		(625857089_u128, 349989_u128, 71595913_u128, 625857293_u128),
		(567721341_u128, 220776_u128, 75561456_u128, 567721683_u128),
		(962034430_u128, 196031_u128, 40199198_u128, 962034635_u128),
		(548598381_u128, 457172_u128, 37345481_u128, 548598462_u128),
		(869164975_u128, 172541_u128, 4635196_u128, 869165001_u128),
		(776275145_u128, 419601_u128, 32861993_u128, 776275223_u128),
		(684419217_u128, 396975_u128, 24222103_u128, 684419278_u128),
		(967509392_u128, 352488_u128, 77778911_u128, 967509612_u128),
	];

	for (accumulated_rps_now, total_shares, reward, expected_accumulated_rps) in testing_values.iter() {
		assert_eq!(
			LiquidityMining::get_accumulated_rps(*accumulated_rps_now, *total_shares, *reward).unwrap(),
			*expected_accumulated_rps
		);
	}
}

#[test]
fn get_user_reward_should_work() {
	let testing_values = vec![
		(
			79_u128,
			1733800371_u128,
			259_u128,
			2333894_u128,
			FixedU128::from_inner(456_446_123_846_332_000_u128),
			142447228701_u128,
			169634504185_u128,
		),
		(
			61_u128,
			3117_u128,
			1148_u128,
			34388_u128,
			FixedU128::from_inner(621_924_695_680_678_000_u128),
			2072804_u128,
			1280987_u128,
		),
		(
			0_u128,
			3232645500_u128,
			523_u128,
			1124892_u128,
			FixedU128::from_inner(1_000_000_000_000_u128),
			565781_u128,
			1690671905827_u128,
		),
		(
			159_u128,
			3501142339_u128,
			317_u128,
			3309752_u128,
			FixedU128::from_inner(384_109_209_525_475_000_u128),
			212478410818_u128,
			340698768992_u128,
		),
		(
			352_u128,
			156_u128,
			596_u128,
			2156_u128,
			FixedU128::from_inner(100_703_041_057_143_000_u128),
			1677_u128,
			34231_u128,
		),
		(
			0_u128,
			192208478782_u128,
			4_u128,
			534348_u128,
			FixedU128::from_inner(104_779_339_071_984_000_u128),
			80557375135_u128,
			688276005645_u128,
		),
		(
			138_u128,
			36579085_u128,
			213_u128,
			1870151_u128,
			FixedU128::from_inner(129_927_485_118_411_000_u128),
			354576988_u128,
			2386984236_u128,
		),
		(
			897_u128,
			1_u128,
			970_u128,
			1_u128,
			FixedU128::from_inner(502_367_859_476_566_000_u128),
			35_u128,
			37_u128,
		),
		(
			4_u128,
			38495028244_u128,
			6_u128,
			2568893_u128,
			FixedU128::from_inner(265_364_053_378_152_000_u128),
			20427824566_u128,
			56559663029_u128,
		),
		(
			10_u128,
			13343864050_u128,
			713_u128,
			1959317_u128,
			FixedU128::from_inner(279_442_586_539_696_000_u128),
			2621375291532_u128,
			6759359176301_u128,
		),
		(
			29_u128,
			18429339175_u128,
			833_u128,
			3306140_u128,
			FixedU128::from_inner(554_635_100_856_657_000_u128),
			8218129641066_u128,
			6599055749494_u128,
		),
		(
			224_u128,
			39102822603_u128,
			586_u128,
			1839083_u128,
			FixedU128::from_inner(654_427_828_000_143_000_u128),
			9263569206758_u128,
			4891650736445_u128,
		),
		(
			36_u128,
			55755691086_u128,
			251_u128,
			3521256_u128,
			FixedU128::from_inner(802_407_775_824_621_000_u128),
			9618838494628_u128,
			2368631567606_u128,
		),
		(
			36_u128,
			258339226986_u128,
			77_u128,
			2106922_u128,
			FixedU128::from_inner(743_748_274_128_360_000_u128),
			7877711415708_u128,
			2714194783796_u128,
		),
		(
			383_u128,
			34812134025_u128,
			2491_u128,
			1442758_u128,
			FixedU128::from_inner(130_076_146_093_442_000_u128),
			9545503668738_u128,
			63838473413204_u128,
		),
		(
			117_u128,
			44358629274_u128,
			295_u128,
			2076570_u128,
			FixedU128::from_inner(495_172_207_692_510_000_u128),
			3909796472461_u128,
			3986037461741_u128,
		),
		(
			172_u128,
			64667747645_u128,
			450_u128,
			33468_u128,
			FixedU128::from_inner(326_047_919_016_893_000_u128),
			5861570070642_u128,
			12116063741200_u128,
		),
		(
			37_u128,
			68875501378_u128,
			82_u128,
			230557_u128,
			FixedU128::from_inner(176_816_131_903_196_000_u128),
			548023257587_u128,
			2551374073866_u128,
		),
		(
			41_u128,
			100689735793_u128,
			81_u128,
			2268544_u128,
			FixedU128::from_inner(376_605_306_400_251_000_u128),
			1516809283443_u128,
			2510777879733_u128,
		),
		(
			252_u128,
			16283442689_u128,
			266_u128,
			3797763_u128,
			FixedU128::from_inner(189_489_655_763_324_000_u128),
			43193817533_u128,
			184770582350_u128,
		),
		(
			20_u128,
			205413646819_u128,
			129_u128,
			3184799_u128,
			FixedU128::from_inner(543_081_681_209_601_000_u128),
			12159643178907_u128,
			10230441139565_u128,
		),
		(
			23_u128,
			100000_u128,
			155_u128,
			1210762_u128,
			FixedU128::from_inner(404_726_206_620_574_000_u128),
			4131623_u128,
			7857615_u128,
		),
		(
			11_u128,
			84495025009_u128,
			166_u128,
			468012_u128,
			FixedU128::from_inner(735_133_167_032_114_000_u128),
			9627839308653_u128,
			3468889099730_u128,
		),
		(
			198_u128,
			79130076897_u128,
			571_u128,
			830256_u128,
			FixedU128::from_inner(689_497_061_649_446_000_u128),
			20350862574442_u128,
			9164655277883_u128,
		),
		(
			30_u128,
			68948735954_u128,
			72_u128,
			3278682_u128,
			FixedU128::from_inner(238_786_980_081_793_000_u128),
			691487259752_u128,
			2204356371634_u128,
		),
		(
			54_u128,
			280608075911_u128,
			158_u128,
			0_u128,
			FixedU128::from_inner(504_409_653_378_878_000_u128),
			14720307919780_u128,
			14462931974964_u128,
		),
		(
			193_u128,
			22787841433_u128,
			1696_u128,
			2962625_u128,
			FixedU128::from_inner(623_942_971_029_398_000_u128),
			21370122208415_u128,
			12880000502759_u128,
		),
		(
			193_u128,
			22787841433_u128,
			193_u128,
			2962625_u128,
			FixedU128::from_inner(623_942_971_029_398_000_u128),
			0_u128,
			0_u128,
		),
	];

	for (
		accumulated_rpvs,
		valued_shares,
		accumulated_rpvs_now,
		accumulated_claimed_rewards,
		loyalty_multiplier,
		expected_user_rewards,
		expected_unchaimable_rewards,
	) in testing_values.iter()
	{
		assert_eq!(
			LiquidityMining::get_user_reward(
				*accumulated_rpvs,
				*valued_shares,
				*accumulated_claimed_rewards,
				*accumulated_rpvs_now,
				*loyalty_multiplier
			)
			.unwrap(),
			(*expected_user_rewards, *expected_unchaimable_rewards)
		);
	}
}

#[test]
fn update_global_pool_should_work() {
	let testing_values = vec![
		(
			26_u64,
			2501944769_u128,
			259_u128,
			HDX,
			BSX_FARM,
			0_u128,
			206_u64,
			65192006_u128,
			55563662_u128,
			259_u128,
			55563662_u128,
		),
		(
			188_u64,
			33769603_u128,
			1148_u128,
			BSX,
			BSX_FARM,
			30080406306_u128,
			259_u64,
			1548635_u128,
			56710169_u128,
			1151_u128,
			166663254_u128,
		),
		(
			195_u64,
			26098384286056_u128,
			523_u128,
			ACA,
			KSM_FARM,
			32055_u128,
			326_u64,
			1712797_u128,
			61424428_u128,
			523_u128,
			61456483_u128,
		),
		(
			181_u64,
			9894090144_u128,
			317_u128,
			KSM,
			ACA_FARM,
			36806694280_u128,
			1856_u64,
			19009156_u128,
			52711084_u128,
			320_u128,
			31893047384_u128,
		),
		(
			196_u64,
			26886423482043_u128,
			596_u128,
			ACA,
			KSM_FARM,
			30560755872_u128,
			954_u64,
			78355_u128,
			34013971_u128,
			596_u128,
			93407061_u128,
		),
		(
			68_u64,
			1138057342_u128,
			4_u128,
			ACA,
			KSM_FARM,
			38398062768_u128,
			161_u64,
			55309798233_u128,
			71071995_u128,
			37_u128,
			38469134763_u128,
		),
		(
			161_u64,
			24495534649923_u128,
			213_u128,
			KSM,
			BSX_FARM,
			11116735745_u128,
			448_u64,
			326_u128,
			85963452_u128,
			213_u128,
			86057014_u128,
		),
		(
			27_u64,
			22108444_u128,
			970_u128,
			KSM,
			KSM_FARM,
			8572779460_u128,
			132_u64,
			1874081_u128,
			43974403_u128,
			978_u128,
			240752908_u128,
		),
		(
			97_u64,
			1593208_u128,
			6_u128,
			HDX,
			BSX_FARM,
			18440792496_u128,
			146_u64,
			741803_u128,
			14437690_u128,
			28_u128,
			50786037_u128,
		),
		(
			154_u64,
			27279119649838_u128,
			713_u128,
			BSX,
			BSX_FARM,
			28318566664_u128,
			202_u64,
			508869_u128,
			7533987_u128,
			713_u128,
			31959699_u128,
		),
		(
			104_u64,
			20462312838954_u128,
			833_u128,
			BSX,
			ACA_FARM,
			3852003_u128,
			131_u64,
			1081636_u128,
			75149021_u128,
			833_u128,
			79001024_u128,
		),
		(
			90_u64,
			37650830596054_u128,
			586_u128,
			HDX,
			KSM_FARM,
			27990338179_u128,
			110_u64,
			758482_u128,
			36765518_u128,
			586_u128,
			51935158_u128,
		),
		(
			198_u64,
			318777215_u128,
			251_u128,
			ACA,
			ACA_FARM,
			3615346492_u128,
			582_u64,
			69329_u128,
			12876432_u128,
			251_u128,
			39498768_u128,
		),
		(
			29_u64,
			33478250_u128,
			77_u128,
			BSX,
			ACA_FARM,
			39174031245_u128,
			100_u64,
			1845620_u128,
			26611087_u128,
			80_u128,
			157650107_u128,
		),
		(
			91_u64,
			393922835172_u128,
			2491_u128,
			ACA,
			KSM_FARM,
			63486975129400_u128,
			260_u64,
			109118678233_u128,
			85100506_u128,
			2537_u128,
			18441141721883_u128,
		),
		(
			67_u64,
			1126422_u128,
			295_u128,
			HDX,
			ACA_FARM,
			7492177402_u128,
			229_u64,
			1227791_u128,
			35844776_u128,
			471_u128,
			234746918_u128,
		),
		(
			168_u64,
			28351324279041_u128,
			450_u128,
			ACA,
			KSM_FARM,
			38796364068_u128,
			361_u64,
			1015284_u128,
			35695723_u128,
			450_u128,
			231645535_u128,
		),
		(
			3_u64,
			17631376575792_u128,
			82_u128,
			HDX,
			BSX_FARM,
			20473946880_u128,
			52_u64,
			1836345_u128,
			93293564_u128,
			82_u128,
			183274469_u128,
		),
		(
			49_u64,
			94059_u128,
			81_u128,
			HDX,
			BSX_FARM,
			11126653978_u128,
			132_u64,
			1672829_u128,
			75841904_u128,
			1557_u128,
			214686711_u128,
		),
		(
			38_u64,
			14085_u128,
			266_u128,
			KSM,
			ACA_FARM,
			36115448964_u128,
			400000_u64,
			886865_u128,
			52402278_u128,
			2564373_u128,
			36167851242_u128,
		),
		(
			158_u64,
			762784_u128,
			129_u128,
			BSX,
			ACA_FARM,
			21814882774_u128,
			158_u64,
			789730_u128,
			86085676_u128,
			129_u128,
			86085676_u128,
		),
	];

	for (
		updated_at,
		total_shares_z,
		accumulated_rpz,
		reward_currency,
		id,
		rewards_left_to_distribute,
		now_period,
		reward_per_period,
		accumulated_rewards,
		expected_accumulated_rpz,
		expected_accumulated_rewards,
	) in testing_values.iter()
	{
		let yield_per_period = Permill::from_percent(50);
		let planned_yielding_periods = 100;
		let blocks_per_period = 0;
		let owner = ALICE;
		let incentivized_token = BSX;
		let max_reward_per_period = 10_000_u128;

		let mut global_pool = GlobalPool::new(
			*id,
			*updated_at,
			*reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		global_pool.total_shares_z = *total_shares_z;
		global_pool.accumulated_rewards = *accumulated_rewards;
		global_pool.accumulated_rpz = *accumulated_rpz;
		global_pool.paid_accumulated_rewards = 10;

		let mut ext = new_test_ext();

		ext.execute_with(|| {
			let farm_account_id = LiquidityMining::pool_account_id(*id).unwrap();
			let _ = Tokens::transfer(
				Origin::signed(TREASURY),
				farm_account_id,
				*reward_currency,
				*rewards_left_to_distribute,
			);
			assert_eq!(
				Tokens::free_balance(*reward_currency, &farm_account_id),
				*rewards_left_to_distribute
			);

			LiquidityMining::update_global_pool(&mut global_pool, *now_period, *reward_per_period).unwrap();

			let mut expected_global_pool = GlobalPool::new(
				*id,
				*now_period,
				*reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			expected_global_pool.total_shares_z = *total_shares_z;
			expected_global_pool.paid_accumulated_rewards = 10;
			expected_global_pool.accumulated_rpz = *expected_accumulated_rpz;
			expected_global_pool.accumulated_rewards = *expected_accumulated_rewards;

			assert_eq!(global_pool, expected_global_pool);
		});
	}
}

#[test]
fn claim_from_global_pool_should_work() {
	let testing_values = vec![
		(
			26_u64,
			2501944769_u128,
			259_u128,
			299_u128,
			HDX,
			5556613662_u128,
			0_u128,
			55563662_u128,
			2222546480_u128,
			299_u128,
			3334067182_u128,
			2222546480_u128,
		),
		(
			188_u64,
			33769603_u128,
			1148_u128,
			1151_u128,
			BSX,
			166663254_u128,
			30080406306_u128,
			5671016_u128,
			17013048_u128,
			1151_u128,
			149650206_u128,
			30097419354_u128,
		),
		(
			195_u64,
			26098384286056_u128,
			523_u128,
			823_u128,
			ACA,
			61456483_u128,
			32055_u128,
			61428_u128,
			18428400_u128,
			823_u128,
			43028083_u128,
			18460455_u128,
		),
		(
			181_u64,
			9894090144_u128,
			317_u128,
			320_u128,
			KSM,
			31893047384_u128,
			36806694280_u128,
			527114_u128,
			1581342_u128,
			320_u128,
			31891466042_u128,
			36808275622_u128,
		),
		(
			196_u64,
			26886423482043_u128,
			596_u128,
			5684_u128,
			ACA,
			93407061_u128,
			30560755872_u128,
			3011_u128,
			15319968_u128,
			5684_u128,
			78087093_u128,
			30576075840_u128,
		),
		(
			68_u64,
			1138057342_u128,
			4_u128,
			37_u128,
			ACA,
			38469134763_u128,
			38398062768_u128,
			71071995_u128,
			2345375835_u128,
			37_u128,
			36123758928_u128,
			40743438603_u128,
		),
		(
			161_u64,
			24495534649923_u128,
			213_u128,
			678_u128,
			KSM,
			86057014_u128,
			11116735745_u128,
			85452_u128,
			39735180_u128,
			678_u128,
			46321834_u128,
			11156470925_u128,
		),
		(
			27_u64,
			22108444_u128,
			970_u128,
			978_u128,
			KSM,
			240752908_u128,
			8572779460_u128,
			474403_u128,
			3795224_u128,
			978_u128,
			236957684_u128,
			8576574684_u128,
		),
		(
			97_u64,
			1593208_u128,
			6_u128,
			28_u128,
			HDX,
			50786037_u128,
			18440792496_u128,
			147690_u128,
			3249180_u128,
			28_u128,
			47536857_u128,
			18444041676_u128,
		),
		(
			154_u64,
			27279119649838_u128,
			713_u128,
			876_u128,
			BSX,
			319959699_u128,
			28318566664_u128,
			75987_u128,
			12385881_u128,
			876_u128,
			307573818_u128,
			28330952545_u128,
		),
		(
			104_u64,
			20462312838954_u128,
			833_u128,
			8373_u128,
			BSX,
			790051024_u128,
			3852003_u128,
			7521_u128,
			56708340_u128,
			8373_u128,
			733342684_u128,
			60560343_u128,
		),
		(
			90_u64,
			37650830596054_u128,
			586_u128,
			5886_u128,
			HDX,
			519356158_u128,
			27990338179_u128,
			318_u128,
			1685400_u128,
			5886_u128,
			517670758_u128,
			27992023579_u128,
		),
		(
			198_u64,
			318777215_u128,
			251_u128,
			2591_u128,
			ACA,
			3949876895_u128,
			3615346492_u128,
			28732_u128,
			67232880_u128,
			2591_u128,
			3882644015_u128,
			3682579372_u128,
		),
		(
			29_u64,
			33478250_u128,
			77_u128,
			80_u128,
			BSX,
			157650107_u128,
			39174031245_u128,
			26611087_u128,
			79833261_u128,
			80_u128,
			77816846_u128,
			39253864506_u128,
		),
		(
			91_u64,
			393922835172_u128,
			2491_u128,
			2537_u128,
			ACA,
			18441141721883_u128,
			63486975129400_u128,
			85100506_u128,
			3914623276_u128,
			2537_u128,
			18437227098607_u128,
			63490889752676_u128,
		),
		(
			67_u64,
			1126422_u128,
			295_u128,
			471_u128,
			HDX,
			234746918_u128,
			7492177402_u128,
			358776_u128,
			63144576_u128,
			471_u128,
			171602342_u128,
			7555321978_u128,
		),
		(
			168_u64,
			28351324279041_u128,
			450_u128,
			952_u128,
			ACA,
			231645535_u128,
			38796364068_u128,
			356723_u128,
			179074946_u128,
			952_u128,
			52570589_u128,
			38975439014_u128,
		),
		(
			3_u64,
			17631376575792_u128,
			82_u128,
			357_u128,
			HDX,
			1832794469_u128,
			20473946880_u128,
			932564_u128,
			256455100_u128,
			357_u128,
			1576339369_u128,
			20730401980_u128,
		),
		(
			49_u64,
			94059_u128,
			81_u128,
			1557_u128,
			HDX,
			21495686711_u128,
			11126653978_u128,
			758404_u128,
			1119404304_u128,
			1557_u128,
			20376282407_u128,
			12246058282_u128,
		),
		(
			38_u64,
			14085_u128,
			266_u128,
			2564373_u128,
			KSM,
			36167851242_u128,
			36115448964_u128,
			5278_u128,
			13533356746_u128,
			2564373_u128,
			22634494496_u128,
			49648805710_u128,
		),
		(
			158_u64,
			762784_u128,
			129_u128,
			129_u128,
			BSX,
			86085676_u128,
			21814882774_u128,
			86085676_u128,
			0_u128,
			129_u128,
			86085676_u128,
			21814882774_u128,
		),
	];

	for (
		updated_at,
		total_shares_z,
		liq_pool_accumulated_rpz,
		global_pool_accumulated_rpz,
		reward_currency,
		accumulated_rewards,
		paid_accumulated_rewards,
		liq_pool_stake_in_global_pool,
		expected_rewards_from_global_pool,
		expected_liq_pool_accumulated_rpz,
		expected_global_pool_accumulated_rewards,
		expected_global_pool_paid_accumulated_rewards,
	) in testing_values.iter()
	{
		let global_pool_id = 1;
		let liq_pool_id = 2;
		let yield_per_period = Permill::from_percent(50);
		let planned_yielding_periods = 100;
		let blocks_per_period = 1;
		let owner = ALICE;
		let incentivized_token = BSX;
		let max_reward_per_period = Balance::from(10_000_u32);

		let mut global_pool = GlobalPool::new(
			global_pool_id,
			*updated_at,
			*reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		global_pool.total_shares_z = *total_shares_z;
		global_pool.accumulated_rpz = *global_pool_accumulated_rpz;
		global_pool.accumulated_rewards = *accumulated_rewards;
		global_pool.paid_accumulated_rewards = *paid_accumulated_rewards;

		let mut liq_pool = LiquidityPoolYieldFarm::new(liq_pool_id, *updated_at, None, FixedU128::from(10_u128));
		liq_pool.accumulated_rpz = *liq_pool_accumulated_rpz;

		assert_eq!(
			LiquidityMining::claim_from_global_pool(&mut global_pool, &mut liq_pool, *liq_pool_stake_in_global_pool)
				.unwrap(),
			*expected_rewards_from_global_pool
		);

		let mut expected_global_pool = GlobalPool::new(
			global_pool_id,
			*updated_at,
			*reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		expected_global_pool.total_shares_z = *total_shares_z;
		expected_global_pool.accumulated_rpz = *global_pool_accumulated_rpz;
		expected_global_pool.accumulated_rewards = *expected_global_pool_accumulated_rewards;
		expected_global_pool.paid_accumulated_rewards = *expected_global_pool_paid_accumulated_rewards;

		assert_eq!(global_pool, expected_global_pool);

		let mut expected_liq_pool =
			LiquidityPoolYieldFarm::new(liq_pool_id, *updated_at, None, FixedU128::from(10_u128));
		expected_liq_pool.accumulated_rpz = *expected_liq_pool_accumulated_rpz;

		assert_eq!(liq_pool, expected_liq_pool);
	}
}

#[test]
fn update_pool_should_work() {
	let testing_values = vec![
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			26_u64,
			206_u64,
			299_u128,
			0_u128,
			2222546480_u128,
			BSX,
			299_u128,
			26_u64,
			0_u128,
			9000000000000_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			188_u64,
			259_u64,
			1151_u128,
			33769603_u128,
			170130593048_u128,
			BSX,
			6188_u128,
			259_u64,
			170130593048_u128,
			8829869406952_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			195_u64,
			326_u64,
			823_u128,
			2604286056_u128,
			8414312431200_u128,
			BSX,
			4053_u128,
			326_u64,
			8414312431200_u128,
			585687568800_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			181_u64,
			1856_u64,
			320_u128,
			8940144_u128,
			190581342_u128,
			BSX,
			341_u128,
			1856_u64,
			190581342_u128,
			8999809418658_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			196_u64,
			954_u64,
			5684_u128,
			282043_u128,
			15319968_u128,
			BSX,
			5738_u128,
			954_u64,
			15319968_u128,
			8999984680032_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			68_u64,
			161_u64,
			37_u128,
			1138057342_u128,
			2345375835_u128,
			BSX,
			39_u128,
			161_u64,
			2345375835_u128,
			8997654624165_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			161_u64,
			448_u64,
			678_u128,
			49923_u128,
			39735180_u128,
			BSX,
			1473_u128,
			448_u64,
			39735180_u128,
			8999960264820_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			27_u64,
			132_u64,
			978_u128,
			2444_u128,
			3795224_u128,
			BSX,
			2530_u128,
			132_u64,
			3795224_u128,
			8999996204776_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			97_u64,
			146_u64,
			28_u128,
			1593208_u128,
			3249180_u128,
			BSX,
			30_u128,
			146_u64,
			3249180_u128,
			8999996750820_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			154_u64,
			202_u64,
			876_u128,
			9838_u128,
			12385881_u128,
			BSX,
			2134_u128,
			202_u64,
			12385881_u128,
			8999987614119_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			104_u64,
			131_u64,
			8373_u128,
			2046838954_u128,
			56708340909_u128,
			BSX,
			8400_u128,
			131_u64,
			56708340909_u128,
			8943291659091_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			90_u64,
			110_u64,
			5886_u128,
			596054_u128,
			1685400_u128,
			BSX,
			5888_u128,
			110_u64,
			1685400_u128,
			8999998314600_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			198_u64,
			582_u64,
			2591_u128,
			377215_u128,
			67232880_u128,
			BSX,
			2769_u128,
			582_u64,
			67232880_u128,
			8999932767120_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			29_u64,
			100_u64,
			80_u128,
			8250_u128,
			79833261_u128,
			BSX,
			9756_u128,
			100_u64,
			79833261_u128,
			8999920166739_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			91_u64,
			260_u64,
			2537_u128,
			35172_u128,
			3914623276_u128,
			BSX,
			113836_u128,
			260_u64,
			3914623276_u128,
			8996085376724_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			67_u64,
			229_u64,
			471_u128,
			1126422_u128,
			63144576_u128,
			BSX,
			527_u128,
			229_u64,
			63144576_u128,
			8999936855424_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			168_u64,
			361_u64,
			952_u128,
			28279041_u128,
			179074946_u128,
			BSX,
			958_u128,
			361_u64,
			179074946_u128,
			8999820925054_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			3_u64,
			52_u64,
			357_u128,
			2_u128,
			256455100_u128,
			BSX,
			128227907_u128,
			52_u64,
			256455100_u128,
			8999743544900_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			49_u64,
			132_u64,
			1557_u128,
			94059_u128,
			1119404304_u128,
			BSX,
			13458_u128,
			132_u64,
			1119404304_u128,
			8998880595696_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			38_u64,
			38_u64,
			2564373_u128,
			14085_u128,
			13533356746_u128,
			BSX,
			2564373_u128,
			38_u64,
			0_u128,
			9000000000000_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			158_u64,
			158_u64,
			129_u128,
			762784_u128,
			179074933_u128,
			BSX,
			129_u128,
			158_u64,
			0_u128,
			9000000000000_u128,
		),
	];

	for (
		global_pool_id,
		liq_pool_id,
		liq_pool_updated_at,
		now_period,
		liq_pool_accumulated_rpvs,
		liq_pool_total_valued_shares,
		liq_pool_rewards,
		reward_currency,
		expected_liq_pool_accumulated_rpvs,
		expected_updated_at,
		expected_liq_pool_reward_currency_balance,
		expected_global_pool_reward_currency_balance,
	) in testing_values.iter()
	{
		let owner = ALICE;
		let yield_per_period = Permill::from_percent(50);
		let blocks_per_period = BlockNumber::from(1_u32);
		let planned_yielding_periods = 100;
		let incentivized_token = BSX;
		let updated_at = 200_u64;
		let max_reward_per_period = Balance::from(10_000_u32);

		let mut global_pool = GlobalPool::<Test>::new(
			*global_pool_id,
			updated_at,
			*reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		global_pool.total_shares_z = 1_000_000_u128;
		global_pool.accumulated_rpz = 200_u128;
		global_pool.accumulated_rewards = 1_000_000_u128;
		global_pool.paid_accumulated_rewards = 1_000_000_u128;

		let mut liq_pool = LiquidityPoolYieldFarm {
			id: *liq_pool_id,
			updated_at: *liq_pool_updated_at,
			total_shares: 200_u128,
			total_valued_shares: *liq_pool_total_valued_shares,
			accumulated_rpvs: *liq_pool_accumulated_rpvs,
			accumulated_rpz: 200_u128,
			loyalty_curve: None,
			stake_in_global_pool: Balance::from(10_000_u32),
			multiplier: FixedU128::from(10_u128),
			canceled: false,
		};

		let mut ext = new_test_ext();

		let farm_account_id = LiquidityMining::pool_account_id(*global_pool_id).unwrap();
		let pool_account_id = LiquidityMining::pool_account_id(*liq_pool_id).unwrap();

		ext.execute_with(|| {
			let _ = Tokens::transfer(
				Origin::signed(TREASURY),
				farm_account_id,
				global_pool.reward_currency,
				9_000_000_000_000,
			);
			assert_eq!(
				Tokens::free_balance(global_pool.reward_currency, &farm_account_id),
				9_000_000_000_000_u128
			);

			assert_eq!(Tokens::free_balance(*reward_currency, &pool_account_id), 0);

			assert_ok!(LiquidityMining::update_liq_pool(
				&mut liq_pool,
				*liq_pool_rewards,
				*now_period,
				*global_pool_id,
				*reward_currency
			));

			let mut rhs_global_pool = GlobalPool::new(
				*global_pool_id,
				updated_at,
				*reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			rhs_global_pool.updated_at = 200_u64;
			rhs_global_pool.total_shares_z = 1_000_000_u128;
			rhs_global_pool.accumulated_rpz = 200_u128;
			rhs_global_pool.accumulated_rewards = 1_000_000_u128;
			rhs_global_pool.paid_accumulated_rewards = 1_000_000_u128;

			assert_eq!(global_pool, rhs_global_pool);

			assert_eq!(
				liq_pool,
				LiquidityPoolYieldFarm {
					id: *liq_pool_id,
					updated_at: *expected_updated_at,
					total_shares: 200_u128,
					total_valued_shares: *liq_pool_total_valued_shares,
					accumulated_rpvs: *expected_liq_pool_accumulated_rpvs,
					accumulated_rpz: 200_u128,
					loyalty_curve: None,
					stake_in_global_pool: Balance::from(10_000_u32),
					multiplier: FixedU128::from(10_u128),
					canceled: false,
				}
			);

			assert_eq!(
				Tokens::free_balance(global_pool.reward_currency, &farm_account_id),
				*expected_global_pool_reward_currency_balance
			);
			assert_eq!(
				Tokens::free_balance(global_pool.reward_currency, &pool_account_id),
				*expected_liq_pool_reward_currency_balance
			);
		});
	}
}

#[test]
fn get_next_pool_id_should_work() {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 1);
		assert_eq!(LiquidityMining::pool_id(), 1);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 2);
		assert_eq!(LiquidityMining::pool_id(), 2);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 3);
		assert_eq!(LiquidityMining::pool_id(), 3);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 4);
		assert_eq!(LiquidityMining::pool_id(), 4);
	});
}

#[test]
fn pool_account_id_should_work() {
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::max_value()];

	for id in ids {
		assert_ok!(LiquidityMining::pool_account_id(id));
	}
}

#[test]
fn pool_account_id_should_not_work() {
	let ids: Vec<PoolId> = vec![0];

	for id in ids {
		assert_err!(LiquidityMining::pool_account_id(id), Error::<Test>::InvalidPoolId);
	}
}

#[test]
fn validate_pool_id_should_work() {
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::max_value()];

	for id in ids {
		assert_ok!(LiquidityMining::validate_pool_id(id));
	}
}

#[test]
fn validate_pool_id_should_not_work() {
	assert_eq!(
		LiquidityMining::validate_pool_id(0).unwrap_err(),
		Error::<Test>::InvalidPoolId
	);
}

#[test]
fn validate_create_farm_data_should_work() {
	assert_ok!(LiquidityMining::validate_create_farm_data(
		1_000_000,
		100,
		1,
		Permill::from_percent(50)
	));

	assert_ok!(LiquidityMining::validate_create_farm_data(
		9_999_000_000_000,
		2_000_000,
		500,
		Permill::from_percent(100)
	));

	assert_ok!(LiquidityMining::validate_create_farm_data(
		10_000_000,
		101,
		16_986_741,
		Permill::from_perthousand(1)
	));
}

#[test]
fn validate_create_farm_data_should_not_work() {
	assert_err!(
		LiquidityMining::validate_create_farm_data(999_999, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(9, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(0, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 99, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 0, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 87, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 0, Permill::from_percent(50)),
		Error::<Test>::InvalidBlocksPerPeriod
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 10, Permill::from_percent(0)),
		Error::<Test>::InvalidYieldPerPeriod
	);
}

#[test]
fn create_farm_should_work() {
	new_test_ext().execute_with(|| {
		let pool_id = 1;
		let total_rewards: Balance = 50_000_000_000;
		let reward_currency = BSX;
		let planned_yielding_periods: BlockNumber = 1_000_000_000_u64;
		let blocks_per_period = 20_000;
		let incentivized_token = BSX;
		let owner = ALICE;
		let yield_per_period = Permill::from_percent(20);
		let max_reward_per_period: Balance = total_rewards.checked_div(planned_yielding_periods.into()).unwrap();

		let created_at_block = 15_896;

		set_block_number(created_at_block);

		let pool_account = LiquidityMining::pool_account_id(pool_id).unwrap();

		assert_eq!(Tokens::free_balance(reward_currency, &pool_account), 0);

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			total_rewards,
			planned_yielding_periods,
			blocks_per_period,
			incentivized_token,
			reward_currency,
			owner,
			yield_per_period
		));

		//check if total_rewards was transferd to pool account
		assert_eq!(Tokens::free_balance(reward_currency, &pool_account), total_rewards);
		assert_eq!(
			Tokens::free_balance(reward_currency, &ALICE),
			(INITIAL_BALANCE - total_rewards)
		);

		let updated_at = created_at_block / blocks_per_period;

		let global_pool = GlobalPool::new(
			pool_id,
			updated_at,
			reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		expect_events(vec![mock::Event::LiquidityMining(Event::FarmCreated {
			farm_id: global_pool.id,
			owner: global_pool.owner,
			reward_currency: global_pool.reward_currency,
			yield_per_period: global_pool.yield_per_period,
			planned_yielding_periods: global_pool.planned_yielding_periods,
			blocks_per_period: global_pool.blocks_per_period,
			incentivized_asset: global_pool.incentivized_asset,
			max_reward_per_period: global_pool.max_reward_per_period,
		})]);

		assert_eq!(LiquidityMining::global_pool(pool_id).unwrap(), global_pool);
	});
}

#[test]
fn create_farm_from_basic_origin_should_not_work() {
	new_test_ext().execute_with(|| {
		let created_at_block = 15_896;

		set_block_number(created_at_block);

		assert_noop!(
			LiquidityMining::create_farm(
				Origin::signed(ALICE),
				1_000_000,
				1_000,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			BadOrigin
		);
	});
}

#[test]
fn create_farm_invalid_data_should_not_work() {
	new_test_ext().execute_with(|| {
		let created_at_block = 15_896;

		set_block_number(created_at_block);

		//total_rewards bellow min. limit
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				100,
				1_000,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			Error::<Test>::InvalidTotalRewards
		);

		//planned_yielding_periods bellow min. limit
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_000,
				10,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			Error::<Test>::InvalidPlannedYieldingPeriods
		);

		//blocks_per_period is 0.
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_000,
				1_000,
				0,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			Error::<Test>::InvalidBlocksPerPeriod
		);

		//yield_per_period is 0.
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_000,
				1_000,
				1,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(0)
			),
			Error::<Test>::InvalidYieldPerPeriod
		);
	});
}

#[test]
fn create_farm_with_inssufficient_balance_should_not_work() {
	//owner account balance is 1M BSX
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_001,
				1_000,
				1,
				BSX,
				BSX,
				ACCOUNT_WITH_1M,
				Permill::from_percent(20)
			),
			Error::<Test>::InsufficientRewardCurrencyBalance
		);
	});
}

#[test]
fn add_liquidity_pool_should_work() {
	//Note: global_pool.updated_at isn't changed because pool is empty (no liq. pool stake in globalPool)
	let test_data = vec![
		(
			AssetPair {
				asset_in: BSX,
				asset_out: ACA,
			},
			LiquidityPoolYieldFarm {
				id: 8,
				updated_at: 17,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(20_000_u128),
				loyalty_curve: Some(LoyaltyCurve::default()),
				canceled: false,
			},
			BSX_ACA_AMM,
			ALICE,
			ALICE_FARM,
			17_850,
			GlobalPool {
				liq_pools_count: 1,
				..PREDEFINED_GLOBAL_POOLS[0].clone()
			},
		),
		(
			AssetPair {
				asset_in: KSM,
				asset_out: BSX,
			},
			LiquidityPoolYieldFarm {
				id: 9,
				updated_at: 17,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(10_000_u128),
				loyalty_curve: None,
				canceled: false,
			},
			BSX_KSM_AMM,
			ALICE,
			ALICE_FARM,
			17_850,
			GlobalPool {
				liq_pools_count: 2,
				..PREDEFINED_GLOBAL_POOLS[0].clone()
			},
		),
		(
			AssetPair {
				asset_in: BSX,
				asset_out: ETH,
			},
			LiquidityPoolYieldFarm {
				id: 10,
				updated_at: 20,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(10_000_u128),
				loyalty_curve: Some(LoyaltyCurve {
					initial_reward_percentage: FixedU128::from_inner(100_000_000_000_000_000),
					scale_coef: 50,
				}),
				canceled: false,
			},
			BSX_ETH_AMM,
			ALICE,
			ALICE_FARM,
			20_000,
			GlobalPool {
				liq_pools_count: 3,
				..PREDEFINED_GLOBAL_POOLS[0].clone()
			},
		),
		(
			AssetPair {
				asset_in: BSX,
				asset_out: ETH,
			},
			LiquidityPoolYieldFarm {
				id: 11,
				updated_at: 2,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(50_000_128),
				loyalty_curve: Some(LoyaltyCurve {
					initial_reward_percentage: FixedU128::from_inner(1),
					scale_coef: 0,
				}),
				canceled: false,
			},
			BSX_ETH_AMM,
			BOB,
			BOB_FARM,
			20_000,
			GlobalPool {
				liq_pools_count: 1,
				..PREDEFINED_GLOBAL_POOLS[1].clone()
			},
		),
	];

	predefined_test_ext().execute_with(|| {
		for (assets, pool, _amm_id, who, farm_id, now, global_pool) in test_data.clone() {
			set_block_number(now);

			assert_ok!(LiquidityMining::add_liquidity_pool(
				Origin::signed(who),
				farm_id,
				assets,
				pool.multiplier,
				pool.loyalty_curve.clone()
			));

			expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolAdded {
				farm_id,
				liq_pool_farm_id: pool.id,
				multiplier: pool.multiplier,
				nft_class: LIQ_MINING_NFT_CLASS,
				loyalty_curve: pool.loyalty_curve.clone(),
				asset_pair: assets,
			})]);

			assert_eq!(LiquidityMining::global_pool(farm_id).unwrap(), global_pool);
		}

		for (_, pool, amm_id, _, farm_id, _, _) in test_data {
			assert_eq!(LiquidityMining::liquidity_pool(farm_id, amm_id).unwrap(), pool);
		}
	});
}

#[test]
fn add_liquidity_pool_missing_incentivized_asset_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					//neither KSM nor DOT is incetivized in farm
					asset_in: KSM,
					asset_out: DOT,
				},
				FixedU128::from(10_000_u128),
				None
			),
			Error::<Test>::MissingIncentivizedAsset
		);
	});
}

#[test]
fn add_liquidity_pool_not_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(BOB),
				ALICE_FARM,
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
				FixedU128::from(10_000_u128),
				None
			),
			Error::<Test>::Forbidden
		);

		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(BOB),
				ALICE_FARM,
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
				FixedU128::from(10_000_u128),
				Some(LoyaltyCurve::default())
			),
			Error::<Test>::Forbidden
		);
	});
}

#[test]
fn add_liquidity_pool_invalid_loyalty_curve_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let curves = vec![
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::one(),
				scale_coef: 0,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from_float(1.0),
				scale_coef: 1_000_000,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from_float(1.000_000_000_000_000_001),
				scale_coef: 25_996_000,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from(1_u128),
				scale_coef: 25_996_000,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from(5_u128),
				scale_coef: 25_996_000,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from(16_874_354_654_u128),
				scale_coef: 25_996_000,
			}),
		];

		for c in curves {
			assert_noop!(
				LiquidityMining::add_liquidity_pool(
					Origin::signed(ALICE),
					ALICE_FARM,
					AssetPair {
						asset_in: BSX,
						asset_out: HDX,
					},
					FixedU128::from(10_000_u128),
					c
				),
				Error::<Test>::InvalidInitialRewardPercentage
			);
		}
	});
}

#[test]
fn add_liquidity_pool_invalid_multiplier_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
				FixedU128::from(0_u128),
				Some(LoyaltyCurve::default())
			),
			Error::<Test>::InvalidMultiplier
		);
	});
}

#[test]
fn add_liquidity_pool_non_existing_amm_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					//AMM for this assetPair does not exist
					asset_in: BSX,
					asset_out: 999_999_999,
				},
				FixedU128::from(10_000_u128),
				Some(LoyaltyCurve::default())
			),
			Error::<Test>::AmmPoolDoesNotExist
		);
	});
}

#[test]
fn add_liquidity_pool_add_duplicate_amm_should_not_work() {
	predefined_test_ext().execute_with(|| {
		set_block_number(20_000);

		let aca_ksm_assets = AssetPair {
			asset_in: ACA,
			asset_out: KSM,
		};

		let aca_ksm_amm_account = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(aca_ksm_assets)).unwrap().0);

		//check if liq. pool for aca ksm assets pair exist
		assert!(LiquidityMining::liquidity_pool(CHARLIE_FARM, aca_ksm_amm_account).is_some());

		//try to add same amm second time in the same block(period)
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(CHARLIE),
				CHARLIE_FARM,
				aca_ksm_assets,
				FixedU128::from(9_000_u128),
				Some(LoyaltyCurve::default()),
			),
			Error::<Test>::LiquidityPoolAlreadyExists
		);

		//try to add same amm second time in later block(period)
		set_block_number(30_000);

		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(CHARLIE),
				CHARLIE_FARM,
				aca_ksm_assets,
				FixedU128::from(9_000_u128),
				Some(LoyaltyCurve::default()),
			),
			Error::<Test>::LiquidityPoolAlreadyExists
		);
	});
}

#[test]
fn destroy_farm_should_work() {
	predefined_test_ext().execute_with(|| {
		//transfer all rewards from farm account
		let farm_account = LiquidityMining::pool_account_id(BOB_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_POOLS[1].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[1].reward_currency, &farm_account),
			0
		);

		assert_ok!(LiquidityMining::destroy_farm(Origin::signed(BOB), BOB_FARM));

		expect_events(vec![mock::Event::LiquidityMining(Event::FarmDestroyed {
			id: BOB_FARM,
			who: BOB,
		})]);

		assert!(LiquidityMining::global_pool(BOB_FARM).is_none());
	});
}

#[test]
fn destroy_farm_not_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		//transfer all rewards from farm account
		let farm_account = LiquidityMining::pool_account_id(BOB_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_POOLS[1].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[1].reward_currency, &farm_account),
			0
		);

		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(ALICE), BOB_FARM),
			Error::<Test>::Forbidden
		);

		assert_eq!(
			LiquidityMining::global_pool(BOB_FARM).unwrap(),
			PREDEFINED_GLOBAL_POOLS[1]
		);
	});
}

#[test]
fn destroy_farm_farm_not_exists_should_not_work() {
	predefined_test_ext().execute_with(|| {
		const NON_EXISTING_FARM: u32 = 999_999_999;
		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(ALICE), NON_EXISTING_FARM),
			Error::<Test>::FarmNotFound
		);
	});
}

#[test]
fn destroy_farm_with_pools_should_not_work() {
	//all rewards was distributed but liq. pool still exist in the farm
	predefined_test_ext().execute_with(|| {
		//transfer all rewards from farm account
		let farm_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_POOLS[2].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[2].reward_currency, &farm_account),
			0
		);

		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(GC), GC_FARM),
			Error::<Test>::FarmIsNotEmpty
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			PREDEFINED_GLOBAL_POOLS[2]
		);
	});
}

#[test]
fn destroy_farm_with_undistributed_rewards_and_no_pools_should_not_work() {
	//all liq. pool was removed from the farm but there are undistributed rewards on farm account
	predefined_test_ext().execute_with(|| {
		let farm_account = LiquidityMining::pool_account_id(BOB_FARM).unwrap();
		assert!(!Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[1].reward_currency, &farm_account).is_zero());

		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(BOB), BOB_FARM),
			Error::<Test>::RewardBalanceIsNotZero
		);

		assert_eq!(
			LiquidityMining::global_pool(BOB_FARM).unwrap(),
			PREDEFINED_GLOBAL_POOLS[1]
		);
	});
}

#[test]
fn destroy_farm_healthy_farm_should_not_work() {
	//farm with undistributed rewards and liq. pools
	predefined_test_ext().execute_with(|| {
		let farm_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		assert!(!Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[2].reward_currency, &farm_account).is_zero());

		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(GC), GC_FARM),
			Error::<Test>::FarmIsNotEmpty
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			PREDEFINED_GLOBAL_POOLS[2]
		);
	});
}

#[test]
fn deposit_shares_should_work() {
	//NOTE: farm incentivize BSX token
	predefined_test_ext().execute_with(|| {
		let farm_id = GC_FARM;
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		let bsx_tkn2_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN2,
		};

		let pallet_account = LiquidityMining::account_id();
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let bsx_tkn1_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn2_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN2_LIQ_POOL_ID).unwrap();
		let bsx_tkn1_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(bsx_tkn1_assets)).unwrap().0);
		let bsx_tkn2_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(bsx_tkn2_assets)).unwrap().0);
		//DEPOSIT 1:
		set_block_number(1_800); //18-th period

		let bsx_tkn1_alice_shares = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 50, 0).unwrap();
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 0);

		let deposited_amount = 50;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn1_assets,
			deposited_amount,
		));

		expect_events(vec![
			mock::Event::NFT(pallet_nft::Event::InstanceMinted {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[0],
			}),
			mock::Event::LiquidityMining(Event::SharesDeposited {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: deposited_amount,
				nft_class: LIQ_MINING_NFT_CLASS,
				nft_instance_id: PREDEFINED_NFT_IDS[0],
			}),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 0,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 0,
				liq_pools_count: 2,
				paid_accumulated_rewards: 0,
				total_shares_z: 12_500,
				accumulated_rewards: 0
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				total_shares: 50,
				total_valued_shares: 2_500,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 12_500,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 1, GC_FARM)
		);

		assert_eq!(
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
			Deposit {
				shares: deposited_amount,
				valued_shares: 2_500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 0,
				entered_at: 18,
				updated_at: 18,
			},
		);

		//check if shares was transferd from extrinsic caller
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_shares - deposited_amount
		);

		//check if shares was transferd to liq. mining pallet account
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account),
			deposited_amount
		);

		// DEPOSIT 2 (deposit in same period):
		let bsx_tkn1_bob_shares = Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB);

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 52, 0).unwrap();

		let deposited_amount = 80;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn1_assets,
			deposited_amount
		));

		expect_events(vec![
			mock::Event::NFT(pallet_nft::Event::InstanceMinted {
				owner: BOB,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[1],
			}),
			mock::Event::LiquidityMining(Event::SharesDeposited {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: BOB,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: deposited_amount,
				nft_class: LIQ_MINING_NFT_CLASS,
				nft_instance_id: PREDEFINED_NFT_IDS[1],
			}),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 18,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 9,
				liq_pools_count: 2,
				paid_accumulated_rewards: 112_500,
				total_shares_z: 33_300,
				accumulated_rewards: 0,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 18,
				accumulated_rpvs: 45,
				accumulated_rpz: 9,
				total_shares: 130,
				total_valued_shares: 6_660,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 33_300,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 2, GC_FARM)
		);

		assert_eq!(
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[1]).unwrap(),
			Deposit {
				shares: deposited_amount,
				valued_shares: 4_160,
				accumulated_rpvs: 45,
				accumulated_claimed_rewards: 0,
				entered_at: 18,
				updated_at: 18,
			},
		);

		//check if shares was transfered from deposit owner
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB),
			bsx_tkn1_bob_shares - deposited_amount
		);
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 130); //130 - sum of all deposited shares until now

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			(30_000_000_000 - 112_500) //total_rewards - sum(claimed rewards by all liq. pools until now)
		);

		//check if claim from global pool was transfered to liq. pool account
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account), 112_500);

		// DEPOSIT 3 (same period, second liq pool yield farm):
		let bsx_tkn2_bob_shares = Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB);

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 8, 0).unwrap();

		let deposited_amount = 25;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount
		));

		expect_events(vec![
			mock::Event::NFT(pallet_nft::Event::InstanceMinted {
				owner: BOB,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[2],
			}),
			mock::Event::LiquidityMining(Event::SharesDeposited {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: BOB,
				lp_token: BSX_TKN2_SHARE_ID,
				amount: deposited_amount,
				nft_class: LIQ_MINING_NFT_CLASS,
				nft_instance_id: PREDEFINED_NFT_IDS[2],
			}),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 18,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 9,
				liq_pools_count: 2,
				paid_accumulated_rewards: 112_500,
				total_shares_z: 35_300,
				accumulated_rewards: 0,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				total_shares: 25,
				total_valued_shares: 200,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 2_000,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 1, GC_FARM)
		);

		assert_eq!(
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[2]).unwrap(),
			Deposit {
				shares: deposited_amount,
				valued_shares: 200,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 0,
				entered_at: 18,
				updated_at: 18,
			},
		);

		//check if shares was transfered from deposit owner
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB),
			bsx_tkn2_bob_shares - deposited_amount
		);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account), 25); //25 - sum of all deposited shares until now

		//pool wasn't updated in this period so no claim from global pool
		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			(30_000_000_000 - 112_500) //total_rewards - claimed rewards by liq. pool
		);

		// no claim happed for this pool so this is same as after previous deposit
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account), 112_500);
		//check if claim from global pool was transfered to liq. pool account
		//(there was no clai for this pool)
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account), 0);

		// DEPOSIT 4 (new period):
		set_block_number(2051); //period 20
		let bsx_tkn2_bob_shares = Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB);

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 58, 0).unwrap();

		let deposited_amount = 800;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount
		));

		expect_events(vec![
			mock::Event::NFT(pallet_nft::Event::InstanceMinted {
				owner: BOB,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[3],
			}),
			mock::Event::LiquidityMining(Event::SharesDeposited {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: BOB,
				lp_token: BSX_TKN2_SHARE_ID,
				amount: deposited_amount,
				nft_class: LIQ_MINING_NFT_CLASS,
				nft_instance_id: PREDEFINED_NFT_IDS[3],
			}),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 20,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 10,
				liq_pools_count: 2,
				paid_accumulated_rewards: 132_500,
				total_shares_z: 499_300,
				accumulated_rewards: 15_300,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 20,
				accumulated_rpvs: 100,
				accumulated_rpz: 10,
				total_shares: 825,
				total_valued_shares: 46_600,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 466_000,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 2, GC_FARM)
		);

		assert_eq!(
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[3]).unwrap(),
			Deposit {
				shares: deposited_amount,
				valued_shares: 46_400,
				accumulated_rpvs: 100,
				accumulated_claimed_rewards: 0,
				entered_at: 20,
				updated_at: 20,
			},
		);

		//check if shares was transfered from deposit owner
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB),
			bsx_tkn2_bob_shares - deposited_amount
		);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account), 825); //825 - sum of all deposited shares until now

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			(30_000_000_000 - 132_500) //total_rewards - sum(claimed rewards by all liq. pools until now)
		);

		//check if claim from global pool was transfered to liq. pool account
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account), 112_500);
		//check if claim from global pool was transfered to liq. pool account
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account), 20_000);

		// DEPOSIT 5 (same period, second liq pool yield farm):
		set_block_number(2_586); //period 20
		let bsx_tkn2_alice_shares = Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE);

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 3, 0).unwrap();

		let deposited_amount = 87;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount,
		));

		expect_events(vec![
			mock::Event::NFT(pallet_nft::Event::InstanceMinted {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[4],
			}),
			mock::Event::LiquidityMining(Event::SharesDeposited {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN2_SHARE_ID,
				amount: 87,
				nft_class: LIQ_MINING_NFT_CLASS,
				nft_instance_id: PREDEFINED_NFT_IDS[4],
			}),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
				total_shares_z: 501_910,
				accumulated_rewards: 331_550,
				paid_accumulated_rewards: 1_064_500,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 25,
				accumulated_rpvs: 120,
				accumulated_rpz: 12,
				total_shares: 912,
				total_valued_shares: 46_861,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 468_610,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 3, GC_FARM)
		);

		assert_eq!(
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[4]).unwrap(),
			Deposit {
				shares: deposited_amount,
				valued_shares: 261,
				accumulated_rpvs: 120,
				accumulated_claimed_rewards: 0,
				entered_at: 25,
				updated_at: 25,
			},
		);

		//check if shares was transfered from deposit owner
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE),
			bsx_tkn2_alice_shares - 87
		);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account), 912); //912 - sum of all deposited shares until now

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			(30_000_000_000 - 1_064_500) //total_rewards - sum(claimed rewards by all liq. pools until now)
		);

		//check if claim from global pool was transfered to liq. pool account
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account), 112_500); //total_rewards - sum(claimed rewards by all liq. pools until now)
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account), 952_000); //total_rewards - sum(claimed rewards by all liq. pools until now)

		// DEPOSIT 6 (same period):
		set_block_number(2_596); //period 20
		let bsx_tkn2_alice_shares = Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE);

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 16, 0).unwrap();

		let deposited_amount = 48;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount,
		));

		expect_events(vec![
			mock::Event::NFT(pallet_nft::Event::InstanceMinted {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[5],
			}),
			mock::Event::LiquidityMining(Event::SharesDeposited {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN2_SHARE_ID,
				amount: deposited_amount,
				nft_class: LIQ_MINING_NFT_CLASS,
				nft_instance_id: PREDEFINED_NFT_IDS[5],
			}),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
				total_shares_z: 509_590,
				accumulated_rewards: 331_550,
				paid_accumulated_rewards: 1_064_500,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN2_LIQ_POOL_ID,
				updated_at: 25,
				accumulated_rpvs: 120,
				accumulated_rpz: 12,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: FixedU128::from(10_u128),
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(),
			(bsx_tkn2_assets, 4, GC_FARM)
		);

		assert_eq!(
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[5]).unwrap(),
			Deposit {
				shares: deposited_amount,
				valued_shares: 768,
				accumulated_rpvs: 120,
				accumulated_claimed_rewards: 0,
				entered_at: 25,
				updated_at: 25,
			},
		);

		//check if shares was transfered from deposit owner
		assert_eq!(
			Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE),
			bsx_tkn2_alice_shares - deposited_amount
		);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account), 960); //960 - sum of all deposited shares until now

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			(30_000_000_000 - 1_064_500) //total_rewards - sum(claimed rewards by all liq. pools until now)
		);

		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account), 112_500); //total_rewards - sum(claimed rewards by all liq. pools until now)
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account), 952_000); //total_rewards - sum(claimed rewards by all liq. pools until now)

		// DEPOSIT 7 : (same period differen liq poll farm)
		set_block_number(2_596); //period 20
		let bsx_tkn1_alice_shares = Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE);

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 80, 0).unwrap();

		let deposited_amount = 486;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn1_assets,
			486
		));

		expect_events(vec![
			mock::Event::NFT(pallet_nft::Event::InstanceMinted {
				owner: ALICE,
				class_id: LIQ_MINING_NFT_CLASS,
				instance_id: PREDEFINED_NFT_IDS[6],
			}),
			mock::Event::LiquidityMining(Event::SharesDeposited {
				farm_id: GC_FARM,
				liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
				who: ALICE,
				lp_token: BSX_TKN1_SHARE_ID,
				amount: deposited_amount,
				nft_class: LIQ_MINING_NFT_CLASS,
				nft_instance_id: PREDEFINED_NFT_IDS[6],
			}),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
				total_shares_z: 703_990,
				accumulated_rewards: 231_650,
				paid_accumulated_rewards: 1_164_400,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: BSX_TKN1_LIQ_POOL_ID,
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 227_700,
				multiplier: FixedU128::from(5_u128),
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 3, GC_FARM)
		);

		assert_eq!(
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[6]).unwrap(),
			Deposit {
				shares: deposited_amount,
				valued_shares: 38_880,
				accumulated_rpvs: 60,
				accumulated_claimed_rewards: 0,
				entered_at: 25,
				updated_at: 25,
			},
		);

		//check if shares was transfered from deposit owner
		assert_eq!(
			Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE),
			bsx_tkn1_alice_shares - deposited_amount
		);
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 616); //616 - sum of all deposited shares until now

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			(30_000_000_000 - 1_164_400) //total_rewards - sum(claimed rewards by all liq. pools until now)
		);

		//check if claim from global pool was transfered to liq. pool account
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account), 212_400); //total_rewards - sum(claimed rewards by all liq. pools until now)
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account), 952_000); //total_rewards - sum(claimed rewards by all liq. pools until now)
	});

	//deposit to farm with different incentivized_asset and reward_currency
	//charlie's farm inncetivize KSM and reward currency is ACA
	//This test only check if valued shares are correctly calculated if reward and incentivized
	//assts are different, otherwise pool behaviour is same as in test above.
	predefined_test_ext().execute_with(|| {
		let aca_ksm_assets = AssetPair {
			asset_in: ACA,
			asset_out: KSM,
		};

		let aca_ksm_amm_account = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(aca_ksm_assets)).unwrap().0);
		let ksm_balance_in_amm = 16;

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), aca_ksm_amm_account, KSM, ksm_balance_in_amm, 0).unwrap();
		Tokens::set_balance(Origin::root(), aca_ksm_amm_account, ACA, 20, 0).unwrap();

		set_block_number(2_596); //period 25

		let deposited_amount = 1_000_000;
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
				valued_shares: deposited_amount * ksm_balance_in_amm,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 0,
				entered_at: 25,
				updated_at: 25,
			}
		);
	});
}

#[test]
fn deposit_shares_zero_deposit_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		assert_noop!(
			LiquidityMining::deposit_shares(Origin::signed(ALICE), GC_FARM, bsx_tkn1_assets, 0),
			Error::<Test>::InvalidDepositAmount
		);
	});
}

#[test]
fn deposit_shares_insufficient_shares_balance_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		assert_noop!(
			LiquidityMining::deposit_shares(Origin::signed(ALICE), GC_FARM, bsx_tkn1_assets, 4_000_000),
			Error::<Test>::InsufficientAmmSharesBalance
		);
	});
}

#[test]
fn deposit_shares_non_existing_liq_pool_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_dot_assets = AssetPair {
			asset_in: BSX,
			asset_out: DOT,
		};

		assert_noop!(
			LiquidityMining::deposit_shares(Origin::signed(ALICE), GC_FARM, bsx_dot_assets, 10_000),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn deposit_shares_canceled_liq_pool_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		assert_noop!(
			LiquidityMining::deposit_shares(Origin::signed(ALICE), GC_FARM, bsx_tkn1_assets, 10_000),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn claim_rewards_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_tkn1_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn2_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN2_LIQ_POOL_ID).unwrap();
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
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
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
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[4]).unwrap(),
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
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
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
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
			LiquidityMining::deposit(4294967303).unwrap(),
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
		let bsx_tkn1_liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
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
			LiquidityMining::deposit(PREDEFINED_NFT_IDS[0]).unwrap(),
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
			Error::<Test>::DoubleClaimInThePeriod
		);
	});
}

#[test]
fn claim_rewards_invalid_nft_id_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		const INVALID_NFT_CLASS: u128 = 5486;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), INVALID_NFT_CLASS),
			Error::<Test>::InvalidNftId
		);

		//liq. pool metadata not found
		//not_found_id is combination of: liq. pool id: u32::max_value() nftIdSequence: 168_453_145
		const NOT_FOUND_ID: u128 = 723_500_752_978_313_215;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), NOT_FOUND_ID),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn claim_rewards_from_canceled_pool_should_not_work() {
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

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), PREDEFINED_NFT_IDS[0]),
			Error::<Test>::LiquidityMiningCanceled
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
			Error::<Test>::LiquidityPoolNotFound
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
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
			LiquidityMining::global_pool(GC_FARM).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
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
		assert!(LiquidityMining::liquidity_pool(GC_FARM, bsx_tkn1_amm_account).is_some());
		assert!(LiquidityMining::liquidity_pool(GC_FARM, bsx_tkn2_amm_account).is_some());
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
		assert!(LiquidityMining::liquidity_pool(GC_FARM, bsx_tkn1_amm_account).is_none());
		assert!(LiquidityMining::liquidity_pool(GC_FARM, bsx_tkn2_amm_account).is_none());
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
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();

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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();

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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();

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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
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

		assert_eq!(LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM), None);

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

#[test]
fn cancel_liquidity_pool_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	//same period
	predefined_test_ext_with_deposits().execute_with(|| {
		let liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityMiningCanceled {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				stake_in_global_pool: 0,
				canceled: true,
				multiplier: 0.into(),
				..liq_pool
			}
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				total_shares_z: global_pool
					.total_shares_z
					.checked_sub(liq_pool.stake_in_global_pool)
					.unwrap(),
				..global_pool
			}
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 3, GC_FARM)
		);

		assert_eq!(Tokens::free_balance(BSX, &liq_pool_account), liq_pool_bsx_balance);
		assert_eq!(Tokens::free_balance(BSX, &global_pool_account), global_pool_bsx_balance);
	});

	//canelc liq. pool with pools update
	predefined_test_ext_with_deposits().execute_with(|| {
		let liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		set_block_number(10_000);

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityMiningCanceled {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				updated_at: 100,
				accumulated_rpvs: 245,
				accumulated_rpz: 49,
				stake_in_global_pool: 0,
				canceled: true,
				multiplier: 0.into(),
				..liq_pool
			}
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				updated_at: 100,
				accumulated_rpz: 49,
				total_shares_z: global_pool
					.total_shares_z
					.checked_sub(liq_pool.stake_in_global_pool)
					.unwrap(),
				accumulated_rewards: 18_206_375,
				paid_accumulated_rewards: 9_589_300,
				..global_pool
			}
		);

		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 3, GC_FARM)
		);

		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_account),
			liq_pool_bsx_balance + 8_424_900 //8_424_900 - liq. pool last claim from global pool
		);

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance - 8_424_900 //8_424_900 - liq. pool last claim from global pool
		);
	});
}

#[test]
fn cancel_liquidity_pool_invalid_liq_pool_should_not_work() {
	let bsx_dot_assets = AssetPair {
		asset_in: BSX,
		asset_out: DOT,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_dot_assets),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn cancel_liquidity_pool_liq_pool_already_canceled() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		//1-th cancel should pass ok
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_tkn1_assets),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn cancel_liquidity_pool_not_owner_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_LIQ_POOL_OWNER: u128 = ALICE;

		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(NOT_LIQ_POOL_OWNER), GC_FARM, bsx_tkn1_assets),
			Error::<Test>::Forbidden
		);
	});
}

#[test]
fn remove_liquidity_pool_with_deposits_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);

		// cancel liq. pool before removing
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolRemoved {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				liq_pools_count: global_pool.liq_pools_count.checked_sub(1).unwrap(),
				..global_pool
			}
		);

		//liq. pool should be removed from storage
		assert_eq!(LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM), None);

		//liq. pool meta should stay in storage until all deposits are withdrawn
		assert_eq!(
			LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(),
			(bsx_tkn1_assets, 3, GC_FARM)
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
		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);

		//cancel pool before removing
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolRemoved {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: GC,
			asset_pair: bsx_tkn1_assets,
		})]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				liq_pools_count: global_pool.liq_pools_count.checked_sub(1).unwrap(),
				..global_pool
			}
		);

		//liq. pool should be removed from storage
		assert_eq!(LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM), None);

		//liq. pool metadata should be removed from storage if no deposits are left
		assert_eq!(LiquidityMining::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID), None);

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
			LiquidityMining::remove_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_tkn1_assets),
			Error::<Test>::LiquidityMiningIsNotCanceled
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
			LiquidityMining::remove_liquidity_pool(Origin::signed(NOT_OWNER), GC_FARM, bsx_tkn1),
			Error::<Test>::Forbidden
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
			LiquidityMining::remove_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_dot_assets),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn update_liquidity_pool_should_work() {
	//liq. pool without deposits
	predefined_test_ext().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		let new_multiplier: PoolMultiplier = FixedU128::from(5_000_u128);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets,
			new_multiplier
		));

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				multiplier: new_multiplier,
				..liq_pool
			}
		);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), global_pool);
	});

	//liq. pool with deposits
	predefined_test_ext_with_deposits().execute_with(|| {
		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		//same period as last pool update so no pool(global or liq. pool) updated
		let new_multiplier: PoolMultiplier = FixedU128::from(10_000_u128);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets,
			new_multiplier
		));

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				stake_in_global_pool: 455_400_000,
				multiplier: new_multiplier,
				..liq_pool
			}
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				total_shares_z: 455_876_290,
				..global_pool
			}
		);

		//different period so pool update should happen
		set_block_number(5_000);
		let new_multiplier: PoolMultiplier = FixedU128::from(5_000_u128);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		let global_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_account = LiquidityMining::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();

		let global_pool_bsx_balance = Tokens::free_balance(BSX, &global_pool_account);
		let liq_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets,
			new_multiplier
		));

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				updated_at: 50,
				accumulated_rpvs: 30_060,
				accumulated_rpz: 15,
				multiplier: new_multiplier,
				stake_in_global_pool: 227_700_000,
				..liq_pool
			}
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				updated_at: 50,
				accumulated_rpz: 15,
				total_shares_z: 228_176_290,
				accumulated_rewards: global_pool.accumulated_rewards + 133_800_000,
				paid_accumulated_rewards: global_pool.paid_accumulated_rewards + 1_366_200_000,
				..global_pool
			}
		);

		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			global_pool_bsx_balance - 1_366_200_000 //1_366_200_000 - liq. pool claim from global pool
		);
		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_account),
			liq_pool_bsx_balance + 1_366_200_000 //1_366_200_000 - liq. pool claim from global pool
		);
	});
}

#[test]
fn update_liquidity_pool_zero_multiplier_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::update_liquidity_pool(
				Origin::signed(GC),
				GC_FARM,
				bsx_tkn1_assets,
				FixedU128::from(0_u128)
			),
			Error::<Test>::InvalidMultiplier
		);
	});
}

#[test]
fn update_liquidity_pool_canceled_pool_should_not_work() {
	let bsx_tkn1_liq_pool = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_liq_pool
		));

		assert_noop!(
			LiquidityMining::update_liquidity_pool(
				Origin::signed(GC),
				GC_FARM,
				bsx_tkn1_liq_pool,
				FixedU128::from(10_001)
			),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn update_liquidity_pool_not_owner_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let not_owner = ALICE;
		assert_noop!(
			LiquidityMining::update_liquidity_pool(
				Origin::signed(not_owner),
				GC_FARM,
				bsx_tkn1_assets,
				FixedU128::from(10_001_u128)
			),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn resume_liquidity_pool_should_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		//cancel liq. pool before resuming
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets
		));

		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap();
		let global_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		let new_multiplier = FixedU128::from(7490_000);

		assert!(liq_pool.canceled);
		assert!(liq_pool.stake_in_global_pool.is_zero());
		assert!(liq_pool.multiplier.is_zero());

		set_block_number(13_420_000);

		assert_ok!(LiquidityMining::resume_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			bsx_tkn1_assets,
			new_multiplier
		));

		let liq_pool_stake_in_global_pool = new_multiplier.checked_mul_int(45_540).unwrap();

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				canceled: false,
				stake_in_global_pool: liq_pool_stake_in_global_pool,
				accumulated_rpz: 62_996,
				multiplier: new_multiplier,
				updated_at: 134_200,
				..liq_pool
			}
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				total_shares_z: global_pool.total_shares_z + liq_pool_stake_in_global_pool,
				updated_at: 134_200,
				accumulated_rpz: 62_996,
				accumulated_rewards: 29_999_067_250,
				..global_pool
			}
		);
	});
}

#[test]
fn resume_liquidity_pool_non_existing_pool_should_not_work() {
	let bsx_ksm_assets = AssetPair {
		asset_in: BSX,
		asset_out: KSM,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7_490_000);

		assert_noop!(
			LiquidityMining::resume_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_ksm_assets, new_multiplier),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn resume_liquidity_pool_non_canceled_pool_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7490_000);

		assert_noop!(
			LiquidityMining::resume_liquidity_pool(Origin::signed(GC), GC_FARM, bsx_tkn1_assets, new_multiplier),
			Error::<Test>::LiquidityMiningIsNotCanceled
		);
	});
}

#[test]
fn resume_liquidity_pool_not_owner_should_not_work() {
	let bsx_tkn1_assets = AssetPair {
		asset_in: BSX,
		asset_out: TKN1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7490_000);

		assert_noop!(
			LiquidityMining::resume_liquidity_pool(Origin::signed(ALICE), GC_FARM, bsx_tkn1_assets, new_multiplier),
			Error::<Test>::LiquidityMiningIsNotCanceled
		);
	});
}

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
			Error::<Test>::FarmNotFound
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
			Error::<Test>::Forbidden
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
			Error::<Test>::FarmIsNotEmpty
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
			Error::<Test>::FarmIsNotEmpty
		);
	});
}

#[test]
fn do_claim_rewards_should_work() {
	predefined_test_ext().execute_with(|| {
		let empty_liq_pool: LiquidityPoolYieldFarm<Test> = LiquidityPoolYieldFarm {
			id: 1,
			updated_at: 0,
			total_shares: 0,
			total_valued_shares: 0,
			accumulated_rpvs: 0,
			accumulated_rpz: 0,
			loyalty_curve: Some(LoyaltyCurve::default()),
			stake_in_global_pool: 0,
			multiplier: FixedU128::from(100),
			canceled: false,
		};

		let test_data: [(
			Deposit<Test>,
			LiquidityPoolYieldFarm<Test>,
			PeriodOf<Test>,
			(Balance, Balance),
		); 3] = [
			(
				Deposit {
					shares: 100,
					valued_shares: 500,
					accumulated_claimed_rewards: 0,
					accumulated_rpvs: 56,
					entered_at: 12,
					updated_at: 45,
				},
				LiquidityPoolYieldFarm {
					loyalty_curve: Some(LoyaltyCurve::default()),
					accumulated_rpvs: 7_789,
					..empty_liq_pool
				},
				45,
				(0, 0),
			),
			(
				Deposit {
					shares: 12_315_314,
					valued_shares: 1_454_565_765_765,
					accumulated_claimed_rewards: 65_454,
					accumulated_rpvs: 9_809,
					entered_at: 3,
					updated_at: 3,
				},
				LiquidityPoolYieldFarm {
					loyalty_curve: Some(LoyaltyCurve {
						initial_reward_percentage: FixedU128::from_float(0.674_651_900_400_000_000_000f64),
						scale_coef: 360,
					}),
					accumulated_rpvs: 10_743,
					..empty_liq_pool
				},
				50,
				(967_600_574_016_191, 390_963_851_142_865),
			),
			(
				Deposit {
					shares: 97_634,
					valued_shares: 7_483_075,
					accumulated_claimed_rewards: 1_657_649,
					accumulated_rpvs: 10_989,
					entered_at: 39,
					updated_at: 329,
				},
				LiquidityPoolYieldFarm {
					loyalty_curve: None, //no loyalty factor
					accumulated_rpvs: 11_000,
					..empty_liq_pool
				},
				1002,
				(80_656_176, 0),
			),
		];

		let liq_pool_account = LiquidityMining::pool_account_id(1).unwrap();
		assert_ok!(Tokens::set_balance(
			Origin::root(),
			liq_pool_account,
			BSX,
			1_000_0000_0000_0000_0000_000,
			0
		));

		for (mut deposit, liq_pool, now_period, expected_result) in test_data {
			let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
			let lib_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);

			assert_eq!(
				LiquidityMining::do_claim_rewards(ALICE, &mut deposit, &liq_pool, now_period, BSX).unwrap(),
				expected_result
			);

			let expected_alice_balance = alice_bsx_balance + expected_result.0;
			let expected_pool_balance = lib_pool_bsx_balance - expected_result.0;

			assert_eq!(Tokens::free_balance(BSX, &ALICE), expected_alice_balance);
			assert_eq!(Tokens::free_balance(BSX, &liq_pool_account), expected_pool_balance);
		}
	});
}

#[test]
fn get_next_nft_id_should_work() {
	new_test_ext().execute_with(|| {
		//(pool_id, result)
		let test_data = vec![
			(1, 4_294_967_297),
			(6_886, 8_589_941_478),
			(87_321, 12_884_989_209),
			(56, 17_179_869_240),
			(789, 21_474_837_269),
			(248, 25_769_804_024),
			(1_000_000_200, 31_064_771_272),
			(u32::max_value(), 38_654_705_663),
		];

		for (pool_id, expected_nft_id) in test_data {
			assert_eq!(LiquidityMining::get_next_nft_id(pool_id).unwrap(), expected_nft_id);
		}

		//This is last allowed sequencer number - 1, test with max pool id
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0])
				.checked_sub(1_u128)
				.unwrap();

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_334
		);

		assert_eq!(
			LiquidityMining::get_next_nft_id(u32::max_value()).unwrap(),
			u128::max_value()
		);

		//This is last allowed sequencer number - 1, test with min. pool id
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0])
				.checked_sub(1_u128)
				.unwrap();

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_334
		);

		assert_eq!(
			LiquidityMining::get_next_nft_id(1).unwrap(),
			340_282_366_920_938_463_463_374_607_427_473_244_161
		);
	});
}

#[test]
fn get_next_nft_id_should_not_work() {
	new_test_ext().execute_with(|| {
		//This is last allowed sequencer number, next should throw error
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0]);

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_335
		);

		assert_noop!(
			LiquidityMining::get_next_nft_id(u32::max_value()),
			Error::<Test>::NftIdOwerflow
		);

		assert_noop!(LiquidityMining::get_next_nft_id(1), Error::<Test>::NftIdOwerflow);
	});
}

#[test]
fn get_pool_id_from_nft_id_should_work() {
	new_test_ext().execute_with(|| {
		//(nft_id, liq. pool id)
		let test_data = vec![
			(4_294_967_297, 1),
			(8_589_941_478, 6_886),
			(12_884_989_209, 87_321),
			(17_179_869_240, 56),
			(21_474_837_269, 789),
			(25_769_804_024, 248),
			(31_064_771_272, 1_000_000_200),
			(38_654_705_663, u32::max_value()),
			(u128::max_value(), u32::max_value()),
			(340_282_366_920_938_463_463_374_607_427_473_244_161, 1),
			(340_282_366_920_938_463_463_374_607_427_473_244_161, 1),
		];

		for (nft_id, expected_pool_id) in test_data {
			assert_eq!(
				LiquidityMining::get_pool_id_from_nft_id(nft_id).unwrap(),
				expected_pool_id
			);
		}
	});
}

//NOTE: look at approx pallet - https://github.com/brendanzab/approx
fn is_approx_eq_fixedu128(num_1: FixedU128, num_2: FixedU128, delta: FixedU128) -> bool {
	let diff = match num_1.cmp(&num_2) {
		Ordering::Less => num_2.checked_sub(&num_1).unwrap(),
		Ordering::Greater => num_1.checked_sub(&num_2).unwrap(),
		Ordering::Equal => return true,
	};

	if diff.cmp(&delta) == Ordering::Greater {
		println!("diff: {:?}; delta: {:?}; n1: {:?}; n2: {:?}", diff, delta, num_1, num_2);

		false
	} else {
		true
	}
}

fn last_events(n: usize) -> Vec<TestEvent> {
	frame_system::Pallet::<Test>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

fn expect_events(e: Vec<TestEvent>) {
	assert_eq!(last_events(e.len()), e);
}
