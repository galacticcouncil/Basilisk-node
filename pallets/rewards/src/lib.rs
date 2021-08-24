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

//TODO:
//  * add overview comment
//  * check math (tests)

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

pub use pallet::*;

use sp_runtime::RuntimeDebug;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::traits::Get;
use sp_arithmetics::Percent;
pub type PoolId<T> = <T as frame_system::Config>::AccountId;
pub type PeriodIndex = u128;
pub type Balance = u128;
pub type Share = u128;
pub type LoyaltyWeight = u128;

/// Pool state at the end of the i-th period
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
		/// Maximum snapshots in storage per pool. Oldest snapshtots are destroyed and rewards are
		/// moved to the next oldest snapshot in case of overflow.
		#[pallet::constant]
		type MaxSnapshots: Get<u16>;

		/// Loyalty bonus(exponent) for NOT cliaming rewards. This vlaue is used in loyalty weight calculation.
		/// Loyalty weight calculation: (start_period - end_period) ^ LoyaltyWeightBonus
		#[pallet::constant]
		type LoyaltyWeightBonus: Get<u32>;

		///LoyaltyWeight percentage slash for claiming rewards [0-100%]
		type LoyaltySlash: Get<Percent>;
	}

	#[pallet::error]
	#[derive(PartialEq)]
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
			let w = Self::get_loyalty_weight_for(claimable_period, now, T::LoyaltyWeightBonus::get())?;
			let s_weighted = Self::get_weighted_shares(amount, w)?;
			CurrentState::<T>::try_mutate(pool_id.clone(), |current_state| -> Result<(), Error<T>> {
				if let Some(v) = current_state.total_weighted_shares.checked_add(s_weighted) {
					current_state.total_weighted_shares = v;
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
				let lp = match lp {
					Some(lp) => lp,
					None => {
						return Ok(());
					}
				};

				let mut snapshots = Snapshots::<T>::try_get(pool_id.clone()).unwrap_or_default();

				let offset: usize = sp_std::cmp::max(snapshots.len().try_into().unwrap_or(0) - lp.claim_from, 0)
					.try_into()
					.unwrap_or(0);

				let mut acc_rewards: Balance = 0;
				let weight = Self::get_loyalty_weight_for(lp.claim_from, now, T::LoyaltyWeightBonus::get())?;
				if weight.is_zero() {
					return Ok(());
				}
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
				lp.claim_from = Self::slash_loyalty_weight(lp.claim_from, now, T::LoyaltySlash::get());

				Ok(())
			})?;

			//TODO: add reward handler
			Ok(())
		}

		pub fn remove_shares(who: &T::AccountId, pool_id: PoolId<T>, now: PeriodIndex) -> Result<(), Error<T>> {
			let lp = match LiquidityProviders::<T>::take(pool_id.clone(), who.clone()) {
				Some(lp) => lp,
				None => {
					return Ok(());
				}
			};

			Self::claim_rewards(who, pool_id.clone(), now)?;

			CurrentState::<T>::try_mutate(pool_id.clone(), |current_state| -> Result<(), Error<T>> {
				//this will work if loyalty weight change is 1 bettween period
				//FIXME: calculate with different loyalty weight change

				if let Some(v) = current_state.total_weighted_shares.checked_sub(lp.shares) {
					current_state.total_weighted_shares = v;
				} else {
					return Err(Error::<T>::Overflow);
				}

				Ok(())
			})?;

			Ok(())
		}

		/// Create snapshot from "current state"(running period) and reward it. It will initialize new "current
		/// state" for pool. Number of stored snapshots is limitted and rewards from discarded
		/// snapshot will be added to next oldedst snapshot rewards.
		///
		/// pool_id - pool id to create snapshot for and reward it
		/// now_idx - index ending period. This index will be used for crated snapshot
		/// rewards - rewas for current period
		pub fn snapshot_and_reward(pool_id: PoolId<T>, now_idx: PeriodIndex, rewards: Balance) {
			CurrentState::<T>::mutate(pool_id.clone(), |current_state| {
				current_state.rewards = rewards;
				let new_current_state = PoolState::default();

				let mut snapshots = Snapshots::<T>::get(pool_id);

				if snapshots.len() >= T::MaxSnapshots::get().into() {
					let removed_s = snapshots.remove(0);

					//move rewards from discarded snapshot to next oldest
					snapshots[0].rewards = snapshots[0].rewards.saturating_add(removed_s.rewards);
				}
				current_state.period = now_idx;
				snapshots.push(current_state.clone());

				*current_state = new_current_state;
			});
		}
	}
}

impl<T: Config> Pallet<T> {
	/// This function compute and return new index to `claim_from` after claiming rewards.
	/// This will result in lower loyalty weight for reward calulation in the next claim.
	/// Maximum slash(100%) will result in returning `now` index which means reset to 0.
	///
	/// New index calculation: `claim_from + floor(slash[%] * (now - claim_from))`
	///
	/// Parameters:
	/// - `claim_from`: current `claim_from` period index of liquidity provider
	/// - `now`: actual period index
	/// - `slash`: percentage slash amount [0 - 100%]
	///
	/// Return new index to `claim_from`
	pub fn slash_loyalty_weight(claim_from: PeriodIndex, now: PeriodIndex, slash: Percent) -> PeriodIndex {
		claim_from + slash.mul_floor(now - claim_from)
	}

	/// This function calculate and return loyalty weight for periods range e.g from period 10 to
	/// period 20.
	///
	/// Weight calculation: `(to - from) ^ weight_increment`
	///
	/// Parameters:
	/// - `from`: start of the range to calculate loyalty weight for
	/// - `to`: end of the range to calculate loyalty for
	/// - `weight_increment`: weight increment for each period in range
	///
	/// Return loyalty weight
	fn get_loyalty_weight_for(
		from: PeriodIndex,
		to: PeriodIndex,
		weight_increment: u32,
	) -> Result<LoyaltyWeight, Error<T>> {
		to.checked_sub(from)
			.ok_or(Error::<T>::Overflow)?
			.checked_pow(weight_increment)
			.ok_or(Error::<T>::Overflow)
	}

	/// This function compute weighted shares used to calculate amout of reward.
	///
	/// Weighted shares calulation: `shares * weight`
	///
	/// Parameters:
	/// - `shares`: amount of shares liquidity provider own
	/// - `weight`: loyalty weight of liquidity provider for NOT claiming rewards
	///
	/// Return weighted shares amount
	fn get_weighted_shares(shares: Share, weight: LoyaltyWeight) -> Result<Balance, Error<T>> {
		shares.checked_mul(weight).ok_or(Error::<T>::Overflow)
	}

	/// This function compute and return amount of rewards account can claim in snapshot based on weighted shares amount.
	/// This function should be called for every snapshot account is claiming rewards
	///
	/// Rewards calculation: `(weighted_shares * totalt_rewards) / total_weighted_shares`
	///
	/// Parameters:
	///
	/// - `weighted_shares`: amount of weighted shares owend by account
	/// - `total_rewards`: amount of total rewards accumulated in the pool in snapshot
	/// - `total_weighted_shares`:  sum of all weighted shares in snapshot
	///
	/// Return amount of reward to pay
	fn get_weighted_rewards(
		weighted_shares: Share,
		total_rewards: Balance,
		total_weighted_shares: Balance,
	) -> Result<Balance, Error<T>> {
		Ok(weighted_shares
			.checked_mul(total_rewards)
			.ok_or(Error::<T>::Overflow)?
			.checked_div(total_weighted_shares)
			.ok_or(Error::<T>::Overflow)?
			.min(total_rewards))
	}
}
