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

#![cfg(feature = "runtime-benchmarks")]

use crate::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use hydradx_traits_stableswap::Registry;
use pallet_stableswap::types::AssetLiquidity;
use pallet_stableswap as stableswap;
use sp_arithmetic::Permill;
use sp_runtime::traits::One;
use orml_traits::MultiCurrency;
use orml_traits::MultiCurrencyExtended;

const ONE: Balance = 1_000_000_000_000;
const BSX: AssetId = 1;
const DAI: AssetId = 2;

const INITIAL_BALANCE: Balance = 1_000_000 * ONE;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn fund<T: Config>(to: T::AccountId, currency: AssetId, amount: Balance) -> DispatchResult {
	T::MultiCurrency::deposit(currency, &to, amount)
}

fn initialize_global_farm<T: Config>(owner: T::AccountId) -> DispatchResult {
	Pallet::<T>::create_global_farm(
		RawOrigin::Root.into(),
		1_000_000 * ONE,
		T::BlockNumber::from(10_000_u32),
		T::BlockNumber::from(10_u32),
		BSX,
		BSX,
		owner.clone(),
		Perquintill::from_percent(20),
		100,
		FixedU128::from(14_u128),
	)
}

fn initialize_stableswap_pool_with_liquidity<T: Config>(owner: T::AccountId,seller: T::AccountId) -> AssetId {
	let token_a = T::AssetRegistry::create_asset(&b"one".to_vec(), 1u128).unwrap();
	let token_b = T::AssetRegistry::create_asset(&b"two".to_vec(), 1u128).unwrap();

	let initial_liquidity = 1_000_000_000_000_000u128;

	let amplification = 100u16;
	let trade_fee = Permill::from_percent(1);
	let withdraw_fee = Permill::from_percent(2);

	<T as stableswap::Config>::Currency::update_balance(token_a, &owner, 1_000_000_000_000_000i128).unwrap();
	T::Currency::update_balance(token_b, &owner, 2_000_000_000_000_000i128).unwrap();

	stableswap::Pallet::<T>::create_pool(
		RawOrigin::Signed(owner.clone()).into(),
		vec![token_a, token_b],
		amplification,
		trade_fee,
		withdraw_fee,
	).unwrap();

	// Pool id will be next asset id in registry storage.
	let next_asset_id: u32 = Into::<u32>::into(token_b) + 1u32;
	let pool_id: T::AssetId = next_asset_id.into();

	// Initial liquidity
	stableswap::Pallet::<T>::add_liquidity(
		RawOrigin::Signed(owner).into(),
		pool_id,
		vec![
			AssetLiquidity {
				asset_id: token_a,
				amount: initial_liquidity,
			},
			AssetLiquidity {
				asset_id: token_b,
				amount: initial_liquidity,
			},
		],
	).unwrap();

	T::MultiCurrency::update_balance(token_a, &seller, 100_000_000_000_000i128).unwrap();

	let amount_sell = 100_000_000_000_000u128;
	let buy_min_amount = 1_000u128;

    pool_id
}

benchmarks! {
	 where_clause {  where
		T::AssetId: From<u32> + Into<u32>,
		T::AssetRegistry: Registry<T::AssetId, Vec<u8>, Balance, DispatchError>,
		T: crate::pallet::Config
	}

	create_global_farm {
	   let caller: T::AccountId = account("caller", 0, 1);

	   let total_rewards = 1_000_000 * ONE;
	   let planned_yielding_periods = T::BlockNumber::from(10_000_u32);
	   let blocks_per_period = T::BlockNumber::from(100_u32);
	   let incentivized_asset = BSX;
	   let reward_currency = BSX;
	   let owner = caller.clone();
	   let yield_per_period = Perquintill::from_percent(20);
	   let min_deposit = 1_000;
	   let price_adjustment = FixedU128::from(10_u128);

	   let planned_periods =
			TryInto::<u128>::try_into(planned_yielding_periods).map_err(|_| "Type conversion overflow").unwrap();

	   let max_reward_per_period = total_rewards.checked_div(planned_periods).unwrap();

		fund::<T>(caller.clone(), BSX, 2_000_000 * ONE)?;
	}: _(RawOrigin::Root, total_rewards, planned_yielding_periods, blocks_per_period, incentivized_asset, reward_currency, owner, yield_per_period, min_deposit, price_adjustment)
	verify {
		assert_last_event::<T>(Event::<T>::GlobalFarmCreated {
			id: 1,
			owner: caller,
			reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			incentivized_asset,
			max_reward_per_period
		}.into());
	}

	destroy_global_farm {
	   let owner: T::AccountId = account("caller", 0, 1);

		initialize_global_farm::<T>(owner.clone())?;
	}: _(RawOrigin::Signed(owner.clone()),  1)
	verify {
		assert_last_event::<T>(Event::<T>::GlobalFarmDestroyed {
			id: 1,
			who: owner,
			reward_currency: BSX,
			undistributed_rewards: 1_000_000 * ONE
		}.into());
	}

	update_global_farm_price_adjustment {
		let owner: T::AccountId = account("caller", 0, 1);
		let new_price_adjustment = FixedU128::from_inner(500_000_000_000_000_000_u128);
		let global_farm_id = 2;

		initialize_global_farm::<T>(owner.clone())?;
	}: _(RawOrigin::Signed(owner.clone()), global_farm_id, new_price_adjustment)
	verify {
		assert_last_event::<T>(Event::<T>::GlobalFarmUpdated {
			id: global_farm_id,
			who: owner,
			price_adjustment: new_price_adjustment
		}.into());
	}

    create_yield_farm {
		let owner: T::AccountId = account("caller", 0, 1);
		let global_farm_id = 3;
		let new_yield_farm_id = 4;
		let multiplier = FixedU128::one();
		let loyalty_curve = LoyaltyCurve::default();


		initialize_global_farm::<T>(owner.clone())?;

        let pool_id = 1;

	}: _(RawOrigin::Signed(owner.clone()), global_farm_id, pool_id, multiplier, Some(loyalty_curve))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, crate::tests::mock::ExtBuilder::default().build(), super::Test);
}
