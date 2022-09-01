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
fn redeposit_lp_shares_should_work() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
			(BOB, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
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
		.with_global_farm(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			BOB,
			Perquintill::from_float(0.2),
			1_000,
			One::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_yield_farm(BOB, BOB_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_deposit(ALICE, GC_FARM, 3, 3, 100_000)
		.build()
		.execute_with(|| {
			let who = ALICE;
			let global_farm_id = BOB_FARM;
			let yield_farm_id = 4; //Bob's yield farm id
			let nft_instance_id = get_deposit_id_at(0);

			assert_ok!(StableswapMining::redeposit_lp_shares(
				Origin::signed(who),
				global_farm_id,
				yield_farm_id,
				nft_instance_id
			));

			assert_last_event!(crate::Event::LPSharesRedeposited {
				who,
				global_farm_id,
				yield_farm_id,
				nft_instance_id
			}
			.into())
		});
}

#[test]
fn redeposit_lp_shares_should_fail_when_origin_is_not_nft_owner() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000 * ONE),
			(BOB, BSX, 2_000_000 * ONE),
		])
		.with_registered_asset("BSX".as_bytes().to_vec(), BSX)
		.with_registered_asset("DAI".as_bytes().to_vec(), DAI)
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
		.with_global_farm(
			100 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			BOB,
			Perquintill::from_float(0.2),
			1_000,
			One::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_yield_farm(BOB, BOB_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_deposit(ALICE, GC_FARM, 3, 3, 100_000)
		.build()
		.execute_with(|| {
			let not_owner = CHARLIE;
			let global_farm_id = BOB_FARM;
			let yield_farm_id = 4; //Bob's yield farm id
			let nft_instance_id = get_deposit_id_at(0);

			assert_noop!(
				StableswapMining::redeposit_lp_shares(
					Origin::signed(not_owner),
					global_farm_id,
					yield_farm_id,
					nft_instance_id
				),
				Error::<Test>::NotDepositOwner
			);
		});
}
