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

//! # Rewards Pallet
//!
//! ## Overview
//!

//NOTE:

//TODO:
//  * add overview comment
//  * replace unwrap() - may panic
//  * remove dependencies on our types

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

pub use pallet::*;

use primitives::Balance;
use sp_runtime::RuntimeDebug;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::traits::Get;
use primitives::Balance as LoyaltyWeight;
use primitives::Balance as Share;
use sp_arithmetics::Perbill;
pub type PoolId<T> = <T as frame_system::Config>::AccountId;
pub type PeriodIndex = u128;

/// Pool state at the end of i-th period
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, Default)]
pub struct PoolState {
	total_weighted_shares: Share,
	rewards: Balance,
	period: PeriodIndex,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, Default)]
pub struct LpInfo {
	shares: Balance,
	in_snaphost: PeriodIndex,
	claim_from: PeriodIndex,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::BlockNumberFor;
	use sp_runtime::traits::Zero;
	use std::convert::TryInto;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Number of blocks per snapshot. Rewards are acumulated for each snaphot
		#[pallet::constant]
		type SnaphotSize: Get<Self::BlockNumber>;

		/// Maximum snapshots in storage per pool. Oldest snapshtots are destroyed and rewards are
		/// moved to next oldest snapshot.
		#[pallet::constant]
		type MaxSnapshots: Get<u16>;

		/// Increment for loyalty bonus.
		/// TWi = (CS - i) ^ LWI
		/// TWi - time weight in snapshot i
		///  CS - current snapshot number
		///  i - i-th snapshot number
		///  LWI - loayalty weight increment
		#[pallet::constant]
		type LoyaltyWeightIncrement: Get<u32>;

