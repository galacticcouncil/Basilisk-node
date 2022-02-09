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
	asset_pair_to_map_key, run_to_block, BlockNumber, Event as TestEvent, ExtBuilder, LiquidityMining, Origin, Test,
	Tokens, ACA, ACA_FARM, ACC_1M, ALICE, AMM_POOLS, BOB, BSX, BSX_ACA_AMM, BSX_ACA_LM_POOL, BSX_ACA_SHARE_ID,
	BSX_DOT_AMM, BSX_DOT_LM_POOL, BSX_DOT_SHARE_ID, BSX_ETH_AMM, BSX_ETH_SHARE_ID, BSX_FARM, BSX_HDX_AMM,
	BSX_HDX_SHARE_ID, BSX_KSM_AMM, BSX_KSM_LM_POOL, BSX_KSM_SHARE_ID, BSX_TO1_AMM, BSX_TO1_SHARE_ID, BSX_TO2_AMM,
	BSX_TO2_SHARE_ID, DOT, ETH, GC, GC_FARM, HDX, INITIAL_BALANCE, KSM, KSM_FARM, TO1, TO2, TREASURY,
};

use frame_support::{assert_err, assert_noop, assert_ok};
use primitives::Balance;

use sp_arithmetic::traits::CheckedSub;
use sp_runtime::traits::BadOrigin;

use std::cmp::Ordering;

const ALICE_FARM: u32 = BSX_FARM;
const BOB_FARM: u32 = KSM_FARM;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| run_to_block(1));
	ext
}

const PREDEFINED_GLOBAL_POOLS: [GlobalPool<Test>; 3] = [
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
];

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			100_000_000_000,
			BlockNumber::from(300_u32),
			BlockNumber::from(1_000_u32),
			BSX,
			BSX,
			ALICE,
			Permill::from_percent(20),
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			1_000_000_000,
			BlockNumber::from(5_000_u32),
			BlockNumber::from(10_000_u32),
			BSX,
			KSM,
			BOB,
			Permill::from_percent(38),
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			30_000_000_000,
			BlockNumber::from(500_u32),
			BlockNumber::from(100_u32),
			BSX,
			BSX,
			GC,
			Permill::from_percent(50),
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::NewFarm {
				farm_id: PREDEFINED_GLOBAL_POOLS[0].id,
				owner: PREDEFINED_GLOBAL_POOLS[0].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[0].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[0].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[0].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[0].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[0].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[0].max_reward_per_period,
			}),
			frame_system::Event::NewAccount(187989685649991564771226578797).into(),
			orml_tokens::Event::Endowed(4_000, 187989685649991564771226578797, 1_000_000_000).into(),
			mock::Event::LiquidityMining(Event::NewFarm {
				farm_id: PREDEFINED_GLOBAL_POOLS[1].id,
				owner: PREDEFINED_GLOBAL_POOLS[1].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[1].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[1].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[1].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[1].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[1].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[1].max_reward_per_period,
			}),
			frame_system::Event::NewAccount(267217848164255902364770529133).into(),
			orml_tokens::Event::Endowed(1_000, 267217848164255902364770529133, 30_000_000_000).into(),
			mock::Event::LiquidityMining(Event::NewFarm {
				farm_id: PREDEFINED_GLOBAL_POOLS[2].id,
				owner: PREDEFINED_GLOBAL_POOLS[2].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[2].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[2].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[2].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[2].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[2].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[2].max_reward_per_period,
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
					asset_in: BSX,
					asset_out: KSM,
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
					asset_out: TO1,
				},
				(BSX_TO1_AMM, BSX_TO1_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: TO2,
				},
				(BSX_TO2_AMM, BSX_TO2_SHARE_ID),
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
				asset_out: TO1,
			},
			FixedU128::from(5_u128),
			Some(LoyaltyCurve::default()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::NewLiquidityPoolFarm {
			farm_id: GC_FARM,
			liq_pool_farm_id: 4,
			multiplier: FixedU128::from(5),
			nft_class: 0,
			asset_pair: AssetPair {
				asset_in: BSX,
				asset_out: TO1,
			},
		})]);

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			AssetPair {
				asset_in: BSX,
				asset_out: TO2,
			},
			FixedU128::from(10_u128),
			Some(LoyaltyCurve::default()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::NewLiquidityPoolFarm {
			farm_id: GC_FARM,
			liq_pool_farm_id: 5,
			multiplier: FixedU128::from(10),
			nft_class: 1,
			asset_pair: AssetPair {
				asset_in: BSX,
				asset_out: TO2,
			},
		})]);
	});

	ext
}

pub fn predefined_test_ext_with_deposits() -> sp_io::TestExternalities {
	let mut ext = predefined_test_ext();

	ext.execute_with(|| {
		let farm_id = GC_FARM;
		let amm_1 = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		let amm_2 = AssetPair {
			asset_in: BSX,
			asset_out: TO2,
		};

		let pallet_acc = LiquidityMining::account_id();
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_amm_1_farm_acc = LiquidityMining::pool_account_id(4).unwrap();
		let liq_pool_amm_2_farm_acc = LiquidityMining::pool_account_id(5).unwrap();
		let amm_1_acc = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(amm_1)).unwrap().0);
		let amm_2_acc = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(amm_2)).unwrap().0);
		//DEPOSIT 1:
		run_to_block(1_800); //18-th period

		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 50, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_1,
			50
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::DepositShares {
			farm_id: GC_FARM,
			liq_pool_farm_id: 5,
			who: ALICE,
			lp_token: BSX_TO1_SHARE_ID,
			amount: 50,
			nft_class: 1,
			nft_instance_id: 0,
		})]);

		// DEPOSIT 2 (deposit in same period):
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 52, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(Origin::signed(BOB), farm_id, amm_1, 80));

		expect_events(vec![mock::Event::LiquidityMining(Event::DepositShares {
			farm_id: GC_FARM,
			liq_pool_farm_id: 4,
			who: BOB,
			lp_token: BSX_TO1_SHARE_ID,
			amount: 80,
			nft_class: 1,
			nft_instance_id: 1,
		})]);

		// DEPOSIT 3 (same period, second liq pool yield farm):
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 8, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(Origin::signed(BOB), farm_id, amm_2, 25));

		expect_events(vec![mock::Event::LiquidityMining(Event::DepositShares {
			farm_id: GC_FARM,
			liq_pool_farm_id: 5,
			who: BOB,
			lp_token: BSX_TO2_SHARE_ID,
			amount: 25,
			nft_class: 1,
			nft_instance_id: 1,
		})]);

		// DEPOSIT 4 (new period):
		run_to_block(2051); //period 20
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 58, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			amm_2,
			800
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::DepositShares {
			farm_id: GC_FARM,
			liq_pool_farm_id: 5,
			who: BOB,
			lp_token: BSX_TO2_SHARE_ID,
			amount: 800,
			nft_class: 1,
			nft_instance_id: 1,
		})]);

		// DEPOSIT 5 (same period, second liq pool yield farm):
		run_to_block(2_586); //period 20
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 3, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_2,
			87
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::DepositShares {
			farm_id: GC_FARM,
			liq_pool_farm_id: 5,
			who: BOB,
			lp_token: BSX_TO2_SHARE_ID,
			amount: 87,
			nft_class: 1,
			nft_instance_id: 2,
		})]);

		// DEPOSIT 6 (same period):
		run_to_block(2_596); //period 20
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 16, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_2,
			48
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::DepositShares {
			farm_id: GC_FARM,
			liq_pool_farm_id: 5,
			who: ALICE,
			lp_token: BSX_TO2_SHARE_ID,
			amount: 48,
			nft_class: 1,
			nft_instance_id: 3,
		})]);

		// DEPOSIT 7 : (same period differen liq poll farm)
		run_to_block(2_596); //period 20
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 80, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_1,
			486
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::DepositShares {
			farm_id: GC_FARM,
			liq_pool_farm_id: 4,
			who: ALICE,
			lp_token: BSX_TO1_SHARE_ID,
			amount: 486,
			nft_class: 1,
			nft_instance_id: 22,
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 227_700,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 25,
				accumulated_rpvs: 120,
				accumulated_rpz: 12,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 3, GC_FARM));
		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 4, GC_FARM));

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 616);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 960);

		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 1_164_400));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 212_400);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), 952_000);

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE), 3_000_000 - 536);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE), 3_000_000 - 135);

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB), 2_000_000 - 80);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB), 2_000_000 - 825);
	});

	ext
}

