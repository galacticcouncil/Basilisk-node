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
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::sp_runtime::traits::Zero;
use primitives::{
	asset::AssetPair,
	traits::{AMMHandlers, AMMTransfer},
	AssetId, Balance,
};
use sp_std::convert::TryInto;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;
pub use types::*;

pub mod weights;
use weights::WeightInfo;

mod benchmarking;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub type AssetPairId = Vec<u8>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for the extrinsics.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Calculation error occurred while calculating average price
		PriceComputationError,

		/// An unexpected overflow occurred
		UpdateDataOverflow,
	}

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::storage]
	#[pallet::getter(fn price_buffer)]
	pub type PriceBuffer<T: Config> = StorageMap<_, Blake2_128Concat, AssetPairId, Vec<PriceEntry>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_data_ten)]
	pub type PriceDataTen<T: Config> = StorageValue<_, Vec<(AssetPairId, BucketQueue)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_data_hundred)]
	pub type PriceDataHundred<T: Config> = StorageMap<_, Blake2_128Concat, AssetPairId, BucketQueue, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_data_thousand)]
	pub type PriceDataThousand<T: Config> = StorageMap<_, Blake2_128Concat, AssetPairId, BucketQueue, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			let (num_of_processed_buckets, num_of_processed_trades) = Self::update_data().unwrap_or_else(|_| (0, 0));

			PriceBuffer::<T>::remove_all(None);

			T::WeightInfo::on_initialize_multiple_entries_multiple_tokens(
				num_of_processed_buckets,
				num_of_processed_trades,
			) // TODO: rebenchmark
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn on_create_pool(asset_pair: AssetPair) {
		let data = PriceDataTen::<T>::get();
		if !data.iter().any(|bucket_tuple| bucket_tuple.0 == asset_pair.name()) {
			PriceDataTen::<T>::append((asset_pair.name(), BucketQueue::default()));
		}
	}

	pub fn on_trade(asset_pair: AssetPair, price_entry: PriceEntry) {
		PriceBuffer::<T>::append(asset_pair.name(), price_entry);
	}

	fn update_data() -> Result<(u32, u32), DispatchError> {
		let mut num_of_processed_buckets = 0u32;
		let mut num_of_processed_trades = 0u32;

		PriceDataTen::<T>::mutate(|data_ten| -> DispatchResult {
			for (asset_pair_id, data) in data_ten.iter_mut() {
				let maybe_price = <PriceBuffer<T>>::try_get(asset_pair_id);
				let result = if let Ok(prices) = maybe_price {
					num_of_processed_buckets
						.checked_add(1)
						.ok_or(Error::<T>::UpdateDataOverflow)?;
					num_of_processed_trades
						.checked_add(prices.len().try_into().map_err(|_| Error::<T>::UpdateDataOverflow)?)
						.ok_or(Error::<T>::UpdateDataOverflow)?;
					PriceInfo::calculate_price_info(prices.as_slice()).ok_or(Error::<T>::PriceComputationError)?
				} else {
					data.get_last()
				};

				data.update_last(result);
			}

			Ok(())
		})?;

		let now = <frame_system::Pallet<T>>::block_number();
		if now.is_zero() {
			return Ok((num_of_processed_buckets, num_of_processed_trades));
		} // TODO: delete me. It is here just to make testing easier.

		if (now % T::BlockNumber::from(BUCKET_SIZE)) == T::BlockNumber::from(BUCKET_SIZE - 1) {
			for element_from_ten in PriceDataTen::<T>::get().iter() {
				PriceDataHundred::<T>::mutate(element_from_ten.0.clone(), |data| -> DispatchResult {
					data.update_last(element_from_ten.1.calculate_average());
					Ok(())
				})?;
			}
		}

		if (now % T::BlockNumber::from(BUCKET_SIZE.pow(2))) == T::BlockNumber::from(BUCKET_SIZE.pow(2) - 1) {
			for element_from_hundred in PriceDataHundred::<T>::iter() {
				PriceDataThousand::<T>::mutate(element_from_hundred.0.clone(), |data| -> DispatchResult {
					data.update_last(element_from_hundred.1.calculate_average());
					Ok(())
				})?;
			}
		}

		Ok((num_of_processed_buckets, num_of_processed_trades))
	}
}

pub struct PriceOracleHandler<T>(PhantomData<T>);
impl<T: Config> AMMHandlers<T::AccountId, AssetId, AssetPair, Balance> for PriceOracleHandler<T> {
	fn on_create_pool(asset_pair: AssetPair) {
		Pallet::<T>::on_create_pool(asset_pair);
	}

	fn on_trade(amm_transfer: &AMMTransfer<T::AccountId, AssetId, AssetPair, Balance>, liq_amount: Balance) {
		let (price, amount) = if let Some(price_tuple) = amm_transfer.normalize_price() {
			price_tuple
		} else {
			return;
		};

		// we assume that zero prices are not valid
		// zero values are ignored and not added to the queue
		if price.is_zero() || amount.is_zero() || liq_amount.is_zero() {
			return;
		}

		let price_entry = PriceEntry {
			price,
			trade_amount: amount,
			liquidity_amount: liq_amount,
		};

		Pallet::<T>::on_trade(amm_transfer.assets, price_entry);
	}
}
