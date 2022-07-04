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
pub fn get_lp_token_should_work() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, BSX, 1_000 * ONE), (ALICE, DAI, 1_000 * ONE)])
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
			pretty_assertions::assert_eq!(StableswapMining::get_lp_token(get_pool_id_at(0)).unwrap(), BSX);
		});
}

#[test]
pub fn get_lp_token_should_fail_when_stableswap_pool_doesnt_exists() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			StableswapMining::get_lp_token(PoolId(1)),
			Error::<Test>::StableswapPoolNotFound
		);
	});
}

#[test]
pub fn get_asset_balance_in_stableswap_pool_should_work() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, BSX, 1_000 * ONE), (ALICE, DAI, 1_000 * ONE)])
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
			pretty_assertions::assert_eq!(
				StableswapMining::get_asset_balance_in_stableswap_pool(BSX, get_pool_id_at(0)).unwrap(),
				100 * ONE
			);
		});
}

#[test]
pub fn get_asset_balance_in_stableswap_pool_should_fail_when_asset_is_not_in_stableswap_pool() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, BSX, 1_000 * ONE), (ALICE, DAI, 1_000 * ONE)])
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
			assert_noop!(
				StableswapMining::get_asset_balance_in_stableswap_pool(HDX, get_pool_id_at(0)),
				Error::<Test>::AssetNotInStableswapPool
			);
		});
}

#[test]
pub fn get_asset_balance_in_stableswap_pool_should_fail_when_stableswap_pool_doesnt_exists() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, BSX, 1_000 * ONE), (ALICE, DAI, 1_000 * ONE)])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::get_asset_balance_in_stableswap_pool(HDX, PoolId(1)),
				Error::<Test>::StableswapPoolNotFound
			);
		});
}