#[test]
fn get_period_number_should_work() {
	let num_1: BlockNumber = 1_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1, 1).unwrap(), 1);

	let num_1: BlockNumber = 1_000_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1, 1).unwrap(), 1_000);

	let num_1: BlockNumber = 23_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1, 15).unwrap(), 1);

	let num_1: BlockNumber = 843_712_398_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1, 13_412_341).unwrap(), 62);

	let num_1: BlockNumber = 843_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1, 2_000).unwrap(), 0);

	let num_1: BlockNumber = 10_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1, 10).unwrap(), 1);
}

#[test]
fn get_period_number_should_not_work() {
	let num_1: BlockNumber = 10_u64;
	assert_err!(LiquidityMining::get_period_number(num_1, 0), Error::<Test>::Overflow);
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=432121354
fn get_loyalty_multiplier_should_work() {
	let c1 = LoyaltyCurve::default();
	let c2 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from(1),
		scale_coef: 50,
	};
	let c3 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from_inner(123_580_000_000_000_000), // 0.12358
		scale_coef: 23,
	};
	let c4 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from_inner(0), // 0.12358
		scale_coef: 15,
	};

	//vec[(periods, c1-multiplier, c2-multiplier, c3-multiplier, c4-multiplier),...]
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
	for t in testing_values.iter() {
		//1th curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, Some(c1.clone())).unwrap();
		assert!(is_approx_eq_fixedu128(m, t.1, precission_delta));

		//2nd curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, Some(c2.clone())).unwrap();
		assert!(is_approx_eq_fixedu128(m, t.2, precission_delta));

		//3rd curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, Some(c3.clone())).unwrap();
		assert!(is_approx_eq_fixedu128(m, t.3, precission_delta));

		//4th curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, Some(c4.clone())).unwrap();
		assert!(is_approx_eq_fixedu128(m, t.4, precission_delta));
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=906912221
fn get_reward_per_period_should_work() {
	//vec[(yield_per_period, total_global_farm_shares (spec: Z), max_reward_per_period, reward_per_period),...]
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

	for t in testing_values.iter() {
		assert_eq!(
			LiquidityMining::get_global_pool_reward_per_period(t.0, t.1, t.2).unwrap(),
			t.3
		);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=478231890
fn get_accumulated_rps_should_work() {
	//vec[(AccPRSprevious, total_shares,reward,  newAccRPS),...]
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

	for t in testing_values.iter() {
		assert_eq!(LiquidityMining::get_accumulated_rps(t.0, t.1, t.2).unwrap(), t.3);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=1775700162
fn get_user_reward_should_work() {
	//[(user_accumulated_claimed_rewards, loyalty_multiplier, user_reward, unclaimable_rewards),...]
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

	for t in testing_values.iter() {
		assert_eq!(
			LiquidityMining::get_user_reward(t.0, t.1, t.3, t.2, t.4).unwrap(),
			(t.5, t.6)
		);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=2010118745
fn update_global_pool_should_work() {
	//[(pool.updated_at, pool.total_shares, pool.accumulated_rps, pool.reward_currency,
	//pool_id, reward_left_to_distribute, period_now, reward_per_period, pool.accumulated_reward, pool.accumulated_rps, pool.accumulated_reward),...]
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

	//[(pool.updated_at, pool.total_shares, pool.accumulated_rps, pool.reward_currency,
	//pool_id, reward_left_to_distribute, period_now, reward_per_period, pool.accumulated_rps),...]
	for t in testing_values.iter() {
		let yield_per_period = Permill::from_percent(50);
		let planned_yielding_periods = 100;
		let blocks_per_period = 0;
		let owner = ALICE;
		let incentivized_token = BSX;
		let max_reward_per_period = 10_000_u128;

		let mut p = GlobalPool::new(
			t.4,
			t.0,
			t.3,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		p.total_shares_z = t.1;
		p.accumulated_rewards = t.8;
		p.accumulated_rpz = t.2;
		p.paid_accumulated_rewards = 10;

		let mut ext = new_test_ext();

		ext.execute_with(|| {
			let farm_account_id = LiquidityMining::pool_account_id(t.4).unwrap();
			let _ = Tokens::transfer(Origin::signed(TREASURY), farm_account_id, t.3, t.5);
			assert_eq!(Tokens::free_balance(t.3, &farm_account_id), t.5);

			LiquidityMining::update_global_pool(&mut p, t.6, t.7).unwrap();

			let mut rhs_p = GlobalPool::new(
				t.4,
				t.6,
				t.3,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			rhs_p.total_shares_z = t.1;
			rhs_p.paid_accumulated_rewards = 10;
			rhs_p.accumulated_rpz = t.9;
			rhs_p.accumulated_rewards = t.10;

			assert_eq!(p, rhs_p);
		});
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=1562134162
fn claim_from_global_pool_should_work() {
	//(pool.updated_at, pool.total_shares, pool.accumulated_rps_start, pool.accumulated_rps, pool.reward_currency, pool.accumululated_rewards, pool.paid_accumularted_rewards, shares , reward, pool.accumulated_rps_start, pool.accumululated_rewards, pool.paid_accumularted_rewards)
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

	//(pool.updated_at, pool.total_shares, pool.accumulated_rps_start, pool.accumulated_rps, pool.reward_currency, pool.accumululated_rewards, pool.paid_accumularted_rewards, shares , reward, pool.accumulated_rps_start, pool.accumululated_rewards, pool.paid_accumularted_rewards)
	for t in testing_values.iter() {
		let g_pool_id = 1;
		let liq_pool_id = 2;
		let yield_per_period = Permill::from_percent(50);
		let planned_yielding_periods = 100;
		let blocks_per_period = 1;
		let owner = ALICE;
		let incentivized_token = BSX;
		let max_reward_per_period = Balance::from(10_000_u32);

		let mut g_pool = GlobalPool::new(
			g_pool_id,
			t.0,
			t.4,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		g_pool.total_shares_z = t.1;
		g_pool.accumulated_rpz = t.3;
		g_pool.accumulated_rewards = t.5;
		g_pool.paid_accumulated_rewards = t.6;

		let mut liq_pool = LiquidityPoolYieldFarm::new(liq_pool_id, t.0, None, FixedU128::from(10_u128), 1);
		liq_pool.accumulated_rpz = t.2;

		assert_eq!(
			LiquidityMining::claim_from_global_pool(&mut g_pool, &mut liq_pool, t.7).unwrap(),
			t.8
		);

		let mut rhs_g_pool = GlobalPool::new(
			g_pool_id,
			t.0,
			t.4,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		rhs_g_pool.total_shares_z = t.1;
		rhs_g_pool.accumulated_rpz = t.3;
		rhs_g_pool.accumulated_rewards = t.10;
		rhs_g_pool.paid_accumulated_rewards = t.11;

		assert_eq!(g_pool, rhs_g_pool);

		let mut rhs_liq_pool = LiquidityPoolYieldFarm::new(liq_pool_id, t.0, None, FixedU128::from(10_u128), 1);
		rhs_liq_pool.accumulated_rpz = t.9;

		assert_eq!(liq_pool, rhs_liq_pool);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=1639947555
fn update_pool_should_work() {
	//(globaPoolId, PoolId, pool.updated_at, period_now, pool.accRPS,pool.total_shares, globaPool.reward_currency, pool.accRPS-new, pool.updated_at-new, pool.account-balance, global_pool.account-balance)
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

	for t in testing_values.iter() {
		let owner = ALICE;
		let gid = t.0;
		let yield_per_period = Permill::from_percent(50);
		let blocks_per_period = BlockNumber::from(1_u32);
		let planned_yielding_periods = 100;
		let incentivized_token = BSX;
		let updated_at = 200_u64;
		let reward_currency = t.7;
		let max_reward_per_period = Balance::from(10_000_u32);

		let mut g_pool = GlobalPool::<Test>::new(
			gid,
			updated_at,
			reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		g_pool.total_shares_z = 1_000_000_u128;
		g_pool.accumulated_rpz = 200_u128;
		g_pool.accumulated_rewards = 1_000_000_u128;
		g_pool.paid_accumulated_rewards = 1_000_000_u128;

		let mut liq_pool = LiquidityPoolYieldFarm {
			id: t.1,
			updated_at: t.2,
			total_shares: 200_u128,
			total_valued_shares: t.5,
			accumulated_rpvs: t.4,
			accumulated_rpz: 200_u128,
			loyalty_curve: None,
			stake_in_global_pool: Balance::from(10_000_u32),
			multiplier: FixedU128::from(10_u128),
			nft_class: 1,
			canceled: false,
		};

		let mut ext = new_test_ext();

		let farm_account_id = LiquidityMining::pool_account_id(t.0).unwrap();
		let pool_account_id = LiquidityMining::pool_account_id(t.1).unwrap();

		ext.execute_with(|| {
			let _ = Tokens::transfer(
				Origin::signed(TREASURY),
				farm_account_id,
				g_pool.reward_currency,
				9_000_000_000_000,
			);
			assert_eq!(
				Tokens::free_balance(g_pool.reward_currency, &farm_account_id),
				9_000_000_000_000_u128
			);

			assert_eq!(Tokens::free_balance(t.7, &pool_account_id), 0);

			assert_ok!(LiquidityMining::update_liq_pool(&mut liq_pool, t.6, t.3, t.0, t.7));

			let mut rhs_g_pool = GlobalPool::new(
				gid,
				updated_at,
				reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			rhs_g_pool.updated_at = 200_u64;
			rhs_g_pool.total_shares_z = 1_000_000_u128;
			rhs_g_pool.accumulated_rpz = 200_u128;
			rhs_g_pool.accumulated_rewards = 1_000_000_u128;
			rhs_g_pool.paid_accumulated_rewards = 1_000_000_u128;

			assert_eq!(g_pool, rhs_g_pool);

			assert_eq!(
				liq_pool,
				LiquidityPoolYieldFarm {
					id: t.1,
					updated_at: t.9,
					total_shares: 200_u128,
					total_valued_shares: t.5,
					accumulated_rpvs: t.8,
					accumulated_rpz: 200_u128,
					loyalty_curve: None,
					stake_in_global_pool: Balance::from(10_000_u32),
					multiplier: FixedU128::from(10_u128),
					nft_class: 1,
					canceled: false,
				}
			);

			assert_eq!(Tokens::free_balance(g_pool.reward_currency, &farm_account_id), t.11);
			assert_eq!(Tokens::free_balance(g_pool.reward_currency, &pool_account_id), t.10);
		});
	}
}

#[test]
fn next_id_should_work() {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_eq!(LiquidityMining::get_next_id().unwrap(), 1);
		assert_eq!(LiquidityMining::pool_id(), 1);

		assert_eq!(LiquidityMining::get_next_id().unwrap(), 2);
		assert_eq!(LiquidityMining::pool_id(), 2);

		assert_eq!(LiquidityMining::get_next_id().unwrap(), 3);
		assert_eq!(LiquidityMining::pool_id(), 3);

		assert_eq!(LiquidityMining::get_next_id().unwrap(), 4);
		assert_eq!(LiquidityMining::pool_id(), 4);
	});
}

#[test]
fn pool_account_id_should_work() {
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::MAX];

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
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::MAX];

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
	// total rawards
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

	//invalid min_farming_periods
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

	//invalid block per period
	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 0, Permill::from_percent(50)),
		Error::<Test>::InvalidBlocksPerPeriod
	);

	//invalid yield per period
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

		run_to_block(created_at_block);

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

		expect_events(vec![mock::Event::LiquidityMining(Event::NewFarm {
			farm_id: global_pool.id,
			owner: global_pool.owner,
			reward_currency: global_pool.reward_currency,
			yield_per_period: global_pool.yield_per_period,
			planned_yielding_periods: global_pool.planned_yielding_periods,
			blocks_per_period: global_pool.blocks_per_period,
			incentivized_asset: global_pool.incentivized_asset,
			max_reward_per_period: global_pool.max_reward_per_period,
		})]);

		assert_eq!(LiquidityMining::global_pool(pool_id), Some(global_pool));
	});
}

#[test]
fn create_farm_from_basic_origin_should_not_work() {
	new_test_ext().execute_with(|| {
		let created_at_block = 15_896;

		run_to_block(created_at_block);

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

		run_to_block(created_at_block);

		//total_rewards bellow min.
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

		//planned_yielding_periods bellow min.
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
	//owner accont have 10K bsx
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_001,
				1_000,
				1,
				BSX,
				BSX,
				ACC_1M,
				Permill::from_percent(20)
			),
			Error::<Test>::InsufficientRewardCurrencyBalance
		);
	});
}

#[test]
fn add_liquidity_pool_should_work() {
	//(AssetPair, LiqudityPoo, ammPoolId, Origin, farmId, now)

	//Note: global_pool.updated_at isn't changed because pool is empty (no. liq. pool stake in
	//globalPool)
	let test_data = vec![
		(
			AssetPair {
				asset_in: BSX,
				asset_out: ACA,
			},
			LiquidityPoolYieldFarm {
				id: 6,
				updated_at: 17,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(20_000_u128),
				loyalty_curve: Some(LoyaltyCurve::default()),
				nft_class: 2,
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
				asset_in: BSX,
				asset_out: KSM,
			},
			LiquidityPoolYieldFarm {
				id: 7,
				updated_at: 17,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(10_000_u128),
				loyalty_curve: None,
				nft_class: 3,
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
				id: 8,
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
				nft_class: 4,
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
				id: 9,
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
				nft_class: 5,
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
		for (assets, pool, amm_id, who, farm_id, now, g_pool) in test_data.clone() {
			run_to_block(now);
			assert_ok!(LiquidityMining::add_liquidity_pool(
				Origin::signed(who),
				farm_id,
				assets,
				pool.multiplier,
				pool.loyalty_curve.clone()
			));

			expect_events(vec![mock::Event::LiquidityMining(Event::NewLiquidityPoolFarm {
				farm_id,
				liq_pool_farm_id: pool.id,
				multiplier: pool.multiplier,
				nft_class: pool.nft_class,
				asset_pair: assets,
			})]);

			assert_eq!(LiquidityMining::global_pool(farm_id).unwrap(), g_pool);
		}

		for (_, pool, amm_id, _, farm_id, _, _) in test_data {
			assert_eq!(LiquidityMining::liquidity_pool(farm_id, amm_id).unwrap(), pool);
		}
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
fn add_liquidity_pool_invalid_weight_should_not_work() {
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
		run_to_block(20_000);
		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(ALICE),
			ALICE_FARM,
			AssetPair {
				//AMM for this assetPair does not exist
				asset_in: BSX,
				asset_out: ACA,
			},
			FixedU128::from(10_000_u128),
			Some(LoyaltyCurve::default())
		));

		let existing_pool = LiquidityPoolYieldFarm {
			id: 6,
			updated_at: 20,
			total_shares: 0,
			total_valued_shares: 0,
			accumulated_rpvs: 0,
			accumulated_rpz: 0,
			loyalty_curve: Some(LoyaltyCurve::default()),
			stake_in_global_pool: 0,
			multiplier: FixedU128::from(10_000_u128),
			nft_class: 2,
			canceled: false,
		};
		assert_eq!(
			LiquidityMining::liquidity_pool(ALICE_FARM, BSX_ACA_AMM).unwrap(),
			existing_pool
		);

		expect_events(vec![
			Event::LiquidityPoolAdded(ALICE_FARM, BSX_ACA_AMM, existing_pool).into()
		]);

		//try to add duplicate pool
		//in the same block(period)
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					//AMM for this assetPair does not exist
					asset_in: BSX,
					asset_out: ACA,
				},
				FixedU128::from(9_000_u128),
				Some(LoyaltyCurve::default()),
			),
			Error::<Test>::LiquidityPoolAlreadyExists
		);

		run_to_block(30_000);
		//in later block(period)
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					//AMM for this assetPair does not exist
					asset_in: BSX,
					asset_out: ACA,
				},
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
		//remove all rewards from reward account
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

		expect_events(vec![Event::FarmDestroyed(BOB_FARM, BOB).into()]);

		assert!(LiquidityMining::global_pool(BOB_FARM).is_none());
	});
}

#[test]
fn destroy_farm_not_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		//remove all rewards from reward account
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
	//in this case all rewards was distributed but liq. pool still exists in farm
	predefined_test_ext().execute_with(|| {
		//remove all rewards from reward account
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
fn destroy_farm_healthy_should_not_work() {
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
		let amm_1 = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		let amm_2 = AssetPair {
			asset_in: BSX,
			asset_out: TO2,
		};

		let pallet_acc = LiquidityMining::account_id();
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_amm_1_farm_acc = LiquidityMining::pool_account_id(4).unwrap();
		let liq_pool_amm_2_farm_acc = LiquidityMining::pool_account_id(5).unwrap();
		let amm_1_acc = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(amm_1)).unwrap().0);
		let amm_2_acc = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(amm_2)).unwrap().0);
		//DEPOSIT 1:
		run_to_block(1_800); //18-th period

		let alice_bsx_to1_shares = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 50, 0).unwrap();
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 0);

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_1,
			50
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 4, ALICE, 50, BSX_TO1_SHARE_ID, 0).into()
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				total_shares: 50,
				total_valued_shares: 2_500,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 12_500,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 1, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(0, 0).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 0,
				entered_at: 18,
				updated_at: 18,
			},
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_bsx_to1_shares - 50
		);
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 50);

		// DEPOSIT 2 (deposit in same period):
		let bob_bsx_to1_shares = Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB);
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 52, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(Origin::signed(BOB), farm_id, amm_1, 80));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 4, BOB, 80, BSX_TO1_SHARE_ID, 1).into()
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 18,
				accumulated_rpvs: 45,
				accumulated_rpz: 9,
				total_shares: 130,
				total_valued_shares: 6_660,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 33_300,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 2, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(0, 1).unwrap(),
			Deposit {
				shares: 80,
				valued_shares: 4_160,
				accumulated_rpvs: 45,
				accumulated_claimed_rewards: 0,
				entered_at: 18,
				updated_at: 18,
			},
		);

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB), bob_bsx_to1_shares - 80);
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 130);

		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 112_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);

		// DEPOSIT 3 (same period, second liq pool yield farm):
		let bob_bsx_to2_shares = Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB);
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 8, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(Origin::signed(BOB), farm_id, amm_2, 25));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, BOB, 25, BSX_TO2_SHARE_ID, 0).into()
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 0,
				accumulated_rpvs: 0,
				accumulated_rpz: 0,
				total_shares: 25,
				total_valued_shares: 200,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 2_000,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 1, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(1, 0).unwrap(),
			Deposit {
				shares: 25,
				valued_shares: 200,
				accumulated_rpvs: 0,
				accumulated_claimed_rewards: 0,
				entered_at: 18,
				updated_at: 18,
			},
		);

		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB), bob_bsx_to2_shares - 25);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 25);

		//no pools update no transfers
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 112_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), 0);

		// DEPOSIT 4 (new period):
		run_to_block(2051); //period 20
		let bob_bsx_to2_shares = Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB);
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 58, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			amm_2,
			800
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, BOB, 800, BSX_TO2_SHARE_ID, 1).into()
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 20,
				accumulated_rpvs: 100,
				accumulated_rpz: 10,
				total_shares: 825,
				total_valued_shares: 46_600,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 466_000,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 2, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(1, 1).unwrap(),
			Deposit {
				shares: 800,
				valued_shares: 46_400,
				accumulated_rpvs: 100,
				accumulated_claimed_rewards: 0,
				entered_at: 20,
				updated_at: 20,
			},
		);

		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB), bob_bsx_to2_shares - 800);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 825);

		//no pools update no transfers
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 132_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), 20_000);

		// DEPOSIT 5 (same period, second liq pool yield farm):
		run_to_block(2_586); //period 20
		let alice_bsx_to2_shares = Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE);
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 3, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_2,
			87
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, ALICE, 87, BSX_TO2_SHARE_ID, 2).into()
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 25,
				accumulated_rpvs: 120,
				accumulated_rpz: 12,
				total_shares: 912,
				total_valued_shares: 46_861,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 468_610,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 3, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(1, 2).unwrap(),
			Deposit {
				shares: 87,
				valued_shares: 261,
				accumulated_rpvs: 120,
				accumulated_claimed_rewards: 0,
				entered_at: 25,
				updated_at: 25,
			},
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE),
			alice_bsx_to2_shares - 87
		);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 912);

		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 1_064_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), (20_000 + 932_000)); //NOTE: 20k from prew deposit

		// DEPOSIT 6 (same period):
		run_to_block(2_596); //period 20
		let alice_bsx_to2_shares = Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE);
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 16, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_2,
			48
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, ALICE, 48, BSX_TO2_SHARE_ID, 3).into()
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 25,
				accumulated_rpvs: 120,
				accumulated_rpz: 12,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 4, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(1, 3).unwrap(),
			Deposit {
				shares: 48,
				valued_shares: 768,
				accumulated_rpvs: 120,
				accumulated_claimed_rewards: 0,
				entered_at: 25,
				updated_at: 25,
			},
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE),
			alice_bsx_to2_shares - 48
		);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 960);

		//no pools update no transfers
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 1_064_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), (20_000 + 932_000)); //NOTE: 20k from prew deposit

		// DEPOSIT 7 : (same period differen liq poll farm)
		run_to_block(2_596); //period 20
		let alice_bsx_to1_shares = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 80, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_1,
			486
		));

		expect_events(vec![Event::SharesDeposited(
			GC_FARM,
			4,
			ALICE,
			486,
			BSX_TO1_SHARE_ID,
			2,
		)
		.into()]);

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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 227_700,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 3, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(0, 2).unwrap(),
			Deposit {
				shares: 486,
				valued_shares: 38_880,
				accumulated_rpvs: 60,
				accumulated_claimed_rewards: 0,
				entered_at: 25,
				updated_at: 25,
			},
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_bsx_to1_shares - 486
		);
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 616);

		//no pools update no transfers
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 1_164_400));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 212_400);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), 952_000);
	});
}

