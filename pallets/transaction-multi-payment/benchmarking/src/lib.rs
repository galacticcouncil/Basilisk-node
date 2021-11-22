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

mod mock;

use sp_std::prelude::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use orml_traits::MultiCurrencyExtended;
use pallet_transaction_multi_payment::Pallet as MultiPaymentModule;
use primitives::{Amount, AssetId, Balance, Price};

#[cfg(test)]
use orml_traits::MultiCurrency;

use frame_support::dispatch;
use pallet_xyk as xykpool;

pub struct Pallet<T: Config>(pallet_transaction_multi_payment::Pallet<T>);

pub trait Config:
	pallet_transaction_payment::Config + pallet_transaction_multi_payment::Config + xykpool::Config
{
}

const SEED: u32 = 0;
const ASSET_ID: u32 = 2;
const HDX: u32 = 0;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId
where
	T::Currencies: MultiCurrencyExtended<T::AccountId, CurrencyId = AssetId, Balance = Balance, Amount = Amount>,
{
	let caller: T::AccountId = account(name, index, SEED);

	T::Currencies::update_balance(ASSET_ID, &caller, 10_000_000_000_000).unwrap();
	T::Currencies::update_balance(HDX, &caller, 10_000_000_000_000).unwrap();

	caller
}

fn initialize_pool<T: Config>(
	caller: T::AccountId,
	asset: AssetId,
	amount: Balance,
	price: Price,
) -> dispatch::DispatchResult {
	xykpool::Pallet::<T>::create_pool(RawOrigin::Signed(caller).into(), HDX, asset, amount, price)?;
	Ok(())
}

benchmarks! {
	swap_currency {
		let maker = funded_account::<T>("maker", 1);
		initialize_pool::<T>(maker, ASSET_ID, 10_000_000_000_000, Price::from(1))?;

		let caller = funded_account::<T>("caller", 2);
		MultiPaymentModule::<T>::set_currency(RawOrigin::Signed(caller.clone()).into(), ASSET_ID)?;

	}: { MultiPaymentModule::<T>::swap_currency(&caller, 1_000_000)? }
	verify{
		assert_eq!(MultiPaymentModule::<T>::get_currency(&caller), Some(ASSET_ID));
		#[cfg(test)]
		assert_eq!(T::Currencies::free_balance(ASSET_ID, &caller), 9999688747087);
	}

	set_currency {
		let maker = funded_account::<T>("maker", 1);
		initialize_pool::<T>(maker, ASSET_ID, 10_000_000_000_000, Price::from(1))?;

		let caller = funded_account::<T>("caller", 123);

		let currency_id: u32 = ASSET_ID;

	}: { MultiPaymentModule::<T>::set_currency(RawOrigin::Signed(caller.clone()).into(), currency_id)? }
	verify{
		assert_eq!(MultiPaymentModule::<T>::get_currency(caller), Some(currency_id));
	}

	add_currency {
		let price = Price::from(10);

	}: { MultiPaymentModule::<T>::add_currency(RawOrigin::Root.into(), 10, price)? }
	verify {
		assert_eq!(MultiPaymentModule::<T>::currencies(10), Some(price));
	}

	remove_currency {
		MultiPaymentModule::<T>::add_currency(RawOrigin::Root.into(), 10, Price::from(2))?;

		assert_eq!(MultiPaymentModule::<T>::currencies(10), Some(Price::from(2)));

	}: { MultiPaymentModule::<T>::remove_currency(RawOrigin::Root.into(), 10)? }
	verify {
		assert_eq!(MultiPaymentModule::<T>::currencies(10), None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{ExtBuilder, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		ExtBuilder::default().base_weight(5).build().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_swap_currency());
			assert_ok!(Pallet::<Test>::test_benchmark_set_currency());
			assert_ok!(Pallet::<Test>::test_benchmark_add_currency());
			assert_ok!(Pallet::<Test>::test_benchmark_remove_currency());
		});
	}
}
