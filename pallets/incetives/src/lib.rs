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

//NOTE:
// in_period - period used to calculate weights
// claimable_period - period from which user can claim rewards (may be different from in_period if
// user claim rewards

//TODO:
//  * add overview comment
//  * claim call
//  * remove_shares call
//  * on_finalize add reward and update next period if exist

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

pub use pallet::*;

use primitives::Balance;
use sp_runtime::{traits::Saturating, FixedU128, RuntimeDebug};

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
	use std::convert::TryInto;

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

		/// Max saved snapshots - user will be able claim rewards for MaxSnapshots. Older snapshots
		/// will be dropped
		#[pallet::constant]
		type MaxSnapshots: Get<u16>;

		#[pallet::constant]
		type BonusWeightIncrement: Get<u32>;

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
	#[pallet::getter(fn snapshots)]
	pub(super) type PoolSnapshots<T: Config> =
		StorageMap<_, Twox64Concat, PoolId<T>, Vec<PoolInfo<T::BlockNumber>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn shares)]
	/// (T:BlockNumber, Balance) - (entered period,first claimable period, shares)
	pub(super) type Shares<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		PoolId<T>,
		Twox64Concat,
		T::AccountId,
		(T::BlockNumber, T::BlockNumber, Balance),
		ValueQuery,
	>;

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
		/// New liquidity added. Rewards will be conting from next period
		#[pallet::weight(<T as Config>::WeightInfo::add_shares())]
		pub fn add_shares(origin: OriginFor<T>, pool: PoolId<T>, add_amount: Share) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let now_period = <frame_system::Pallet<T>>::block_number() % T::AccumulatePeriod::get();
			let next_period = now_period + T::BlockNumber::from(1_u32);
			let mut snapshots = PoolSnapshots::<T>::try_get(pool.clone()).unwrap_or(vec![]);

			let default_info = PoolInfo::default();
			let last_snapshot = snapshots.last().unwrap_or(&default_info);

			let mut next_snapshot: PoolInfo<T::BlockNumber>;
			if last_snapshot.period == next_period {
				next_snapshot = last_snapshot.clone();
			} else {
				next_snapshot = default_info.clone();
			}

			next_snapshot.period = next_period;
			next_snapshot.total_shares = next_snapshot.total_shares.saturating_add(add_amount);
			next_snapshot.lp_count = next_snapshot.lp_count.saturating_add(1);

			Self::push_or_replace_last_snapshot(&mut snapshots, next_snapshot, T::MaxSnapshots::get());

			PoolSnapshots::<T>::insert(pool.clone(), snapshots);
			Shares::<T>::mutate(pool, who, |(in_period, claimable_period, shares)| {
				*in_period = now_period;
				*claimable_period = next_period;
				*shares = add_amount;
			});

			//TODO: add_event
			Ok(().into())
		}

		/// Only full reward can be claimed till last finisehd period
		#[pallet::weight(<T as Config>::WeightInfo::claim_rewards())]
		pub fn claim_rewards(origin: OriginFor<T>, pool: PoolId<T>) -> DispatchResultWithPostInfo {
			//TODO: should be time bonus reseted? -> make it configurable
			let who = ensure_signed(origin)?;

			let now_period = <frame_system::Pallet<T>>::block_number() % T::AccumulatePeriod::get();
			Shares::<T>::mutate(pool.clone(), who, |(in_period, claimable_period, shares)| {
				if shares.is_zero() {
					return;
				}

				let mut snapshots = PoolSnapshots::<T>::get(pool.clone());

				//claimable_period may be bigger than snapshots.len() if user doesn't claimed
				//let skip = sp_std::cmp::max(T::BlockNumber::from(snapshots.len() as u64) - *claimable_period), T::BlockNumber::from(0));
				let offset = {
					let this = sp_std::cmp::max(
						T::BlockNumber::from(snapshots.len() as u32) - *claimable_period,
						T::BlockNumber::from(0_u32),
					)
					.try_into();
					match this {
						Ok(x) => x,
						Err(_) => 0,
					}
				};

				let total_claimable_rewards: Balance = 0;
				let cp: u128 = claimable_period.try_into().unwrap_or(0);
				snapshots.iter_mut().skip(offset).for_each(|s| {
					//TODO: consult this with kubo. Should be weight calculated for every period or
					//should I use highest weight for all periods
					let p: u128 = s.period.try_into().unwrap_or(0);
					let weight_in_period = Self::get_weight_in_period(
						FixedU128::from(p),
						FixedU128::from(cp),
						T::BonusWeightIncrement::get(),
					);

					//TODO: calculate reward in snapshot
					//TODO: update snapshot
				});

				//TODO: "slash" weight

				//update acc data
				//in_period & claimable_period - weight sa pocita od in_period. Calaimuje sa len od
				//claimable_period
				*claimable_period = T::BlockNumber::from(1_u32) + now_period;

				//TODO: make this configurable and handle all cases
				//"slash" for claiming - slash weight to 50%
				*in_period += (now_period - *in_period) / T::BlockNumber::from(2_u32);
			});

			//TODO: add_event
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// This function check if last elements snapshot.period == p.period and replace it if equals
	/// or push new element to vec
	fn push_or_replace_last_snapshot(
		s: &mut Vec<PoolInfo<<T as frame_system::Config>::BlockNumber>>,
		p: PoolInfo<T::BlockNumber>,
		max_cap: u16,
	) {
		if !s.is_empty() && s.last().unwrap().period == p.period {
			let last = s.len() - 1;
			s[last] = p;
		} else {
			Self::push_to_snapshots(s, p, max_cap);
		}
	}
	/// This function push new element to snapshots. Firt snapshot may be dropped to stay in maxCap
	fn push_to_snapshots(
		s: &mut Vec<PoolInfo<<T as frame_system::Config>::BlockNumber>>,
		p: PoolInfo<T::BlockNumber>,
		max_cap: u16,
	) {
		if s.len() == max_cap as usize {
			s.remove(0);
			s.push(p);
		} else {
			s.push(p);
		}
	}

	fn get_weight_in_period(
		current_period: FixedU128,
		claimable_period: FixedU128,
		weight_increment: u32,
	) -> FixedU128 {
		//current_period - claimable_period).pow(weight_increment)
		(current_period.saturating_sub(claimable_period)).saturating_pow(weight_increment as usize)
	}

	fn get_weighted_shares(
		acc_shares: Balance,
		acc_weight: Balance,
		total_shares: Balance,
		total_weights: Balance,
	) -> Balance {
		(acc_shares * acc_weight as Balance) / (total_shares * total_weights)
	}

	fn get_weighted_rewards(
		acc_weighted_shares: Balance,
		total_rewards: Balance,
		total_weighted_shares: Balance,
	) -> Balance {
		(acc_weighted_shares * total_rewards) / total_weighted_shares
	}
}
