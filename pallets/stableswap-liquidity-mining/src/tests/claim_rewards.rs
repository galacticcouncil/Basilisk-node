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
fn claim_rewards_should_work() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000_000_000 * ONE),
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
			2_000_000_000_000 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Perquintill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_deposit(ALICE, GC_FARM, 2, 3, 100_000)
		.build()
		.execute_with(|| {
			let who = ALICE;
			let global_farm_id = GC_FARM;
			let yield_farm_id = 2;
			let nft_instance_id = get_deposit_id_at(0);

			set_block_number(10_000);

			assert_ok!(StableswapMining::claim_rewards(
				Origin::signed(who),
				nft_instance_id,
				yield_farm_id
			));

			assert_last_event!(crate::Event::RewardClaimed {
				who,
				global_farm_id,
				yield_farm_id,
				nft_instance_id,
				reward_currency: BSX,
				claimed: 20_000_000 * ONE
			}
			.into());
		});
}

#[test]
fn claim_rewards_should_fail_when_second_claim_in_same_period_from_same_farm() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000_000_000 * ONE),
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
			2_000_000_000_000 * ONE,
			1_000_000,
			1,
			BSX,
			BSX,
			GC,
			Perquintill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_deposit(ALICE, GC_FARM, 2, 3, 100_000)
		.build()
		.execute_with(|| {
			let who = ALICE;
			let global_farm_id = GC_FARM;
			let yield_farm_id = 2;
			let nft_instance_id = get_deposit_id_at(0);

			set_block_number(10_000);

			//1-th claim in parriod should pass
			assert_ok!(StableswapMining::claim_rewards(
				Origin::signed(who),
				nft_instance_id,
				yield_farm_id
			));

			assert_last_event!(crate::Event::RewardClaimed {
				who,
				global_farm_id,
				yield_farm_id,
				nft_instance_id,
				reward_currency: BSX,
				claimed: 20_000_000 * ONE
			}
			.into());

			//2-nd claim should fail
			assert_noop!(
				StableswapMining::claim_rewards(Origin::signed(who), nft_instance_id, yield_farm_id),
				"Dummy Double Claim"
			);
		});
}

#[test]
fn claim_rewards_should_fail_when_account_is_not_deposit_owner() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000_000_000 * ONE),
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
			2_000_000_000_000 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Perquintill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_deposit(ALICE, GC_FARM, 2, 3, 100_000)
		.build()
		.execute_with(|| {
			let not_owner = CHARLIE;
			let yield_farm_id = 2;
			let nft_instance_id = get_deposit_id_at(0);

			set_block_number(10_000);

			assert_noop!(
				StableswapMining::claim_rewards(Origin::signed(not_owner), nft_instance_id, yield_farm_id),
				Error::<Test>::NotDepositOwner
			);
		});
}