#[test]
fn deposit_shares_zero_deposit_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let assets = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		assert_noop!(
			LiquidityMining::deposit_shares(Origin::signed(ALICE), GC_FARM, assets, 0),
			Error::<Test>::InvalidDepositAmount
		);
	});
}

#[test]
fn deposit_shares_insufficient_shares_balance_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let assets = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		assert_noop!(
			LiquidityMining::deposit_shares(Origin::signed(ALICE), GC_FARM, assets, 4_000_000),
			Error::<Test>::InsufficientAmmSharesBalance
		);
	});
}

#[test]
fn deposit_shares_non_existing_liq_pool_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let assets = AssetPair {
			asset_in: BSX,
			asset_out: DOT,
		};

		assert_noop!(
			LiquidityMining::deposit_shares(Origin::signed(ALICE), GC_FARM, assets, 10_000),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn deposit_shares_canceled_liq_pool_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let assets = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		assert_noop!(
			LiquidityMining::deposit_shares(Origin::signed(ALICE), GC_FARM, assets, 10_000),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn claim_rewards_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_to1_lm_account = LiquidityMining::pool_account_id(4).unwrap();
		let bsx_to2_lm_account = LiquidityMining::pool_account_id(5).unwrap();
		let liq_pool_bsx_to1_rewarad_balance = Tokens::free_balance(BSX, &bsx_to1_lm_account);

		//claim A1.1  (dep A1 1-th time)
		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0));

		expect_events(vec![Event::RewardClaimed(ALICE, GC_FARM, 4, 79_906, BSX).into()]);

		assert_eq!(
			LiquidityMining::deposit(0, 0).unwrap(),
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
			Tokens::free_balance(BSX, &bsx_to1_lm_account),
			liq_pool_bsx_to1_rewarad_balance - 79_906
		);

		// claim B3.1
		run_to_block(3_056);
		let liq_pool_bsx_to2_rewarad_balance = Tokens::free_balance(BSX, &bsx_to2_lm_account);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 1, 2));

		expect_events(vec![Event::RewardClaimed(ALICE, GC_FARM, 5, 2_734, BSX).into()]);

		assert_eq!(
			LiquidityMining::deposit(1, 2).unwrap(),
			Deposit {
				shares: 87,
				valued_shares: 261,
				accumulated_rpvs: 120,
				accumulated_claimed_rewards: 2_734,
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 30,
				accumulated_rpvs: 140,
				accumulated_rpz: 14,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + 2_734);
		//NOTE: + claim from global pool - paid reward to user
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_to2_lm_account),
			liq_pool_bsx_to2_rewarad_balance + 952_580 - 2_734
		);

		//run for log time(longer than planned_yielding_periods) without interaction or claim.
		//planned_yielding_periods = 500; 100 blocks per period
		//claim A1.2
		run_to_block(125_879);
		let liq_pool_bsx_to1_rewarad_balance = Tokens::free_balance(BSX, &bsx_to1_lm_account);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0));

		expect_events(vec![Event::RewardClaimed(ALICE, GC_FARM, 4, 7_477_183, BSX).into()]);

		assert_eq!(
			LiquidityMining::deposit(0, 0).unwrap(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 1_258,
				accumulated_rpvs: 3_140,
				accumulated_rpz: 628,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 227_700,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 30,
				accumulated_rpvs: 140,
				accumulated_rpz: 14,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + 7_477_183);
		//NOTE: + claim from global pool - paid reward to user
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_to1_lm_account),
			liq_pool_bsx_to1_rewarad_balance + 140_263_200 - 7_477_183
		);
	});
}

