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
use codec::Decode;
use frame_support::{
	codec, log,
	traits::{Get, PalletInfoAccess, StorageVersion},
	weights::Weight,
};

/// Royalty amount type is changed from u8 to u16.
pub mod v1 {
	use super::*;
	use frame_support::{
		migration::move_prefix,
		storage::{storage_prefix, unhashed},
		storage_alias, StoragePrefixedMap, Twox64Concat,
	};
	use sp_arithmetic::traits::Saturating;
	use sp_io::hashing::twox_128;

	#[derive(Decode)]
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
		RoyaltyOf<T>,
	>;

	// rename the storage and transform the revenue type
	pub mod move_and_transform_old_storage {
		use super::*;

		pub fn pre_migrate<T: Config>() {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "Storage version too high.");

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

			// move MarketplaceInstances to MarketplaceItems
			let pallet_name = <Pallet<T> as PalletInfoAccess>::name().as_bytes();
			let new_storage_prefix = storage_prefix(pallet_name, MarketplaceItems::<T>::storage_prefix());
			let old_storage_prefix = storage_prefix(pallet_name, MarketplaceInstances::<T>::storage_prefix());

			move_prefix(&old_storage_prefix, &new_storage_prefix);
			if let Some(value) = unhashed::get_raw(&old_storage_prefix) {
				unhashed::put_raw(&new_storage_prefix, &value);
				unhashed::kill(&old_storage_prefix);
			}

			// change Royalty type
			let mut translated = 0u64;

			<MarketplaceItems<T>>::translate(|_key_1, _key_2, old: OldRoyalty<T::AccountId>| {
				translated.saturating_inc();
				Some(RoyaltyOf::<T> {
					author: old.author,
					// multiply the value by 100 to transform percentage to basis points
					royalty: Into::<u16>::into(old.royalty)
						.checked_mul(100)
						.unwrap_or(MAX_ROYALTY - 1),
				})
			});

			StorageVersion::new(1).put::<Pallet<T>>();

			let reads = translated
				.checked_mul(2)
				.and_then(|v| v.checked_add(3))
				.unwrap_or(Weight::MAX);
			let writes = reads; // the number of writes is the same as the number of reads

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
			assert_eq!(old_key_iter.count(), 0);

			for (_key_1, _key_2, royalty) in MarketplaceItems::<T>::iter() {
				assert!(royalty.royalty < MAX_ROYALTY, "Invalid value.");
			}

			log::info!(
				target: "runtime::marketplace",
				"Marketplace migration: POST checks successful!"
			);
		}
	}

	// rename the storage without transforming the revenue type
	pub mod move_old_storage {
		use super::*;
		use core::convert::TryInto;

		pub fn pre_migrate<T: Config>() {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "Storage version too high.");

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

			// move MarketplaceInstances to MarketplaceItems
			let pallet_name = <Pallet<T> as PalletInfoAccess>::name().as_bytes();
			let new_storage_prefix = storage_prefix(pallet_name, MarketplaceItems::<T>::storage_prefix());
			let old_storage_prefix = storage_prefix(pallet_name, MarketplaceInstances::<T>::storage_prefix());

			// If the number of items overflows the max weight, return the max weight.
			// Make sure this won't happen by running try-runtime command before executing the migration.
			let num_of_instances = MarketplaceItems::<T>::iter().count().try_into().unwrap_or(Weight::MAX);

			move_prefix(&old_storage_prefix, &new_storage_prefix);
			if let Some(value) = unhashed::get_raw(&old_storage_prefix) {
				unhashed::put_raw(&new_storage_prefix, &value);
				unhashed::kill(&old_storage_prefix);
			}

			StorageVersion::new(1).put::<Pallet<T>>();

			let reads = num_of_instances
				.checked_mul(2)
				.and_then(|v| v.checked_add(3))
				.unwrap_or(Weight::MAX);
			let writes = num_of_instances.checked_add(3).unwrap_or(Weight::MAX);

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
			assert_eq!(old_key_iter.count(), 0);

			log::info!(
				target: "runtime::marketplace",
				"Marketplace migration: POST checks successful!"
			);
		}
	}
}
