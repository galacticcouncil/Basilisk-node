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
//  * payout handler
//
//  NOTE: weight increment have to be linear. Can't be variable(exponential e.g)

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

pub use pallet::*;

use orml_traits::RewardHandler;
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
	/// Sum of all shares in period
	total_shares: Share,
	/// Sum of weighted shares for all cccounts in period
	total_weighted_shares: Share,
	/// Rewards accumulated for period
	rewards: Balance,
	/// Period to which data is snapshoted
	period: PeriodIndex,
}

/// Liquidity provider info
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, Default)]
pub struct LpInfo {
	shares: Balance,
	/// Period in which account added shares
	loyalty_from: PeriodIndex,
	/// Period from which loyalty weight is calucated. Account can claim from this period.
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
		/// Maximum number of snapshots saved in storage per pool. Oldest snapshot accumulate
		/// rewards from discarded snapshot. In case of owerflow, oldest snapshot is destroyed and
		/// rewards from this snapshot are moved to next oldest snapshot.
		#[pallet::constant]
		type MaxSnapshots: Get<u16>;

		/// Loyalty bonus for leaving shares in pool and not claiming rewards. This value is used
		/// to calculate loyalty weight.
		/// Loyalty weight calculation: (end_period - start_period ) * LoyaltyWeightBonus
		#[pallet::constant]
		type LoyaltyWeightBonus: Get<u32>;

		/// Loyalty weight slash[%] for claiming rewards [0-100%].
		/// `0` is equal to not slash anything. Loyalty weight continue to accumulate as before.
		/// `100` is same as reset to 0. Loyalty weight will be accumulated from scratch from this
		/// point.
		type LoyaltySlash: Get<Percent>;

		type Handler: RewardHandler<Self::AccountId, Balance = Balance, PoolId = PoolId<Self>>;
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// Math owerflow error
		Overflow,

