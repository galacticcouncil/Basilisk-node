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

//! # Incentives Pallet
//!
//! ## Overview
//!

//TODO:
//  * add overview comment
//  * limit max periods in storage

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

pub use pallet::*;

use primitives::Balance;
use sp_runtime::RuntimeDebug;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

//mod benchmarking;

pub mod weights;

use weights::WeightInfo;

pub type Share = Balance;
pub type PoolId<T> = <T as frame_system::Config>::AccountId;

/// Pool state at the end of i-th period
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, Default)]
pub struct PoolInfo<BlockNumber> {
	total_shares: Balance,
	total_rewards: Balance,
	total_weights: Balance,
	lp_count: u64,
	period: BlockNumber,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, Default)]
pub struct LiqProvider<AccountId, BlockNumber> {
	/// Liquidity provider account
	account: AccountId,

	/// Index of first rewardable period for account (first full period)
	in_period: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Zero;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Period to accumulate rewards
		#[pallet::constant]
		type AccumulatePeriod: Get<Self::BlockNumber>;

		/// Weight information for the extrinsics
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(super) type PoolSnapshots<T: Config> =
		StorageMap<_, Twox64Concat, PoolId<T>, Vec<PoolInfo<T::BlockNumber>>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(now: BlockNumberFor<T>) {
			if now % T::AccumulatePeriod::get() == Zero::zero() {
				//TODO: calculate rewards
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::add_shares())]
		pub fn add_shares(origin: OriginFor<T>, pool: PoolId<T>, add_amount: Share) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let now_period = <frame_system::Pallet<T>>::block_number() % T::AccumulatePeriod::get();
            let next_period = now_period + T::BlockNumber::from(1_u32);
			let mut snapshots = PoolSnapshots::<T>::try_get(pool).unwrap_or(vec![]);

			let last_snapshot = snapshots.last().unwrap_or(&PoolInfo {
				total_shares: 0,
				total_rewards: 0,
				total_weights: 0,
				lp_count: 0,
				period: now_period,
			});

			let mut next_snapshot = last_snapshot.clone();

            next_snapshot.period = next_period;
                /*
						next_snapshot.total_shares.saturating_add(add_amount);
						next_snapshot.lp_count.saturating_add(1);
			*/
			//find or create pool
			//calculate current period
			//update pollInfo in next period
			//save user shares for pool

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {}

/*
pub fn get_weight_in_period(current_period: Period, in_period: Period, weight_increment: u32) -> u32 {
	(current_period - in_period + 1).pow(weight_increment)
}

pub fn get_weighted_shares(acc_shares: Share, acc_weight: u32, total_shares: Share, total_weights: Balance) -> Balance {
	(acc_shares * acc_weight as Balance) / (total_shares * total_weights)
}

pub fn get_weighted_rewards(
	acc_weighted_shares: Share,
	total_rewards: Balance,
	total_weighted_shares: Share,
) -> Balance {
	(acc_weighted_shares * total_rewards) / total_weighted_shares
}
*/