#[test]
fn claim_rewards_double_claim_in_the_same_period_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_to1_lm_account = LiquidityMining::pool_account_id(4).unwrap();
		let liq_pool_bsx_to1_rewarad_balance = Tokens::free_balance(BSX, &bsx_to1_lm_account);

		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0));

		expect_events(vec![Event::RewardClaimed(ALICE, GC_FARM, 4, 79_906, BSX).into()]);

		assert_eq!(
			LiquidityMining::deposit(0, 0).unwrap(),
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
			Tokens::free_balance(BSX, &bsx_to1_lm_account),
			liq_pool_bsx_to1_rewarad_balance - 79_906
		);

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0),
			Error::<Test>::DoubleClaimInThePeriod
		);
	});
}

#[test]
fn claim_rewards_invalid_nft_class_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let invalid_nft_class = 5486;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), invalid_nft_class, 0),
			Error::<Test>::NftClassDoesNotExists
		);
	});
}

#[test]
fn claim_rewards_invalid_nft_id_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let invalid_nft_id = 684;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, invalid_nft_id),
			Error::<Test>::NftDoesNotExist
		);
	});
}

#[test]
fn claim_rewards_from_canceled_pool_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let assets = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn claim_rewards_from_removed_pool_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let assets = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn claim_rewards_not_deposit_owner_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_OWNER: u128 = BOB;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(NOT_OWNER), 0, 0),
			Error::<Test>::NotDepositOwner
		);
	});
}

