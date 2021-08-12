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
	register{
		let caller = funded_account::<T>("caller", 0);

		let name = b"NAME".to_vec();

	}: _(RawOrigin::Signed(caller.clone()), name, AssetType::Token)
	verify {
		assert!(crate::Pallet::<Test>::asset_ids(&name).is_some());
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
			assert_ok!(test_benchmark_register::<Test>());
		});
	}
}
