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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const SEED: u32 = 1;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	caller
}

benchmarks! {
	register{
		let caller = funded_account::<T>("caller", 0);

		let name = vec![1; T::StringLimit::get() as usize];

		// This makes sure that next asset id is equal to native asset id
		// In such case, one additional operation is performed to skip the id (aka worst case)
		assert_eq!(crate::Pallet::<T>::next_asset_id(), T::AssetId::from(0u8));

	}: _(RawOrigin::Signed(caller.clone()), name.clone(), AssetType::Token)
	verify {
		let bname = crate::Pallet::<T>::to_bounded_name(name).unwrap();
		assert_eq!(crate::Pallet::<T>::asset_ids(bname), Some(T::AssetId::from(1u8)));
	}

	update{
		let caller = funded_account::<T>("caller", 0);

		let name = b"NAME".to_vec();
		assert_eq!(crate::Pallet::<T>::next_asset_id(), T::AssetId::from(0u8));
		let _ = crate::Pallet::<T>::register(RawOrigin::Signed(caller.clone()).into(), name.clone(), AssetType::Token);

		let new_name= vec![1; T::StringLimit::get() as usize];

		let asset_id = T::AssetId::from(1u8);

	}: _(RawOrigin::Signed(caller.clone()), asset_id, new_name.clone(), AssetType::PoolShare(T::AssetId::from(10u8),T::AssetId::from(20u8)))
	verify {
		let bname = crate::Pallet::<T>::to_bounded_name(new_name).unwrap();
		assert_eq!(crate::Pallet::<T>::asset_ids(&bname), Some(T::AssetId::from(1u8)));
		assert_eq!(crate::Pallet::<T>::assets(asset_id), Some(AssetDetails{
			asset_type: AssetType::PoolShare(T::AssetId::from(10u8), T::AssetId::from(20u8)),
			locked: false,
			name: bname,
		}));
	}

	set_metadata{
		let caller = funded_account::<T>("caller", 0);

		let name = b"NAME".to_vec();
		assert_eq!(crate::Pallet::<T>::next_asset_id(), T::AssetId::from(0u8));
		let _ = crate::Pallet::<T>::register(RawOrigin::Signed(caller.clone()).into(), name.clone(), AssetType::Token);

		let asset_id = T::AssetId::from(1u8);

		let max_symbol = vec![1; T::StringLimit::get() as usize];

	}: _(RawOrigin::Signed(caller.clone()), asset_id, max_symbol.clone(), 10u8)
	verify {
		let bname = crate::Pallet::<T>::to_bounded_name(name).unwrap();
		let bsymbol= crate::Pallet::<T>::to_bounded_name(max_symbol).unwrap();
		assert_eq!(crate::Pallet::<T>::asset_ids(&bname), Some(T::AssetId::from(1u8)));
		assert_eq!(crate::Pallet::<T>::asset_metadata(asset_id), Some(AssetMetadata{
			symbol: bsymbol,
			decimals: 10u8
		}));
	}

	set_location{
		let caller = funded_account::<T>("caller", 0);

		let name = b"NAME".to_vec();
		assert_eq!(crate::Pallet::<T>::next_asset_id(), T::AssetId::from(0u8));
		let _ = crate::Pallet::<T>::register(RawOrigin::Signed(caller.clone()).into(), name.clone(), AssetType::Token);

		let asset_id = T::AssetId::from(1u8);

	}: _(RawOrigin::Signed(caller.clone()), asset_id, Default::default())
	verify {
		let bname = crate::Pallet::<T>::to_bounded_name(name).unwrap();
		let bsymbol= crate::Pallet::<T>::to_bounded_name(b"SYMBOL".to_vec()).unwrap();
		assert_eq!(crate::Pallet::<T>::asset_ids(&bname), Some(T::AssetId::from(1u8)));
		assert_eq!(crate::Pallet::<T>::locations(asset_id), Some(Default::default()));
		assert_eq!(crate::Pallet::<T>::location_assets(T::AssetNativeLocation::default()), Some(asset_id));
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
			assert_ok!(Pallet::<Test>::test_benchmark_register());
			assert_ok!(Pallet::<Test>::test_benchmark_update());
			assert_ok!(Pallet::<Test>::test_benchmark_set_metadata());
			assert_ok!(Pallet::<Test>::test_benchmark_set_location());
		});
	}
}
