// This file is part of HydraDX.

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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const SEED: u32 = 1;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId
where
	<T as pallet::Config>::CurrencyId: From<u32>,
{
	let caller: T::AccountId = account(name, index, SEED);
	caller
}

benchmarks! {
	dust_account{
		let caller = funded_account::<T>("caller", 0);
		let to_dust_account = funded_account::<T>("dust", 0);

		let asset_id: T::CurrencyId = T::CurrencyId::from(1u32);
		let reward = T::Reward::get();

		let min_deposit = T::MinCurrencyDeposits::get(&asset_id);

		let dust_amount : T::Amount = T::Amount::try_from(min_deposit).ok().unwrap() - T::Amount::from(1u32);

		T::MultiCurrency::update_balance(asset_id, &to_dust_account, dust_amount).unwrap();
		assert_eq!(T::MultiCurrency::free_balance(1u32.into(), &to_dust_account), dust_amount.try_into().ok().unwrap());

		let current_balance = T::MultiCurrency::free_balance(asset_id, &crate::Pallet::<T>::dust_dest_account());

	}: _(RawOrigin::Signed(caller.clone()), to_dust_account.clone(),1u32.into())
	verify {
		assert_eq!(T::MultiCurrency::free_balance(1u32.into(), &to_dust_account), 0u32.into());
		assert_eq!(T::MultiCurrency::free_balance(0u32.into(), &caller), reward);
		assert_eq!(T::MultiCurrency::free_balance(1u32.into(), &crate::Pallet::<T>::dust_dest_account()), current_balance + dust_amount.try_into().ok().unwrap());
	}

	add_nondustable_account{
		let caller = funded_account::<T>("caller", 0);
		let nondustable_account = funded_account::<T>("dust", 0);
	}: _(RawOrigin::Root, nondustable_account.clone())
	verify {
		assert!(crate::Pallet::<T>::blacklisted(&nondustable_account).is_some());
	}

	remove_nondustable_account{
		let caller = funded_account::<T>("caller", 0);
		let nondustable_account = funded_account::<T>("dust", 0);
		let _ = crate::Pallet::<T>::add_nondustable_account(RawOrigin::Root.into(), nondustable_account.clone());

	}: _(RawOrigin::Root, nondustable_account.clone())
	verify {
		assert!(crate::Pallet::<T>::blacklisted(&nondustable_account).is_none());
	}
}

#[cfg(test)]
mod tests {
	use super::mock::Test;
	use super::*;
	use crate::mock::ExtBuilder;
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_dust_account());
			assert_ok!(Pallet::<Test>::test_benchmark_add_nondustable_account());
			assert_ok!(Pallet::<Test>::test_benchmark_remove_nondustable_account());
		});
	}
}