#[test]
fn withdraw_shares_should_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	let amm_2 = AssetPair {
		asset_in: BSX,
		asset_out: TO2,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		const REWARD_CURRENCY: u32 = BSX;

		let pallet_acc = LiquidityMining::account_id();
		let liq_pool_amm_1_acc = LiquidityMining::pool_account_id(4).unwrap();
		let liq_pool_amm_2_acc = LiquidityMining::pool_account_id(5).unwrap();
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();

		// withdraw 1A
		let alice_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let pallet_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 0, 0));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, 4, 79_906, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, 50).into(),
			pallet_uniques::Event::Burned(0, 0, 1).into(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 566,
				total_valued_shares: 43_040,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 215_200,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_rew_curr_balance + 79_906
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_amm_1_shares_balance + 50
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_amm_1_shares_balance - 50
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance
		);

		//liq pool farms balance checks
		//NOTE ... - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance - (79_906 + 70_094)
		);
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 70_094
		);

		assert_eq!(LiquidityMining::deposit(0, 0), None);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 2, GC_FARM));

		run_to_block(12_800);

		// withdraw 3B
		let alice_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let pallet_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 1, 2));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, 5, 100_324, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO2_SHARE_ID, 87).into(),
			pallet_uniques::Event::Burned(1, 2, 1).into(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 566,
				total_valued_shares: 43_040,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 215_200,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 128,
				accumulated_rpvs: 630,
				accumulated_rpz: 63,
				total_shares: 873,
				total_valued_shares: 47_368,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 473_680,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_rew_curr_balance + 100_324
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE),
			alice_amm_2_shares_balance + 87
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_amm_1_shares_balance
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance - 87
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance
		);
		//NOTE ... pool reward - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			(liq_pool_amm_2_rew_curr_balance + 24_290_790 - (100_324 + 32_786))
		);

		//global pool balance checks
		//note ... + unclaimabe - pool reward
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 32_786 - 24_290_790
		);

		assert_eq!(LiquidityMining::deposit(1, 2), None);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 3, GC_FARM));

		// withdraw 3A
		let alice_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let pallet_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 0, 2));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, 4, 7_472_429, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, 486).into(),
			pallet_uniques::Event::Burned(0, 2, 1).into(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 128,
				accumulated_rpvs: 315,
				accumulated_rpz: 63,
				total_shares: 80,
				total_valued_shares: 4_160,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 20_800,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_rew_curr_balance + 7_472_429
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_amm_1_shares_balance + 486
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_amm_1_shares_balance - 486
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance + 10_975_200 - (7_472_429 + 2_441_971)
		);
		//NOTE ... pool reward - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance
		);

		//global pool balance checks
		//note ... + unclaimabe - pool reward
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 2_441_971 - 10_975_200
		);

		assert_eq!(LiquidityMining::deposit(0, 2), None);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 1, GC_FARM));

		// withdraw 2A
		let bob_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB);
		let bob_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(BOB), 0, 1));

		expect_events(vec![
			Event::RewardClaimed(BOB, GC_FARM, 4, 855_771, BSX).into(),
			Event::SharesWithdrawn(BOB, BSX_TO1_SHARE_ID, 80).into(),
			pallet_uniques::Event::Burned(0, 1, 2).into(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 128,
				accumulated_rpvs: 315,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_rew_curr_balance + 855_771
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB),
			bob_amm_1_shares_balance + 80
		);

		//pallet balances checks
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 0);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance - (855_771 + 267_429)
		);
		//NOTE ... pool reward - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance
		);

		//global pool balance checks
		//note ... + unclaimabe - pool reward
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 267_429
		);

		assert_eq!(LiquidityMining::deposit(0, 1), None);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 0, GC_FARM));

		// withdraw 1B
		let bob_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB);
		let bob_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(BOB), 1, 0));

		expect_events(vec![
			Event::RewardClaimed(BOB, GC_FARM, 5, 95_999, BSX).into(),
			Event::SharesWithdrawn(BOB, BSX_TO2_SHARE_ID, 25).into(),
			pallet_uniques::Event::Burned(1, 0, 2).into(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 128,
				accumulated_rpvs: 315,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(5_u128),
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 128,
				accumulated_rpvs: 630,
				accumulated_rpz: 63,
				total_shares: 848,
				total_valued_shares: 47_168,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 471_680,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_rew_curr_balance + 95_999
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB),
			bob_amm_2_shares_balance + 25
		);

		//pallet balances checks
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 0);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance - 25
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance
		);
		//NOTE ... - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance - (95_999 + 30_001)
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 30_001
		);

		assert_eq!(LiquidityMining::deposit(1, 0), None);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 2, GC_FARM));

		// withdraw 4B
		let alice_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 1, 3));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, 5, 295_207, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO2_SHARE_ID, 48).into(),
			pallet_uniques::Event::Burned(1, 3, 1).into(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 128,
				accumulated_rpvs: 630,
				accumulated_rpz: 63,
				total_shares: 800,
				total_valued_shares: 46_400,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 464_000,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_rew_curr_balance + 29_5207
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE),
			alice_amm_2_shares_balance + 48
		);

		//pallet balances checks
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 0);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance - 48
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance
		);
		//NOTE ... - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance - (29_5207 + 96_473)
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 96_473
		);

		assert_eq!(LiquidityMining::deposit(1, 3), None);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 1, GC_FARM));

		// withdraw 2B
		let bob_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB);
		let bob_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(BOB), 1, 1));

		expect_events(vec![
			Event::RewardClaimed(BOB, GC_FARM, 5, 18_680_461, BSX).into(),
			frame_system::Event::KilledAccount(29533360621462889584138678125).into(),
			Event::SharesWithdrawn(BOB, BSX_TO2_SHARE_ID, 800).into(),
			pallet_uniques::Event::Burned(1, 1, 2).into(),
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
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 128,
				accumulated_rpvs: 630,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: FixedU128::from(10_u128),
				nft_class: 1,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_rew_curr_balance + 18_680_461
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB),
			bob_amm_2_shares_balance + 800
		);

		//pallet balances checks
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 0);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 0);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance
		);
		//NOTE ... - (reward + unclaimabe)
		assert_eq!(Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc), 0);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 5_911_539
		);

		assert_eq!(LiquidityMining::deposit(1, 1), None);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 0, GC_FARM));
	});
}

