// This file is part of Basilisk-node.

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
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
// limitations under the License..

use crate::{Config, MarketplaceItems, Pallet, RoyaltyOf, MAX_ROYALTY};
use codec::{Encode, Decode};
use frame_support::{
	codec, log,
	traits::{Get, PalletInfoAccess, StorageVersion},
	weights::Weight,
};

/// MarketplaceInstances storage item is renamed to MarketplaceItems and the hashing algorithm is changed from TwoX to Blake2.
pub mod v1 {
	use super::*;
	use frame_support::{storage_alias, StoragePrefixedMap, Twox64Concat};
	use sp_io::hashing::twox_128;

	/// rename the storage and transform the royalty amount type from u8 to u16
	pub mod move_and_transform_old_storage {
		use frame_support::migration;
		use super::*;

		#[derive(Encode, Decode)]
		pub struct OldRoyalty<AccountId> {
			pub author: AccountId,
			/// Royalty in percent in range 0-99
			pub royalty: u8,
		}

		#[storage_alias]
		type MarketplaceInstances<T: Config> = StorageDoubleMap<
			Pallet<T>,
			Twox64Concat,
			<T as pallet_nft::Config>::NftCollectionId,
			Twox64Concat,
			<T as pallet_nft::Config>::NftItemId,
			OldRoyalty<<T as frame_system::Config>::AccountId>,
		>;

		pub fn pre_migrate<T: Config>() {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "Storage version too high.");

			// Assert that `MarketplaceItems` storage is empty
			let pallet_name = <Pallet<T> as PalletInfoAccess>::name().as_bytes();
			let key = [&twox_128(pallet_name), &twox_128(MarketplaceItems::<T>::storage_prefix())[..]].concat();
			let key_iter =
				frame_support::storage::KeyPrefixIterator::new(key.to_vec(), key.to_vec(), |_| Ok(()));
			assert_eq!(key_iter.count(), 0, "MarketplaceItems storage is not empty");

			log::info!(
				target: "runtime::marketplace",
				"Marketplace migration: PRE checks successful!"
			);

		}

		pub fn migrate<T: Config>() -> Weight {
			log::info!(
				target: "runtime::marketplace",
				"Running migration to v1 for Marketplace"
			);

			let mut count: u64 = 0;
			for (collection_id, item_id, royalty) in MarketplaceInstances::<T>::iter() {
				MarketplaceItems::<T>::insert(&collection_id, &item_id, RoyaltyOf::<T> {
					author: royalty.author,
					// multiply the value by 100 to transform percentage to basis points
					royalty: Into::<u16>::into(royalty.royalty)
						.checked_mul(100)
						.unwrap_or(MAX_ROYALTY - 1),
				});
				count += 1;
			}

			let res = migration::clear_storage_prefix(
				<Pallet<T>>::name().as_bytes(),
				MarketplaceInstances::<T>::storage_prefix(),
				b"",
				None,
				None,
			);
			if res.maybe_cursor.is_some() {
				log::info!(
					target: "runtime::marketplace",
					"Not all storage items has been removed"
				);
			}

			StorageVersion::new(1).put::<Pallet<T>>();

			let reads = count.checked_add(1).unwrap_or(Weight::MAX);
			let writes = count.checked_add(2).unwrap_or(Weight::MAX);

			T::DbWeight::get().reads_writes(reads, writes)
		}

		pub fn post_migrate<T: Config>() {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "Unexpected storage version.");

			// Assert that no `MarketplaceInstances` storage remains at the old prefix.
			let pallet_name = <Pallet<T> as PalletInfoAccess>::name().as_bytes();
			let old_storage_prefix = MarketplaceInstances::<T>::storage_prefix();
			let old_key = [&twox_128(pallet_name), &twox_128(old_storage_prefix)[..]].concat();
			let old_key_iter =
				frame_support::storage::KeyPrefixIterator::new(old_key.to_vec(), old_key.to_vec(), |_| Ok(()));
			assert_eq!(old_key_iter.count(), 0, "MarketplaceInstances storage is not empty");

			for (collection_id, item_id, royalty) in MarketplaceItems::<T>::iter() {
				assert!(royalty.royalty < MAX_ROYALTY, "Invalid value for collection {:?} and item {:?}.", collection_id, item_id);
			}

			log::info!(
				target: "runtime::marketplace",
				"Marketplace migration: POST checks successful!"
			);
		}
	}

	/// rename the storage without transforming the royalty amount type.
	pub mod move_and_rehash_old_storage {
		use super::*;
		use frame_support::migration;

		#[storage_alias]
		type MarketplaceInstances<T: Config> = StorageDoubleMap<
			Pallet<T>,
			Twox64Concat,
			<T as pallet_nft::Config>::NftCollectionId,
			Twox64Concat,
			<T as pallet_nft::Config>::NftItemId,
			RoyaltyOf<T>,
		>;

		pub fn pre_migrate<T: Config>() {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "Storage version too high.");

			// Assert that `MarketplaceItems` storage is empty
			let pallet_name = <Pallet<T> as PalletInfoAccess>::name().as_bytes();
			let key = [&twox_128(pallet_name), &twox_128(MarketplaceItems::<T>::storage_prefix())[..]].concat();
			let key_iter =
				frame_support::storage::KeyPrefixIterator::new(key.to_vec(), key.to_vec(), |_| Ok(()));
			assert_eq!(key_iter.count(), 0, "MarketplaceItems storage is not empty");

			log::info!(
				target: "runtime::marketplace",
				"Marketplace migration: PRE checks successful!"
			);
		}

		pub fn migrate<T: Config>() -> Weight {
			log::info!(
				target: "runtime::marketplace",
				"Running migration to v1 for Marketplace"
			);

			let mut count: u64 = 0;
			for (collection_id, item_id, royalty) in MarketplaceInstances::<T>::iter() {
				MarketplaceItems::<T>::insert(&collection_id, &item_id, royalty);
				count += 1;
			}

			let res = migration::clear_storage_prefix(
				<Pallet<T>>::name().as_bytes(),
				MarketplaceInstances::<T>::storage_prefix(),
				b"",
				None,
				None,
			);
			if res.maybe_cursor.is_some() {
				log::info!(
					target: "runtime::marketplace",
					"Not all storage items has been removed"
				);
			}

			StorageVersion::new(1).put::<Pallet<T>>();

			let reads = count.checked_add(1).unwrap_or(Weight::MAX);
			let writes = count.checked_add(2).unwrap_or(Weight::MAX);

			T::DbWeight::get().reads_writes(reads, writes)
		}

		pub fn post_migrate<T: Config>() {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "Unexpected storage version.");

			// Assert that no `MarketplaceInstances` storage remains at the old prefix.
			let pallet_name = <Pallet<T> as PalletInfoAccess>::name().as_bytes();
			let old_storage_prefix = MarketplaceInstances::<T>::storage_prefix();
			let old_key = [&twox_128(pallet_name), &twox_128(old_storage_prefix)[..]].concat();
			let old_key_iter =
				frame_support::storage::KeyPrefixIterator::new(old_key.to_vec(), old_key.to_vec(), |_| Ok(()));
			assert_eq!(old_key_iter.count(), 0, "MarketplaceInstances storage is not empty");

			for (collection_id, item_id, royalty) in MarketplaceItems::<T>::iter() {
				assert!(royalty.royalty < MAX_ROYALTY, "Invalid value for collection {:?} and item {:?}.", collection_id, item_id);
			}
			
			log::info!(
				target: "runtime::marketplace",
				"Marketplace migration: POST checks successful!"
			);
		}
	}
}
