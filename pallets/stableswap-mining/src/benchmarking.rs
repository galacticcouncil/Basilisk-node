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
use hydradx_traits::registry;
//use hydradx_traits::registry::Registry;

const ONE: Balance = 1_000_000_000_000;
const BSX: AssetId = 1;
const DAI: AssetId = 2;

const INITIAL_BALANCE: Balance = 1_000_000 * ONE;

type AssetRegistryOf<T> = <T as pallet_stableswap::Config>::AssetRegistry;

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
		Permill::from_percent(20),
		100,
		FixedU128::from(14_u128),
	)
}

fn initialize_stableswap_pool<T: Config>(owner: T::AccountId) -> DispatchResult {
    let asset_a = AssetRegistryOf::<T>::create_asset(&b"one".to_vec(), 1u128)?;
    
    Ok(()) 
}

benchmarks! {
	 where_clause {  where 
        T::AssetId: From<u32> + Into<u32>,
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
	   let yield_per_period = Permill::from_percent(20);
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
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, crate::tests::mock::ExtBuilder::default().build(), super::Test);
}
