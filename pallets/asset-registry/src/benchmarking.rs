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

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

use sp_std::vec;

benchmarks! {
	register{
		let name = vec![1; T::StringLimit::get() as usize];
		let ed = T::Balance::from(1_000_000u32);

		// This makes sure that next asset id is equal to native asset id
		// In such case, one additional operation is performed to skip the id (aka worst case)
		assert_eq!(crate::Pallet::<T>::next_asset_id(), T::AssetId::from(0u8));

	}: _(RawOrigin::Root, name.clone(), AssetType::Token, ed)
	verify {
		let bname = crate::Pallet::<T>::to_bounded_name(name).unwrap();
		assert_eq!(crate::Pallet::<T>::asset_ids(bname), Some(T::AssetId::from(1u8)));
	}

	update{
		let name = b"NAME".to_vec();
		let ed = T::Balance::from(1_000_000u32);
		assert_eq!(crate::Pallet::<T>::next_asset_id(), T::AssetId::from(0u8));
		let _ = crate::Pallet::<T>::register(RawOrigin::Root.into(), name, AssetType::Token, ed);

		let new_name= vec![1; T::StringLimit::get() as usize];

		let asset_id = T::AssetId::from(1u8);

		let new_ed = T::Balance::from(2_000_000u32);

	}: _(RawOrigin::Root, asset_id, new_name.clone(), AssetType::PoolShare(T::AssetId::from(10u8),T::AssetId::from(20u8)), Some(new_ed))
	verify {
		let bname = crate::Pallet::<T>::to_bounded_name(new_name).unwrap();
		assert_eq!(crate::Pallet::<T>::asset_ids(&bname), Some(T::AssetId::from(1u8)));

		let stored = crate::Pallet::<T>::assets(asset_id);

		assert!(stored.is_some());
		let stored = stored.unwrap();

		let expected = AssetDetails{
			asset_type: AssetType::PoolShare(T::AssetId::from(10u8), T::AssetId::from(20u8)),
			locked: false,
			existential_deposit: new_ed,
			name: bname,};

		assert_eq!(stored.asset_type, expected.asset_type);
		assert_eq!(stored.locked, expected.locked);
		assert_eq!(stored.existential_deposit, expected.existential_deposit);
		assert_eq!(stored.name.to_vec(), expected.name.to_vec());
	}

	set_metadata{
		let name = b"NAME".to_vec();
		let ed = T::Balance::from(1_000_000u32);
		assert_eq!(crate::Pallet::<T>::next_asset_id(), T::AssetId::from(0u8));
		let _ = crate::Pallet::<T>::register(RawOrigin::Root.into(), name.clone(), AssetType::Token, ed);

		let asset_id = T::AssetId::from(1u8);

		let max_symbol = vec![1; T::StringLimit::get() as usize];

	}: _(RawOrigin::Root, asset_id, max_symbol.clone(), 10u8)
	verify {
		let bname = crate::Pallet::<T>::to_bounded_name(name).unwrap();
		let bsymbol= crate::Pallet::<T>::to_bounded_name(max_symbol).unwrap();
		assert_eq!(crate::Pallet::<T>::asset_ids(&bname), Some(T::AssetId::from(1u8)));

		let stored = crate::Pallet::<T>::asset_metadata(asset_id);

		assert!(stored.is_some());

		let stored = stored.unwrap();

		let expected =AssetMetadata{
			symbol: bsymbol,
			decimals: 10u8
		};

		assert_eq!(stored.symbol.to_vec(), expected.symbol.to_vec());
		assert_eq!(stored.decimals, expected.decimals);
	}

	set_location{
		let name = b"NAME".to_vec();
		let ed = T::Balance::from(1_000_000u32);
		assert_eq!(crate::Pallet::<T>::next_asset_id(), T::AssetId::from(0u8));
		let _ = crate::Pallet::<T>::register(RawOrigin::Root.into(), name.clone(), AssetType::Token, ed);

		let asset_id = T::AssetId::from(1u8);

	}: _(RawOrigin::Root, asset_id, Default::default())
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