#[test]
fn withdraw_shares_from_canceled_pool_should_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		run_to_block(10_000);

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		let liq_pool_id = 4;
		let pallet_acc = LiquidityMining::account_id();
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_acc = LiquidityMining::pool_account_id(liq_pool_id).unwrap();

		//1-th withdraw
		let liq_pool_rew_balance = Tokens::free_balance(BSX, &liq_pool_acc);
		let g_pool_rew_balance = Tokens::free_balance(BSX, &g_pool_acc);
		let pallet_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let alice_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 0, 0));

		let user_reward = 444_230;
		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, liq_pool_id, user_reward, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, 50).into(),
			pallet_uniques::Event::Burned(0, 0, 1).into(),
		]);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), g_pool);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				total_shares: liq_pool.total_shares - 50,
				total_valued_shares: liq_pool.total_valued_shares - 2500,
				..liq_pool
			}
		);

		assert_eq!(LiquidityMining::deposit(0, 0), None);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_shares_balance - 50
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_shares_balance + 50
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + user_reward);

		let unclaimable_rewards = 168_270;
		assert_eq!(
			Tokens::free_balance(BSX, &g_pool_acc),
			g_pool_rew_balance + unclaimable_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_acc),
			liq_pool_rew_balance - user_reward - unclaimable_rewards
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (assets, 2, GC_FARM));

		//2-nd withdraw
		let liq_pool_rew_balance = Tokens::free_balance(BSX, &liq_pool_acc);
		let g_pool_rew_balance = Tokens::free_balance(BSX, &g_pool_acc);
		let pallet_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let alice_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();

		let user_reward = 5_137_714;
		let unclaimable_rewards = 2_055_086;
		let shares_amount = 486;
		let valued_shares_amount = 38_880;

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 0, 2));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, liq_pool_id, user_reward, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, shares_amount).into(),
			pallet_uniques::Event::Burned(0, 2, 1).into(),
		]);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), g_pool);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				total_shares: liq_pool.total_shares - shares_amount,
				total_valued_shares: liq_pool.total_valued_shares - valued_shares_amount,
				..liq_pool
			}
		);

		assert_eq!(LiquidityMining::deposit(0, 2), None);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_shares_balance - shares_amount
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_shares_balance + shares_amount
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + user_reward);

		assert_eq!(
			Tokens::free_balance(BSX, &g_pool_acc),
			g_pool_rew_balance + unclaimable_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_acc),
			liq_pool_rew_balance - user_reward - unclaimable_rewards
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (assets, 1, GC_FARM));

		//3-th withdraw
		let liq_pool_rew_balance = Tokens::free_balance(BSX, &liq_pool_acc);
		let g_pool_rew_balance = Tokens::free_balance(BSX, &g_pool_acc);
		let pallet_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let bob_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB);
		let bob_bsx_balance = Tokens::free_balance(BSX, &BOB);

		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();

		let user_reward = 603_428;
		let unclaimable_rewards = 228_572;
		let shares_amount = 80;

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(BOB), 0, 1));

		expect_events(vec![
			Event::RewardClaimed(BOB, GC_FARM, liq_pool_id, user_reward, BSX).into(),
			Event::SharesWithdrawn(BOB, BSX_TO1_SHARE_ID, shares_amount).into(),
			pallet_uniques::Event::Burned(0, 1, 2).into(),
			pallet_uniques::Event::Destroyed(0).into(),
		]);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), g_pool);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				total_shares: 0,
				total_valued_shares: 0,
				..liq_pool
			}
		);

		assert_eq!(LiquidityMining::deposit(0, 3), None);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_shares_balance - shares_amount
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB),
			bob_shares_balance + shares_amount
		);

		assert_eq!(Tokens::free_balance(BSX, &BOB), bob_bsx_balance + user_reward);

		assert_eq!(
			Tokens::free_balance(BSX, &g_pool_acc),
			g_pool_rew_balance + unclaimable_rewards
		);

		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_acc),
			liq_pool_rew_balance - user_reward - unclaimable_rewards
		);

		//last withdraw should destroy nft class
		assert_eq!(LiquidityMining::nft_class(0), None);
	});
}

#[test]
fn claim_and_reward_in_same_period_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_to1_lm_account = LiquidityMining::pool_account_id(4).unwrap();
		let liq_pool_bsx_to1_rewarad_balance = Tokens::free_balance(BSX, &bsx_to1_lm_account);
		let alice_lp_tokens_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		const LIQ_POOL_ID: PoolId = 4;

		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, LIQ_POOL_ID, 79_906, BSX).into()
		]);

		assert_eq!(
			LiquidityMining::deposit(0, 0).unwrap(),
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
			Tokens::free_balance(BSX, &bsx_to1_lm_account),
			liq_pool_bsx_to1_rewarad_balance - 79_906
		);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 0, 0));

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_lp_tokens_balance + 50
		);

		expect_events(vec![
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, 50).into(),
			pallet_uniques::Event::Burned(0, 0, 1).into(),
		]);
	});
}