		///LoyaltyWeight slash for claiming rewards - e.g 2 LW = LWc / 2;
		///LW - loyalty weight
		///LWc - loyalty weight current
		type LoyaltySlash: Get<Perbill>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Math owerflow error
		Overflow,
	}

	#[pallet::storage]
	#[pallet::getter(fn snapshots)]
	pub(super) type Snapshots<T: Config> = StorageMap<_, Twox64Concat, PoolId<T>, Vec<PoolState>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_state)]
	pub(super) type CurrentState<T: Config> = StorageMap<_, Twox64Concat, PoolId<T>, PoolState, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_providers)]
	pub(super) type LiquidityProviders<T: Config> =
		StorageDoubleMap<_, Twox64Concat, PoolId<T>, Twox64Concat, T::AccountId, LpInfo, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {
		/// New liquidity added. Rewards will be conting from next period
		pub fn add_shares(
			who: &T::AccountId,
			pool_id: PoolId<T>,
			amount: Share,
			now: PeriodIndex,
		) -> Result<(), Error<T>> {
			if amount.is_zero() {
				return Ok(());
			}

			let claimable_period = now + 1;
			let w = Self::get_loyalty_weight_in(now, claimable_period, T::LoyaltyWeightIncrement::get())?;
			let s_weighted = Self::get_weighted_shares(amount, w)?;
			CurrentState::<T>::try_mutate(pool_id.clone(), |cs| -> Result<(), Error<T>> {
				if let Some(v) = cs.total_weighted_shares.checked_add(s_weighted) {
					cs.total_weighted_shares = v;
					Ok(())
				} else {
					return Err(Error::<T>::Overflow);
				}
			})?;

			LiquidityProviders::<T>::insert(
				pool_id,
				who,
				LpInfo {
					shares: amount,
					in_snaphost: now,
					claim_from: claimable_period,
				},
			);

			Ok(())
		}

		pub fn claim_rewards(who: &T::AccountId, pool_id: PoolId<T>, now: PeriodIndex) -> Result<(), Error<T>> {
			LiquidityProviders::<T>::try_mutate(pool_id.clone(), who, |lp| -> Result<(), Error<T>> {
				let mut snapshots = Snapshots::<T>::try_get(pool_id.clone()).unwrap_or_default();

				let offset: usize = sp_std::cmp::max(snapshots.len().try_into().unwrap_or(0) - lp.claim_from, 0)
					.try_into()
					.unwrap_or(0);

				let mut acc_rewards: Balance = 0;
				//TODO: ckeck math if some period is not skipped
				let weight = Self::get_loyalty_weight_in(now, lp.claim_from, T::LoyaltyWeightIncrement::get())?;
				snapshots
					.iter_mut()
					.skip(offset)
					.try_for_each(|s| -> Result<(), Error<T>> {
						let ws = Self::get_weighted_shares(lp.shares, weight)?;
						let mut reward = Self::get_weighted_rewards(ws, s.rewards, s.total_weighted_shares)?;

						reward = if reward >= s.rewards { s.rewards } else { reward };

						acc_rewards += reward;
						s.rewards -= reward;
						Ok(())
					})?;

				Snapshots::<T>::insert(pool_id, snapshots);
				lp.claim_from = Self::slash_loyalty_weight(lp.claim_from, now);

				Ok(())
			})?;

			//TODO: add reward handler
			Ok(())
		}

		pub fn remove_shares(who: &T::AccountId, pool_id: PoolId<T>, now: PeriodIndex) -> Result<(), Error<T>> {
			//TODO: First check if user is in pool
			let lp = LiquidityProviders::<T>::take(pool_id.clone(), who.clone());
            
            

			Self::claim_rewards(who, pool_id.clone(), now)?;

			CurrentState::<T>::try_mutate(pool_id.clone(), |cs| -> Result<(), Error<T>> {
				//this will work if loyalty weight change is 1 bettween period
				//FIXME: calculate with different loyalty weight change

				if let Some(v) = cs.total_weighted_shares.checked_sub(lp.shares) {
					cs.total_weighted_shares = v;
				} else {
					return Err(Error::<T>::Overflow);
				}

				Ok(())
			})?;

			Ok(())
		}

		/// Create snapshot from "current state" and reward it. It will initialize new "current
		/// state"
		/// NOTE: this should never fail (on_initialize)
		pub fn snapshot_and_reward(pool_id: PoolId<T>, i: PeriodIndex, rewards: Balance) {
			CurrentState::<T>::mutate(pool_id.clone(), |cs| {
				cs.rewards = rewards;
				let new_current_state = PoolState::default();

				let mut snapshots = Snapshots::<T>::get(pool_id);

				if snapshots.len() >= T::MaxSnapshots::get().into() {
					let removed_s = snapshots.remove(0);

					//move rewards from discarded snapshot to next oldest
					snapshots[0].rewards = snapshots[0].rewards.checked_add(removed_s.rewards).unwrap();
				}
				cs.period = i;
				snapshots.push(cs.clone());

				*cs = new_current_state;
			});
		}
	}
}

impl<T: Config> Pallet<T> {
	/// This function calculate new index used to calculate loyalty weight.
	/// calculation of new claim from: floor(lolaylty-slash * (now - current-claim-from))
	pub fn slash_loyalty_weight(claim_from: PeriodIndex, now: PeriodIndex) -> PeriodIndex {
		claim_from + T::LoyaltySlash::get().mul_floor(now - claim_from)
	}

	fn get_loyalty_weight_in(
		now_index: PeriodIndex,
		claimable_index: PeriodIndex,
		weight_increment: u32,
	) -> Result<LoyaltyWeight, Error<T>> {
		now_index
			.checked_sub(claimable_index)
			.ok_or(Error::<T>::Overflow)?
			.checked_pow(weight_increment)
			.ok_or(Error::<T>::Overflow)
	}

	fn get_weighted_shares(shares: Share, weight: LoyaltyWeight) -> Result<Balance, Error<T>> {
		shares.checked_mul(weight).ok_or(Error::<T>::Overflow)
	}

	fn get_weighted_rewards(
		weighted_shares: LoyaltyWeight,
		total_rewards: Balance,
		total_weighted_shares: Balance,
	) -> Result<Balance, Error<T>> {
		weighted_shares
			.checked_mul(total_rewards)
			.ok_or(Error::<T>::Overflow)?
			.checked_div(total_weighted_shares)
			.ok_or(Error::<T>::Overflow)
	}
}