		/// One account try to add shares multiple times
		DuplicateShares,
	}

	#[pallet::storage]
	#[pallet::getter(fn snapshots)]
	pub(super) type Snapshots<T: Config> = StorageMap<_, Twox64Concat, PoolId<T>, Vec<PoolState>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	/// Currently running period. Shares can be added to this state. Snapshot will be created from
	/// this state at the end of the period. This storage hold tuple `(running_period_state,
	/// next_period_state)`. `running_period_state` is state of currenlty running period. Shares
	/// can be only removed from this state. New shares should be added to `next_period_state`.
	pub(super) type Pools<T: Config> = StorageMap<_, Twox64Concat, PoolId<T>, (PoolState, PoolState), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_providers)]
	pub(super) type LiquidityProviders<T: Config> =
		StorageDoubleMap<_, Twox64Concat, PoolId<T>, Twox64Concat, T::AccountId, LpInfo, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {
		/// Add new shares to pool.
		///
		/// !!!Warn: Adding more shares from the same account is not supported. This case will
		/// result in unfair rewards. e.g account will accumulate loyalty weight for 1 share and
		/// adding more shares later will result in incorrect loaylty weight for locked shares amount.
		/// Loyalty weight is calculated only for time (finished periods).
		///
		/// Parameters:
		/// - `who`: account adding shares
		/// - `pool_id`: pool identifier to which shares are added
		/// - `amount`: amount of shares added to pool
		/// - `now`: period index of currently running period
		pub fn add_shares(
			who: &T::AccountId,
			pool_id: PoolId<T>,
			amount: Share,
			now: PeriodIndex,
		) -> Result<(), Error<T>> {
			if amount.is_zero() {
				return Ok(());
			}

			LiquidityProviders::<T>::try_mutate(pool_id.clone(), who, |lp| -> Result<(), Error<T>> {
				ensure!(lp.is_none(), Error::<T>::DuplicateShares);

				// claiming rewards is possible from next whole period.
				let claimable_period = now + 1;
				let w = Self::get_loyalty_weight_for(now, claimable_period, T::LoyaltyWeightBonus::get())?;
				let shares_weighted = Self::get_weighted_shares(amount, w)?;
				Pools::<T>::try_mutate(
					pool_id.clone(),
					|(_current_period, next_period)| -> Result<(), Error<T>> {
						next_period.total_weighted_shares = next_period
							.total_weighted_shares
							.checked_add(shares_weighted)
							.ok_or(Error::<T>::Overflow)?;

						next_period.total_shares = next_period
							.total_shares
							.checked_add(amount)
							.ok_or(Error::<T>::Overflow)?;
						Ok(())
					},
				)?;

				*lp = Some(LpInfo {
					shares: amount,
					loyalty_from: claimable_period,
					claim_from: claimable_period,
				});
                
				Ok(())
			})
		}

		/// Payoff rewards without removing shares from pool.
		///
		/// Parameters:
		/// - `who`: account claiming shares
		/// - `pool_id`: pool identifier to claim rewards for
		/// - `now`: period index of currently running period. Will be used to caculate loyalty
		/// weight
		pub fn claim_rewards(who: &T::AccountId, pool_id: PoolId<T>, now: PeriodIndex) -> Result<(), Error<T>> {
			LiquidityProviders::<T>::try_mutate(pool_id.clone(), who, |lp| -> Result<(), Error<T>> {
				let lp = match lp {
					Some(lp) => lp,
					None => {
						return Ok(());
					}
				};

				if lp.claim_from < now {
					return Ok(());
				}

				// rewards to payoff for all snapshots
				let mut rewards: Balance = 0;

				let mut snapshots = Snapshots::<T>::try_get(&pool_id).unwrap_or_default();
				// 0 offset is ok, check is in the iterator
				let offset: usize = (snapshots.len().try_into().unwrap_or(0) - (now - lp.claim_from))
					.max(0)
					.try_into()
					.unwrap_or(0);

				snapshots
					.iter_mut()
					.skip(offset)
					.try_for_each(|s| -> Result<(), Error<T>> {
						// this should never happen but better safe than sorry
						if s.period < lp.claim_from {
							return Ok(());
						}

						let lw = Self::get_loyalty_weight_for(lp.loyalty_from, s.period, T::LoyaltyWeightBonus::get())?;
						let weighted_shares = Self::get_weighted_shares(lp.shares, lw)?;
						let reward = Self::get_weighted_rewards(weighted_shares, s.rewards, s.total_weighted_shares)?
							.min(s.rewards);

						rewards += reward;
						s.rewards -= reward;
						s.total_weighted_shares = s
							.total_weighted_shares
							.checked_sub(weighted_shares)
							.ok_or(Error::<T>::Overflow)?
							.max(0);

						s.total_shares = s
							.total_shares
							.checked_sub(lp.shares)
							.ok_or(Error::<T>::Overflow)?
							.max(0);

						Ok(())
					})?;

				Snapshots::<T>::insert(&pool_id, snapshots);

				lp.loyalty_from = Self::slash_loyalty_weight(lp.loyalty_from, now, T::LoyaltySlash::get());
				lp.claim_from = now;

				T::Handler::payout(who, &pool_id, rewards);
				Ok(())
			})?;

			Ok(())
		}

		/// Remove shares from pool and payoff rewards.
		///
		/// Parameters:
		/// - `who`: account removing shares
		/// - `pool_id`: pool identifier to remove shares from
		/// - `now`: period index of currently running period. Will be used to caculate loyalty
		/// weight
		pub fn remove_shares(who: &T::AccountId, pool_id: PoolId<T>, now: PeriodIndex) -> Result<(), Error<T>> {
			let lp = match LiquidityProviders::<T>::take(pool_id.clone(), who.clone()) {
				Some(lp) => lp,
				None => {
					return Ok(());
				}
			};

			Self::claim_rewards(who, pool_id.clone(), now)?;

			Pools::<T>::try_mutate(
				pool_id.clone(),
				|(current_period, next_period)| -> Result<(), Error<T>> {
					let weight_per_one_period =
						Self::get_loyalty_weight_for(now - 1, now, T::LoyaltyWeightBonus::get())?;
					let weight_shares_per_one_period = Self::get_weighted_shares(lp.shares, weight_per_one_period)?;

					if lp.loyalty_from == now {
						next_period.total_weighted_shares = next_period
							.total_weighted_shares
							.checked_sub(weight_shares_per_one_period)
							.ok_or(Error::<T>::Overflow)?;

						next_period.total_shares = next_period
							.total_shares
							.checked_sub(lp.shares)
							.ok_or(Error::<T>::Overflow)?;
					} else {
						current_period.total_weighted_shares = current_period
							.total_weighted_shares
							.checked_sub(weight_shares_per_one_period)
							.ok_or(Error::<T>::Overflow)?;

						current_period.total_shares = current_period
							.total_shares
							.checked_sub(lp.shares)
							.ok_or(Error::<T>::Overflow)?;
					}

					Ok(())
				},
			)?;

			Ok(())
		}

		/// Create snapshot from "current state"(running period) reward it, initialize new "current state"
		/// from "next state" and reset "next_state".
		/// Number of stored snapshots is limitted and rewards from discarded
		/// snapshot will be added to next oldedst snapshot rewards.
		///
		/// pool_id - pool id to create snapshot for and reward it
		/// now - index ending period. This index will be used for crated snapshot
		/// rewards - rewas for current period
		pub fn snapshot_and_reward(pool_id: PoolId<T>, now: PeriodIndex, rewards: Balance) {
			Pools::<T>::mutate(pool_id.clone(), |(current_period, next_period)| {
				current_period.rewards = rewards;
				// This should never overflow with this values
				let weight_per_one_period = Self::get_loyalty_weight_for(1, 2, T::LoyaltyWeightBonus::get()).unwrap();

				let weighted_shares_per_one_period =
					match Self::get_weighted_shares(current_period.total_shares, weight_per_one_period) {
						Ok(v) => v,
						Err(_) => {
							//NOTE: is this ok?
							u128::MAX
						}
					};

				current_period.total_weighted_shares += weighted_shares_per_one_period;

				let mut snapshots = Snapshots::<T>::get(pool_id);

				if snapshots.len() >= T::MaxSnapshots::get().into() {
					let removed_s = snapshots.remove(0);

					//move rewards from discarded snapshot to next oldest
					snapshots[0].rewards = snapshots[0].rewards.saturating_add(removed_s.rewards);
				}

				current_period.period = now;
				snapshots.push(current_period.clone());

				*current_period = next_period.clone();
				*next_period = PoolState::default();
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
	/// - `loaylty_from`: period index since when is lolalty weight currenlty computed
	/// - `now`: current period index
	/// - `slash`: percentage slash amount [0 - 100%]
	///
	/// Return new index to `claim_from`
	pub fn slash_loyalty_weight(loyalty_from: PeriodIndex, now: PeriodIndex, slash: Percent) -> PeriodIndex {
		loyalty_from + slash.mul_floor(now - loyalty_from)
	}

	/// This function calculate and return loyalty weight for periods range e.g from period 10 to
	/// period 20.
	///
	/// Weight calculation: `(to - from) * weight_increment`
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
			.checked_mul(weight_increment.into())
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
