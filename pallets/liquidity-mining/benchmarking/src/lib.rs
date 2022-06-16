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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_wraps)]

mod mock;

use pallet_liquidity_mining::Pallet as LiquidityMining;
use warehouse_liquidity_mining::Pallet as WarehouseLM;
use warehouse_liquidity_mining::{GlobalFarmId, YieldFarmId, YieldFarmState};

use frame_benchmarking::{account, benchmarks};
use frame_system::{Pallet as System, RawOrigin};

use frame_support::dispatch;
use orml_traits::MultiCurrency;
use primitives::{asset::AssetPair, AssetId, Balance, Price};
use sp_arithmetic::FixedU128;
use sp_arithmetic::Permill;
use sp_std::convert::From;

use pallet_xyk as xykpool;
use primitives::constants::currency::NATIVE_EXISTENTIAL_DEPOSIT;
use warehouse_liquidity_mining::LoyaltyCurve;

pub const GLOBAL_FARM_ID: GlobalFarmId = 1;
pub const YIELD_FARM_ID: YieldFarmId = 2;
pub const DEPOSIT_ID: u128 = 1;

const SEED: u32 = 0;

const BSX: AssetId = 0;
const KSM: AssetId = 1;

pub trait Config: pallet_liquidity_mining::Config + pallet_xyk::Config {}

pub struct Pallet<T: Config>(LiquidityMining<T>);

const INITIAL_BALANCE: Balance = 100_000_000_000_000_000;

type MultiCurrencyOf<T> = <T as pallet_liquidity_mining::Config>::MultiCurrency;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);

	<T as pallet_liquidity_mining::Config>::MultiCurrency::deposit(
		BSX,
		&caller,
		INITIAL_BALANCE * NATIVE_EXISTENTIAL_DEPOSIT,
	)
	.unwrap();

	<T as pallet_liquidity_mining::Config>::MultiCurrency::deposit(
		KSM,
		&caller,
		INITIAL_BALANCE * NATIVE_EXISTENTIAL_DEPOSIT,
	)
	.unwrap();

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

