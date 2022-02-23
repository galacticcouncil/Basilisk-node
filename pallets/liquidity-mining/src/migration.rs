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

use super::*;
use frame_support::traits::GetStorageVersion;
use primitives::nft::ClassType;

#[allow(dead_code)]
pub fn init_nft_class<T: Config>() -> frame_support::weights::Weight {
	let version = <Pallet<T> as GetStorageVersion>::current_storage_version();

	if version == 0 {
		let pallet_account = <Pallet<T>>::account_id();

		pallet_nft::Pallet::<T>::do_create_class(
			pallet_account,
			T::NftClass::get(),
			ClassType::LiquidityMining,
			vec![].try_into().unwrap(),
		)
		.unwrap();

		//TODO: this is not correct weight, set this proper values
		T::DbWeight::get().reads_writes(2, 3)
	} else {
		0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;
	use frame_support::assert_noop;

	#[test]
	fn init_nft_class_migration_should_work() {
		sp_io::TestExternalities::default().execute_with(|| {
			init_nft_class::<Test>();

			let pallet_account = <Pallet<Test>>::account_id();

			assert_noop!(
				pallet_nft::Pallet::<Test>::do_create_class(
					pallet_account,
					mock::LIQ_MINING_NFT_CLASS,
					ClassType::LiquidityMining,
					vec![].try_into().unwrap(),
				),
				pallet_uniques::Error::<Test>::InUse
			);
		});
	}
}
