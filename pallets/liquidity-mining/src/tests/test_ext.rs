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
use crate::mock::PALLET_SERVICE_ACCOUNT;
use pretty_assertions::assert_eq;
use warehouse_liquidity_mining::GlobalFarmData;
use warehouse_liquidity_mining::YieldFarmData;

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
		assert_ok!(LiquidityMining::create_global_farm(
			Origin::root(),
			100_000_000_000,
			PREDEFINED_GLOBAL_FARMS[0].planned_yielding_periods,
			PREDEFINED_GLOBAL_FARMS[0].blocks_per_period,
			PREDEFINED_GLOBAL_FARMS[0].incentivized_asset,
			PREDEFINED_GLOBAL_FARMS[0].reward_currency,
			ALICE,
			PREDEFINED_GLOBAL_FARMS[0].yield_per_period,
			PREDEFINED_GLOBAL_FARMS[0].min_deposit,
			PREDEFINED_GLOBAL_FARMS[0].price_adjustment
		));

		assert_ok!(LiquidityMining::create_global_farm(
			Origin::root(),
			1_000_000_000,
			PREDEFINED_GLOBAL_FARMS[1].planned_yielding_periods,
			PREDEFINED_GLOBAL_FARMS[1].blocks_per_period,
			PREDEFINED_GLOBAL_FARMS[1].incentivized_asset,
			PREDEFINED_GLOBAL_FARMS[1].reward_currency,
			BOB,
			PREDEFINED_GLOBAL_FARMS[1].yield_per_period,
			PREDEFINED_GLOBAL_FARMS[1].min_deposit,
			PREDEFINED_GLOBAL_FARMS[1].price_adjustment
		));

		assert_ok!(LiquidityMining::create_global_farm(
			Origin::root(),
			30_000_000_000,
			PREDEFINED_GLOBAL_FARMS[2].planned_yielding_periods,
			PREDEFINED_GLOBAL_FARMS[2].blocks_per_period,
			PREDEFINED_GLOBAL_FARMS[2].incentivized_asset,
			PREDEFINED_GLOBAL_FARMS[2].reward_currency,
			GC,
			PREDEFINED_GLOBAL_FARMS[2].yield_per_period,
			PREDEFINED_GLOBAL_FARMS[2].min_deposit,
			PREDEFINED_GLOBAL_FARMS[2].price_adjustment
		));

		assert_ok!(LiquidityMining::create_global_farm(
			Origin::root(),
			30_000_000_000,
			PREDEFINED_GLOBAL_FARMS[3].planned_yielding_periods,
			PREDEFINED_GLOBAL_FARMS[3].blocks_per_period,
			PREDEFINED_GLOBAL_FARMS[3].incentivized_asset,
			PREDEFINED_GLOBAL_FARMS[3].reward_currency,
			CHARLIE,
			PREDEFINED_GLOBAL_FARMS[3].yield_per_period,
			PREDEFINED_GLOBAL_FARMS[3].min_deposit,
			PREDEFINED_GLOBAL_FARMS[3].price_adjustment
		));

		assert_ok!(LiquidityMining::create_global_farm(
			Origin::root(),
			30_000_000_000,
			PREDEFINED_GLOBAL_FARMS[4].planned_yielding_periods,
			PREDEFINED_GLOBAL_FARMS[4].blocks_per_period,
			PREDEFINED_GLOBAL_FARMS[4].incentivized_asset,
			PREDEFINED_GLOBAL_FARMS[4].reward_currency,
			DAVE,
			PREDEFINED_GLOBAL_FARMS[4].yield_per_period,
			PREDEFINED_GLOBAL_FARMS[4].min_deposit,
			PREDEFINED_GLOBAL_FARMS[4].price_adjustment
		));

		assert_ok!(LiquidityMining::create_global_farm(
			Origin::root(),
			30_000_000_000,
			PREDEFINED_GLOBAL_FARMS[5].planned_yielding_periods,
			PREDEFINED_GLOBAL_FARMS[5].blocks_per_period,
			PREDEFINED_GLOBAL_FARMS[5].incentivized_asset,
			PREDEFINED_GLOBAL_FARMS[5].reward_currency,
			EVE,
			PREDEFINED_GLOBAL_FARMS[5].yield_per_period,
			PREDEFINED_GLOBAL_FARMS[5].min_deposit,
			PREDEFINED_GLOBAL_FARMS[5].price_adjustment
		));

		expect_events(vec![
			mock::Event::LiquidityMining(Event::GlobalFarmCreated {
				id: PREDEFINED_GLOBAL_FARMS[0].id,
				owner: PREDEFINED_GLOBAL_FARMS[0].owner,
				reward_currency: PREDEFINED_GLOBAL_FARMS[0].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_FARMS[0].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_FARMS[0].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_FARMS[0].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_FARMS[0].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_FARMS[0].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 192282548550198434755674140525,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 4_000,
				who: 192282548550198434755674140525,
				amount: 1_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::GlobalFarmCreated {
				id: PREDEFINED_GLOBAL_FARMS[1].id,
				owner: PREDEFINED_GLOBAL_FARMS[1].owner,
				reward_currency: PREDEFINED_GLOBAL_FARMS[1].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_FARMS[1].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_FARMS[1].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_FARMS[1].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_FARMS[1].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_FARMS[1].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 271510711064462772349218090861,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 1_000,
				who: 271510711064462772349218090861,
				amount: 30_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::GlobalFarmCreated {
				id: PREDEFINED_GLOBAL_FARMS[2].id,
				owner: PREDEFINED_GLOBAL_FARMS[2].owner,
				reward_currency: PREDEFINED_GLOBAL_FARMS[2].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_FARMS[2].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_FARMS[2].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_FARMS[2].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_FARMS[2].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_FARMS[2].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 350738873578727109942762041197,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 3_000,
				who: 350738873578727109942762041197,
				amount: 30_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::GlobalFarmCreated {
				id: PREDEFINED_GLOBAL_FARMS[3].id,
				owner: PREDEFINED_GLOBAL_FARMS[3].owner,
				reward_currency: PREDEFINED_GLOBAL_FARMS[3].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_FARMS[3].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_FARMS[3].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_FARMS[3].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_FARMS[3].incentivized_asset,
				max_reward_per_period: PREDEFINED_GLOBAL_FARMS[3].max_reward_per_period,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 429967036092991447536305991533,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 3_000,
				who: 429967036092991447536305991533,
				amount: 30_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::GlobalFarmCreated {
				id: PREDEFINED_GLOBAL_FARMS[4].id,
				owner: PREDEFINED_GLOBAL_FARMS[4].owner,
				reward_currency: PREDEFINED_GLOBAL_FARMS[4].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_FARMS[4].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_FARMS[4].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_FARMS[4].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_FARMS[4].incentivized_asset,
				max_reward_per_period: 100000000,
			}),
			mock::Event::System(frame_system::Event::NewAccount {
				account: 509195198607255785129849941869,
			}),
			mock::Event::Tokens(orml_tokens::Event::Endowed {
				currency_id: 4_000,
				who: 509195198607255785129849941869,
				amount: 30_000_000_000,
			}),
			mock::Event::LiquidityMining(Event::GlobalFarmCreated {
				id: PREDEFINED_GLOBAL_FARMS[5].id,
				owner: PREDEFINED_GLOBAL_FARMS[5].owner,
				reward_currency: PREDEFINED_GLOBAL_FARMS[5].reward_currency,
				yield_per_period: PREDEFINED_GLOBAL_FARMS[5].yield_per_period,
				planned_yielding_periods: PREDEFINED_GLOBAL_FARMS[5].planned_yielding_periods,
				blocks_per_period: PREDEFINED_GLOBAL_FARMS[5].blocks_per_period,
				incentivized_asset: PREDEFINED_GLOBAL_FARMS[5].incentivized_asset,
				max_reward_per_period: 100000000,
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
			(BSX_DOT_AMM, BSX_DOT_SHARE_ID, BSX_DOT_ASSET_PAIR),
			(BSX_ETH_AMM, BSX_ETH_SHARE_ID, BSX_ETH_ASSET_PAIR),
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
			(BSX_TKN2_AMM, BSX_TKN2_SHARE_ID, BSX_TKN2_ASSET_PAIR),
			(
				KSM_DOT_AMM,
				KSM_DOT_SHARE_ID,
				AssetPair {
					asset_in: KSM,
					asset_out: DOT,
				},
			),
			(ACA_KSM_AMM, ACA_KSM_SHARE_ID, ACA_KSM_ASSET_PAIR),
		];

		AMM_POOLS.with(|h| {
			let mut hm = h.borrow_mut();
			for v in amm_mock_data {
				hm.insert(asset_pair_to_map_key(v.2), v);
			}
		});

		assert_ok!(LiquidityMining::create_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN1_ASSET_PAIR,
			PREDEFINED_YIELD_FARMS.with(|v| v[0].multiplier),
			PREDEFINED_YIELD_FARMS.with(|v| v[0].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmCreated {
			global_farm_id: GC_FARM,
			yield_farm_id: PREDEFINED_YIELD_FARMS.with(|v| v[0].id),
			multiplier: PREDEFINED_YIELD_FARMS.with(|v| v[0].multiplier),
			loyalty_curve: PREDEFINED_YIELD_FARMS.with(|v| v[0].loyalty_curve.clone()),
			asset_pair: BSX_TKN1_ASSET_PAIR,
		})]);

		assert_ok!(LiquidityMining::create_yield_farm(
			Origin::signed(GC),
			GC_FARM,
			BSX_TKN2_ASSET_PAIR,
			PREDEFINED_YIELD_FARMS.with(|v| v[1].multiplier),
			PREDEFINED_YIELD_FARMS.with(|v| v[1].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmCreated {
			global_farm_id: GC_FARM,
			yield_farm_id: PREDEFINED_YIELD_FARMS.with(|v| v[1].id),
			multiplier: PREDEFINED_YIELD_FARMS.with(|v| v[1].multiplier),
			loyalty_curve: PREDEFINED_YIELD_FARMS.with(|v| v[1].loyalty_curve.clone()),
			asset_pair: BSX_TKN2_ASSET_PAIR,
		})]);

		assert_ok!(LiquidityMining::create_yield_farm(
			Origin::signed(CHARLIE),
			CHARLIE_FARM,
			ACA_KSM_ASSET_PAIR,
			PREDEFINED_YIELD_FARMS.with(|v| v[2].multiplier),
			PREDEFINED_YIELD_FARMS.with(|v| v[2].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmCreated {
			global_farm_id: CHARLIE_FARM,
			yield_farm_id: PREDEFINED_YIELD_FARMS.with(|v| v[2].id),
			multiplier: PREDEFINED_YIELD_FARMS.with(|v| v[2].multiplier),
			loyalty_curve: PREDEFINED_YIELD_FARMS.with(|v| v[2].loyalty_curve.clone()),
			asset_pair: ACA_KSM_ASSET_PAIR,
		})]);

		assert_ok!(LiquidityMining::create_yield_farm(
			Origin::signed(DAVE),
			DAVE_FARM,
			BSX_TKN1_ASSET_PAIR,
			PREDEFINED_YIELD_FARMS.with(|v| v[3].multiplier),
			PREDEFINED_YIELD_FARMS.with(|v| v[3].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmCreated {
			global_farm_id: DAVE_FARM,
			yield_farm_id: PREDEFINED_YIELD_FARMS.with(|v| v[3].id),
			multiplier: PREDEFINED_YIELD_FARMS.with(|v| v[3].multiplier),
			loyalty_curve: PREDEFINED_YIELD_FARMS.with(|v| v[3].loyalty_curve.clone()),
			asset_pair: BSX_TKN1_ASSET_PAIR,
		})]);

		assert_ok!(LiquidityMining::create_yield_farm(
			Origin::signed(EVE),
			EVE_FARM,
			BSX_TKN1_ASSET_PAIR,
			PREDEFINED_YIELD_FARMS.with(|v| v[4].multiplier),
			PREDEFINED_YIELD_FARMS.with(|v| v[4].loyalty_curve.clone()),
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmCreated {
			global_farm_id: EVE_FARM,
			yield_farm_id: PREDEFINED_YIELD_FARMS.with(|v| v[4].id),
			multiplier: PREDEFINED_YIELD_FARMS.with(|v| v[4].multiplier),
			loyalty_curve: PREDEFINED_YIELD_FARMS.with(|v| v[4].loyalty_curve.clone()),
			asset_pair: BSX_TKN1_ASSET_PAIR,
		})]);
	});

	ext
}

pub fn predefined_test_ext_with_deposits() -> sp_io::TestExternalities {
	let mut ext = predefined_test_ext();

	ext.execute_with(|| {
		let farm_id = GC_FARM; //global pool

		let global_farm_account = WarehouseLM::farm_account_id(GC_FARM).unwrap();
		let bsx_tkn1_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN1_YIELD_FARM_ID).unwrap();
		let bsx_tkn2_yield_farm_account = WarehouseLM::farm_account_id(BSX_TKN2_YIELD_FARM_ID).unwrap();
		let bsx_tkn1_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(BSX_TKN1_ASSET_PAIR)).unwrap().0);
		let bsx_tkn2_amm_account =
			AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(BSX_TKN2_ASSET_PAIR)).unwrap().0);

		//DEPOSIT 1:
		set_block_number(1_800); //18-th period

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 50, 0).unwrap();

		let deposited_amount = 50;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount
		})]);

		// DEPOSIT 2 (deposit in same period):

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 52, 0).unwrap();

		let deposited_amount = 80;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: BOB,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount
		})]);

		// DEPOSIT 3 (same period, second liq pool yield farm):

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 8, 0).unwrap();

		let deposited_amount = 25;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			BSX_TKN2_YIELD_FARM_ID,
			BSX_TKN2_ASSET_PAIR,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN2_YIELD_FARM_ID,
			who: BOB,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount
		})]);

		// DEPOSIT 4 (new period):
		set_block_number(2051); //period 20

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 58, 0).unwrap();

		let deposited_amount = 800;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			BSX_TKN2_YIELD_FARM_ID,
			BSX_TKN2_ASSET_PAIR,
			deposited_amount
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN2_YIELD_FARM_ID,
			who: BOB,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount
		})]);

		// DEPOSIT 5 (same period, second liq pool yield farm):
		set_block_number(2_586); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 3, 0).unwrap();

		let deposited_amount = 87;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			BSX_TKN2_YIELD_FARM_ID,
			BSX_TKN2_ASSET_PAIR,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN2_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount
		})]);

		// DEPOSIT 6 (same period):
		set_block_number(2_596); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn2_amm_account, BSX, 16, 0).unwrap();

		let deposited_amount = 48;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			BSX_TKN2_YIELD_FARM_ID,
			BSX_TKN2_ASSET_PAIR,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN2_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN2_SHARE_ID,
			amount: deposited_amount
		})]);

		// DEPOSIT 7 : (same period differen liq poll farm)
		set_block_number(2_596); //period 25

		//this is done because amount of incetivized token in AMM is used in calculations.
		Tokens::set_balance(Origin::root(), bsx_tkn1_amm_account, BSX, 80, 0).unwrap();

		let deposited_amount = 486;
		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			BSX_TKN1_YIELD_FARM_ID,
			BSX_TKN1_ASSET_PAIR,
			deposited_amount,
		));

		expect_events(vec![mock::Event::LiquidityMining(Event::SharesDeposited {
			global_farm_id: GC_FARM,
			yield_farm_id: BSX_TKN1_YIELD_FARM_ID,
			who: ALICE,
			lp_token: BSX_TKN1_SHARE_ID,
			amount: deposited_amount
		})]);

		assert_eq!(
			WarehouseLM::global_farm(GC_FARM).unwrap(),
			GlobalFarmData {
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
				yield_farms_count: (2, 2),
				total_shares_z: 703_990,
				accumulated_rewards: 231_650,
				paid_accumulated_rewards: 1_164_400,
				state: FarmState::Active,
				min_deposit: 1,
				price_adjustment: One::one()
			}
		);

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN1_AMM, GC_FARM, BSX_TKN1_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				updated_at: 25,
				accumulated_rpvs: 60,
				accumulated_rpz: 12,
				total_shares: 616,
				total_valued_shares: 45_540,
				entries_count: 3,
				..PREDEFINED_YIELD_FARMS.with(|v| v[0].clone())
			},
		);

		assert_eq!(
			WarehouseLM::yield_farm((BSX_TKN2_AMM, GC_FARM, BSX_TKN2_YIELD_FARM_ID)).unwrap(),
			YieldFarmData {
				updated_at: 25,
				accumulated_rpvs: 120,
				accumulated_rpz: 12,
				total_shares: 960,
				total_valued_shares: 47_629,
				entries_count: 4,
				..PREDEFINED_YIELD_FARMS.with(|v| v[1].clone())
			},
		);

		//shares amount check on pallet account, sum of all deposits grouped by shares id
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &PALLET_SERVICE_ACCOUNT), 616);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &PALLET_SERVICE_ACCOUNT), 960);

		//reward currency balance check. total_rewards - sum(claimes from global pool)
		assert_eq!(
			Tokens::free_balance(BSX, &global_farm_account),
			(30_000_000_000 - 1_164_400)
		);

		//check of claimed amount from global pool (sum of all claims)
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn1_yield_farm_account), 212_400);
		assert_eq!(Tokens::free_balance(BSX, &bsx_tkn2_yield_farm_account), 952_000);

		//balance check after transfer amm shares
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &ALICE), 3_000_000 - 536);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &ALICE), 3_000_000 - 135);

		//balance check after transfer amm shares
		assert_eq!(Tokens::free_balance(BSX_TKN1_SHARE_ID, &BOB), 2_000_000 - 80);
		assert_eq!(Tokens::free_balance(BSX_TKN2_SHARE_ID, &BOB), 2_000_000 - 825);
	});

	ext
}
