// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Stableswap pallet
//!
//! Curve/stableswap AMM implementation.
//!
//! ### Terminology
//!
//! * **LP** - liquidity provider
//! * **Share Token** - a token representing share asset of specific pool. Each pool has its own share token.
//! * **Amplification** - curve AMM pool amplification parameter
//!
//! ## Assumptions
//!
//! Maximum number of assets in pool is 5.
//!
//! A pool can be created only by allowed `CreatePoolOrigin`.
//!
//! First LP to provided liquidity must add initial liquidity of all pool assets. Subsequent calls to add_liquidity, LP can provide only 1 asset.
//!
//! Initial liquidity is first liquidity added to the pool (that is first call of `add_liquidity`).
//!
//! LP is given certain amount of shares by minting a pool's share token.
//!
//! When LP decides to withdraw liquidity, it receives selected asset.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::{DispatchResult, Get};
use frame_support::transactional;
use sp_runtime::Permill;
use sp_std::prelude::*;

pub use pallet::*;

pub mod traits;
pub mod types;
pub mod weights;

use crate::types::Balance;
use weights::WeightInfo;

#[cfg(test)]
pub(crate) mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarks;

/// Stableswap share token and account id identifier.
/// Used as identifier to create share token unique names and account ids.
pub const POOL_IDENTIFIER: &[u8] = b"sts";

pub const MAX_ASSETS_IN_POOL: u32 = 5;

