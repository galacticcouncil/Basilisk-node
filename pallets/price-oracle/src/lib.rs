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
use primitives::{Balance, Price};
use sp_std::iter::Sum;
use sp_std::ops::{Add, Index, IndexMut, Mul};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// mod benchmarking;

// pub mod weights;

// use weights::WeightInfo;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

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

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct PriceEntry {
	price: Price,
	amount: Balance,
	liq_amount: Balance,
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
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		Placeholder(),
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_count)]
	pub type NumOfTrackedAssets<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_buffer)]
	pub type PriceBuffer<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<PriceEntry>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_data_ten)]
	pub type PriceDataTen<T: Config> = StorageValue<_, Vec<(T::AccountId, BucketQueue)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_data_hundred)]
	pub type PriceDataHundred<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BucketQueue, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn price_data_thousand)]
	pub type PriceDataThousand<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BucketQueue, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			Self::update_data();
			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	fn new_entry(asset_pair: T::AccountId) -> DispatchResult {
		PriceDataTen::<T>::append((asset_pair, BucketQueue::default()));

		let incremented_asset_count = Self::asset_count()
			.checked_add(1)
			.ok_or(Error::<T>::AssetCountOverflow)?;
		<NumOfTrackedAssets<T>>::put(incremented_asset_count);

		Ok(())
	}

	fn on_trade(asset_pair: T::AccountId, entry: PriceEntry) -> DispatchResult {
		if PriceBuffer::<T>::contains_key(&asset_pair) {
			PriceBuffer::<T>::append(asset_pair, entry);
		} else {
			PriceBuffer::<T>::insert(asset_pair, vec![entry]);
		}

		Ok(())
	}

	fn update_data() -> DispatchResult {
		PriceDataTen::<T>::try_mutate(|data_ten| -> DispatchResult {
			for (asset_pair, data) in data_ten.iter_mut() {
				let maybe_price = <PriceBuffer<T>>::try_get(asset_pair);
				let result = if let Ok(prices) = maybe_price {
					PriceInfo::calculate_price_info(prices.as_slice()).ok_or(Error::<T>::PriceComputationError)?
				} else {
					PriceInfo::default()
				};

				data.update_last(result);
			}

			PriceBuffer::<T>::remove_all();

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
