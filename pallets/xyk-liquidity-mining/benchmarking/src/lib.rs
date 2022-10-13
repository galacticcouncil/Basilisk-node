// This file is part of Basilisk-node.

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_wraps)]

mod mock;

use pallet_xyk_liquidity_mining::Pallet as LiquidityMining;
use warehouse_liquidity_mining::{GlobalFarmId, LoyaltyCurve, YieldFarmId};

use frame_benchmarking::{account, benchmarks};
use frame_system::{Pallet as System, RawOrigin};

use frame_support::dispatch;
use orml_traits::arithmetic::One;
use orml_traits::MultiCurrency;
use primitives::{asset::AssetPair, AssetId, Balance, Price};
use sp_arithmetic::FixedU128;
use sp_arithmetic::Perquintill;
use sp_std::convert::From;

use pallet_xyk as xykpool;

pub const GLOBAL_FARM_ID: GlobalFarmId = 1;
pub const GLOBAL_FARM_ID_2: GlobalFarmId = 3;
pub const YIELD_FARM_ID: YieldFarmId = 2;
pub const YIELD_FARM_ID_2: YieldFarmId = 4;
pub const YIELD_FARM_ID_3: YieldFarmId = 4;
pub const DEPOSIT_ID: u128 = 1;

const SEED: u32 = 0;

const BSX: AssetId = 0;
const KSM: AssetId = 1;
const DOT: AssetId = 2;
const ASSET_PAIR: AssetPair = AssetPair {
	asset_in: BSX,
	asset_out: KSM,
};

const INITIAL_BALANCE: Balance = 100_000_000;
const ONE: Balance = 1_000_000_000_000;

pub trait Config: pallet_xyk_liquidity_mining::Config + pallet_xyk::Config {}

pub struct Pallet<T: Config>(LiquidityMining<T>);

type MultiCurrencyOf<T> = <T as pallet_xyk_liquidity_mining::Config>::MultiCurrency;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);

	<T as pallet_xyk_liquidity_mining::Config>::MultiCurrency::deposit(BSX, &caller, INITIAL_BALANCE * ONE).unwrap();

	<T as pallet_xyk_liquidity_mining::Config>::MultiCurrency::deposit(KSM, &caller, INITIAL_BALANCE * ONE).unwrap();

	<T as pallet_xyk_liquidity_mining::Config>::MultiCurrency::deposit(DOT, &caller, INITIAL_BALANCE * ONE).unwrap();

	caller
}

fn initialize_pool<T: Config>(
	caller: T::AccountId,
	asset_a: AssetId,
	asset_b: AssetId,
	amount: Balance,
	price: Price,
) -> dispatch::DispatchResult {
	xykpool::Pallet::<T>::create_pool(RawOrigin::Signed(caller).into(), asset_a, asset_b, amount, price)?;

	Ok(())
}

fn xyk_add_liquidity<T: Config>(
	caller: T::AccountId,
	assets: AssetPair,
	amount_a: Balance,
	amount_b_max: Balance,
) -> dispatch::DispatchResult {
	xykpool::Pallet::<T>::add_liquidity(
		RawOrigin::Signed(caller).into(),
		assets.asset_in,
		assets.asset_out,
		amount_a,
		amount_b_max,
	)?;

	Ok(())
}

fn lm_crete_global_farm<T: Config>(
	total_rewards: Balance,
	owner: T::AccountId,
	yield_per_period: Perquintill,
) -> dispatch::DispatchResult {
	LiquidityMining::<T>::create_global_farm(
		RawOrigin::Root.into(),
		total_rewards,
		T::BlockNumber::from(1_000_000_u32),
		T::BlockNumber::from(1_u32),
		BSX,
		BSX,
		owner,
		yield_per_period,
		1,
		One::one(),
	)?;

	Ok(())
}

fn lm_deposit_shares<T: Config>(caller: T::AccountId, assets: AssetPair, amount: Balance) -> dispatch::DispatchResult {
	LiquidityMining::<T>::deposit_shares(
		RawOrigin::Signed(caller).into(),
		GLOBAL_FARM_ID,
		YIELD_FARM_ID,
		assets,
		amount,
	)?;

	Ok(())
}

fn lm_create_yield_farm<T: Config>(
	caller: T::AccountId,
	farm_id: GlobalFarmId,
	assets: AssetPair,
	multiplier: FixedU128,
) -> dispatch::DispatchResult {
	LiquidityMining::<T>::create_yield_farm(
		RawOrigin::Signed(caller).into(),
		farm_id,
		assets,
		multiplier,
		Some(LoyaltyCurve::default()),
	)?;

	Ok(())
}

fn set_block_number<T: Config>(block: u32) {
	System::<T>::set_block_number(block.into());
}