fn init_farm<T: Config>(
	total_rewards: Balance,
	owner: T::AccountId,
	yield_per_period: Permill,
) -> dispatch::DispatchResult {
	LiquidityMining::<T>::create_global_farm(
		RawOrigin::Root.into(),
		total_rewards * NATIVE_EXISTENTIAL_DEPOSIT,
		T::BlockNumber::from(1_000_000_u32),
		T::BlockNumber::from(1_u32),
		BSX,
		BSX,
		owner,
		yield_per_period,
	)?;

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
	assets: AssetPair,
	multiplier: FixedU128,
) -> dispatch::DispatchResult {
	LiquidityMining::<T>::create_yield_farm(
		RawOrigin::Signed(caller).into(),
		1,
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
		let caller = funded_account::<T>("caller", 0);
	}: {
		LiquidityMining::<T>::create_global_farm(RawOrigin::Root.into(), 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, T::BlockNumber::from(1_000_000_u32), T::BlockNumber::from(1_u32), BSX, BSX, caller.clone(), Permill::from_percent(20))? }
	verify {
		assert!(WarehouseLM::<T>::global_farm(GLOBAL_FARM_ID).is_some());
	}

	destroy_farm {
		let caller = funded_account::<T>("caller", 0);

		init_farm::<T>(1_000_000, caller.clone(), Permill::from_percent(20))?;

		assert!(WarehouseLM::<T>::global_farm(GLOBAL_FARM_ID).is_some());

	}: {
		LiquidityMining::<T>::destroy_global_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID)?
	}
	verify {
		assert!(WarehouseLM::<T>::global_farm(GLOBAL_FARM_ID).is_none());
	}


	create_yield_farm {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);

		initialize_pool::<T>(xyk_caller, BSX, KSM, 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, Price::from(10))?;

		init_farm::<T>(1_000_000, caller.clone(), Permill::from_percent(20))?;

		let global_pool_account = WarehouseLM::<T>::farm_account_id(1).unwrap();
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &global_pool_account) == 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT);

		let assets = AssetPair {
			asset_in: BSX,
			asset_out: KSM,
		};

	}: {
		LiquidityMining::<T>::create_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, assets, FixedU128::from(50_000_u128), Some(LoyaltyCurve::default()))?
	}
	verify {
		assert_eq!(WarehouseLM::<T>::global_farm(GLOBAL_FARM_ID).unwrap().yield_farms_count, (1,1));

		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(assets.asset_in, assets.asset_out);

		assert!(WarehouseLM::<T>::yield_farm((xyk_id,GLOBAL_FARM_ID,YIELD_FARM_ID)).is_some());
	}

	update_yield_farm {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);

		initialize_pool::<T>(xyk_caller, BSX, KSM, 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, Price::from(10))?;

		init_farm::<T>(1_000_000, caller.clone(), Permill::from_percent(20))?;

		let global_pool_account = WarehouseLM::<T>::farm_account_id(1).unwrap();
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &global_pool_account) == 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT);

		let assets = AssetPair {
			asset_in: BSX,
			asset_out: KSM,
		};

		LiquidityMining::<T>::create_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, assets, FixedU128::from(50_000_u128), Some(LoyaltyCurve::default()))?;

		assert_eq!(WarehouseLM::<T>::global_farm(GLOBAL_FARM_ID).unwrap().yield_farms_count, (1,1));

		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(assets.asset_in, assets.asset_out);
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(), GLOBAL_FARM_ID, YIELD_FARM_ID)).unwrap().multiplier, FixedU128::from(50_000_u128));
	}: {
		LiquidityMining::<T>::update_yield_farm(RawOrigin::Signed(caller.clone()).into(), 1, assets, FixedU128::from(10_000_u128))?
	}
	verify {
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id, GLOBAL_FARM_ID, YIELD_FARM_ID)).unwrap().multiplier, FixedU128::from(10_000_u128));
	}

	stop_yield_farm {
		//init nft class for liq. mining
		pallet_liquidity_mining::migration::init_nft_class::<T>();

		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, BSX, KSM, 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, Price::from(10))?;

		init_farm::<T>(1_000_000, caller.clone(), Permill::from_percent(20))?;

		let global_pool_account = WarehouseLM::<T>::farm_account_id(1).unwrap();
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &global_pool_account) == 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT);

		let assets = AssetPair {
			asset_in: BSX,
			asset_out: KSM,
		};

		lm_create_yield_farm::<T>(caller.clone(), assets, FixedU128::from(50_000_u128))?;

		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(assets.asset_in, assets.asset_out);

		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Active);

		xyk_add_liquidity::<T>(liq_provider.clone(), assets, 10_000, 1_000_000_000)?;

		lm_deposit_shares::<T>(liq_provider, assets, 10_000)?;
		set_block_number::<T>(200_000);

		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().updated_at, 0_u32.into());

	}: {
		LiquidityMining::<T>::stop_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, assets)?
	}
	verify {
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Stopped);

		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().updated_at, 200_000_u32.into());
	}

	destroy_yield_farm {
		//init nft class for liq. mining
		pallet_liquidity_mining::migration::init_nft_class::<T>();

		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, BSX, KSM, 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, Price::from(10))?;

		init_farm::<T>(1_000_000, caller.clone(), Permill::from_percent(20))?;

		let global_pool_account = WarehouseLM::<T>::farm_account_id(1).unwrap();
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &global_pool_account) == 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT);

		let assets = AssetPair {
			asset_in: BSX,
			asset_out: KSM,
		};

		lm_create_yield_farm::<T>(caller.clone(), assets, FixedU128::from(50_000_u128))?;

		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(assets.asset_in, assets.asset_out);
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Active);
		xyk_add_liquidity::<T>(liq_provider.clone(), assets, 10_000, 1_000_000_000)?;

		lm_deposit_shares::<T>(liq_provider.clone(), assets, 10_000)?;

		set_block_number::<T>(200_000);

		LiquidityMining::<T>::withdraw_shares(
			RawOrigin::Signed(liq_provider).into(),
			DEPOSIT_ID,
			YIELD_FARM_ID
		)?;

		LiquidityMining::<T>::stop_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID, assets)?;

		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Stopped);
	}: {
		LiquidityMining::<T>::destroy_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID,YIELD_FARM_ID, assets)?
	}
	verify {
		assert!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).is_none());
		assert_eq!(WarehouseLM::<T>::global_farm(GLOBAL_FARM_ID).unwrap().yield_farms_count, (0,0));
	}

	deposit_shares {
		//init nft class for liq. mining
		pallet_liquidity_mining::migration::init_nft_class::<T>();

		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, BSX, KSM, 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, Price::from(10))?;

		init_farm::<T>(1_000_000, caller.clone(), Permill::from_percent(20))?;

		let global_pool_account = WarehouseLM::<T>::farm_account_id(1).unwrap();
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &global_pool_account) == 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT);

		let assets = AssetPair {
			asset_in: BSX,
			asset_out: KSM,
		};

		lm_create_yield_farm::<T>(caller, assets, FixedU128::from(50_000_u128))?;

		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(assets.asset_in, assets.asset_out);
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Active);

		xyk_add_liquidity::<T>(liq_provider.clone(), assets, 10_000, 1_000_000_000)?;

		set_block_number::<T>(200_000);

		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().updated_at, 0_u32.into());
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id,GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().updated_at, 0_u32.into());

		assert!(WarehouseLM::<T>::deposit(
			DEPOSIT_ID
		).is_none());

	}: {
		LiquidityMining::<T>::deposit_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID, YIELD_FARM_ID, assets, 10_000)?
	}
	verify {
		assert!(WarehouseLM::<T>::deposit(
			DEPOSIT_ID
		).is_some());
	}

	claim_rewards {
		//init nft class for liq. mining
		pallet_liquidity_mining::migration::init_nft_class::<T>();

		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, BSX, KSM, 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, Price::from(10))?;

		init_farm::<T>(INITIAL_BALANCE, caller.clone(), Permill::from_percent(20))?;

		let global_pool_account = WarehouseLM::<T>::farm_account_id(1).unwrap();
		assert_eq!(MultiCurrencyOf::<T>::free_balance(BSX, &global_pool_account), INITIAL_BALANCE * NATIVE_EXISTENTIAL_DEPOSIT);

		let assets = AssetPair {
			asset_in: BSX,
			asset_out: KSM,
		};

		lm_create_yield_farm::<T>(caller, assets, FixedU128::from(50_000_u128))?;

		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(assets.asset_in, assets.asset_out);
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Active);

		xyk_add_liquidity::<T>(liq_provider.clone(), assets, 10_000, 1_000_000_000)?;

		set_block_number::<T>(200_000);

		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().updated_at, 0_u32.into());
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id,GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().updated_at, 0_u32.into());

		LiquidityMining::<T>::deposit_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID,YIELD_FARM_ID, assets, 10_000)?;

		assert!(WarehouseLM::<T>::deposit(DEPOSIT_ID).is_some());

		set_block_number::<T>(400_000);

		let liq_provider_bsx_balance = MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider);
	}: {
		LiquidityMining::<T>::claim_rewards(RawOrigin::Signed(liq_provider.clone()).into(), DEPOSIT_ID, YIELD_FARM_ID)?
	}
	verify {
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider).gt(&liq_provider_bsx_balance));
	}

	withdraw_shares {
		//init nft class for liq. mining
		pallet_liquidity_mining::migration::init_nft_class::<T>();

		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);
		let liq_provider = funded_account::<T>("liq_provider", 2);

		initialize_pool::<T>(xyk_caller, BSX, KSM, 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, Price::from(10))?;

		init_farm::<T>(INITIAL_BALANCE, caller.clone(), Permill::from_percent(20))?;

		let global_pool_account = WarehouseLM::<T>::farm_account_id(1).unwrap();
		assert_eq!(MultiCurrencyOf::<T>::free_balance(BSX, &global_pool_account), INITIAL_BALANCE * NATIVE_EXISTENTIAL_DEPOSIT);

		let assets = AssetPair {
			asset_in: BSX,
			asset_out: KSM,
		};

		lm_create_yield_farm::<T>(caller, assets, FixedU128::from(50_000_u128))?;

		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(assets.asset_in, assets.asset_out);
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Active);

		xyk_add_liquidity::<T>(liq_provider.clone(), assets, 10_000, 1_000_000_000)?;

		set_block_number::<T>(200_000);

		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().updated_at, 0_u32.into());
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id,GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().updated_at, 0_u32.into());

		LiquidityMining::<T>::deposit_shares(RawOrigin::Signed(liq_provider.clone()).into(), GLOBAL_FARM_ID, YIELD_FARM_ID, assets, 10_000)?;

		assert!(WarehouseLM::<T>::deposit(DEPOSIT_ID).is_some());

		set_block_number::<T>(400_000);

		let liq_provider_bsx_balance = MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider);
	}: {
		LiquidityMining::<T>::withdraw_shares(RawOrigin::Signed(liq_provider.clone()).into(), DEPOSIT_ID, YIELD_FARM_ID)?
	}
	verify {
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &liq_provider).gt(&liq_provider_bsx_balance));
		assert!(WarehouseLM::<T>::deposit(DEPOSIT_ID).is_none());

	}

	//NOTE: This is same no matter if `update_global_pool()` is called because `GlobalFarm`will be
	//read/written either way.
	resume_yield_farm {
		let caller = funded_account::<T>("caller", 0);
		let xyk_caller = funded_account::<T>("xyk_caller", 1);

		initialize_pool::<T>(xyk_caller, BSX, KSM, 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT, Price::from(10))?;

		init_farm::<T>(1_000_000, caller.clone(), Permill::from_percent(20))?;

		let global_pool_account = WarehouseLM::<T>::farm_account_id(1).unwrap();
		assert!(MultiCurrencyOf::<T>::free_balance(BSX, &global_pool_account) == 1_000_000 * NATIVE_EXISTENTIAL_DEPOSIT);

		let assets = AssetPair {
			asset_in: BSX,
			asset_out: KSM,
		};

		lm_create_yield_farm::<T>(caller.clone(), assets, FixedU128::from(50_000_u128))?;

		let xyk_id = xykpool::Pallet::<T>::pair_account_from_assets(assets.asset_in, assets.asset_out);
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Active);

		LiquidityMining::<T>::stop_yield_farm(RawOrigin::Signed(caller.clone()).into(), 1, assets)?;

		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Stopped);
	}: {
		LiquidityMining::<T>::resume_yield_farm(RawOrigin::Signed(caller.clone()).into(), GLOBAL_FARM_ID,YIELD_FARM_ID, assets, FixedU128::from(12_452))?
	}
	verify {
		assert_eq!(WarehouseLM::<T>::yield_farm((xyk_id.clone(),GLOBAL_FARM_ID,YIELD_FARM_ID)).unwrap().state, YieldFarmState::Active);
	}
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
