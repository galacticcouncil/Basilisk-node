// This file is part of Basilisk-node

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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::account;
use frame_benchmarking::benchmarks;
use frame_support::pallet_prelude::DispatchError;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use orml_traits::MultiCurrencyExtended;
use sp_runtime::Permill;

use hydradx_traits::Registry;

use crate::types::{Balance, PoolId};

// Stable benchmarks
// Worst case scenarios in any stableswap calculates is scenarios where "math" does max number of iterations.
// Therefore, hydra-dx-features build with "runtime-benchmarks" features forces calculations of D and Y to perform all iterations.
// it is no longer needed to come up with some extreme scenario where it would do as many as iterations as possible.
// AS it is, it would not be possible to come up with scenarios where D/Y does not converge( or does max iterations).

benchmarks! {
	 where_clause {  where T::AssetId: From<u32> + Into<u32>,
		T::Currency: MultiCurrencyExtended<T::AccountId, Amount=i128>,
		T::AssetRegistry: Registry<T::AssetId, Vec<u8>, Balance, DispatchError>,
		T: crate::pallet::Config
	}

	create_pool{
		let token_a = T::AssetRegistry::create_asset(&b"one".to_vec(), 1u128)?;
		let token_b = T::AssetRegistry::create_asset(&b"two".to_vec(), 1u128)?;

		let initial_liquidity = (1_000_000_000_000_000u128, 2_000_000_000_000_000u128);

		let amplification = 100u32;
		let fee = Permill::from_percent(1);
		let caller: T::AccountId = account("caller", 0, 1);

		T::Currency::update_balance(token_a, &caller, 1_000_000_000_000_000i128)?;
		T::Currency::update_balance(token_b, &caller, 2_000_000_000_000_000i128)?;

		// Pool id will be next asset id in registry storage.
		let next_asset_id:u32 = Into::<u32>::into(token_b) + 1u32;
		let pool_id = PoolId( next_asset_id.into());

	}: _(RawOrigin::Signed(caller), (token_a,token_b), initial_liquidity, amplification, fee)
	verify {
		assert!(<Pools<T>>::get(pool_id).is_some());
	}

	add_liquidity{
		let token_a = T::AssetRegistry::create_asset(&b"one".to_vec(), 1u128)?;
		let token_b = T::AssetRegistry::create_asset(&b"two".to_vec(), 1u128)?;

		let initial_liquidity = (1_000_000_000_000_000u128, 2_000_000_000_000_000u128);

		let amplification = 100u32;
		let fee = Permill::from_percent(1);
		let caller: T::AccountId = account("caller", 0, 1);

		T::Currency::update_balance(token_a, &caller, 1_000_000_000_000_000i128)?;
		T::Currency::update_balance(token_b, &caller, 2_000_000_000_000_000i128)?;

		crate::Pallet::<T>::create_pool(RawOrigin::Signed(caller).into(),
			(token_a,token_b),
			initial_liquidity,
			amplification,
			fee
		)?;

		// Pool id will be next asset id in registry storage.
		let next_asset_id:u32 = Into::<u32>::into(token_b) + 1u32;
		let pool_id = PoolId( next_asset_id.into());

		let lp_provider: T::AccountId = account("provider", 0, 1);

		T::Currency::update_balance(token_a, &lp_provider, 1_000_000_000_000_000_000_000i128)?;
		T::Currency::update_balance(token_b, &lp_provider, 1_000_000_000_000_000_000_000i128)?;

		let liquidity_added = 300_000_000_000_000u128;
	}: _(RawOrigin::Signed(lp_provider.clone()), pool_id, token_a, liquidity_added)
	verify {
		assert!(T::Currency::free_balance(pool_id.0, &lp_provider) > 0u128);
	}


	remove_liquidity{
		let token_a = T::AssetRegistry::create_asset(&b"one".to_vec(), 1u128)?;
		let token_b = T::AssetRegistry::create_asset(&b"two".to_vec(), 1u128)?;

		let initial_liquidity = (1_000_000_000_000_000u128, 2_000_000_000_000_000u128);

		let amplification = 100u32;
		let fee = Permill::from_percent(1);
		let caller: T::AccountId = account("caller", 0, 1);

		T::Currency::update_balance(token_a, &caller, 1_000_000_000_000_000i128)?;
		T::Currency::update_balance(token_b, &caller, 2_000_000_000_000_000i128)?;

		crate::Pallet::<T>::create_pool(RawOrigin::Signed(caller).into(),
			(token_a,token_b),
			initial_liquidity,
			amplification,
			fee
		)?;

		// Pool id will be next asset id in registry storage.
		let next_asset_id:u32 = Into::<u32>::into(token_b) + 1u32;
		let pool_id = PoolId( next_asset_id.into());

		let lp_provider: T::AccountId = account("provider", 0, 1);

		T::Currency::update_balance(token_a, &lp_provider, 1_000_000_000_000_000_000_000i128)?;
		T::Currency::update_balance(token_b, &lp_provider, 1_000_000_000_000_000_000_000i128)?;

		let liquidity_added = 300_000_000_000_000u128;
		crate::Pallet::<T>::add_liquidity(RawOrigin::Signed(lp_provider.clone()).into(),
			pool_id,
			token_a,
			liquidity_added
		)?;

		let shares = T::Currency::free_balance(pool_id.0, &lp_provider);

	}: _(RawOrigin::Signed(lp_provider.clone()), pool_id, shares)
	verify {
		assert!(T::Currency::free_balance(pool_id.0, &lp_provider) == 0u128);
	}

	sell{
		let token_a = T::AssetRegistry::create_asset(&b"one".to_vec(), 1u128)?;
		let token_b = T::AssetRegistry::create_asset(&b"two".to_vec(), 1u128)?;

		let initial_liquidity = (1_000_000_000_000_000u128, 2_000_000_000_000_000u128);

		let amplification = 100u32;
		let fee = Permill::from_percent(1);
		let caller: T::AccountId = account("caller", 0, 1);

		T::Currency::update_balance(token_a, &caller, 1_000_000_000_000_000i128)?;
		T::Currency::update_balance(token_b, &caller, 2_000_000_000_000_000i128)?;

		crate::Pallet::<T>::create_pool(RawOrigin::Signed(caller).into(),
			(token_a,token_b),
			initial_liquidity,
			amplification,
			fee
		)?;

		// Pool id will be next asset id in registry storage.
		let next_asset_id:u32 = Into::<u32>::into(token_b) + 1u32;
		let pool_id = PoolId( next_asset_id.into());

		let seller : T::AccountId = account("seller", 0, 1);

		T::Currency::update_balance(token_a, &seller, 100_000_000_000_000i128)?;

		let amount_sell  = 100_000_000_000_000u128;
		let buy_min_amount = 1_000u128;

	}: _(RawOrigin::Signed(seller.clone()), pool_id, token_a, token_b, amount_sell, buy_min_amount)
	verify {
		assert!(T::Currency::free_balance(token_a, &seller) ==  0u128);
		assert!(T::Currency::free_balance(token_b, &seller) > 0u128);
	}

	buy{
	let token_a = T::AssetRegistry::create_asset(&b"one".to_vec(), 1u128)?;
		let token_b = T::AssetRegistry::create_asset(&b"two".to_vec(), 1u128)?;

		let initial_liquidity = (1_000_000_000_000_000u128, 2_000_000_000_000_000u128);

		let amplification = 100u32;
		let fee = Permill::from_percent(1);
		let caller: T::AccountId = account("caller", 0, 1);

		T::Currency::update_balance(token_a, &caller, 1_000_000_000_000_000i128)?;
		T::Currency::update_balance(token_b, &caller, 2_000_000_000_000_000i128)?;

		crate::Pallet::<T>::create_pool(RawOrigin::Signed(caller).into(),
			(token_a,token_b),
			initial_liquidity,
			amplification,
			fee
		)?;

		// Pool id will be next asset id in registry storage.
		let next_asset_id:u32 = Into::<u32>::into(token_b) + 1u32;
		let pool_id = PoolId( next_asset_id.into());

		let buyer: T::AccountId = account("buyer", 0, 1);

		T::Currency::update_balance(token_a, &buyer, 100_000_000_000_000i128)?;

		let amount_buy = 10_000_000_000_000u128;
		let sell_max_limit = 20_000_000_000_000_000u128;

	}: _(RawOrigin::Signed(buyer.clone()), pool_id, token_b, token_a, amount_buy, sell_max_limit)
	verify {
		assert!(T::Currency::free_balance(token_b, &buyer) > 0u128);
	}
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::tests::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}
