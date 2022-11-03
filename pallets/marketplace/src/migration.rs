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
use codec::{Decode, Encode};
use frame_support::{
	codec, log,
	traits::{Get, PalletInfoAccess, StorageVersion},
	weights::Weight,
};

/// MarketplaceInstances storage item is renamed to MarketplaceItems and the hashing algorithm is changed from TwoX to Blake2.
/// The royalty type is changed from u8 to u16 on testnet.
pub mod v2 {
	use super::*;
	use frame_support::{migration, storage_alias, StoragePrefixedMap, Twox64Concat};
	use sp_io::hashing::twox_128;

	// Storage type with old hash, used on mainnet
	pub mod v0_storage {
		use super::*;

		#[storage_alias]
		pub type MarketplaceInstances<T: Config> = StorageDoubleMap<
			Pallet<T>,
			Twox64Concat,
			<T as pallet_nft::Config>::NftCollectionId,
			Twox64Concat,
			<T as pallet_nft::Config>::NftItemId,
			RoyaltyOf<T>,
		>;
	}

	// Storage type with old royalty type and hash, used on testnet
	pub mod v1_storage {
		use super::*;

		pub const MAX_ROYALTY: u8 = 100;

		#[derive(Encode, Decode)]
		pub struct OldRoyalty<AccountId> {
			pub author: AccountId,
			/// Royalty in percent in range 0-99
			pub royalty: u8,
		}

		#[storage_alias]
		pub type MarketplaceInstances<T: Config> = StorageDoubleMap<
			Pallet<T>,
			Twox64Concat,
			<T as pallet_nft::Config>::NftCollectionId,
			Twox64Concat,
			<T as pallet_nft::Config>::NftItemId,
			OldRoyalty<<T as frame_system::Config>::AccountId>,
		>;
	}

	pub fn pre_migrate<T: Config>() {
		log::info!(
			target: "runtime::marketplace",
			"Marketplace migration: PRE checks start"
		);

		assert!(StorageVersion::get::<Pallet<T>>() < 2, "Storage version too high.");

		// Assert that `MarketplaceItems` storage is empty
		let pallet_name = <Pallet<T> as PalletInfoAccess>::name().as_bytes();
		let key = [
			&twox_128(pallet_name),
			&twox_128(MarketplaceItems::<T>::storage_prefix())[..],
		]
		.concat();
		let key_iter = frame_support::storage::KeyPrefixIterator::new(key.to_vec(), key.to_vec(), |_| Ok(()));
		assert_eq!(key_iter.count(), 0, "MarketplaceItems storage is not empty");

		log::info!(
			target: "runtime::marketplace",
			"Marketplace migration: PRE checks successful"
		);
	}

	pub fn migrate<T: Config>() -> Weight {
		let storage_version = StorageVersion::get::<Pallet<T>>();

		log::info!(
			target: "runtime::marketplace",
			"Running migration to v2 for Marketplace with storage version {:?}",
			storage_version
		);

		let num_of_instances = if storage_version == 0 {
			v0_storage::MarketplaceInstances::<T>::iter().count()
		} else {
			v1_storage::MarketplaceInstances::<T>::iter().count()
		};

		let mut count: u64 = 0;
		if storage_version == 0 {
			for (collection_id, item_id, royalty) in v0_storage::MarketplaceInstances::<T>::iter() {
				MarketplaceItems::<T>::insert(&collection_id, &item_id, royalty);
				count += 1;
			}
		} else {
			for (collection_id, item_id, royalty) in v1_storage::MarketplaceInstances::<T>::iter() {
				MarketplaceItems::<T>::insert(
					&collection_id,
					&item_id,
					RoyaltyOf::<T> {
						author: royalty.author,
						royalty: Into::<u16>::into(royalty.royalty)
							// multiply the value by 100 to transform percentage to basis points
							.checked_mul(100)
							.unwrap_or(MAX_ROYALTY - 1),
					},
				);
				count += 1;
			}
		};

		let storage_prefix = if storage_version == 0 {
			v0_storage::MarketplaceInstances::<T>::storage_prefix()
		} else {
			v1_storage::MarketplaceInstances::<T>::storage_prefix()
		};

		let res = migration::clear_storage_prefix(<Pallet<T>>::name().as_bytes(), storage_prefix, b"", None, None);
		if res.maybe_cursor.is_some() {
			log::info!(
				target: "runtime::marketplace",
				"Not all storage items has been removed"
			);
		}

		let num_of_items = MarketplaceItems::<T>::iter().count();
		if num_of_instances != num_of_items {
			log::info!(
				target: "runtime::marketplace",
				"Migration to v1 for Marketplace wasn't successful! Not all data was migrated."
			);
		}

		StorageVersion::new(2).put::<Pallet<T>>();

		let reads = count
			.checked_mul(3)
			.unwrap_or(u64::MAX)
			.checked_add(2)
			.unwrap_or(u64::MAX);
		let writes = count.checked_add(2).unwrap_or(u64::MAX);

		log::info!(
			target: "runtime::marketplace",
			"Migration to v2 for Marketplace was successful"
		);

		T::DbWeight::get().reads_writes(reads, writes)
	}

	pub fn post_migrate<T: Config>() {
		log::info!(
			target: "runtime::marketplace",
			"Marketplace migration: POST checks start"
		);

		let storage_version = StorageVersion::get::<Pallet<T>>();
		assert_eq!(storage_version, 2, "Unexpected storage version.");

		// Assert that no `MarketplaceInstances` storage remains at the old prefix.
		let pallet_name = <Pallet<T> as PalletInfoAccess>::name().as_bytes();
		let old_storage_prefix = if storage_version == 0 {
			v0_storage::MarketplaceInstances::<T>::storage_prefix()
		} else {
			v1_storage::MarketplaceInstances::<T>::storage_prefix()
		};

		let old_key = [&twox_128(pallet_name), &twox_128(old_storage_prefix)[..]].concat();
		let old_key_iter =
			frame_support::storage::KeyPrefixIterator::new(old_key.to_vec(), old_key.to_vec(), |_| Ok(()));
		assert_eq!(old_key_iter.count(), 0, "MarketplaceInstances storage is not empty");

		for (collection_id, item_id, royalty) in MarketplaceItems::<T>::iter() {
			assert!(
				royalty.royalty < MAX_ROYALTY,
				"Invalid value for CollectionId {:?} and ItemId {:?}.",
				collection_id,
				item_id
			);
		}

		log::info!(
			target: "runtime::marketplace",
			"Marketplace migration: POST checks successful"
		);
	}
}
