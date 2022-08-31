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

#[test]
fn destroy_global_farm_should_work() {
	let owner = ALICE;
	let total_rewards = 1_000_000_000_000_u128;
	let planned_yielding_periods = 11_000_000_u64;
	let blocks_per_period = 100;
	let incentivized_asset = BSX;
	let reward_currency = BSX;
	let yield_per_period = Perquintill::from_float(0.2);
	let min_deposit = 100;
	let price_adujustment: FixedU128 = One::one();
	let global_farm_id = GC_FARM;

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
		.with_global_farm(
			total_rewards,
			planned_yielding_periods,
			blocks_per_period,
			incentivized_asset,
			reward_currency,
			owner,
			yield_per_period,
			min_deposit,
			price_adujustment,
		)
		.build()
		.execute_with(|| {
			assert_ok!(StableswapMining::destroy_global_farm(
				Origin::signed(owner),
				global_farm_id
			));

			assert_last_event!(crate::Event::GlobalFarmDestroyed {
				who: owner,
				id: global_farm_id,
				reward_currency,
				undistributed_rewards: GLOBAL_FARM_UNDISTRIBUTED_REWARDS,
			}
			.into());
		});
}
