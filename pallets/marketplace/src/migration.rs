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

use crate::{Config, MarketplaceInstances, Pallet, RoyaltyOf, MAX_ROYALTY};
use codec::Decode;
use frame_support::{
	codec,
	traits::{Get, StorageVersion},
	weights::Weight,
};

/// Wrapper for all migrations of this pallet, based on `StorageVersion`.
pub fn migrate<T: Config>() -> Weight {
	let version = StorageVersion::get::<Pallet<T>>();
	let mut weight: Weight = 0;

	if version < 1 {
		weight = weight.saturating_add(v1::migrate::<T>());
		StorageVersion::new(1).put::<Pallet<T>>();
	}

	weight
}

/// Royalty amount type is changed from u8 to u16.
mod v1 {
	use super::*;

	#[derive(Decode)]
	pub struct OldRoyalty<AccountId> {
		/// The user account which receives the royalty
		pub author: AccountId,
		/// Royalty in percent in range 0-99
		pub royalty: u8,
	}

	pub fn migrate<T: Config>() -> Weight {
		let mut weight: Weight = 0;

		<MarketplaceInstances<T>>::translate(|_key_1, _key_2, old: OldRoyalty<T::AccountId>| {
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
			Some(RoyaltyOf::<T> {
				author: old.author,
				// multiply the value by 100 to transform percentage to basis points
				royalty: Into::<u16>::into(old.royalty)
					.checked_mul(100)
					.unwrap_or(MAX_ROYALTY - 1),
			})
		});

		weight
	}
}
