// This file is part of Basilisk-node.

// Copyright (C) 2020-2021  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use warehouse_liquidity_mining::GlobalPool;
use warehouse_liquidity_mining::LiquidityPoolYieldFarm;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| {
		migration::init_nft_class::<Test>();
		set_block_number(1)
	});
	ext
}

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			100_000_000_000,
			PREDEFINED_GLOBAL_POOLS[0].planned_yielding_periods,
			PREDEFINED_GLOBAL_POOLS[0].blocks_per_period,
			PREDEFINED_GLOBAL_POOLS[0].incentivized_asset,
			PREDEFINED_GLOBAL_POOLS[0].reward_currency,
			ALICE,
			PREDEFINED_GLOBAL_POOLS[0].yield_per_period,
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			1_000_000_000,
			PREDEFINED_GLOBAL_POOLS[1].planned_yielding_periods,
			PREDEFINED_GLOBAL_POOLS[1].blocks_per_period,
			PREDEFINED_GLOBAL_POOLS[1].incentivized_asset,
			PREDEFINED_GLOBAL_POOLS[1].reward_currency,
			BOB,
			PREDEFINED_GLOBAL_POOLS[1].yield_per_period,
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			30_000_000_000,
			PREDEFINED_GLOBAL_POOLS[2].planned_yielding_periods,
			PREDEFINED_GLOBAL_POOLS[2].blocks_per_period,
			PREDEFINED_GLOBAL_POOLS[2].incentivized_asset,
			PREDEFINED_GLOBAL_POOLS[2].reward_currency,
			GC,
			PREDEFINED_GLOBAL_POOLS[2].yield_per_period,
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			30_000_000_000,
			PREDEFINED_GLOBAL_POOLS[3].planned_yielding_periods,
			PREDEFINED_GLOBAL_POOLS[3].blocks_per_period,
			PREDEFINED_GLOBAL_POOLS[3].incentivized_asset,
			PREDEFINED_GLOBAL_POOLS[3].reward_currency,
			CHARLIE,
			PREDEFINED_GLOBAL_POOLS[3].yield_per_period,
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::FarmCreated {
				id: PREDEFINED_GLOBAL_POOLS[0].id,
				owner: PREDEFINED_GLOBAL_POOLS[0].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[0].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[0].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[0].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[0].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[0].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[0].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 192282548550198434755674140525,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 4_000,
				who: 192282548550198434755674140525,
				amount: 1_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::FarmCreated {
				id: PREDEFINED_GLOBAL_POOLS[1].id,
				owner: PREDEFINED_GLOBAL_POOLS[1].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[1].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[1].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[1].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[1].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[1].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[1].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 271510711064462772349218090861,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 1_000,
				who: 271510711064462772349218090861,
				amount: 30_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::FarmCreated {
				id: PREDEFINED_GLOBAL_POOLS[2].id,
				owner: PREDEFINED_GLOBAL_POOLS[2].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[2].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[2].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[2].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[2].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[2].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[2].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 350738873578727109942762041197,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 3_000,
				who: 350738873578727109942762041197,
				amount: 30_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::FarmCreated {
				id: PREDEFINED_GLOBAL_POOLS[3].id,
				owner: PREDEFINED_GLOBAL_POOLS[3].owner,
				reward_currency: PREDEFINED_GLOBAL_POOLS[3].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_POOLS[3].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_POOLS[3].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_POOLS[3].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_POOLS[3].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_POOLS[3].max_reward_per_period,
			}),
		]);

		let amm_mock_data = vec![
			(
				BSX_ACA_AMM,
				BSX_ACA_SHARE_ID,
				AssetPair {
					asset_in: BSX,
					asset_out: ACA,
				},
			),
			(
				BSX_KSM_AMM,
				BSX_KSM_SHARE_ID,
				AssetPair {
					asset_in: KSM,
					asset_out: BSX,
				},
			),
			(
				BSX_DOT_AMM,
				BSX_DOT_SHARE_ID,
				AssetPair {
					asset_in: BSX,
					asset_out: DOT,
				},
			),
			(
				BSX_ETH_AMM,
				BSX_ETH_SHARE_ID,
				AssetPair {
					asset_in: BSX,
					asset_out: ETH,
				},
			),
			(
				BSX_HDX_AMM,
				BSX_HDX_SHARE_ID,
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
			),
			(
				BSX_TKN1_AMM,
				BSX_TKN1_SHARE_ID,
				AssetPair {
					asset_in: BSX,
					asset_out: TKN1,
				},
			),
			(
				BSX_TKN2_AMM,
				BSX_TKN2_SHARE_ID,
				AssetPair {
					asset_in: BSX,
					asset_out: TKN2,
				},
			),
			(
				KSM_DOT_AMM,
				KSM_DOT_SHARE_ID,
				AssetPair {
					asset_in: KSM,
					asset_out: DOT,
				},
			),
			(
				ACA_KSM_AMM,
				ACA_KSM_SHARE_ID,
				AssetPair {
					asset_in: ACA,
					asset_out: KSM,
				},
			),
		];

		AMM_POOLS.with(|h| {
			let mut hm = h.borrow_mut();
			for v in amm_mock_data {
				hm.insert(asset_pair_to_map_key(v.2), v);
			}
		});

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			AssetPair {
				asset_in: BSX,
				asset_out: TKN1,
			},
			PREDEFINED_LIQ_POOLS.with(|v| v[0].multiplier),
			PREDEFINED_LIQ_POOLS.with(|v| v[0].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolAdded {
			farm_id: GC_FARM,
			liq_pool_farm_id: PREDEFINED_LIQ_POOLS.with(|v| v[0].id),
			multiplier: PREDEFINED_LIQ_POOLS.with(|v| v[0].multiplier),
			nft_class: LIQ_MINING_NFT_CLASS,
			loyalty_curve: PREDEFINED_LIQ_POOLS.with(|v| v[0].loyalty_curve.clone()),
			asset_pair: AssetPair {
				asset_in: BSX,
				asset_out: TKN1,
			},
		})]);

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			AssetPair {
				asset_in: BSX,
				asset_out: TKN2,
			},
			PREDEFINED_LIQ_POOLS.with(|v| v[1].multiplier),
			PREDEFINED_LIQ_POOLS.with(|v| v[1].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolAdded {
			farm_id: GC_FARM,
			liq_pool_farm_id: PREDEFINED_LIQ_POOLS.with(|v| v[1].id),
			multiplier: PREDEFINED_LIQ_POOLS.with(|v| v[1].multiplier),
			nft_class: LIQ_MINING_NFT_CLASS,
			loyalty_curve: PREDEFINED_LIQ_POOLS.with(|v| v[1].loyalty_curve.clone()),
			asset_pair: AssetPair {
				asset_in: BSX,
				asset_out: TKN2,
			},
		})]);

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(CHARLIE),
			CHARLIE_FARM,
			AssetPair {
				asset_in: ACA,
				asset_out: KSM,
			},
			PREDEFINED_LIQ_POOLS.with(|v| v[2].multiplier),
			PREDEFINED_LIQ_POOLS.with(|v| v[2].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::LiquidityPoolAdded {
			farm_id: CHARLIE_FARM,
			liq_pool_farm_id: PREDEFINED_LIQ_POOLS.with(|v| v[2].id),
			multiplier: PREDEFINED_LIQ_POOLS.with(|v| v[2].multiplier),
			nft_class: LIQ_MINING_NFT_CLASS,
			loyalty_curve: PREDEFINED_LIQ_POOLS.with(|v| v[2].loyalty_curve.clone()),
			asset_pair: AssetPair {
				asset_in: ACA,
				asset_out: KSM,
			},
		})]);
	});

	ext
}

