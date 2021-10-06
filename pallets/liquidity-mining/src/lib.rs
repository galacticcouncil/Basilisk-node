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
//  - lm - liquidity minign
//  - lp - liquidity provider
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
	ensure,
	sp_runtime::traits::{One, Zero},
	traits::{Get, LockIdentifier},
	transactional,
};

use frame_system::ensure_signed;
use sp_arithmetic::{
	traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
	FixedU128,
};

use orml_traits::{MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency};

use codec::{Decode, Encode, HasCompact};
use frame_support::sp_runtime::RuntimeDebug;

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct LmPool<Balance, CurrencyId, Period> {
	accumulated_reward_per_share: FixedU128,
	total_locked_shares: Balance,
	share_id: CurrencyId,
	updated_at: Period,
	unpaid_rewards: Balance,
	paid_rewards: Balance,
	canceled: bool,
	loyalty_curve: LoyaltyCurve,
}

impl Default for LoyaltyCurve {
	fn default() -> Self {
		Self {
			b: FixedU128::from_inner(500_000_000_000_000_000),
			scale_coef: 100,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct LoyaltyCurve {
	b: FixedU128,
	scale_coef: u32,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, Default)]
pub struct LpInfo<Balance: HasCompact> {
	reward_per_share: FixedU128,
	locked_shares: Balance,
	claimed_rewards: Balance, //Rewards claim till now
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

		/// The account holding payable rewards for all existing LM pools
		#[pallet::constant]
		type LmAccount: Get<Self::AccountId>;

		/// Number of blocks per periond. If 1 then period is 1 block
		type AccumulatePeriod: Get<Self::BlockNumber>;

		/// Currency id of reward
		#[pallet::constant]
		type PayoutCurrencyId: Get<Self::CurrencyId>;

		/// Administrator able to manage liquidity minig e.g create/cancel/destroy pools
		type AdminOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// Math computation overflow
		Overflow,

		/// Liquidity mining pool alerady exist
		LmPoolExists,

		/// Shares balance is not sufficient
		InsufficientShareBalance,

		/// Liquidity mining pool does not exist
		PoolNotFound,

		/// Accont depositing shares second time
		DoubleDepositPerAccount,

		/// Deposit not found for combination account + pool_id
		DepositNotFound,

		/// Feature is not implemented yet
		NotImplemented,

		/// Liquidity mining pool is not canceled
		PoolNotCanceled,

		/// Liquidity mining pool have some locked shares
		PoolHaveShares,

		/// Liquidity mining in this pool was canceled
		PoolCanceled,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New liquidity mining pool was created [pool id]
		PoolCreated(PoolId<T>),

		/// Account chlamied redards from liquidity mining. [who, liquidity mining pool id, claimed amount]
		RewaredClaimed(T::AccountId, PoolId<T>, T::Balance),

		/// Shares was withdrawn from liquidity mining. [who, liquidity mining pool id, amount of withdrawn shares]
		SharesWithdrawned(T::AccountId, PoolId<T>, T::Balance),

		/// Liquidity mining pool was canceled and run in limitted mode. [pool id]
		PoolCanceled(PoolId<T>),

		/// Liquidity mining pool was destroyed [pool id]
		PoolDestroyed(PoolId<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn liq_provider)]
	pub type LiqProviders<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		PoolId<T>,
		Blake2_128Concat,
		T::AccountId,
		LpInfo<T::Balance>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pool)]
	pub type Pools<T: Config> =
		StorageMap<_, Blake2_128Concat, PoolId<T>, LmPool<T::Balance, T::CurrencyId, T::BlockNumber>, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000)]
		#[transactional]
		pub fn deposit_shares(origin: OriginFor<T>, pool_id: PoolId<T>, amount: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//TODO: check potencial unnecessary write in case of pool not need update in this
			//period
			LiqProviders::<T>::try_mutate(&pool_id, &who, |lp| -> DispatchResult {
				ensure!(lp.is_none(), Error::<T>::DoubleDepositPerAccount);

				Pools::<T>::try_mutate(&pool_id, |lm_p| -> DispatchResult {
					ensure!(lm_p.is_some(), Error::<T>::PoolNotFound);
					let lm_pool = lm_p.as_mut().unwrap(); //safe because of check above

					ensure!(!lm_pool.canceled, Error::<T>::PoolCanceled);

					ensure!(
						T::MultiCurrency::free_balance(lm_pool.share_id, &who) >= amount,
						Error::<T>::InsufficientShareBalance
					);

					Self::update_pool(pool_id.clone(), lm_pool)?;

					lm_pool.total_locked_shares = lm_pool
						.total_locked_shares
						.checked_add(&amount)
						.ok_or(Error::<T>::Overflow)?;

					*lp = Some(LpInfo {
						reward_per_share: lm_pool.accumulated_reward_per_share,
						locked_shares: amount,
						claimed_rewards: T::Balance::default(),
					});

					T::MultiCurrency::extend_lock(LM_LOCK_ID, lm_pool.share_id, &who, amount)
				})
			})
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn claim_rewards(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut claimed_rewards = T::Balance::default();

			LiqProviders::<T>::try_mutate(&pool_id, &who, |liq_provider| -> DispatchResult {
				ensure!(liq_provider.is_some(), Error::<T>::DepositNotFound);
				let lp = liq_provider.as_mut().unwrap(); //safe bacause of check above

				Pools::<T>::try_mutate(&pool_id, |lm_pool| -> DispatchResult {
					let pool = lm_pool.as_mut().unwrap();

					ensure!(!pool.canceled, Error::<T>::PoolCanceled);

					Self::update_pool(pool_id.clone(), pool)?;

					claimed_rewards = Self::do_claim_rewards(who.clone(), lp, pool)?;

					Ok(())
				})
			})?;

			Self::deposit_event(Event::RewaredClaimed(who, pool_id, claimed_rewards));

			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn withdraw_shares(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut withdrawned_shares = T::Balance::default();
			let mut claimed_rewards = T::Balance::default();

			LiqProviders::<T>::try_mutate_exists(&pool_id, &who, |liq_provider| -> DispatchResult {
				ensure!(liq_provider.is_some(), Error::<T>::DepositNotFound);
				let lp = liq_provider.as_mut().unwrap(); //safe bacause of check above

				Pools::<T>::try_mutate(&pool_id, |lm_pool| -> DispatchResult {
					let pool = lm_pool.as_mut().unwrap();

					Self::update_pool(pool_id.clone(), pool)?;

					claimed_rewards = Self::do_claim_rewards(who.clone(), lp, pool)?;

					pool.total_locked_shares = pool
						.total_locked_shares
						.checked_sub(&lp.locked_shares)
						.ok_or(Error::<T>::Overflow)?
						.min(T::Balance::from(0_u128));

					withdrawned_shares = lp.locked_shares;
					T::MultiCurrency::remove_lock(LM_LOCK_ID, pool.share_id, &who)
				})?;

				*liq_provider = None;

				Ok(())
			})?;

			//deposit claim & withdraw events
			Self::deposit_event(Event::RewaredClaimed(who.clone(), pool_id.clone(), claimed_rewards));
			Self::deposit_event(Event::SharesWithdrawned(who, pool_id, withdrawned_shares));

			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn cancel_pool(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;

			Pools::<T>::try_mutate(&pool_id, |lm_pool| -> DispatchResult {
				ensure!(lm_pool.is_some(), Error::<T>::PoolNotFound);
				let pool = lm_pool.as_mut().unwrap();

				ensure!(!pool.canceled, Error::<T>::PoolCanceled);

				Self::update_pool(pool_id.clone(), pool)?;

				pool.canceled = true;

				Ok(())
			})?;

			Self::deposit_event(Event::PoolCanceled(pool_id));

			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn destroy_pool(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;

			Pools::<T>::try_mutate_exists(&pool_id, |lm_pool| -> DispatchResult {
				ensure!(lm_pool.is_some(), Error::<T>::PoolNotFound);
				let pool = lm_pool.as_mut().unwrap();

				ensure!(pool.canceled, Error::<T>::PoolNotCanceled);

				ensure!(pool.total_locked_shares.is_zero(), Error::<T>::PoolHaveShares);

				*lm_pool = None;

				Ok(())
			})?;

			Self::deposit_event(Event::PoolDestroyed(pool_id));

			Ok(())
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			currency_id: T::CurrencyId,
			loyalty_curve: Option<LoyaltyCurve>,
		) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;

			Pools::<T>::try_mutate(&pool_id, |lm_pool| -> DispatchResult {
				ensure!(lm_pool.is_none(), Error::<T>::LmPoolExists);

				let now_period =
					Self::get_period_number(<frame_system::Pallet<T>>::block_number(), T::AccumulatePeriod::get())?;

				let loyalty_curve = match loyalty_curve {
					Some(v) => v,
					None => LoyaltyCurve::default(),
				};

				*lm_pool = Some(LmPool {
					accumulated_reward_per_share: FixedU128::default(),
					total_locked_shares: T::Balance::default(),
					share_id: currency_id,
					updated_at: now_period,
					unpaid_rewards: T::Balance::default(),
					paid_rewards: T::Balance::default(),
					canceled: false,
					loyalty_curve,
				});

				Ok(())
			})?;

			Self::deposit_event(Event::PoolCreated(pool_id));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_claim_rewards(
		who: T::AccountId,
		lp: &mut LpInfo<T::Balance>,
		pool: &mut LmPool<T::Balance, T::CurrencyId, T::BlockNumber>,
	) -> Result<T::Balance, DispatchError> {
		let lp_reward = Self::calculate_rewards(lp, pool.accumulated_reward_per_share, &pool.loyalty_curve)?;

		pool.paid_rewards = pool.paid_rewards.checked_add(&lp_reward).ok_or(Error::<T>::Overflow)?;
		pool.unpaid_rewards = pool
			.unpaid_rewards
			.checked_sub(&lp_reward)
			.ok_or(Error::<T>::Overflow)?;

		lp.claimed_rewards = lp.claimed_rewards.checked_add(&lp_reward).ok_or(Error::<T>::Overflow)?;

		match T::MultiCurrency::transfer(T::PayoutCurrencyId::get(), &T::LmAccount::get(), &who, lp_reward) {
			Ok(_) => return Ok(lp_reward),
			Err(e) => return Err(e),
		};
	}

	/// reward calculation:
	/// (pool_accumulated_reward_per_share - lp_reward_per_share) * lp_shares * lp_loyalty_multiplier - lp_claimed_reward_til_now
	pub fn calculate_rewards(
		lp: &LpInfo<T::Balance>,
		global_accumulated_reward_per_share: FixedU128,
		loyalty_curve_opts: &LoyaltyCurve,
	) -> Result<T::Balance, Error<T>> {
		let now = <frame_system::Pallet<T>>::block_number();
		let period_now = Self::get_period_number(now, T::AccumulatePeriod::get())?;

		let loyalty_multiplier = Self::get_loyalty_multiplier(period_now, loyalty_curve_opts)?;

		//(global_accumulated_reward_per_share - reward_per_share) * loyalty_weight * locked_shares - already_claimed_rewards
		global_accumulated_reward_per_share
			.checked_sub(&lp.reward_per_share)
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(&loyalty_multiplier)
			.ok_or(Error::<T>::Overflow)?
			.checked_mul_int(lp.locked_shares)
			.ok_or(Error::<T>::Overflow)?
			.checked_sub(&lp.claimed_rewards)
			.ok_or(Error::<T>::Overflow)
	}

	pub fn update_pool(
		pool_id: PoolId<T>,
		pool: &mut LmPool<T::Balance, T::CurrencyId, T::BlockNumber>,
	) -> Result<(), Error<T>> {
		if pool.canceled {
			return Ok(());
		}

		let now = <frame_system::Pallet<T>>::block_number();
		let period_now = Self::get_period_number(now, T::AccumulatePeriod::get())?;
		
        // Do nothing if pool was udpated in this period
		if pool.updated_at == period_now {
			return Ok(());
		}

		//count number of periods since last update
		let periods = period_now - pool.updated_at;
        println!("period now: {:?}", period_now);
		//get rewards for all "untouched" periods
		let rewards = Self::compute_rewards(pool_id, pool, periods)?;

		pool.unpaid_rewards = pool.unpaid_rewards.checked_add(&rewards).ok_or(Error::<T>::Overflow)?;
println!("acc reward {:?}", Self::get_accumulated_reward_per_share(&pool, rewards).unwrap());
		pool.accumulated_reward_per_share = Self::get_accumulated_reward_per_share(&pool, rewards)?;

		pool.updated_at = period_now;

		Ok(())
	}

	pub fn get_accumulated_reward_per_share(
		pool: &LmPool<T::Balance, T::CurrencyId, T::BlockNumber>,
		reward: T::Balance,
	) -> Result<FixedU128, Error<T>> {
		let reward_per_share = FixedU128::from(Into::<u128>::into(reward))
			.checked_div(&FixedU128::from(Into::<u128>::into(pool.total_locked_shares)))
			.ok_or(Error::<T>::Overflow)?;
       
        println!("reward: {:?}, total_locked: {:?}", reward, pool.total_locked_shares);

		pool.accumulated_reward_per_share
			.checked_add(&reward_per_share)
			.ok_or(Error::<T>::Overflow)
	}

	/// This method compute accumulated rewards for X number of periods.
	///
	/// TODO: add real reward calculations
	pub fn compute_rewards(
		pool_id: PoolId<T>,
		pool: &LmPool<T::Balance, T::CurrencyId, T::BlockNumber>,
		periods_count: T::BlockNumber,
	) -> Result<T::Balance, Error<T>> {
		let outstanding_rewards = Self::get_accumulated_reward_for_periods(periods_count)?;

println!("periods: {:?}, outstanding_rewards: {:?}", periods_count, outstanding_rewards);

		let claimed_rewards = pool
			.unpaid_rewards
			.checked_add(&pool.paid_rewards)
			.ok_or(Error::<T>::Overflow)?;
println!("claimed_rewards: {:?}", claimed_rewards);
		let planned_rewards = Self::get_total_planned_rewards(pool_id)?;
println!("planned_rewards: {:?}", planned_rewards);
		let total_rewards = claimed_rewards
			.checked_add(&outstanding_rewards)
			.ok_or(Error::<T>::Overflow)?;

println!("total_rewards: {:?}", total_rewards);
		if total_rewards > planned_rewards {
            //TODO: add abs
			claimed_rewards
				.checked_sub(&planned_rewards)
				.ok_or(Error::<T>::Overflow)
		} else {
			Ok(outstanding_rewards)
		}
	}

	pub fn get_total_planned_rewards(_pool_id: PoolId<T>) -> Result<T::Balance, Error<T>> {
		//TODO: add real compuation
		Ok(T::Balance::from(1_000_000_000_000_000_000_000_000u128))
	}

	// TODO: this method should call real reward function
	pub fn get_accumulated_reward_for_periods(periods_count: T::BlockNumber) -> Result<T::Balance, Error<T>> {
		let p = T::Balance::from(periods_count);

		//TODO: add real computation
		T::Balance::from(2_000_000_000_000_000_u128).checked_mul(&p).ok_or(Error::<T>::Overflow)
	}

	pub fn get_period_number(
		now: T::BlockNumber,
		accumulate_period: T::BlockNumber,
	) -> Result<T::BlockNumber, Error<T>> {
		if accumulate_period.is_one() {
			return Ok(now);
		}

		now.checked_div(&accumulate_period).ok_or(Error::<T>::Overflow)
	}

	/// Loyalty ponus function
	pub fn get_loyalty_multiplier(period: T::BlockNumber, curve_opts: &LoyaltyCurve) -> Result<FixedU128, Error<T>> {
		let denom = curve_opts.b.checked_add(&1.into()).ok_or(Error::<T>::Overflow)?;
		let p: u128 = period.try_into().map_err(|_e| Error::<T>::Overflow)?;

		let t = FixedU128::from(p).checked_div(&denom).ok_or(Error::<T>::Overflow)?;

		let tb = t.checked_mul(&curve_opts.b).ok_or(Error::<T>::Overflow)?;

		let t_add_tb = t.checked_add(&tb).ok_or(Error::<T>::Overflow)?;

		let num = t_add_tb.checked_add(&curve_opts.b).ok_or(Error::<T>::Overflow)?;

		let denom = t_add_tb.checked_add(&1.into()).ok_or(Error::<T>::Overflow)?;

		num.checked_div(&denom).ok_or(Error::<T>::Overflow)
	}
}