const D_ITERATIONS: u8 = 128;
const Y_ITERATIONS: u8 = 64;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::traits::ShareAccountIdFor;
	use crate::types::{AssetLiquidity, Balance, PoolInfo};
	use codec::HasCompact;
	use core::ops::RangeInclusive;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use hydradx_traits::{Registry, ShareTokenRegistry};
	use orml_traits::MultiCurrency;
	use sp_runtime::traits::Zero;
	use sp_runtime::ArithmeticError;
	use sp_runtime::Permill;
	use sp_std::collections::btree_map::BTreeMap;

	#[pallet::pallet]
	#[pallet::generate_store(pub(crate) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Identifier for the class of asset.
		type AssetId: Member
			+ Parameter
			+ Ord
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		/// Multi currency mechanism
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = Self::AssetId, Balance = Balance>;

		/// Account ID constructor
		type ShareAccountId: ShareAccountIdFor<Vec<Self::AssetId>, AccountId = Self::AccountId>;

		/// Asset registry mechanism
		type AssetRegistry: ShareTokenRegistry<Self::AssetId, Vec<u8>, Balance, DispatchError>;

		/// The origin which can create a new pool
		type CreatePoolOrigin: EnsureOrigin<Self::Origin>;

		/// Precision used in Newton's method to solve math equations iteratively.
		#[pallet::constant]
		type Precision: Get<Balance>;

		/// Minimum pool liquidity
		#[pallet::constant]
		type MinPoolLiquidity: Get<Balance>;

		/// Minimum trading amount
		#[pallet::constant]
		type MinTradingLimit: Get<Balance>;

		/// Amplification inclusive range. Pool's amp can be selected from the range only.
		#[pallet::constant]
		type AmplificationRange: Get<RangeInclusive<u16>>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	/// Existing pools
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, PoolInfo<T::AssetId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A pool was created.
		PoolCreated {
			pool_id: T::AssetId,
			assets: Vec<T::AssetId>,
			amplification: u16,
			trade_fee: Permill,
			withdraw_fee: Permill,
		},
		/// Liquidity of an asset was added to a pool.
		LiquidityAdded {
			pool_id: T::AssetId,
			who: T::AccountId,
			shares: Balance,
			assets: Vec<AssetLiquidity<T::AssetId>>,
		},
		/// Liquidity removed.
		LiquidityRemoved {
			pool_id: T::AssetId,
			who: T::AccountId,
			shares: Balance,
			assets: (T::AssetId, T::AssetId),
			amounts: (Balance, Balance),
		},
		/// Sell trade executed. Trade fee paid in asset leaving the pool (already subtracted from amount_out).
		SellExecuted {
			who: T::AccountId,
			pool_id: T::AssetId,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount_in: Balance,
			amount_out: Balance,
			fee: Balance,
		},
		/// Buy trade executed. Trade fee paid in asset entering the pool (already included in amount_in).
		BuyExecuted {
			who: T::AccountId,
			pool_id: T::AssetId,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount_in: Balance,
			amount_out: Balance,
			fee: Balance,
		},
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		/// Creating a pool with same assets is not allowed.
		SameAssets,

		/// Maximum number of assets has been exceeded.
		MaxAssetsExceeded,

		/// A pool with given assets does not exist.
		PoolNotFound,

		/// A pool with given assets already exists.
		PoolExists,

		/// Asset is not in the pool.
		AssetNotInPool,

		/// One or more assets are not registered in AssetRegistry
		AssetNotRegistered,

		/// Invalid asset amount provided. Amount must be greater than zero.
		InvalidAssetAmount,

		/// Balance of an asset is not sufficient to perform a trade.
		InsufficientBalance,

		/// Balance of a share asset is not sufficient to withdraw liquidity.
		InsufficientShares,

		/// Liquidity has not reached the required minimum.
		InsufficientLiquidity,

		/// Insufficient liquidity left in the pool after withdrawal.
		InsufficientLiquidityRemaining,

		/// Amount is less than the minimum trading amount configured.
		InsufficientTradingAmount,

		/// Minimum limit has not been reached during trade.
		BuyLimitNotReached,

		/// Maximum limit has been exceeded during trade.
		SellLimitExceeded,

		/// Initial liquidity of asset must be > 0.
		InvalidInitialLiquidity,

		/// Account balance is too low.
		BalanceTooLow,

		/// Amplification is outside configured range.
		InvalidAmplification,

		/// Remaining balance of share asset is below asset's existential deposit.
		InsufficientShareBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a stableswap pool with given list of asset
		///
		/// All assets must be correctly registered in `T::AssetRegistry`.
		/// Note that this does not seed the pool with liquidity. Use `add_liquidity` to provide
		/// initial liquidity.
		///
		/// Parameters:
		/// - `origin`: Must be T::CreatePoolOrigin
		/// - `assets`: List of Asset ids
		/// - `amplification`: Pool amplification
		/// - `trade_fee`: trade fee to be applied in sell/buy trades
		/// - `withdraw_fee`: fee to be applied when removing liquidity
		///
		/// Emits `PoolCreated` event if successful.
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			assets: Vec<T::AssetId>,
			amplification: u16,
			trade_fee: Permill,
			withdraw_fee: Permill,
		) -> DispatchResult {
			T::CreatePoolOrigin::ensure_origin(origin)?;

			let mut pool_assets = assets;
			pool_assets.sort();

			let pool = PoolInfo {
				assets: pool_assets
					.clone()
					.try_into()
					.map_err(|_| Error::<T>::MaxAssetsExceeded)?,
				amplification,
				trade_fee,
				withdraw_fee,
			};
			ensure!(pool.is_valid(), Error::<T>::SameAssets);
			ensure!(
				T::AmplificationRange::get().contains(&amplification),
				Error::<T>::InvalidAmplification
			);
			for asset in pool.assets.iter() {
				ensure!(T::AssetRegistry::exists(*asset), Error::<T>::AssetNotRegistered);
			}
			let share_asset_ident = T::ShareAccountId::name(&pool.assets, Some(POOL_IDENTIFIER));
			let share_asset = T::AssetRegistry::get_or_create_shared_asset(
				share_asset_ident,
				pool.assets.clone().into(), //TODO: fix the trait to accept refeerence instead
				T::MinPoolLiquidity::get(),
			)?;

			Pools::<T>::try_mutate(&share_asset, |maybe_pool| -> DispatchResult {
				ensure!(maybe_pool.is_none(), Error::<T>::PoolExists);

				*maybe_pool = Some(pool);

				Ok(())
			})?;

			Self::deposit_event(Event::PoolCreated {
				pool_id: share_asset,
				assets: pool_assets,
				amplification,
				trade_fee,
				withdraw_fee,
			});

			Ok(())
		}

		/// Add liquidity to selected pool.
		///
		/// First call of `add_liquidity` adds "initial liquidity" of all assets.
		///
		/// If there is liquidity already in the pool, LP can provide liquidity of any number of pool assets.
		///
		/// LP must have sufficient amount of each assets.
		///
		/// Origin is given corresponding amount of shares.
		///
		/// Parameters:
		/// - `origin`: liquidity provider
		/// - `pool_id`: Pool Id
		/// - `assets`: asset id and liquidity amount provided
		///
		/// Emits `LiquidityAdded` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: T::AssetId,
			assets: Vec<AssetLiquidity<T::AssetId>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			ensure!(assets.len() <= pool.assets.len(), Error::<T>::MaxAssetsExceeded);
			let mut added_assets = BTreeMap::<T::AssetId, Balance>::new();
			for asset in assets.iter() {
				ensure!(
					asset.amount >= T::MinTradingLimit::get(),
					Error::<T>::InsufficientTradingAmount
				);
				ensure!(
					T::Currency::free_balance(asset.asset_id, &who) >= asset.amount,
					Error::<T>::InsufficientBalance
				);
				ensure!(pool.find_asset(asset.asset_id).is_some(), Error::<T>::AssetNotInPool);
				added_assets.insert(asset.asset_id, asset.amount);
			}

			let pool_account = pool.pool_account::<T>();
			let mut initial_reserves = Vec::new();
			let mut updated_reserves = Vec::new();
			for pool_asset in pool.assets.iter() {
				let reserve = T::Currency::free_balance(*pool_asset, &pool_account);
				initial_reserves.push(reserve);
				if let Some(liq_added) = added_assets.get(&pool_asset) {
					updated_reserves.push(reserve.checked_add(*liq_added).ok_or(ArithmeticError::Overflow)?);
				} else {
					ensure!(!reserve.is_zero(), Error::<T>::InvalidInitialLiquidity);
					updated_reserves.push(reserve);
				}
			}

			let share_issuance = T::Currency::total_issuance(pool_id);
			let share_amount = hydra_dx_math::stableswap::calculate_shares::<D_ITERATIONS>(
				&initial_reserves,
				&updated_reserves,
				pool.amplification.into(),
				T::Precision::get(),
				share_issuance,
			)
			.ok_or(ArithmeticError::Overflow)?;
			ensure!(!share_amount.is_zero(), Error::<T>::InvalidAssetAmount);
			let current_share_balance = T::Currency::free_balance(pool_id, &who);
			ensure!(
				current_share_balance.saturating_add(share_amount) >= T::MinPoolLiquidity::get(),
				Error::<T>::InsufficientShareBalance
			);

			T::Currency::deposit(pool_id, &who, share_amount)?;
			for asset in assets.iter() {
				T::Currency::transfer(asset.asset_id, &who, &pool_account, asset.amount)?;
			}

			Self::deposit_event(Event::LiquidityAdded {
				pool_id,
				who,
				shares: share_amount,
				assets,
			});

			Ok(())
		}

		/// Remove liquidity from selected pool.
		///
		/// Withdraws liquidity of selected asset from a pool.
		///
		/// Sahre amount is burn and LP receives corresponding amount of chosen asset.
		///
		/// Withdraw fee is applied to the asset amount.
		///
		/// Parameters:
		/// - `origin`: liquidity provider
		/// - `pool_id`: Pool Id
		/// - `asset_id`: id of asset to receive
		/// - 'share_amount': amount of shares to withdraw
		///
		/// Emits `LiquidityRemoved` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::remove_liquidity_one_asset())]
		#[transactional]
		pub fn remove_liquidity_one_asset(
			origin: OriginFor<T>,
			pool_id: T::AssetId,
			asset_id: T::AssetId,
			share_amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(share_amount > Balance::zero(), Error::<T>::InvalidAssetAmount);

			let current_share_balance = T::Currency::free_balance(pool_id, &who);

			ensure!(current_share_balance >= share_amount, Error::<T>::InsufficientShares);

			ensure!(
				current_share_balance == share_amount
					|| current_share_balance.saturating_sub(share_amount) >= T::MinPoolLiquidity::get(),
				Error::<T>::InsufficientShareBalance
			);

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			let asset_idx = pool.find_asset(asset_id).ok_or(Error::<T>::AssetNotInPool)?;
			let pool_account = pool.pool_account::<T>();
			let balances = pool.balances::<T>();
			let share_issuance = T::Currency::total_issuance(pool_id);

			ensure!(
				share_issuance.saturating_sub(share_amount) >= T::MinPoolLiquidity::get(),
				Error::<T>::InsufficientLiquidityRemaining
			);

			let (amount, _fee) = hydra_dx_math::stableswap::calculate_withdraw_one_asset::<D_ITERATIONS, Y_ITERATIONS>(
				&balances,
				share_amount,
				asset_idx,
				share_issuance,
				pool.amplification.into(),
				T::Precision::get(),
			)
			.ok_or(ArithmeticError::Overflow)?;

			// TODO: what do with fee?

			T::Currency::withdraw(pool_id, &who, share_amount)?;
			T::Currency::transfer(asset_id, &pool_account, &who, amount)?;

			/*
			Self::deposit_event(Event::LiquidityRemoved {
				pool_id,
				who,
				shares: amount,
				assets: (pool.assets[0], pool.assets[1]),
				amounts: (amounts.0, amounts.1),
			});
			 */

			Ok(())
		}

		/// Execute a swap of `asset_in` for `asset_out` by specifying how much to put in.
		///
		/// Parameters:
		/// - `origin`: origin of the caller
		/// - `pool_id`: Id of a pool
		/// - `asset_in`: ID of asset sold to the pool
		/// - `asset_out`: ID of asset bought from the pool
		/// - `amount_in`: Amount of asset to be sold to the pool
		/// - `min_buy_amount`: Minimum amount required to receive
		///
		/// Emits `SellExecuted` event when successful.
		///
		#[pallet::weight(<T as Config>::WeightInfo::sell())]
		#[transactional]
		pub fn sell(
			origin: OriginFor<T>,
			pool_id: T::AssetId,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount_in: Balance,
			min_buy_amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				amount_in >= T::MinTradingLimit::get(),
				Error::<T>::InsufficientTradingAmount
			);

			ensure!(
				T::Currency::free_balance(asset_in, &who) >= amount_in,
				Error::<T>::InsufficientBalance
			);

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			let index_in = pool.find_asset(asset_in).ok_or(Error::<T>::AssetNotInPool)?;
			let index_out = pool.find_asset(asset_out).ok_or(Error::<T>::AssetNotInPool)?;

			let balances = pool.balances::<T>();

			ensure!(balances[index_in] > Balance::zero(), Error::<T>::InsufficientLiquidity);
			ensure!(balances[index_out] > Balance::zero(), Error::<T>::InsufficientLiquidity);

			let amount_out = hydra_dx_math::stableswap::calculate_out_given_in::<D_ITERATIONS, Y_ITERATIONS>(
				&balances,
				index_in,
				index_out,
				amount_in,
				pool.amplification.into(),
				T::Precision::get(),
			)
			.ok_or(ArithmeticError::Overflow)?;

			let pool_account = pool.pool_account::<T>();

			let fee_amount = Self::calculate_fee_amount(amount_out, pool.trade_fee, Rounding::Down);

			let amount_out = amount_out.checked_sub(fee_amount).ok_or(ArithmeticError::Underflow)?;

			ensure!(amount_out >= min_buy_amount, Error::<T>::BuyLimitNotReached);

			T::Currency::transfer(asset_in, &who, &pool_account, amount_in)?;
			T::Currency::transfer(asset_out, &pool_account, &who, amount_out)?;

			Self::deposit_event(Event::SellExecuted {
				who,
				pool_id,
				asset_in,
				asset_out,
				amount_in,
				amount_out,
				fee: fee_amount,
			});

			Ok(())
		}

		/// Execute a swap of `asset_in` for `asset_out` by specifying how much to get out.
		///
		/// Parameters:
		/// - `origin`:
		/// - `pool_id`: Id of a pool
		/// - `asset_out`: ID of asset bought from the pool
		/// - `asset_in`: ID of asset sold to the pool
		/// - `amount_out`: Amount of asset to receive from the pool
		/// - `max_sell_amount`: Maximum amount allowed to be sold
		///
		/// Emits `BuyExecuted` event when successful.
		///
		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		#[transactional]
		pub fn buy(
			origin: OriginFor<T>,
			pool_id: T::AssetId,
			asset_out: T::AssetId,
			asset_in: T::AssetId,
			amount_out: Balance,
			max_sell_amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				amount_out >= T::MinTradingLimit::get(),
				Error::<T>::InsufficientTradingAmount
			);

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			let index_in = pool.find_asset(asset_in).ok_or(Error::<T>::AssetNotInPool)?;
			let index_out = pool.find_asset(asset_out).ok_or(Error::<T>::AssetNotInPool)?;

			let pool_account = pool.pool_account::<T>();

			let balances = pool.balances::<T>();

			ensure!(balances[index_out] > amount_out, Error::<T>::InsufficientLiquidity);
			ensure!(balances[index_in] > Balance::zero(), Error::<T>::InsufficientLiquidity);

			let amount_in = hydra_dx_math::stableswap::calculate_in_given_out::<D_ITERATIONS, Y_ITERATIONS>(
				&balances,
				index_in,
				index_out,
				amount_out,
				pool.amplification.into(),
				T::Precision::get(),
			)
			.ok_or(ArithmeticError::Overflow)?;

			let fee_amount = Self::calculate_fee_amount(amount_in, pool.trade_fee, Rounding::Up);

			let amount_in = amount_in.checked_add(fee_amount).ok_or(ArithmeticError::Overflow)?;

			ensure!(amount_in <= max_sell_amount, Error::<T>::SellLimitExceeded);

			ensure!(
				T::Currency::free_balance(asset_in, &who) >= amount_in,
				Error::<T>::InsufficientBalance
			);

			T::Currency::transfer(asset_in, &who, &pool_account, amount_in)?;
			T::Currency::transfer(asset_out, &pool_account, &who, amount_out)?;

			Self::deposit_event(Event::BuyExecuted {
				who,
				pool_id,
				asset_in,
				asset_out,
				amount_in,
				amount_out,
				fee: fee_amount,
			});

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}

/// Rounding method to use
enum Rounding {
	Down,
	Up,
}

impl<T: Config> Pallet<T> {
	fn calculate_fee_amount(amount: Balance, fee: Permill, rounding: Rounding) -> Balance {
		match rounding {
			Rounding::Down => fee.mul_floor(amount),
			Rounding::Up => fee.mul_ceil(amount),
		}
	}
}
