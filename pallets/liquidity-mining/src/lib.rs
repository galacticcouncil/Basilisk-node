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
//
// Abbr:
//  rps - reward per share
//
//  TODO:
//      * weights and benchmarking
//      * mining nft on deposit_shares()
//      * weighted pools
//      * add real reward curve
//      * add canceled check to all user actions

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;
pub mod weights;

pub use pallet::*;

type PoolId<T> = <T as frame_system::Config>::AccountId;

use frame_support::{
	sp_runtime::traits::{One, Zero},
	traits::LockIdentifier,
	transactional,
};

use sp_arithmetic::{
	traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
	FixedU128,
};

use orml_traits::{MultiCurrencyExtended, MultiLockableCurrency};

use codec::{Decode, Encode};
use frame_support::sp_runtime::RuntimeDebug;

impl Default for LoyaltyCurve {
	fn default() -> Self {
		Self {
			b: FixedU128::from_inner(500_000_000_000_000_000), // 0.5
			scale_coef: 100,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct GlobalPool<Period, Balance> {
	updated_at: Period,
	total_shares: Balance,
	accumulated_rps: Balance,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct LoyaltyCurve {
	b: FixedU128,
	scale_coef: u32,
}

pub const LM_LOCK_ID: LockIdentifier = *b"LM_LOCK_";

use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::{FixedPointNumber, FixedPointOperand};
use sp_std::convert::{From, Into, TryInto};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Balance type
		type Balance: Parameter
			+ From<u128>
			+ Into<u128>
			+ From<Self::BlockNumber>
			//+ From<FixedU128>
			+ Member
			+ AtLeast32BitUnsigned
			+ FixedPointOperand
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ CheckedAdd;

		/// Asset type
		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + From<u32>;

		/// Currency for transfers
		type MultiCurrency: MultiLockableCurrency<Self::AccountId>
			+ MultiCurrencyExtended<Self::AccountId, CurrencyId = Self::CurrencyId, Balance = Self::Balance>;

		/// Administrator able to create liquidity mining program
		type AdminOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// Math computation overflow
		Overflow,

		/// Feature is not implemented yet
		NotImplemented,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {}

	/*
	#[pallet::storage]
	#[pallet::getter(fn pool)]
	pub type Pools<T: Config> =
		StorageMap<_, Blake2_128Concat, PoolId<T>, LmPool<T::Balance, T::CurrencyId, T::BlockNumber>, OptionQuery>;
	*/

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Craete new liquidity mining program
		#[pallet::weight(1000)]
		#[transactional]
		pub fn create_new_program(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			currency_id: T::CurrencyId,
			loyalty_curve: Option<LoyaltyCurve>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Return period number from block number now and number of blocks in one period
	pub fn get_period_number(
		now: T::BlockNumber,
		accumulate_period: T::BlockNumber,
	) -> Result<T::BlockNumber, Error<T>> {
		if accumulate_period.is_one() {
			return Ok(now);
		}

		now.checked_div(&accumulate_period).ok_or(Error::<T>::Overflow)
	}

	/// Loyalty multiplier  
	///
	// theta = periods/[(b + 1) * scale_coef];
	//
	// loyalty-multiplier = [theta + (theta * b) + b]/[theta + (theta * b) + 1]
	//
	pub fn get_loyalty_multiplier(periods: T::BlockNumber, curve: &LoyaltyCurve) -> Result<FixedU128, Error<T>> {
		//b.is_one() is special case
		if FixedPointNumber::is_one(&curve.b) {
			return Ok(1.into());
		}

		let denom = curve
			.b
			.checked_add(&1.into())
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(&FixedU128::from(Into::<u128>::into(curve.scale_coef)))
			.ok_or(Error::<T>::Overflow)?;

		let p = FixedU128::from(TryInto::<u128>::try_into(periods).map_err(|_e| Error::<T>::Overflow)?);
		let theta = p.checked_div(&denom).ok_or(Error::<T>::Overflow)?;

		let theta_mul_b = theta.checked_mul(&curve.b).ok_or(Error::<T>::Overflow)?;

		let theta_add_theta_mul_b = theta.checked_add(&theta_mul_b).ok_or(Error::<T>::Overflow)?;

		let num = theta_add_theta_mul_b
			.checked_add(&curve.b)
			.ok_or(Error::<T>::Overflow)?;

		let denom = theta_add_theta_mul_b
			.checked_add(&1.into())
			.ok_or(Error::<T>::Overflow)?;

		num.checked_div(&denom).ok_or(Error::<T>::Overflow)
	}

	pub fn get_reward_per_period(
		yield_per_period: FixedU128,
		total_global_farm_shares: T::Balance,
		max_reward_per_period: T::Balance,
	) -> Result<T::Balance, Error<T>> {
		Ok(yield_per_period
			.checked_mul_int(total_global_farm_shares)
			.ok_or(Error::<T>::Overflow)?
			.min(max_reward_per_period))
	}

	pub fn update_global_pool(
		pool: &mut GlobalPool<T::BlockNumber, T::Balance>,
		now_period: T::BlockNumber,
	) -> Result<(), Error<T>> {
		if pool.updated_at == now_period {
			return Ok(());
		}

		if pool.total_shares.is_zero() {
			return Ok(());
		}

		let periods_since_last_update = now_period.checked_sub(&pool.updated_at).ok_or(Error::<T>::Overflow)?;

		Err(Error::<T>::NotImplemented)
	}

	pub fn get_new_accumulated_rps(
		accumulated_rps_now: T::Balance,
		total_shares: T::Balance,
		reward: T::Balance,
	) -> Result<T::Balance, Error<T>> {
		reward
			.checked_div(&total_shares)
			.ok_or(Error::<T>::Overflow)?
			.checked_add(&accumulated_rps_now)
			.ok_or(Error::<T>::Overflow)
	}

	/// (user_rewards, unclaimable_rewards)
	/// NOTE: claimable_reward and user_rewards is not the same !!!
	pub fn get_user_reward(
		user_accumulated_rps: T::Balance,
		user_shares: T::Balance,
		accumulated_rps_now: T::Balance,
		user_accumulated_claimed_rewards: T::Balance,
		loyalty_multiplier: FixedU128,
	) -> Result<(T::Balance, T::Balance), Error<T>> {
		let max_rewards = accumulated_rps_now
			.checked_sub(&user_accumulated_rps)
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(&user_shares)
			.ok_or(Error::<T>::Overflow)?;

		let claimable_rewards = loyalty_multiplier
			.checked_mul_int(max_rewards)
			.ok_or(Error::<T>::Overflow)?;

		let unclaimable_rewards = max_rewards
			.checked_sub(&claimable_rewards)
			.ok_or(Error::<T>::Overflow)?;

		let user_rewards = claimable_rewards
			.checked_sub(&user_accumulated_claimed_rewards)
			.ok_or(Error::<T>::Overflow)?;

        Ok((user_rewards, unclaimable_rewards))
	}
}
