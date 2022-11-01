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
use frame_support::traits::StorageVersion;

const STORAGE_VERSION: u16 = 1;

pub mod v1 {
	use super::*;
	use hydradx_traits::pools::DustRemovalAccountWhitelist;

	pub fn pre_migrate<T: Config>() {
		assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "Storage version too high.");

		log::info!(
			target: "runtime::xyk-liquidity-mining",
			"xyk-liquidity-mining migration: PRE checks successful!"
		);
	}

	#[allow(dead_code)]
	pub fn migrate<T: Config>() -> Weight {
		//offset for storage version update
		let mut weight: Weight = T::DbWeight::get().reads(1).saturating_add(T::DbWeight::get().writes(1));

		let pallet_account = <Pallet<T>>::account_id();

		//add to non-dustable whitelist
		match T::NonDustableWhitelistHandler::add_account(&pallet_account) {
			Ok(()) => {
				weight = weight
					.saturating_add(T::DbWeight::get().reads(1))
					.saturating_add(T::DbWeight::get().writes(1))
			}
			Err(e) => {
				log::error!(
					target: "runtime: xyk-liquidity-mining",
					"Error to add pallet account to non dustable whitelist: {:?}",
					e
				);
			}
		}

		StorageVersion::new(STORAGE_VERSION).put::<Pallet<T>>();
		weight
	}

	pub fn post_migrate<T: Config>() {
		assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "Unexpected storage version.");

		log::info!(
			target: "runtime::xyk-liquidity-mining",
			"xyk-liquidity-mining migration: POST checks successful!"
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::mock::Test;

	#[test]
	fn v1_migration() {
		sp_io::TestExternalities::default().execute_with(|| {
			let weight = v1::migrate::<Test>();

			assert_eq!(StorageVersion::get::<Pallet<Test>>(), STORAGE_VERSION);

			let storage_version_weight: Weight = 2;
			let duster_weight: Weight = 2;
			let expected_weight = duster_weight.saturating_add(storage_version_weight);

			assert_eq!(weight, expected_weight);
		});
	}
}