pub fn predefined_test_ext_with_deposits() -> sp_io::TestExternalities {
	let mut ext = predefined_test_ext();

	ext.execute_with(|| {
		let farm_id = GC_FARM; //global pool

		let bsx_tkn1_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN1,
		};

		let bsx_tkn2_assets = AssetPair {
			asset_in: BSX,
			asset_out: TKN2,
		};

		//NOTE: this should be LiquidityMining - shares are transfered to LiquidityMining account
		//not to WarehouseLM.
		let pallet_account = LiquidityMining::account_id();
		let global_pool_account = WarehouseLM::pool_account_id(GC_FARM).unwrap();
		let bsx_tkn1_liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN1_LIQ_POOL_ID).unwrap();
		let bsx_tkn2_liq_pool_account = WarehouseLM::pool_account_id(BSX_TKN2_LIQ_POOL_ID).unwrap();
		let bsx_tkn1_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(bsx_tkn1_assets)).unwrap().0);
		let bsx_tkn2_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(bsx_tkn2_assets)).unwrap().0);

		//DEPOSIT 1:
		set_block_number(1_800); //18-th period

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 50, 0).unwrap();

		let deposited_amount = 50;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn1_assets,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[0],
		})]);

		// DEPOSIT 2 (deposit in same period):

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 52, 0).unwrap();

		let deposited_amount = 80;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn1_assets,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: BOB,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[1],
		})]);

		// DEPOSIT 3 (same period, second liq pool yield farm):

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 8, 0).unwrap();

		let deposited_amount = 25;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: BOB,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[2],
		})]);

		// DEPOSIT 4 (new period):
		set_block_number(2051); //period 20

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 58, 0).unwrap();

		let deposited_amount = 800;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: BOB,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[3],
		})]);

		// DEPOSIT 5 (same period, second liq pool yield farm):
		set_block_number(2_586); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 3, 0).unwrap();

		let deposited_amount = 87;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: ALICE,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[4],
		})]);

		// DEPOSIT 6 (same period):
		set_block_number(2_596); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 16, 0).unwrap();

		let deposited_amount = 48;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn2_assets,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN2_LIQ_POOL_ID,
			who: ALICE,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[5],
		})]);

		// DEPOSIT 7 : (same period differen liq poll farm)
		set_block_number(2_596); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 80, 0).unwrap();

		let deposited_amount = 486;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			bsx_tkn1_assets,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			farm_id: GC_FARM,
			liq_pool_farm_id: BSX_TKN1_LIQ_POOL_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount,
			nft_class_id: LIQ_MINING_NFT_CLASS,
			nft_instance_id: PREDEFINED_NFT_IDS[6],
		})]);

		assert_eq!(
			WarehouseLM::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 25,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_asset: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 12,
				liq_pools_count: 2,
				total_shares_z: 703_990,
				accumulated_rewards: 231_650,
				paid_accumulated_rewards: 1_164_400,
			}
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 616,
				total_valued_shares: 45_540,
				stake_in_global_pool: 227_700,
				..PREDEFINED_LIQ_POOLS.with(|v| v[0].clone())
			},
		);

		assert_eq!(
			WarehouseLM::liquidity_pool(GC_FARM, BSX_TKN2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				updated_at: 25,
				accumulated_rpvs: 120,
				accumulated_rpz: 12,
				total_shares: 960,
				total_valued_shares: 47_629,
				stake_in_global_pool: 476_290,
				..PREDEFINED_LIQ_POOLS.with(|v| v[1].clone())
			},
		);

		//liq. pool meta check (nfts count)
		assert_eq!(WarehouseLM::liq_pool_meta(BSX_TKN1_LIQ_POOL_ID).unwrap(), (3, GC_FARM));

		//liq. pool meta check (nfts count)
		assert_eq!(WarehouseLM::liq_pool_meta(BSX_TKN2_LIQ_POOL_ID).unwrap(), (4, GC_FARM));

		//shares amount check on pallet account, sum of all deposits grouped by shares id
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &pallet_account), 616);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &pallet_account), 960);

		//reward currency balance check. total_rewards - sum(claimes from global pool)
		assert_eq!(
			Tokens::free_balance(BSX, &global_pool_account),
			(30_000_000_000 - 1_164_400)
		);

		//check of claimed amount from global pool (sum of all claims)
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_liq_pool_account), 212_400);
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn2_liq_pool_account), 952_000);

		//balance check after transfer amm shares
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE), 3_000_000 - 536);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE), 3_000_000 - 135);

		//balance check after transfer amm shares
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB), 2_000_000 - 80);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB), 2_000_000 - 825);
	});

	ext
}
