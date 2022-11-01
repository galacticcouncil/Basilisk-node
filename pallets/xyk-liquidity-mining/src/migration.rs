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
use hydradx_traits::nft::CreateTypedCollection;

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

		//create nft collection
		match T::NFTHandler::create_typed_collection(
			pallet_account,
			T::NftCollectionId::get(),
			pallet_nft::CollectionType::LiquidityMining,
		) {
			Ok(()) => {
				//NOTE: create_typed_collection is not benchamrked but it's calling same
				//funtion as create_collection so weight should be the same.
				weight = weight.saturating_add(
					<pallet_nft::weights::BasiliskWeight<T> as pallet_nft::weights::WeightInfo>::create_collection(),
				);
			}
			Err(e) => {
				log::error!(
					target: "runtime: xyk-liquidity-mining",
					"Error to create NFT collection: {:?}",
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
	use crate::tests::{mock, mock::Test};
	use std::borrow::Borrow;
	use std::cell::RefCell;

	#[test]
	fn v1_migration() {
		sp_io::TestExternalities::default().execute_with(|| {
			let pallet_account = <Pallet<Test>>::account_id();

			let weight = v1::migrate::<Test>();

			assert_that_nft_collecion_is_created(pallet_account);

			assert_eq!(StorageVersion::get::<Pallet<Test>>(), STORAGE_VERSION);

			let storage_version_weight: Weight = Weight::from_ref_time(2);
			let duster_weight: Weight = Weight::from_ref_time(2);
			let expected_weight =
				<pallet_nft::weights::BasiliskWeight<Test> as pallet_nft::weights::WeightInfo>::create_collection()
					.saturating_add(duster_weight)
					.saturating_add(storage_version_weight);

			assert_eq!(weight, expected_weight);
		});
	}

	fn assert_that_nft_collecion_is_created(pallet_account: u128) {
		mock::NFT_COLLECTION.borrow().with(|v| {
			assert_eq!(
				*v,
				RefCell::new((mock::LM_NFT_COLLECTION, pallet_account, pallet_account))
			)
		});
	}
}
