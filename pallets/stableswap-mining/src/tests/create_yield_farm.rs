// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

//Happy path. Creation of the yield farm should work and `YieldFarmCreated` event should be
//emitted. Multiple yield farms can be created in the same global farm.
#[test]
fn create_yield_farm_should_work() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
			(ALICE, HDX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_registered_asset("KSM".as_bytes().to_vec(), KSM)
		.with_registered_asset("HDX".as_bytes().to_vec(), HDX)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, DAI),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, HDX),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			let global_farm_id = GC_FARM;
			let pool_id = get_pool_id_at(0);
			let multiplier = FixedU128::from_float(5.2);

			assert_ok!(StableswapMining::create_yield_farm(
				Origin::signed(GC),
				global_farm_id,
				pool_id,
				multiplier,
				None
			));

			assert_last_event!(crate::Event::YieldFarmCreated {
				global_farm_id,
				yield_farm_id: 2,
				pool_id,
				multiplier,
				loyalty_curve: None,
			}
			.into());

			// second yield farm with loyalty curve
			let pool_id = get_pool_id_at(1);
			let multiplier = FixedU128::from_float(1.0);
			let loyalty_curve = Some(LoyaltyCurve::default());

			assert_ok!(StableswapMining::create_yield_farm(
				Origin::signed(GC),
				global_farm_id,
				pool_id,
				multiplier,
				loyalty_curve.clone()
			));

			assert_last_event!(crate::Event::YieldFarmCreated {
				global_farm_id,
				yield_farm_id: 3,
				pool_id,
				multiplier,
				loyalty_curve,
			}
			.into());
		});
}

// Create 2 yield farms for same stableswap pool in the same global farm should fail with error
// `YieldFarmAlreadyExists`.
#[test]
fn create_same_yield_farm_in_same_global_farm_should_fail() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, DAI),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.with_yield_farms(GC, GC_FARM, FixedU128::one(), None, PoolId(3), (BSX, DAI))
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::create_yield_farm(Origin::signed(GC), GC_FARM, PoolId(3), FixedU128::one(), None),
				warehouse_liquidity_mining::Error::<Test, Instance1>::YieldFarmAlreadyExists
			);
		});
}

// Create second yield farm for same stableswap pool in the different global farm should work.
#[test]
fn create_same_yield_farm_in_different_global_farm_should_work() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, DAI),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			ALICE,
			Permill::from_float(0.2),
			100,
			One::one(),
		)
		.with_yield_farms(GC, GC_FARM, FixedU128::one(), None, PoolId(3), (BSX, DAI))
		.build()
		.execute_with(|| {
			let alice_farm: GlobalFarmId = 2;
			let pool_id: PoolId<AssetId> = PoolId(3);
			let multiplier: FarmMultiplier = FixedU128::one();
			let loyalty_curve = None;

			assert_ok!(StableswapMining::create_yield_farm(
				Origin::signed(ALICE),
				alice_farm,
				pool_id,
				multiplier,
				loyalty_curve.clone()
			));

			assert_last_event!(crate::Event::YieldFarmCreated {
				global_farm_id: alice_farm,
				yield_farm_id: 4,
				pool_id,
				multiplier,
				loyalty_curve,
			}
			.into());
		});
}

#[test]
fn create_yield_farm_should_fail_when_stableswap_pool_doesnt_exists() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
			(ALICE, HDX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::create_yield_farm(
					Origin::signed(GC),
					GC_FARM,
					PoolId(4),
					FixedU128::from_float(5.2),
					None
				),
				Error::<Test>::StableswapPoolNotFound
			);
		});
}

#[test]
fn create_yield_farm_should_fail_when_multiplier_is_not_valid() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
			(ALICE, HDX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, HDX),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::create_yield_farm(
					Origin::signed(GC),
					GC_FARM,
					get_pool_id_at(0),
					FixedU128::from(0),
					None
				),
				warehouse_liquidity_mining::Error::<Test, Instance1>::InvalidMultiplier
			);
		});
}

// Only global farm owner can create yield farms in global farm.
#[test]
fn create_yield_farm_should_fail_when_origin_is_not_owner() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
			(ALICE, HDX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, HDX),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			let not_global_farm_owner = ALICE;

			assert_noop!(
				StableswapMining::create_yield_farm(
					Origin::signed(not_global_farm_owner),
					GC_FARM,
					get_pool_id_at(0),
					FixedU128::one(),
					None
				),
				warehouse_liquidity_mining::Error::<Test, Instance1>::Forbidden
			);
		});
}

