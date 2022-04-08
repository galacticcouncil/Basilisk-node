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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;

use frame_support::traits::Get;

use orml_traits::MultiCurrency;
use sp_runtime::RuntimeAppPublic;
use sp_std::vec::Vec;
use frame_support::weights::DispatchClass;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
	use frame_system::pallet_prelude::BlockNumberFor;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Balance type
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;

		/// Currency for transfers
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = Self::CurrencyId, Balance = Self::Balance>;

		/// Reward amount per one collator.
		#[pallet::constant]
		type RewardPerCollator: Get<Self::Balance>;

		/// Reward Asset Id
		#[pallet::constant]
		type RewardCurrencyId: Get<Self::CurrencyId>;

		/// List of collator which will not be rewarded.
		type NotRewardedCollators: Get<Vec<Self::AccountId>>;

		/// The identifier type for an authority.
		type AuthorityId: Member + Parameter + RuntimeAppPublic + MaybeSerializeDeserialize + MaxEncodedLen;
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Account dusted.
		CollatorRewarded {
			who: T::AccountId,
			amount: T::Balance,
			reward_currency: T::CurrencyId,
		},
	}
}

use frame_support::traits::OneSessionHandler;

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::AuthorityId;
}

impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
	type Key = T::AuthorityId;

	fn on_genesis_session<'a, I: 'a>(_collators: I) {
	}

	fn on_new_session<'a, I: 'a>(_changed: bool, collators: I, _queued_validators: I)
	where
		I: Iterator<Item = (&'a T::AccountId, T::AuthorityId)>,
	{

		for (collator, _) in collators {
			if !T::NotRewardedCollators::get().contains(collator) {
				let result = T::Currency::deposit(T::RewardCurrencyId::get(), collator, T::RewardPerCollator::get());
				if result.is_err() {
					log::warn!("Error reward collators: {:?}", result);
					continue;
				}

				Self::deposit_event(Event::CollatorRewarded {
					who: collator.clone(),
					amount: T::RewardPerCollator::get(),
					reward_currency: T::RewardCurrencyId::get(),
				});

				frame_system::Pallet::<T>::register_extra_weight_unchecked(
					20_000,
					DispatchClass::Mandatory,
				);
			}
		}
	}

	fn on_disabled(_i: u32) {
	}
}
