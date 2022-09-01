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
				assets: vec![BSX, DAI].try_into().unwrap(),
				amplification: 100,
				trade_fee: Permill::from_percent(0),
				withdraw_fee: Permill::from_percent(0),
			},
			InitialLiquidity {
				account: ALICE,
				assets: vec![
					AssetLiquidity {
						asset_id: BSX,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: DAI,
						amount: 100 * ONE,
					},
				],
			},
		)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: vec![BSX, HDX].try_into().unwrap(),
				amplification: 100,
				trade_fee: Permill::from_percent(0),
				withdraw_fee: Permill::from_percent(0),
			},
			InitialLiquidity {
				account: ALICE,
				assets: vec![
					AssetLiquidity {
						asset_id: BSX,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: HDX,
						amount: 100 * ONE,
					},
				],
			},
		)
		.with_global_farm(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Perquintill::from_float(0.2),
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
		.with_global_farm(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Perquintill::from_float(0.2),
			1_000,
			One::one(),
		)
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::create_yield_farm(Origin::signed(GC), GC_FARM, 4, FixedU128::from_float(5.2), None),
				Error::<Test>::StableswapPoolNotFound
			);
		});
}
