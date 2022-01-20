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

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

use orml_traits::MultiCurrency;
use pallet_liquidity_mining::Pallet as LiquidityMining;
use primitives::AssetId;
use sp_arithmetic::Permill;

const SEED: u32 = 0;

const BSX: AssetId = 1000;

pub trait Config: pallet_liquidity_mining::Config {}

pub struct Pallet<T: Config>(LiquidityMining<T>);

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	<T as pallet_liquidity_mining::Config>::MultiCurrency::deposit(BSX.into(), &caller, 1_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	create_farm {
		let caller = funded_account::<T>("caller", 0);
	}: { LiquidityMining::<T>::create_farm(RawOrigin::Root.into(), 1_000_000_000_000, T::BlockNumber::from(1_000_000_u32), T::BlockNumber::from(1_u32), BSX.into(), BSX.into(), caller, Permill::from_percent(20))? }
	verify {
		assert!(LiquidityMining::<T>::global_pool(1).is_some());
	}

	destroy_farm {
		let caller = funded_account::<T>("caller", 0);
		LiquidityMining::<T>::create_farm(RawOrigin::Root.into(), 1_000_000_000_000, T::BlockNumber::from(1_000_000_u32), T::BlockNumber::from(1_u32), BSX.into(), BSX.into(), caller.clone(), Permill::from_percent(20))?;

		assert!(LiquidityMining::<T>::global_pool(1).is_some());

		LiquidityMining::<T>::withdraw_undistributed_rewards(RawOrigin::Signed(caller.clone()).into(), 1)?;
	}: { LiquidityMining::<T>::destroy_farm(RawOrigin::Signed(caller.clone()).into(), 1)? }
	verify {
		assert!(LiquidityMining::<T>::global_pool(1).is_none());
	}

	withdraw_undistributed_rewards {
		let caller = funded_account::<T>("caller", 0);
		LiquidityMining::<T>::create_farm(RawOrigin::Root.into(), 1_000_000_000_000, T::BlockNumber::from(1_000_000_u32), T::BlockNumber::from(1_u32), BSX.into(), BSX.into(), caller.clone(), Permill::from_percent(20))?;

		let g_pool_acc = LiquidityMining::<T>::pool_account_id(1).unwrap();
		assert!(T::MultiCurrency::free_balance(BSX.into(), &g_pool_acc) == 1_000_000_000_000);
	}: { LiquidityMining::<T>::withdraw_undistributed_rewards(RawOrigin::Signed(caller.clone()).into(), 1)? }
	verify {
		assert_eq!(T::MultiCurrency::free_balance(BSX.into(), &g_pool_acc), 0);
		assert_eq!(T::MultiCurrency::free_balance(BSX.into(), &caller.clone()), 1_000_000_000_000_000);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_create_farm());
			assert_ok!(Pallet::<Test>::test_benchmark_destroy_farm());
			assert_ok!(Pallet::<Test>::test_benchmark_withdraw_undistributed_rewards());
			//assert_ok!(Pallet::<Test>::test_benchmark_add_liquidity_pool());
		});
	}
}

//TODO:
//  * add_liquidity_pool
//  * update_liquidity_pool
//  * cancel_liquidity_pool
//  * remove_liquidity_pool
//  * deposit_shares
//  * claim_rewards
//  * withdraw_shares
//