#[test]
fn create_yield_farm_should_fail_when_global_farm_doesnt_exists() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
			(ALICE, HDX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, HDX),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			let non_existing_farm_id = 99_999_999;
			assert_noop!(
				StableswapMining::create_yield_farm(
					Origin::signed(GC),
					non_existing_farm_id,
					get_pool_id_at(0),
					FixedU128::one(),
					None
				),
				warehouse_liquidity_mining::Error::<Test, Instance1>::GlobalFarmNotFound
			);
		});
}

// Yield farm can't be created in global farm if global farm reached `T::MaxYieldFarmsPerGlobalFarm`
// limit.
// NOTE: this limit is set to 4 in mock
#[test]
fn create_yield_farm_should_fail_when_global_farm_is_full() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
			(ALICE, HDX, 1_000 * ONE),
			(ALICE, ACA, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(ALICE, DOT, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("KSM".as_bytes().to_vec(), KSM)
		.with_registered_asset("HDX".as_bytes().to_vec(), HDX)
		.with_registered_asset("ACA".as_bytes().to_vec(), ACA)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_registered_asset("DOT".as_bytes().to_vec(), DOT)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, HDX),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, DAI),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, KSM),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, ACA),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, DOT),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.with_yield_farms(GC, GC_FARM, FixedU128::one(), None, PoolId(6), (BSX, DAI))
		.with_yield_farms(GC, GC_FARM, FixedU128::one(), None, PoolId(7), (BSX, KSM))
		.with_yield_farms(GC, GC_FARM, FixedU128::one(), None, PoolId(8), (BSX, ACA))
		.with_yield_farms(GC, GC_FARM, FixedU128::one(), None, PoolId(9), (BSX, HDX))
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::create_yield_farm(
					Origin::signed(GC),
					GC_FARM,
					get_pool_id_at(4),
					FixedU128::one(),
					None
				),
				warehouse_liquidity_mining::Error::<Test, Instance1>::GlobalFarmIsFull
			);
		});
}

// Yield farm can be created only if one of the assets in stableswap pool is `incentivized_asset`.
#[test]
fn create_yield_farm_should_fail_when_pool_has_no_incentivized_asset() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
			(ALICE, HDX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("HDX".as_bytes().to_vec(), HDX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(DAI, HDX),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: HDX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::create_yield_farm(
					Origin::signed(GC),
					GC_FARM,
					get_pool_id_at(0),
					FixedU128::one(),
					None
				),
				warehouse_liquidity_mining::Error::<Test, Instance1>::MissingIncentivizedAsset
			);
		});
}

//Rules for loyalty curver validation:
//`initial_reward_percentage < 1` else `InvalidInitialRewardPercentage`
#[test]
fn create_yield_farm_should_fail_when_loyalty_curve_is_not_valid() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, DAI),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::create_yield_farm(
					Origin::signed(GC),
					GC_FARM,
					get_pool_id_at(0),
					FixedU128::one(),
					Some(LoyaltyCurve {
						scale_coef: 10,
						initial_reward_percentage: FixedU128::one() // invalid param
					})
				),
				warehouse_liquidity_mining::Error::<Test, Instance1>::InvalidInitialRewardPercentage
			);
		});
}

#[test]
fn create_yield_farm_should_fail_when_global_farm_was_removed() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: PoolAssets::new(BSX, DAI),
				amplification: 100,
				fee: Permill::from_float(0.1),
			},
			InitialLiquidity {
				account: ALICE,
				asset: BSX,
				amount: 100 * ONE,
			},
		)
		.with_global_farms(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			// destroy global farm before test
			assert_ok!(StableswapMining::destroy_global_farm(Origin::signed(GC), GC_FARM));

			assert_noop!(
				StableswapMining::create_yield_farm(
					Origin::signed(GC),
					GC_FARM,
					get_pool_id_at(0),
					FixedU128::one(),
					None
				),
				warehouse_liquidity_mining::Error::<Test, Instance1>::GlobalFarmNotFound
			);
		});
}
