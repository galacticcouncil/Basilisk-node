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

/*
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use sp_runtime::FixedU128;

use frame_benchmarking::account;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use orml_traits::MultiCurrencyExtended;

benchmarks! {
	 where_clause {  where T::AssetId: From<u32>,
		T::Currency: MultiCurrencyExtended<T::AccountId, Amount=i128>,
		T: crate::pallet::Config
	}

	create_pool{
	}: _(RawOrigin::Root, stable_price, native_price)
	verify {
	}

	/*
	add_liquidity{
		let lp_provider: T::AccountId = account("provider", 1, 1);
		let token_id = T::AssetRegistry::create_asset(&b"FCK".to_vec(), 1u128)?;
		T::Currency::update_balance(token_id, &caller, 500_000_000_000_000i128)?;
		let liquidity_added = 300_000_000_000_000u128;
	}: _(RawOrigin::Signed(lp_provider), token_id, liquidity_added)
	verify {
	}

	remove_liquidity{
	}: _(RawOrigin::Signed(lp_provider.clone()), current_position_id, liquidity_added)
	verify {
	}

	sell{
	}: _(RawOrigin::Signed(seller.clone()), token_id, T::StableCoinAssetId::get(), amount_sell, buy_min_amount)
	verify {
	}

	buy{
	}: _(RawOrigin::Signed(seller.clone()), T::StableCoinAssetId::get(), token_id, amount_buy, sell_max_limit)
	verify {
	}

	 */
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::tests::mock::ExtBuilder;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}


 */