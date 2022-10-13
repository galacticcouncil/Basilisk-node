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
const READ_WEIGHT: u64 = 3;
const WRITE_WEIGHT: u64 = 5;

#[allow(dead_code)]
pub fn init_nft_collection<T: Config>() -> frame_support::weights::Weight {
	let version = StorageVersion::get::<Pallet<T>>();

	if version == 0 {
		if let Some(pallet_account) = <Pallet<T>>::account_id() {
			if let Err(e) = T::NFTHandler::create_typed_collection(
				pallet_account,
				T::NftCollectionId::get(),
				pallet_nft::CollectionType::LiquidityMining,
			) {
				log::error!(
					target: "basilisk:liquidity-mining",
					"Error to create NFT class: {:?}",
					e
				);
			} else {
				StorageVersion::new(STORAGE_VERSION).put::<Pallet<T>>();
			}
		} else {
			log::error!(
				target: "basilisk:liquidity-mining",
				"Error to get pallet account",
			);
		}

		T::DbWeight::get().reads_writes(READ_WEIGHT, WRITE_WEIGHT)
	} else {
		log::warn!(
			target: "basilisk:liquidity-mining",
			"Attempted to apply migration to v{:?} but failed because storage version is {:?}",
			STORAGE_VERSION, version
		);

		Weight::zero()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{mock, mock::Test};
	use std::borrow::Borrow;
	use std::cell::RefCell;

	#[test]
	fn init_nft_class_migration_should_work() {
		sp_io::TestExternalities::default().execute_with(|| {
			let pallet_account = <Pallet<Test>>::account_id().unwrap();

			let weight = init_nft_collection::<Test>();

			assert_that_nft_collecion_is_created(pallet_account);
			assert_eq!(StorageVersion::get::<Pallet<Test>>(), STORAGE_VERSION);
			assert_eq!(
				weight,
				(READ_WEIGHT * mock::INITIAL_READ_WEIGHT) + (WRITE_WEIGHT * mock::INITIAL_WRITE_WEIGHT)
			);
		});
	}

	fn assert_that_nft_collecion_is_created(pallet_account: u128) {
		mock::NFT_CLASS
			.borrow()
			.with(|v| assert_eq!(*v, RefCell::new((mock::LM_NFT_CLASS, pallet_account, pallet_account))));
	}
}
