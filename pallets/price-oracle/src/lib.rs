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

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::sp_runtime::traits::{CheckedDiv, Zero};
use frame_support::sp_runtime::RuntimeDebug;
use frame_support::traits::Get;
use primitives::{AssetId, Balance, Price, asset::AssetPair, traits::{AMMTransfer, AMMHandlers}};
use sp_std::iter::Sum;
use sp_std::vec;
use sp_std::ops::{Add, Index, IndexMut, Mul};
use sp_std::prelude::*;
use sp_std::marker::PhantomData;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// mod benchmarking;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct PriceEntry {
	pub price: Price,
	pub amount: Balance,
	pub liq_amount: Balance,
}

impl Add for PriceEntry {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		Self {
			price: self.price.add(other.price),
			amount: self.amount.add(other.amount),
			liq_amount: self.liq_amount.add(other.liq_amount),
		}
	}
}

impl Zero for PriceEntry {
	fn zero() -> Self {
		Self {
			price: Price::zero(),
			amount: Balance::zero(),
			liq_amount: Balance::zero(),
		}
	}

	fn is_zero(&self) -> bool {
		self == &PriceEntry::zero()
	}
}

impl Add for &PriceEntry {
	type Output = PriceEntry;
	fn add(self, other: Self) -> Self::Output {
		PriceEntry {
			price: self.price.add(other.price),
			amount: self.amount.add(other.amount),
			liq_amount: self.liq_amount.add(other.liq_amount),
		}
	}
}

impl<'a> Sum<&'a Self> for PriceEntry {
	fn sum<I>(iter: I) -> Self
	where
		I: Iterator<Item = &'a Self>,
	{
		iter.fold(
			PriceEntry {
				price: Price::zero(),
				amount: Balance::zero(),
				liq_amount: Balance::zero(),
			},
			|a, b| &a + b,
		)
	}
}

pub const BUCKET_SIZE: u32 = 10;

pub type Bucket = [PriceInfo; BUCKET_SIZE as usize];

pub trait PriceInfoCalculation<PriceInfo> {
	fn calculate_price_info(entries: &[PriceEntry]) -> Option<PriceInfo>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub struct PriceInfo {
	avg_price: Price,
	volume: Balance,
}

impl Default for PriceInfo {
	fn default() -> Self {
		Self {
			avg_price: Price::zero(),
			volume: Zero::zero(),
		}
	}
}

impl Add for &PriceInfo {
	type Output = PriceInfo;
	fn add(self, other: Self) -> Self::Output {
		PriceInfo {
			avg_price: self.avg_price.add(other.avg_price),
			volume: self.volume.add(other.volume),
		}
	}
}

impl<'a> Sum<&'a Self> for PriceInfo {
	fn sum<I>(iter: I) -> Self
	where
		I: Iterator<Item = &'a Self>,
	{
		iter.fold(
			PriceInfo {
				avg_price: Price::zero(),
				volume: Balance::zero(),
			},
			|a, b| &a + b,
		)
	}
}

impl PriceInfoCalculation<PriceInfo> for PriceInfo {
	fn calculate_price_info(entries: &[PriceEntry]) -> Option<PriceInfo> {
		let intermediate_result: Vec<PriceEntry> = entries
			.iter()
			.map(|x| PriceEntry {
				price: x.price.mul(Price::from(x.liq_amount)),
				amount: x.amount,
				liq_amount: x.liq_amount,
			})
			.collect();

		let sum = intermediate_result.iter().sum::<PriceEntry>();
		let weighted_avg_price = sum.price.checked_div(&Price::from(sum.liq_amount as u128))?;
		Some(PriceInfo {
			avg_price: weighted_avg_price,
			volume: sum.amount,
		})
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub struct BucketQueue {
	bucket: Bucket,
	last: u32,
}

impl BucketQueue {
	const BUCKET_SIZE: u32 = BUCKET_SIZE;
}

impl Default for BucketQueue {
	fn default() -> Self {
		Self {
			bucket: Bucket::default(),
			last: Self::BUCKET_SIZE - 1,
		}
	}
}

pub trait BucketQueueT {
	fn update_last(&mut self, price_info: PriceInfo);
	fn get_last(&self) -> PriceInfo;
	fn calculate_average(&self) -> Option<PriceInfo>;
}

impl BucketQueueT for BucketQueue {
	fn update_last(&mut self, price_info: PriceInfo) {
		self.last = (self.last + 1) % Self::BUCKET_SIZE;
		self.bucket[self.last as usize] = price_info;
	}

	fn get_last(&self) -> PriceInfo {
		self.bucket[self.last as usize]
	}

	fn calculate_average(&self) -> Option<PriceInfo> {
		let sum = self.bucket.iter().sum::<PriceInfo>();
		Some(PriceInfo {
			avg_price: sum.avg_price.checked_div(&Price::from(Self::BUCKET_SIZE as u128))?,
			volume: sum.volume.checked_div(Self::BUCKET_SIZE as u128)?,
		})
	}
}

impl Index<usize> for BucketQueue {
	type Output = PriceInfo;
	fn index(&self, index: usize) -> &Self::Output {
		&self.bucket[index]
	}
}

impl IndexMut<usize> for BucketQueue {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.bucket[index]
	}
}

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

