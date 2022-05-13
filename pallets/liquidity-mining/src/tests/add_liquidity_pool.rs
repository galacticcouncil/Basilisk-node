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

			assert_eq!(WarehouseLM::global_pool(farm_id).unwrap(), global_pool);
		}

		for (_, pool, amm_id, _, farm_id, _, _) in test_data {
			assert_eq!(WarehouseLM::liquidity_pool(farm_id, amm_id).unwrap(), pool);
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
					//neither KSM nor DOT is incetivized in the farm
					asset_in: KSM,
					asset_out: DOT,
				},
				FixedU128::from(10_000_u128),
				None
			),
			pallet_liquidity_mining::Error::<Test>::MissingIncentivizedAsset
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
			pallet_liquidity_mining::Error::<Test>::Forbidden
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
			pallet_liquidity_mining::Error::<Test>::Forbidden
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
				pallet_liquidity_mining::Error::<Test>::InvalidInitialRewardPercentage
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
			pallet_liquidity_mining::Error::<Test>::InvalidMultiplier
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
		assert!(WarehouseLM::liquidity_pool(CHARLIE_FARM, aca_ksm_amm_account).is_some());

		//try to add same amm second time in the same block(period)
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(CHARLIE),
				CHARLIE_FARM,
				aca_ksm_assets,
				FixedU128::from(9_000_u128),
				Some(LoyaltyCurve::default()),
			),
			pallet_liquidity_mining::Error::<Test>::LiquidityPoolAlreadyExists
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
			pallet_liquidity_mining::Error::<Test>::LiquidityPoolAlreadyExists
		);
	});
}
