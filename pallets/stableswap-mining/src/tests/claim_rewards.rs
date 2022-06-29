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
			2_000_000_000_000 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, PoolId(3), (BSX, DAI))
		.with_deposit(ALICE, GC_FARM, 2, PoolId(3), 100_000)
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

			assert_last_event!(crate::Event::RewardsClaimed {
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
fn claim_rewards_in_same_period_should_work_when_claiming_different_farm_entry() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, BSX, 1_000 * ONE),
			(ALICE, DAI, 1_000 * ONE),
			(GC, BSX, 2_000_000_000_000 * ONE),
			(BOB, HDX, 2_000_000_000_000 * ONE),
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
			2_000_000_000_000 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_global_farm(
			2_000_000_000_000 * ONE,
			1_000_000,
			1_000,
			BSX,
			HDX,
			BOB,
			Permill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, PoolId(3), (BSX, DAI))
		.with_yield_farm(BOB, BOB_FARM, FixedU128::one(), None, PoolId(3), (BSX, DAI))
		.with_deposit(ALICE, GC_FARM, 3, PoolId(3), 100_000)
		.build()
		.execute_with(|| {
			let owner = ALICE;
			let global_farm_id = GC_FARM;
			let yield_farm_id = 3;
			let nft_instance_id = get_deposit_id_at(0);

			// redeposit lp shares before test
			assert_ok!(StableswapMining::redeposit_lp_shares(
				Origin::signed(owner),
				BOB_FARM,
				4,
				nft_instance_id
			));

			set_block_number(10_000);

			// 1-th claim
			assert_ok!(StableswapMining::claim_rewards(
				Origin::signed(owner),
				nft_instance_id,
				yield_farm_id
			));

			assert_last_event!(crate::Event::RewardsClaimed {
				who: owner,
				global_farm_id,
				yield_farm_id,
				nft_instance_id,
				reward_currency: BSX,
				claimed: 20_000_000 * ONE
			}
			.into());

			// 2-nd claim
			let global_farm_id = BOB_FARM;
			let yield_farm_id = 4;

			assert_ok!(StableswapMining::claim_rewards(
				Origin::signed(owner),
				nft_instance_id,
				yield_farm_id
			));

			assert_last_event!(crate::Event::RewardsClaimed {
				who: owner,
				global_farm_id,
				yield_farm_id,
				nft_instance_id,
				reward_currency: HDX,
				claimed: 20_000_000 * ONE
			}
			.into());
		});
}

#[test]
fn claim_rewards_should_fail_when_second_claim_in_same_period() {
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
			2_000_000_000_000 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, PoolId(3), (BSX, DAI))
		.with_deposit(ALICE, GC_FARM, 2, PoolId(3), 100_000)
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

			assert_last_event!(crate::Event::RewardsClaimed {
				who,
				global_farm_id,
				yield_farm_id,
				nft_instance_id,
				reward_currency: BSX,
				claimed: 20_000_000 * ONE
			}
			.into());

			//2-nd claim should fail
			set_block_number(10_500);

			assert_noop!(
				StableswapMining::claim_rewards(Origin::signed(who), nft_instance_id, yield_farm_id),
				warehouse_liquidity_mining::Error::<Test, Instance1>::DoubleClaimInPeriod
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
			2_000_000_000_000 * ONE,
			1_000_000,
			1_000,
			BSX,
			BSX,
			GC,
			Permill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, PoolId(3), (BSX, DAI))
		.with_deposit(ALICE, GC_FARM, 2, PoolId(3), 100_000)
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
