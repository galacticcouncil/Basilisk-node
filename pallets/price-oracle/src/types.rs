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

use codec::{Decode, Encode};
use frame_support::sp_runtime::traits::{CheckedDiv, Zero};
use frame_support::sp_runtime::RuntimeDebug;
pub use primitives::{Balance, Price};
use sp_std::iter::Sum;
use sp_std::ops::{Add, Index, IndexMut, Mul};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

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
	pub avg_price: Price,
	pub volume: Balance,
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
	// make sure that BUCKET_SIZE != 0
	pub const BUCKET_SIZE: u32 = BUCKET_SIZE;
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
	fn calculate_average(&self) -> PriceInfo;
}

impl BucketQueueT for BucketQueue {
	fn update_last(&mut self, price_info: PriceInfo) {
		self.last = (self.last + 1) % Self::BUCKET_SIZE;
		self.bucket[self.last as usize] = price_info;
	}

	fn get_last(&self) -> PriceInfo {
		self.bucket[self.last as usize]
	}

	fn calculate_average(&self) -> PriceInfo {
		let sum = self.bucket.iter().sum::<PriceInfo>();
		PriceInfo {
			avg_price: sum.avg_price.checked_div(&Price::from(Self::BUCKET_SIZE as u128)).expect("avg_price is valid value; BUCKET_SIZE is non-zero integer; qed"),
			volume: sum.volume.checked_div(Self::BUCKET_SIZE as u128).expect("avg_price is valid value; BUCKET_SIZE is non-zero integer; qed"),
		}
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