#[test]
fn withdraw_shares_from_removed_pool_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let assets = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		run_to_block(10_000);

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		assert_eq!(LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM), None);

		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		let nft_class_id = 0;
		let pallet_acc = LiquidityMining::account_id();
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let pallet_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let g_pool_rew_balance = Tokens::free_balance(BSX, &g_pool_acc);
		let alice_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(BSX, &ALICE);

		//1-th withdraw
		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), nft_class_id, 0));

		let shares_amount = 50;

		expect_events(vec![
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, shares_amount).into(),
			pallet_uniques::Event::Burned(0, 0, 1).into(),
		]);

		assert_eq!(LiquidityMining::deposit(nft_class_id, 0), None);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), g_pool);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_shares_balance - shares_amount
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_shares_balance + shares_amount
		);

		//no reward paid from removed pool
		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_rew_curr_balance);
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), g_pool_rew_balance);

		//2-nd withdraw
		let alice_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(BSX, &ALICE);
		let pallet_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let shares_amount = 486;

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), nft_class_id, 2));

		expect_events(vec![
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, shares_amount).into(),
			pallet_uniques::Event::Burned(0, 2, 1).into(),
		]);

		assert_eq!(LiquidityMining::deposit(nft_class_id, 2), None);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), g_pool);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_shares_balance - shares_amount
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_shares_balance + shares_amount
		);

		//no reward paid from removed pool
		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_rew_curr_balance);
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), g_pool_rew_balance);

		//3-th withdraw
		let bob_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB);
		let bob_rew_curr_balance = Tokens::free_balance(BSX, &BOB);
		let shares_amount = 80;

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(BOB), nft_class_id, 1));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::WithdrawShares {
				who: BOB,
				lp_token: BSX_TO1_SHARE_ID,
				amount: shares_amount,
			}),
			pallet_uniques::Event::Burned(0, 1, 2).into(),
			pallet_uniques::Event::Destroyed(0).into(),
		]);

		assert_eq!(LiquidityMining::deposit(nft_class_id, 3), None);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), g_pool);

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 0);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB),
			bob_shares_balance + shares_amount
		);

		//no reward paid from removed pool
		assert_eq!(Tokens::free_balance(BSX, &BOB), bob_rew_curr_balance);
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), g_pool_rew_balance);

		assert_eq!(LiquidityMining::nft_class(nft_class_id), None);
	});
}

#[test]
fn withdraw_shares_invalid_nft_class_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let invalid_nft_class = 684;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), invalid_nft_class, 0),
			Error::<Test>::NftClassDoesNotExists
		);
	});
}

#[test]
fn withdraw_shares_invalid_nft_id_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let invalid_nft_id = 684;

		assert_noop!(
			LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, invalid_nft_id),
			Error::<Test>::NftDoesNotExist
		);
	});
}

#[test]
fn withdraw_shares_not_owner_should_not_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_OWNER: u128 = BOB;

		assert_noop!(
			LiquidityMining::withdraw_shares(Origin::signed(NOT_OWNER), 0, 0),
			Error::<Test>::NotDepositOwner
		);
	});
}

#[test]
fn cancel_liquidity_pool_should_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	//same period
	predefined_test_ext_with_deposits().execute_with(|| {
		let liq_pool_id = 4;
		let liq_pool_account = LiquidityMining::pool_account_id(liq_pool_id).unwrap();
		let g_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_rew_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let g_pool_rew_balance = Tokens::free_balance(BSX, &g_pool_account);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();
		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::CancelLiquidityPoolFarm {
			farm_id: GC_FARM,
			liq_pool_farm_id: liq_pool_id,
			who: GC,
			asset_pair: assets,
		})]);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
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
				total_shares_z: g_pool
					.total_shares_z
					.checked_sub(liq_pool.stake_in_global_pool)
					.unwrap(),
				..g_pool
			}
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (assets, 3, GC_FARM));

		assert_eq!(Tokens::free_balance(BSX, &liq_pool_account), liq_pool_rew_balance);
		assert_eq!(Tokens::free_balance(BSX, &g_pool_account), g_pool_rew_balance);
	});

	//with pools update
	predefined_test_ext_with_deposits().execute_with(|| {
		let liq_pool_id = 4;
		let liq_pool_account = LiquidityMining::pool_account_id(liq_pool_id).unwrap();
		let g_pool_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_rew_balance = Tokens::free_balance(BSX, &liq_pool_account);
		let g_pool_rew_balance = Tokens::free_balance(BSX, &g_pool_account);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();
		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		run_to_block(10_000);

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::CancelLiquidityPoolFarm {
			farm_id: GC_FARM,
			liq_pool_farm_id: liq_pool_id,
			who: GC,
			asset_pair: assets,
		})]);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
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
				total_shares_z: g_pool
					.total_shares_z
					.checked_sub(liq_pool.stake_in_global_pool)
					.unwrap(),
				accumulated_rewards: 18_206_375,
				paid_accumulated_rewards: 9_589_300,
				..g_pool
			}
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (assets, 3, GC_FARM));

		//"last reward" from update pool
		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_account),
			liq_pool_rew_balance + 8_424_900
		);

		assert_eq!(
			Tokens::free_balance(BSX, &g_pool_account),
			g_pool_rew_balance - 8_424_900
		);
	});
}

#[test]
fn cancel_liquidity_pool_invalid_liq_pool_should_not_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: DOT,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(GC), GC_FARM, assets),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn cancel_liquidity_pool_lm_already_canceled() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(GC), GC_FARM, assets),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn cancel_liquidity_pool_not_owner_should_not_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_OWNER: u128 = ALICE;

		assert_noop!(
			LiquidityMining::cancel_liquidity_pool(Origin::signed(NOT_OWNER), GC_FARM, assets),
			Error::<Test>::Forbidden
		);
	});
}

#[test]
fn remove_liquidity_pool_with_deposits_should_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let liq_pool_id = 4;
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_acc = LiquidityMining::pool_account_id(liq_pool_id).unwrap();

		let liq_pool_rew_balance = Tokens::free_balance(BSX, &liq_pool_acc);
		let g_pool_rew_balance = Tokens::free_balance(BSX, &g_pool_acc);

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RemoveLiquidityPoolFarm {
			farm_id: GC_FARM,
			liq_pool_farm_id: liq_pool_id,
			who: GC,
			asset_pair: assets,
		})]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				liq_pools_count: g_pool.liq_pools_count.checked_sub(1).unwrap(),
				..g_pool
			}
		);

		//liq pool struct should be removed from storage
		assert_eq!(LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM), None);

		//nft class info should stay in storage until add deposits will be withdrawn
		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (assets, 3, GC_FARM));

		assert_eq!(Tokens::free_balance(BSX, &liq_pool_acc), 0);
		//unpaid rewards should be transfered back to g_pool account
		assert_eq!(
			Tokens::free_balance(BSX, &g_pool_acc),
			g_pool_rew_balance.checked_add(liq_pool_rew_balance).unwrap()
		);
	});
}

#[test]
fn remove_liquidity_pool_without_deposits_should_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext().execute_with(|| {
		let liq_pool_id = 4;
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_acc = LiquidityMining::pool_account_id(liq_pool_id).unwrap();

		let liq_pool_rew_balance = Tokens::free_balance(BSX, &liq_pool_acc);
		let g_pool_rew_balance = Tokens::free_balance(BSX, &g_pool_acc);

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::RemoveLiquidityPoolFarm {
			farm_id: GC_FARM,
			liq_pool_farm_id: liq_pool_id,
			who: GC,
			asset_pair: assets,
		})]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				liq_pools_count: g_pool.liq_pools_count.checked_sub(1).unwrap(),
				..g_pool
			}
		);

		//liq pool struct should be removed from storage
		assert_eq!(LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM), None);

		//nft class info should removed from storage if no deposits are left
		assert_eq!(LiquidityMining::nft_class(0), None);

		assert_eq!(Tokens::free_balance(BSX, &liq_pool_acc), 0);
		//unpaid rewards should be transfered back to g_pool account
		assert_eq!(
			Tokens::free_balance(BSX, &g_pool_acc),
			g_pool_rew_balance.checked_add(liq_pool_rew_balance).unwrap()
		);
	});
}

#[test]
fn remove_liquidity_pool_non_canceled_lm_should_not_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::remove_liquidity_pool(Origin::signed(GC), GC_FARM, assets),
			Error::<Test>::LiquidityMiningIsNotCanceled
		);
	});
}

#[test]
fn remove_liquidity_pool_not_owner_should_not_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		const NOT_OWNER: u128 = ALICE;

		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			assets
		));

		assert_noop!(
			LiquidityMining::remove_liquidity_pool(Origin::signed(NOT_OWNER), GC_FARM, assets),
			Error::<Test>::Forbidden
		);
	});
}

