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
fn withdraw_lp_shares_should_work() {
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
		.with_global_farm(
			2_000_000_000_000 * ONE,
			1_000_000,
			1_000,
			BSX,
			HDX,
			BOB,
			Perquintill::from_float(0.2),
			1_000,
			FixedU128::one(),
		)
		.with_yield_farm(GC, GC_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_yield_farm(BOB, BOB_FARM, FixedU128::one(), None, 3, vec![BSX, DAI])
		.with_deposit(ALICE, GC_FARM, 3, 3, 100_000)
		.build()
		.execute_with(|| {
			let owner = ALICE;
			let nft_instance_id = get_deposit_id_at(0);
			let pool_id = 3;
			let withdrawn_amount = 100_000;
			let lp_token = StableswapMining::get_lp_token(pool_id).unwrap();

			//redeposit lp shares before test
			assert_ok!(StableswapMining::redeposit_lp_shares(
				Origin::signed(owner),
				BOB_FARM,
				4,
				nft_instance_id
			));

			set_block_number(10_000);

			// 1-th withdraw
			let yield_farm_id = 3;
			let owner_lp_token_balance = Tokens::free_balance(lp_token, &owner);
			assert_ok!(StableswapMining::withdraw_lp_shares(
				Origin::signed(owner),
				nft_instance_id,
				yield_farm_id,
				pool_id
			));

			pretty_assertions::assert_eq!(
				has_event(
					crate::Event::<Test>::RewardsClaimed {
						who: owner,
						global_farm_id: GC_FARM,
						yield_farm_id,
						nft_instance_id,
						reward_currency: BSX,
						claimed: 20_000_000 * ONE,
					}
					.into()
				),
				true
			);

			assert_last_event!(crate::Event::<Test>::LPSharesWithdrawn {
				who: owner,
				global_farm_id: GC_FARM,
				yield_farm_id,
				lp_token,
				amount: withdrawn_amount
			}
			.into());

			//NOTE: lp tokens are returned only if deposit was destroy, this is not this case.
			pretty_assertions::assert_eq!(Tokens::free_balance(lp_token, &owner), owner_lp_token_balance);

			//NFT should have not been burned
			pretty_assertions::assert_eq!(NFTS.with(|v| v.borrow().contains_key(&nft_instance_id)), true);

			// 1-th withdraw
			let yield_farm_id = 4;
			assert_ok!(StableswapMining::withdraw_lp_shares(
				Origin::signed(owner),
				nft_instance_id,
				yield_farm_id,
				pool_id
			));

			pretty_assertions::assert_eq!(
				has_event(
					crate::Event::<Test>::RewardsClaimed {
						who: owner,
						global_farm_id: BOB_FARM,
						yield_farm_id,
						nft_instance_id,
						reward_currency: HDX,
						claimed: 20_000_000 * ONE,
					}
					.into()
				),
				true
			);

			pretty_assertions::assert_eq!(
				has_event(
					crate::Event::<Test>::LPSharesWithdrawn {
						who: owner,
						global_farm_id: BOB_FARM,
						yield_farm_id,
						lp_token,
						amount: withdrawn_amount,
					}
					.into()
				),
				true
			);

			assert_last_event!(crate::Event::<Test>::DepositDestroyed {
				who: owner,
				nft_instance_id
			}
			.into());

			//NOTE: deposit was destroyed and LP shares returned.
			pretty_assertions::assert_eq!(
				Tokens::free_balance(lp_token, &owner),
				owner_lp_token_balance + withdrawn_amount
			);

			//Check if NFT was burned
			pretty_assertions::assert_eq!(NFTS.with(|v| v.borrow().contains_key(&nft_instance_id)), false);
		});
}

#[test]
fn withdraw_lp_shares_should_work_when_reward_claim_is_zero() {
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
			let owner = ALICE;
			let nft_instance_id = get_deposit_id_at(0);
			let pool_id = 3;
			let withdrawn_amount = 100_000;
			let lp_token = StableswapMining::get_lp_token(pool_id).unwrap();
			let yield_farm_id = 2;
			let global_farm_id = GC_FARM;

			let owner_lp_token_balance = Tokens::free_balance(lp_token, &owner);
			assert_ok!(StableswapMining::withdraw_lp_shares(
				Origin::signed(owner),
				nft_instance_id,
				yield_farm_id,
				pool_id
			));

			pretty_assertions::assert_eq!(
				has_event(
					crate::Event::<Test>::RewardsClaimed {
						who: owner,
						global_farm_id,
						yield_farm_id,
						nft_instance_id,
						reward_currency: BSX,
						claimed: 20_000_000 * ONE,
					}
					.into()
				),
				false
			);

			pretty_assertions::assert_eq!(
				has_event(
					crate::Event::<Test>::LPSharesWithdrawn {
						who: owner,
						global_farm_id,
						yield_farm_id,
						lp_token,
						amount: withdrawn_amount,
					}
					.into()
				),
				true
			);

			assert_last_event!(crate::Event::<Test>::DepositDestroyed {
				who: owner,
				nft_instance_id
			}
			.into());

			//NOTE: deposit was destroyed and LP tokens returned
			pretty_assertions::assert_eq!(
				Tokens::free_balance(lp_token, &owner),
				owner_lp_token_balance + withdrawn_amount
			);

			// check if NFT was burned
			pretty_assertions::assert_eq!(NFTS.with(|v| v.borrow().contains_key(&nft_instance_id)), false);
		});
}

#[test]
fn withdraw_lp_shares_should_fail_when_account_is_not_deposit_owner() {
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
			let not_owner = BOB;
			let nft_instance_id = get_deposit_id_at(0);
			let pool_id = 3;
			let yield_farm_id = 2;

			assert_noop!(
				StableswapMining::withdraw_lp_shares(
					Origin::signed(not_owner),
					nft_instance_id,
					yield_farm_id,
					pool_id
				),
				Error::<Test>::NotDepositOwner
			);
		});
}

#[test]
fn withdraw_lp_shares_should_fail_when_stableswap_pool_doesnt_exits() {
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
			let not_owner = ALICE;
			let nft_instance_id = get_deposit_id_at(0);
			let pool_id = 3;
			let yield_farm_id = 2;

			//Destroy stableswap pool - stableswap pallet doesn't allow poll destruction
			pallet_stableswap::Pools::<Test>::remove(pool_id);
			pretty_assertions::assert_eq!(pallet_stableswap::Pools::<Test>::get(pool_id).is_none(), true);

			assert_noop!(
				StableswapMining::withdraw_lp_shares(
					Origin::signed(not_owner),
					nft_instance_id,
					yield_farm_id,
					pool_id
				),
				Error::<Test>::StableswapPoolNotFound
			);
		});
}
