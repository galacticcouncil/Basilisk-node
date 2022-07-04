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
use frame_support::error::BadOrigin;

// Happy path. Cration from allowed origin with valid params shoudl work, `GlobalFarmCreated` event
// should be emitted and `total_rewards` should be transferd from farm's owner.
#[test]
fn create_global_farm_should_work() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
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
		.build()
		.execute_with(|| {
			let owner = ALICE;
			let total_rewards = 1_000_000_000_000_u128;
			let planned_yielding_periods = 11_000_000_u64;
			let blocks_per_period = 100;
			let incentivized_asset = BSX;
			let reward_currency = BSX;
			let yield_per_period = Permill::from_float(0.2);
			let min_deposit = 100;
			let price_adujustment: FixedU128 = One::one();
			let max_reward_per_period = total_rewards.checked_div(planned_yielding_periods.into()).unwrap();
			let global_farm_id = GC_FARM;

			assert_ok!(StableswapMining::create_global_farm(
				Origin::root(),
				total_rewards,
				planned_yielding_periods,
				blocks_per_period,
				incentivized_asset,
				reward_currency,
				owner,
				yield_per_period,
				min_deposit,
				price_adujustment,
			));

			assert_last_event!(crate::Event::GlobalFarmCreated {
				owner,
				id: global_farm_id,
				reward_currency,
				yield_per_period,
				planned_yielding_periods,
				incentivized_asset,
				max_reward_per_period,
				blocks_per_period,
			}
			.into());
		});
}

// Create global farm from not allowed origin shuuld fail with error `BadOrigin` and strorage
// should not be nodified.
#[test]
fn create_global_farm_should_fail_when_not_allowed_origin() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, KSM, 1_000 * ONE),
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
		.build()
		.execute_with(|| {
			let not_allowed_origin: Origin = Origin::signed(ALICE);

			assert_noop!(
				StableswapMining::create_global_farm(
					not_allowed_origin,
					1_000_000_000_000_u128,
					11_000,
					1_000,
					BSX,
					BSX,
					ALICE,
					Permill::from_float(0.2),
					10,
					One::one(),
				),
				BadOrigin
			);
		});
}