		#[pallet::constant]
		type BucketLength: Get<u32>;

		#[pallet::constant]
		type BucketDepth: Get<u32>;

		#[pallet::constant]
		type MaxAssetCount: Get<u32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		PriceDataNotFound,
		AssetCountOverflow,
		PriceComputationError,
	}

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::storage]
	#[pallet::getter(fn asset_count)]
	pub type NumOfTrackedAssets<T> = StorageValue<_, u32, ValueQuery>;

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
			Self::update_data();

			PriceBuffer::<T>::remove_all();

			Weight::zero()	// TODO
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn on_create_pool(asset_pair: AssetPair) -> DispatchResult {
		PriceDataTen::<T>::append((asset_pair.name(), BucketQueue::default()));

		let incremented_asset_count = Self::asset_count()
			.checked_add(1)
			.ok_or(Error::<T>::AssetCountOverflow)?;
		<NumOfTrackedAssets<T>>::put(incremented_asset_count);

		Ok(())
	}

	pub fn on_trade(asset_pair: AssetPair, price_entry: PriceEntry) -> DispatchResult {
		let asset_pair_id = asset_pair.name();
		if PriceBuffer::<T>::contains_key(&asset_pair_id) {
			PriceBuffer::<T>::append(asset_pair_id, price_entry);
		} else {
			PriceBuffer::<T>::insert(asset_pair_id, vec![price_entry]);
		}

		Ok(())
	}

	fn update_data() -> DispatchResult {
        // TODO: maybe create a separate storage for liquidity_data and process it separately

		PriceDataTen::<T>::mutate(|data_ten| -> DispatchResult {
			for (asset_pair_id, data) in data_ten.iter_mut() {
				let maybe_price = <PriceBuffer<T>>::try_get(asset_pair_id);
				let result = if let Ok(prices) = maybe_price {
					PriceInfo::calculate_price_info(prices.as_slice()).ok_or(Error::<T>::PriceComputationError)?
				} else {
					PriceInfo::default()
				};

				data.update_last(result);
			}

			Ok(())
		})?;

		let now = <frame_system::Pallet<T>>::block_number();
		if now.is_zero() {
			return Ok(());
		} // TODO ???

		if (now % T::BlockNumber::from(BUCKET_SIZE)) == T::BlockNumber::from(BUCKET_SIZE - 1) {
			for element_from_ten in PriceDataTen::<T>::get().iter() {
				PriceDataHundred::<T>::mutate(element_from_ten.0.clone(), |data| -> DispatchResult {
					data.update_last(
						element_from_ten
							.1
							.calculate_average()
							.ok_or(Error::<T>::PriceComputationError)?,
					);
					Ok(())
				})?;
			}
		}

		if (now % T::BlockNumber::from(BUCKET_SIZE.pow(2))) == T::BlockNumber::from(BUCKET_SIZE.pow(2) - 1) {
			for element_from_hundred in PriceDataHundred::<T>::iter() {
				PriceDataThousand::<T>::mutate(element_from_hundred.0.clone(), |data| -> DispatchResult {
					data.update_last(
						element_from_hundred
							.1
							.calculate_average()
							.ok_or(Error::<T>::PriceComputationError)?,
					);
					Ok(())
				})?;
			}
		}

		Ok(())
	}
}

pub struct PriceOracleHandler<T>(PhantomData<T>);
impl<T: Config> AMMHandlers<T::AccountId, AssetId, AssetPair, Balance> for PriceOracleHandler<T> {
	fn on_create_pool(asset_pair: AssetPair) {
		Pallet::<T>::on_create_pool(asset_pair);
	}

	fn on_trade(amm_transfer: &AMMTransfer<T::AccountId, AssetId, AssetPair, Balance>, liq_amount: Balance) {
		let (price, amount) = amm_transfer.normalize_price().unwrap_or((Zero::zero(), Zero::zero()));

		let price_entry = PriceEntry {
			price,
			amount,
			liq_amount,
		};

		Pallet::<T>::on_trade(amm_transfer.assets, price_entry);
	}
}
