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
//! TBD
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::{DispatchResult, Get};
use frame_support::transactional;
use sp_runtime::Permill;

pub use pallet::*;

mod math;
mod traits;
mod types;
pub mod weights;

use crate::types::Balance;
use weights::WeightInfo;

#[cfg(test)]
pub(crate) mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarks;

const POOL_IDENTIFIER: &str = "sts";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::math::two_asset_pool_math::{
		calculate_add_liquidity_shares, calculate_asset_b_reserve, calculate_in_given_out, calculate_out_given_in,
		calculate_remove_liquidity_amounts,
	};
	use crate::traits::ShareAccountIdFor;
	use crate::types::{AssetAmounts, Balance, PoolAssets, PoolId, PoolInfo};
	use codec::HasCompact;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use hydradx_traits::{Registry, ShareTokenRegistry};
	use orml_traits::MultiCurrency;
	use sp_runtime::traits::Zero;
	use sp_runtime::ArithmeticError;
	use sp_runtime::Permill;

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
			+ PartialOrd
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		/// Multi currency mechanism
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = Self::AssetId, Balance = Balance>;

		/// Account ID constructor
		type ShareAccountId: ShareAccountIdFor<PoolAssets<Self::AssetId>, AccountId = Self::AccountId>;

		/// Asset registry mechnanism
		type AssetRegistry: ShareTokenRegistry<Self::AssetId, Vec<u8>, Balance, DispatchError>;

		/// The origin which can create a new pool
		type CreatePoolOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		/// Precision used in Newton's method to solve math equations iteratively.
		#[pallet::constant]
		type Precision: Get<Balance>;

		/// Minimum pool liquidity
		#[pallet::constant]
		type MinimumLiquidity: Get<Balance>;

		/// Minimum trading amount
		#[pallet::constant]
		type MinimumTradingLimit: Get<Balance>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	/// Existing pools
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T::AssetId>, PoolInfo<T::AssetId, Balance>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A pool was created.
		PoolCreated {
			id: PoolId<T::AssetId>,
			assets: (T::AssetId, T::AssetId),
			initial_liquidity: (Balance, Balance),
			amplification: Balance,
		},
		/// Liquidity of an asset was added to a pool.
		LiquidityAdded {
			id: PoolId<T::AssetId>,
			from: T::AccountId,
			assets: (T::AssetId, T::AssetId),
			amounts: (Balance, Balance),
		},
		/// Liquidity removed.
		LiquidityRemoved {
			id: PoolId<T::AssetId>,
			who: T::AccountId,
			shares: Balance,
			amounts: (Balance, Balance),
		},
		/// Sell trade executed.
		SellExecuted {
			who: T::AccountId,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount_in: Balance,
			amount_out: Balance,
			fee: Balance,
		},
		/// Buy trade executed.
		BuyExecuted {
			who: T::AccountId,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount_in: Balance,
			amount_out: Balance,
		},
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		/// Creating a pool with same assets is not allowed.
		SameAssets,

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

		/// Balance of an asset is nto sufficient to perform a trade.
		InsufficientBalance,

		/// Balance of an share asset is nto sufficient to withdraw liquidity.
		InsufficientShares,

		/// Liquidity has not reached the required minimum.
		InsufficientLiquidity,

		/// Insufficient liquidity left in the pool after withdrawal.
		InsufficientLiquidityRemaining,

		/// Amount is less than min trading limit.
		InsufficientTradingAmount,

		/// Minimum limit has not been reached during trade.
		BuyLimitNotReached,

		/// Maximum limit has been exceeded during trade.
		SellLimitExceeded,

		/// Initial liquidity of asset must be > 0.
		InvalidInitialLiquidity,

		/// Account balance is too low.
		BalanceTooLow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a 2-asset pool with initial liquidity for both assets.
		///
		/// Both assets must be correctly registered in `T::AssetRegistry`
		///
		/// Initial liquidity must be > 0.
		///
		/// Origin is given corresponding amount of shares.
		///
		/// Parameters:
		/// - `origin`: Must be T::CreatePoolOrigin
		/// - `assets`: Asset ids tuple
		/// - `initial_liquidity`: Corresponding initial liquidity of `assets`
		/// - `amplification`: Pool amplification
		/// - `fee`: trade fee to be applied in sell/buy trades
		///
		/// Emits `PoolCreated` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			assets: (T::AssetId, T::AssetId),
			initial_liquidity: (Balance, Balance),
			amplification: Balance,
			fee: Permill,
		) -> DispatchResult {
			let who = T::CreatePoolOrigin::ensure_origin(origin)?;

			let pool_assets: PoolAssets<T::AssetId> = assets.into();

			ensure!(pool_assets.is_valid(), Error::<T>::SameAssets);

			for asset in (&pool_assets).into_iter() {
				ensure!(T::AssetRegistry::exists(asset), Error::<T>::AssetNotRegistered);
			}

			ensure!(
				initial_liquidity.0 > Balance::zero(),
				Error::<T>::InvalidInitialLiquidity
			);
			ensure!(
				initial_liquidity.1 > Balance::zero(),
				Error::<T>::InvalidInitialLiquidity
			);

			ensure!(
				T::Currency::free_balance(assets.0, &who) >= initial_liquidity.0,
				Error::<T>::BalanceTooLow,
			);
			ensure!(
				T::Currency::free_balance(assets.1, &who) >= initial_liquidity.1,
				Error::<T>::BalanceTooLow,
			);

			let share_asset_ident = T::ShareAccountId::name(&pool_assets, Some(POOL_IDENTIFIER));

			let share_asset = T::AssetRegistry::get_or_create_shared_asset(
				share_asset_ident,
				(&pool_assets).into(),
				Balance::zero(),
			)?;

			let pool_id = PoolId(share_asset);

			let pool = Pools::<T>::try_mutate(
				&pool_id,
				|maybe_pool| -> Result<PoolInfo<T::AssetId, Balance>, DispatchError> {
					ensure!(maybe_pool.is_none(), Error::<T>::PoolExists);

					let pool = PoolInfo {
						assets: pool_assets.clone(),
						amplification,
						fee,
					};

					*maybe_pool = Some(pool.clone());

					Ok(pool)
				},
			)?;

			let reserves: AssetAmounts<Balance> = initial_liquidity.into();

			let share_amount = calculate_add_liquidity_shares(
				&AssetAmounts::default(),
				&reserves,
				T::Precision::get(),
				pool.amplification,
				Balance::zero(),
			)
			.ok_or(ArithmeticError::Overflow)?;

			ensure!(
				share_amount >= T::MinimumLiquidity::get(),
				Error::<T>::InsufficientLiquidity
			);

			let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

			T::Currency::transfer(assets.0, &who, &pool_account, initial_liquidity.0)?;
			T::Currency::transfer(assets.1, &who, &pool_account, initial_liquidity.1)?;

			T::Currency::deposit(pool_id.0, &who, share_amount)?;

			Self::deposit_event(Event::PoolCreated {
				id: pool_id,
				assets,
				initial_liquidity,
				amplification,
			});

			Ok(())
		}

		/// Add liquidity to selected 2-asset pool.
		///
		/// LP must provide liquidity of both assets by specifying amount of asset a.
		///
		/// Amount of asset b is calculated so the ratio does not change:
		///
		/// new_reserve_b = (reserve_a + amount) * reserve_b / reserve_a
		/// amount_b = new_reserve_b - reserve_b
		///
		/// LP must have sufficient amount of asset a/b - amount_a and amount_b.
		///
		/// Origin is given corresponding amount of shares.
		///
		/// Parameters:
		/// - `origin`: Must be T::CreatePoolOrigin
		/// - `pool_id`: Pool Id
		/// - `asset`: Asset id
		/// - `amount`: liquidity amount of `asset` to be added to the pool.
		///
		/// Emits `LiquidityAdded` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolId<T::AssetId>,
			asset: T::AssetId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				amount >= T::MinimumTradingLimit::get(),
				Error::<T>::InsufficientTradingAmount
			);

			// Ensure that who has enough balance of given asset
			ensure!(
				T::Currency::free_balance(asset, &who) >= amount,
				Error::<T>::InsufficientBalance
			);

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			ensure!(pool.contains_asset(asset), Error::<T>::AssetNotInPool);

			// Load initial pool assets balances.
			let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));
			let initial_reserves = AssetAmounts(
				T::Currency::free_balance(pool.assets.0, &pool_account),
				T::Currency::free_balance(pool.assets.1, &pool_account),
			);

			// Work out which asset's amount has to be calculated based on given provided asset by LP
			// Calculate correct amount of second asset which LP has to provided too.
			// Update initial reserves.
			let (asset_b_id, asset_b_amount, new_reserves) = if asset == pool.assets.0 {
				let asset_reserve = initial_reserves
					.0
					.checked_add(amount)
					.ok_or(ArithmeticError::Overflow)?;

				let asset_b_reserve = calculate_asset_b_reserve(initial_reserves.1, initial_reserves.0, asset_reserve)
					.ok_or(ArithmeticError::Overflow)?;
				(
					pool.assets.1,
					asset_b_reserve
						.checked_sub(initial_reserves.1)
						.ok_or(ArithmeticError::Underflow)?,
					AssetAmounts(asset_reserve, asset_b_reserve),
				)
			} else {
				let asset_reserve = initial_reserves
					.1
					.checked_add(amount)
					.ok_or(ArithmeticError::Overflow)?;

				let asset_b_reserve = calculate_asset_b_reserve(initial_reserves.0, initial_reserves.1, asset_reserve)
					.ok_or(ArithmeticError::Overflow)?;
				(
					pool.assets.0,
					asset_b_reserve
						.checked_sub(initial_reserves.0)
						.ok_or(ArithmeticError::Underflow)?,
					AssetAmounts(asset_b_reserve, asset_reserve),
				)
			};

			ensure!(
				T::Currency::free_balance(asset_b_id, &who) >= asset_b_amount,
				Error::<T>::InsufficientBalance
			);

			let share_issuance = T::Currency::total_issuance(pool_id.0);

			let share_amount = calculate_add_liquidity_shares(
				&initial_reserves,
				&new_reserves,
				T::Precision::get(),
				pool.amplification,
				share_issuance,
			)
			.ok_or(ArithmeticError::Overflow)?;

			T::Currency::deposit(pool_id.0, &who, share_amount)?;

			T::Currency::transfer(asset, &who, &pool_account, amount)?;
			T::Currency::transfer(asset_b_id, &who, &pool_account, asset_b_amount)?;

			Self::deposit_event(Event::LiquidityAdded {
				id: pool_id,
				from: who,
				assets: (asset, asset_b_id),
				amounts: (amount, asset_b_amount),
			});

			Ok(())
		}

		/// Remove liquidity from selected 2-asset pool in the form of burning shares.
		///
		/// LP receives certain amount of both assets.
		///
		/// Partial withdrawal is allowed.
		///
		/// Parameters:
		/// - `origin`: Must be T::CreatePoolOrigin
		/// - `pool_id`: Pool Id
		/// - `amount`: Amount of shares to burn
		///
		/// Emits `LiquidityRemoved` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::remove_liquidity())]
		#[transactional]
		pub fn remove_liquidity(origin: OriginFor<T>, pool_id: PoolId<T::AssetId>, amount: Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(amount > Balance::zero(), Error::<T>::InvalidAssetAmount);

			ensure!(
				T::Currency::free_balance(pool_id.0, &who) >= amount,
				Error::<T>::InsufficientShares
			);

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

			let initial_reserves = AssetAmounts(
				T::Currency::free_balance(pool.assets.0, &pool_account),
				T::Currency::free_balance(pool.assets.1, &pool_account),
			);

			let share_issuance = T::Currency::total_issuance(pool_id.0);

			ensure!(
				share_issuance.saturating_sub(amount) >= T::MinimumLiquidity::get(),
				Error::<T>::InsufficientLiquidityRemaining
			);

			let amounts = calculate_remove_liquidity_amounts(&initial_reserves, amount, share_issuance)
				.ok_or(ArithmeticError::Overflow)?;

			T::Currency::withdraw(pool_id.0, &who, amount)?;

			// Assets are ordered by id in pool.assets.So amounts provided corresponds.
			for (asset, asset_amount) in pool.assets.into_iter().zip(amounts.into_iter()) {
				T::Currency::transfer(asset, &pool_account, &who, asset_amount)?;
			}

			Self::deposit_event(Event::LiquidityRemoved {
				id: pool_id,
				who,
				shares: amount,
				amounts: (amounts.0, amounts.1),
			});

			Ok(())
		}

		/// Execute a swap of `asset_in` for `asset_out`.
		///
		/// Parameters:
		/// - `origin`:
		/// - `pool_id`: Id of a pool
		/// - `asset_in`: ID of asset sold to the pool
		/// - `asset_out`: ID of asset bought from the pool
		/// - `amount`: Amount of asset sold
		/// - `min_buy_amount`: Minimum amount required to receive
		///
		/// Emits `SellExecuted` event when successful.
		///
		#[pallet::weight(<T as Config>::WeightInfo::sell())]
		#[transactional]
		pub fn sell(
			origin: OriginFor<T>,
			pool_id: PoolId<T::AssetId>,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount_in: Balance,
			min_buy_amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				amount_in >= T::MinimumTradingLimit::get(),
				Error::<T>::InsufficientTradingAmount
			);

			ensure!(
				T::Currency::free_balance(asset_in, &who) >= amount_in,
				Error::<T>::InsufficientBalance
			);

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			ensure!(pool.assets.contains(asset_in), Error::<T>::AssetNotInPool);
			ensure!(pool.assets.contains(asset_out), Error::<T>::AssetNotInPool);

			let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

			let reserve_in = T::Currency::free_balance(asset_in, &pool_account);
			let reserve_out = T::Currency::free_balance(asset_out, &pool_account);

			let amount_out = calculate_out_given_in(
				reserve_in,
				reserve_out,
				amount_in,
				T::Precision::get(),
				pool.amplification,
			)
			.ok_or(ArithmeticError::Overflow)?;

			let fee_amount = Self::calculate_fee_amount(amount_out, pool.fee, true).ok_or(ArithmeticError::Overflow)?;

			let amount_out = amount_out.checked_sub(fee_amount).ok_or(ArithmeticError::Underflow)?;

			ensure!(amount_out >= min_buy_amount, Error::<T>::BuyLimitNotReached);

			T::Currency::transfer(asset_in, &who, &pool_account, amount_in)?;
			T::Currency::transfer(asset_out, &pool_account, &who, amount_out)?;

			Self::deposit_event(Event::SellExecuted {
				who,
				asset_in,
				asset_out,
				amount_in,
				amount_out,
				fee: fee_amount,
			});

			Ok(())
		}

		/// Execute a swap of `asset_out` for `asset_in`.
		///
		/// Parameters:
		/// - `origin`:
		/// - `pool_id`: Id of a pool
		/// - `asset_out`: ID of asset bought from the pool
		/// - `asset_in`: ID of asset sold to the pool
		/// - `amount`: Amount of asset sold
		/// - `max_sell_amount`: Maximum amount allowedto be sold
		///
		/// Emits `buyExecuted` event when successful.
		///
		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		#[transactional]
		pub fn buy(
			origin: OriginFor<T>,
			pool_id: PoolId<T::AssetId>,
			asset_out: T::AssetId,
			asset_in: T::AssetId,
			amount_out: Balance,
			max_sell_amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				amount_out >= T::MinimumTradingLimit::get(),
				Error::<T>::InsufficientTradingAmount
			);

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			ensure!(pool.assets.contains(asset_in), Error::<T>::AssetNotInPool);
			ensure!(pool.assets.contains(asset_out), Error::<T>::AssetNotInPool);

			let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

			let reserve_in = T::Currency::free_balance(asset_in, &pool_account);
			let reserve_out = T::Currency::free_balance(asset_out, &pool_account);

			ensure!(reserve_out > amount_out, Error::<T>::InsufficientLiquidity);

			let amount_in = calculate_in_given_out(
				reserve_in,
				reserve_out,
				amount_out,
				T::Precision::get(),
				pool.amplification,
			)
			.ok_or(ArithmeticError::Overflow)?;

			let fee_amount = Self::calculate_fee_amount(amount_in, pool.fee, false).ok_or(ArithmeticError::Overflow)?;

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
				asset_in,
				asset_out,
				amount_in,
				amount_out,
			});

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	fn calculate_fee_amount(amount: Balance, fee: Permill, rounding_down: bool) -> Option<Balance> {
		if rounding_down {
			Some(fee.mul_floor(amount))
		} else {
			Some(fee.mul_ceil(amount))
		}
	}
}
