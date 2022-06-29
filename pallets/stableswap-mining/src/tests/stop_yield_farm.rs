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
fn stop_yield_farm_should_work() {
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
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, PoolId(3), (BSX, DAI))
		.build()
		.execute_with(|| {
			let owner = GC;
			let global_farm_id = GC_FARM;
			let yield_farm_id = 2;
			let pool_id = get_pool_id_at(0);

			assert_ok!(StableswapMining::stop_yield_farm(
				Origin::signed(owner),
				global_farm_id,
				pool_id
			));

			assert_last_event!(crate::Event::YieldFarmStopped {
				who: owner,
				global_farm_id,
				yield_farm_id,
				pool_id
			}
			.into());
		});
}