#[test]
fn remove_liquidity_pool_liq_pool_does_not_exists_should_not_work() {
	let assets = AssetPair {
		asset_in: BSX,
		asset_out: DOT,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::remove_liquidity_pool(Origin::signed(GC), GC_FARM, assets),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn update_liquidity_pool_should_work() {
	//liq pool yield farms without deposits
	predefined_test_ext().execute_with(|| {
		let amm_1 = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		let new_multiplier: PoolMultiplier = FixedU128::from(5_000_u128);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();
		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1,
			new_multiplier
		));

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				multiplier: new_multiplier,
				..liq_pool
			}
		);

		assert_eq!(LiquidityMining::global_pool(GC_FARM).unwrap(), g_pool);
	});

	predefined_test_ext_with_deposits().execute_with(|| {
		let amm_1 = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		// Same period as last pools update so no update_XXX_pool() is called.
		let new_multiplier: PoolMultiplier = FixedU128::from(10_000_u128);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();
		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1,
			new_multiplier
		));

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
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
				..g_pool
			}
		);

		// Different period update_XXX_pool() should be called
		run_to_block(5_000);
		let new_multiplier: PoolMultiplier = FixedU128::from(5_000_u128);
		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();
		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_acc = LiquidityMining::pool_account_id(4).unwrap();

		let g_pool_rew_curr_balance = Tokens::free_balance(BSX, &g_pool_acc);
		let liq_pool_rew_curr_balance = Tokens::free_balance(BSX, &liq_pool_acc);

		assert_ok!(LiquidityMining::update_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1,
			new_multiplier
		));

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
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
				accumulated_rewards: g_pool.accumulated_rewards + 133_800_000,
				paid_accumulated_rewards: g_pool.paid_accumulated_rewards + 1_366_200_000,
				..g_pool
			}
		);

		assert_eq!(
			Tokens::free_balance(BSX, &g_pool_acc),
			g_pool_rew_curr_balance - 1_366_200_000
		);
		assert_eq!(
			Tokens::free_balance(BSX, &liq_pool_acc),
			liq_pool_rew_curr_balance + 1_366_200_000
		);
	});
}

#[test]
fn update_liquidity_pool_zero_multiplier_should_not_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::update_liquidity_pool(Origin::signed(GC), GC_FARM, amm_1, FixedU128::from(0_u128)),
			Error::<Test>::InvalidMultiplier
		);
	});
}

#[test]
fn update_liquidity_pool_canceled_pool_should_not_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));

		assert_noop!(
			LiquidityMining::update_liquidity_pool(Origin::signed(GC), GC_FARM, amm_1, FixedU128::from(10_001)),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn update_liquidity_pool_not_owner_should_not_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));

		let not_owner = ALICE;
		assert_noop!(
			LiquidityMining::update_liquidity_pool(
				Origin::signed(not_owner),
				GC_FARM,
				amm_1,
				FixedU128::from(10_001_u128)
			),
			Error::<Test>::LiquidityMiningCanceled
		);
	});
}

#[test]
fn resume_liquidity_pool_should_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));

		let liq_pool = LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap();
		let g_pool = LiquidityMining::global_pool(GC_FARM).unwrap();

		let new_multiplier = FixedU128::from(7490_000);

		assert!(liq_pool.canceled);
		assert!(liq_pool.stake_in_global_pool.is_zero());
		assert!(liq_pool.multiplier.is_zero());

		run_to_block(13_420_000);

		assert_ok!(LiquidityMining::resume_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1,
			new_multiplier
		));

		let liq_pool_stake_in_g_pool = new_multiplier.checked_mul_int(45_540).unwrap();

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				canceled: false,
				stake_in_global_pool: liq_pool_stake_in_g_pool,
				accumulated_rpz: 62_996,
				multiplier: new_multiplier,
				..liq_pool
			}
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				total_shares_z: g_pool.total_shares_z + liq_pool_stake_in_g_pool,
				updated_at: 134_200,
				accumulated_rpz: 62_996,
				accumulated_rewards: 29_999_067_250,
				..g_pool
			}
		);
	});
}

#[test]
fn resume_liquidity_pool_not_existing_pool_should_not_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: KSM,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7490_000);

		assert_noop!(
			LiquidityMining::resume_liquidity_pool(Origin::signed(GC), GC_FARM, amm_1, new_multiplier),
			Error::<Test>::LiquidityPoolNotFound
		);
	});
}

#[test]
fn resume_liquidity_pool_not_canceled_pool_should_not_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7490_000);

		assert_noop!(
			LiquidityMining::resume_liquidity_pool(Origin::signed(GC), GC_FARM, amm_1, new_multiplier),
			Error::<Test>::LiquidityMiningIsNotCanceled
		);
	});
}

#[test]
fn resume_liquidity_pool_not_owner_should_not_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		let new_multiplier = FixedU128::from(7490_000);

		assert_noop!(
			LiquidityMining::resume_liquidity_pool(Origin::signed(ALICE), GC_FARM, amm_1, new_multiplier),
			Error::<Test>::LiquidityMiningIsNotCanceled
		);
	});
}

#[test]
fn withdraw_undistributed_rewards_should_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	let amm_2 = AssetPair {
		asset_in: BSX,
		asset_out: TO2,
	};

	predefined_test_ext().execute_with(|| {
		//farm have to empty to be able to withdraw undistributed rewards
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_2
		));

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));
		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_2
		));

		let farm_ower_banalce = Tokens::total_balance(BSX, &GC);

		assert_ok!(LiquidityMining::withdraw_undistributed_rewards(
			Origin::signed(GC),
			GC_FARM
		));

		assert_eq!(Tokens::total_balance(BSX, &GC), farm_ower_banalce + 30_000_000_000);
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
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	let amm_2 = AssetPair {
		asset_in: BSX,
		asset_out: TO2,
	};

	predefined_test_ext().execute_with(|| {
		//farm have to empty to be able to withdraw undistributed rewards
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_2
		));

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));
		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_2
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
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	let amm_2 = AssetPair {
		asset_in: BSX,
		asset_out: TO2,
	};

	//canceled pools
	predefined_test_ext().execute_with(|| {
		//farm have to empty to be able to withdraw undistributed rewards
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_2
		));

		assert_ok!(LiquidityMining::remove_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_2
		));

		assert_noop!(
			LiquidityMining::withdraw_undistributed_rewards(Origin::signed(GC), GC_FARM),
			Error::<Test>::FarmIsNotEmpty
		);
	});

	//active pools
	predefined_test_ext().execute_with(|| {
		//farm have to empty to be able to withdraw undistributed rewards
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_1
		));
		assert_ok!(LiquidityMining::cancel_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			amm_2
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
		let empty_lp: LiquidityPoolYieldFarm<Test> = LiquidityPoolYieldFarm {
			id: 1,
			updated_at: 0,
			total_shares: 0,
			total_valued_shares: 0,
			accumulated_rpvs: 0,
			accumulated_rpz: 0,
			loyalty_curve: Some(LoyaltyCurve::default()),
			stake_in_global_pool: 0,
			multiplier: FixedU128::from(100),
			nft_class: 0,
			canceled: false,
		};

		//(Deposit, LiquidityPoolYieldFarm, period_now, expected_result(user_reward, unclaimable_rewards)
		let mock_data: [(
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
					..empty_lp
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
					..empty_lp
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
					..empty_lp
				},
				1002,
				(80_656_176, 0),
			),
		];

		let liq_pool_acc = LiquidityMining::pool_account_id(1).unwrap();
		assert_ok!(Tokens::set_balance(
			Origin::root(),
			liq_pool_acc,
			BSX,
			1_000_0000_0000_0000_0000_000,
			0
		));

		for (mut deposit, liq_pool, now_period, expected_result) in mock_data {
			let alice_balance = Tokens::free_balance(BSX, &ALICE);
			let pool_acc_balance = Tokens::free_balance(BSX, &liq_pool_acc);

			assert_eq!(
				LiquidityMining::do_claim_rewards(ALICE, &mut deposit, &liq_pool, now_period, BSX).unwrap(),
				expected_result
			);

			let expected_alice_balance = alice_balance + expected_result.0;
			let expected_pool_balance = pool_acc_balance - expected_result.0;

			assert_eq!(Tokens::free_balance(BSX, &ALICE), expected_alice_balance);
			assert_eq!(Tokens::free_balance(BSX, &liq_pool_acc), expected_pool_balance);
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
