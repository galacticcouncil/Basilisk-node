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
fn deposit_lp_should_work() {
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
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.build()
		.execute_with(|| {
			let who = ALICE;
			let global_farm_id = GC_FARM;
			let yield_farm_id = 2;
			let nft_instance_id = 1;
			let pool_id = get_pool_id_at(0);
			let lp_token = StableswapMining::get_lp_token(pool_id).unwrap();
			let deposited_amount = 1_000;
			let alice_lp_shares_balance = Tokens::free_balance(lp_token, &ALICE);

			assert_ok!(StableswapMining::deposit_lp_shares(
				Origin::signed(who),
				global_farm_id,
				yield_farm_id,
				pool_id,
				deposited_amount
			));

			assert_last_event!(crate::Event::LPSharesDeposited {
				who,
				global_farm_id,
				yield_farm_id,
				pool_id,
				nft_instance_id,
				lp_token,
				amount: deposited_amount
			}
			.into());

			// Lp shares lock(transfer) check
			pretty_assertions::assert_eq!(
				Tokens::free_balance(lp_token, &ALICE),
				alice_lp_shares_balance - deposited_amount
			);

			// Check if NFT was minted
			let nft_owner: AccountId = DummyNFT::owner(&LIQ_MINING_NFT_CLASS, &nft_instance_id).unwrap();
			pretty_assertions::assert_eq!(nft_owner, ALICE);
		});
}

#[test]
fn deposit_lp_should_fail_when_account_with_insufficient_balance() {
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
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.build()
		.execute_with(|| {
			assert_noop!(
				StableswapMining::deposit_lp_shares(Origin::signed(ALICE), GC_FARM, 2, get_pool_id_at(0), 901 * ONE),
				Error::<Test>::InsufficientLpShares
			);
		});
}

#[test]
fn deposit_lp_should_fail_when_stableswap_pool_doesnt_exist() {
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
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);

			//Destroy stableswap pool - stableswap pallet doesn't allow poll destruction for now.
			pallet_stableswap::Pools::<Test>::remove(pool_id);
			pretty_assertions::assert_eq!(pallet_stableswap::Pools::<Test>::get(pool_id).is_none(), true);

			assert_noop!(
				StableswapMining::deposit_lp_shares(Origin::signed(ALICE), GC_FARM, 2, get_pool_id_at(0), 1_000),
				Error::<Test>::StableswapPoolNotFound
			);
		});
}
