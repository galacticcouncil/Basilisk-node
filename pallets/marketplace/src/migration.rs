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
// limitations under the License..

use crate::{Config, MarketplaceItems, Pallet, RoyaltyOf, MAX_ROYALTY};
use codec::Decode;
use frame_support::{
	codec, log,
	traits::{Get, StorageVersion},
	weights::Weight,
};

/// Royalty amount type is changed from u8 to u16.
pub mod v1 {
	use super::*;
	use sp_arithmetic::traits::Saturating;

	#[derive(Decode)]
	pub struct OldRoyalty<AccountId> {
		pub author: AccountId,
		/// Royalty in percent in range 0-99
		pub royalty: u8,
	}

	pub fn pre_migrate<T: Config>() {
		assert_eq!(StorageVersion::get::<Pallet<T>>(), 0, "Storage version too high.");
	}

	pub fn migrate<T: Config>() -> Weight {
		log::info!(
			target: "runtime::marketplace",
			"Running migration to v1 for Marketplace"
		);

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

		T::DbWeight::get().reads_writes(translated, translated + 1)
	}

	pub fn post_migrate<T: Config>() {
		assert_eq!(StorageVersion::get::<Pallet<T>>(), 1, "Storage version too high.");

		for (_key_1, _key_2, royalty) in MarketplaceItems::<T>::iter() {
			assert!(royalty.royalty < MAX_ROYALTY, "Invalid value.");
		}
	}
}