benchmarks! {
	create_global_farm {
		let total_rewards = 1_000_000 * ONE;
		let caller = funded_account::<T>("caller", 0);
		let planned_yielding_periods = T::BlockNumber::from(1_000_000_u32);
		let yield_per_period = Perquintill::from_percent(20);
		let blocks_per_period = T::BlockNumber::from(1_u32);
	}: {
		LiquidityMining::<T>::create_global_farm(RawOrigin::Root.into(), total_rewards, planned_yielding_periods, blocks_per_period, BSX, BSX, caller.clone(), yield_per_period, 1, One::one())?
	}
	verify {
		assert_eq!(MultiCurrencyOf::<T>::free_balance(BSX, &caller), (INITIAL_BALANCE * ONE - total_rewards ));
	}

	update_global_farm {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(ASSET_PAIR.asset_in, ASSET_PAIR.asset_out);
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, 10_000, 1_000_000_000)?;

		lm_crete_global_farm::<T>(1_000_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		lm_deposit_shares::<T>(liq_provider, ASSET_PAIR, 10_000)?;
		set_block_number::<T>(200_000);
	}: {
		LiquidityMining::<T>::update_global_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, FixedU128::from_inner(234_456_677_000_000_000_u128))?
	}

	destroy_global_farm {
		let total_rewards = 1_000_000 * ONE;
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(ASSET_PAIR.asset_in, ASSET_PAIR.asset_out);
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, 10_000, 1_000_000_000)?;

		lm_crete_global_farm::<T>(1_000_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		lm_deposit_shares::<T>(liq_provider, ASSET_PAIR, 10_000)?;
		set_block_number::<T>(100_000);

		LiquidityMining::<T>::stop_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, ASSET_PAIR)?;
		LiquidityMining::<T>::destroy_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, YIELD_FARM_ID, ASSET_PAIR)?;
		set_block_number::<T>(200_000);
	}: {
		LiquidityMining::<T>::destroy_global_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID)?
	}

	create_yield_farm {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);
		let bsx_dot = AssetPair {
			asset_in:  BSX,
			asset_out: DOT
		};

		initialize_pool::<T>(xyk_caller.clone(), ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(ASSET_PAIR.asset_in, ASSET_PAIR.asset_out);
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, 10_000, 1_000_000_000)?;

		lm_crete_global_farm::<T>(1_000_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		lm_deposit_shares::<T>(liq_provider, ASSET_PAIR, 10_000)?;
		set_block_number::<T>(100_000);

		initialize_pool::<T>(xyk_caller, bsx_dot.asset_in, bsx_dot.asset_out, 1_000_000 * ONE, Price::from(10))?;
	}: {
		LiquidityMining::<T>::create_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, bsx_dot, FixedU128::from(50_000_000_u128), Some(LoyaltyCurve::default()))?
	}

	update_yield_farm {
		let new_multiplier = FixedU128::one();
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(ASSET_PAIR.asset_in, ASSET_PAIR.asset_out);
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, 10_000, 1_000_000_000)?;

		lm_crete_global_farm::<T>(1_000_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::from_inner(500_000_000_000_000_000_u128))?;

		lm_deposit_shares::<T>(liq_provider, ASSET_PAIR, 10_000)?;
		set_block_number::<T>(100_000);
	}: {
		LiquidityMining::<T>::update_yield_farm(RawOrigin::Signed(caller.clone()).into(), 1, ASSET_PAIR, new_multiplier)?
	}

	stop_yield_farm {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(ASSET_PAIR.asset_in, ASSET_PAIR.asset_out);
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, 10_000, 1_000_000_000)?;

		lm_crete_global_farm::<T>(1_000_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		lm_deposit_shares::<T>(liq_provider, ASSET_PAIR, 10_000)?;
		set_block_number::<T>(100_000);
	}: {
		LiquidityMining::<T>::stop_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, ASSET_PAIR)?
	}

	destroy_yield_farm {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(ASSET_PAIR.asset_in, ASSET_PAIR.asset_out);
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, 10_000, 1_000_000_000)?;

		lm_crete_global_farm::<T>(1_000_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		lm_deposit_shares::<T>(liq_provider, ASSET_PAIR, 10_000)?;
		set_block_number::<T>(100_000);

		LiquidityMining::<T>::stop_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, ASSET_PAIR)?;
	}: {
		LiquidityMining::<T>::destroy_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID,YIELD_FARM_ID, ASSET_PAIR)?
	}

	deposit_shares {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(ASSET_PAIR.asset_in, ASSET_PAIR.asset_out);
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, 100_000, 1_000_000_000)?;

		lm_crete_global_farm::<T>(1_000_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller, GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		lm_deposit_shares::<T>(liq_provider.clone(), ASSET_PAIR, 10_000)?;
		set_block_number::<T>(100_000);
	}: {
		LiquidityMining::<T>::deposit_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID, YIELD_FARM_ID, ASSET_PAIR, 10_000)?
	}

	redeposit_lp_shares {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);
		let shares_amount = 10_000;

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, shares_amount, 1_000_000_000)?;

		//global id: 1, yield id: 2
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		//global id: 3, yield id: 4
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID_2, ASSET_PAIR, FixedU128::one())?;

		//global id: 5, yield id:6
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), 5, ASSET_PAIR, FixedU128::one())?;

		//global id: 7, yield id:8
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), 7, ASSET_PAIR, FixedU128::one())?;

		//global id: 9, yield id:10
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller, 9, ASSET_PAIR, FixedU128::one())?;

		set_block_number::<T>(200_000);

		LiquidityMining::<T>::deposit_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID, YIELD_FARM_ID, ASSET_PAIR, shares_amount)?;
		LiquidityMining::<T>::redeposit_lp_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID_2, YIELD_FARM_ID_2, ASSET_PAIR, DEPOSIT_ID)?;
		LiquidityMining::<T>::redeposit_lp_shares(RawOrigin::Signed(liq_provider.clone()).into(), 5, 6, ASSET_PAIR, DEPOSIT_ID)?;
		LiquidityMining::<T>::redeposit_lp_shares(RawOrigin::Signed(liq_provider.clone()).into(), 7, 8, ASSET_PAIR, DEPOSIT_ID)?;
	}: {
		LiquidityMining::<T>::redeposit_lp_shares(RawOrigin::Signed(liq_provider.clone()).into(), 9, 10, ASSET_PAIR, DEPOSIT_ID)?
	}

	claim_rewards {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);
		let shares_amount = 10_000;

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, shares_amount, 1_000_000_000)?;

		//global id: 1, yield id: 2
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(),GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		//global id: 3, yield id: 4
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID_2, ASSET_PAIR, FixedU128::one())?;

		//global id: 5, yield id:6
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), 5, ASSET_PAIR, FixedU128::one())?;

		//global id: 7, yield id:8
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller.clone(), 7, ASSET_PAIR, FixedU128::one())?;

		//global id: 9, yield id:10
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller, 9, ASSET_PAIR, FixedU128::one())?;

		set_block_number::<T>(200_000);

		LiquidityMining::<T>::deposit_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID, YIELD_FARM_ID, ASSET_PAIR, shares_amount)?;
		LiquidityMining::<T>::redeposit_lp_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID_2, YIELD_FARM_ID_2, ASSET_PAIR, DEPOSIT_ID)?;
		LiquidityMining::<T>::redeposit_lp_shares(RawOrigin::Signed(liq_provider.clone()).into(), 5, 6, ASSET_PAIR, DEPOSIT_ID)?;
		LiquidityMining::<T>::redeposit_lp_shares(RawOrigin::Signed(liq_provider.clone()).into(), 7, 8, ASSET_PAIR, DEPOSIT_ID)?;
		LiquidityMining::<T>::redeposit_lp_shares(RawOrigin::Signed(liq_provider.clone()).into(), 9, 10, ASSET_PAIR, DEPOSIT_ID)?;

		set_block_number::<T>(400_000);
		let liq_provider_bsx_balance = MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider);
	}: {
		LiquidityMining::<T>::claim_rewards(RawOrigin::Signed(liq_provider.clone()).into(), DEPOSIT_ID, 10)?
	} verify {
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider).gt(&liq_provider_bsx_balance));
	}

	withdraw_shares {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);
		let shares_amount = 10_000;

		initialize_pool::<T>(xyk_caller, ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		xyk_add_liquidity::<T>(liq_provider.clone(), ASSET_PAIR, shares_amount, 1_000_000_000)?;

		//global id: 1, yield id: 2
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		lm_create_yield_farm::<T>(caller,GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;

		set_block_number::<T>(200_000);

		LiquidityMining::<T>::deposit_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID, YIELD_FARM_ID, ASSET_PAIR, shares_amount)?;

		set_block_number::<T>(400_000);
		let liq_provider_bsx_balance = MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider);
	}: {
		LiquidityMining::<T>::withdraw_shares(RawOrigin::Signed(liq_provider.clone()).into(), DEPOSIT_ID, YIELD_FARM_ID, ASSET_PAIR)?
	} verify {
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider).gt(&liq_provider_bsx_balance));
	}

	resume_yield_farm {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);
		let shares_amount = 10_000;
		let bsx_dot = AssetPair {
			asset_in:  BSX,
			asset_out: DOT
		};

		initialize_pool::<T>(xyk_caller.clone(), ASSET_PAIR.asset_in, ASSET_PAIR.asset_out, 1_000_000 * ONE, Price::from(10))?;
		initialize_pool::<T>(xyk_caller, bsx_dot.asset_in, bsx_dot.asset_out, 1_000_000 * ONE, Price::from(10))?;
		xyk_add_liquidity::<T>(liq_provider.clone(), bsx_dot, shares_amount, 1_000_000_000)?;

		//global id: 1
		lm_crete_global_farm::<T>(100_000 * ONE, caller.clone(), Perquintill::from_percent(20))?;
		//yield id: 2
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, ASSET_PAIR, FixedU128::one())?;
		//yield id: 3
		lm_create_yield_farm::<T>(caller.clone(), GLOBAL_FARM_ID, bsx_dot, FixedU128::one())?;

		LiquidityMining::<T>::stop_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, ASSET_PAIR)?;

		set_block_number::<T>(200_000);

		LiquidityMining::<T>::deposit_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID, 3, bsx_dot, shares_amount)?;

		set_block_number::<T>(400_000);
		let liq_provider_bsx_balance = MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider);
	}: {
		LiquidityMining::<T>::resume_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, YIELD_FARM_ID, ASSET_PAIR, FixedU128::from(12_452))?
	}
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
