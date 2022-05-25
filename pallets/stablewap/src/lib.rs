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
//!
//! Questions:
//! 1. who can create pools
//! 2. pool owner needed to know ?
//! 3. creation fee?
//! 4. fees = trade fee and admin fee?!
//! 5.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::DispatchResult;
use frame_support::transactional;

pub use pallet::*;

mod math;
mod traits;
mod types;
pub mod weights;

use weights::WeightInfo;

#[cfg(test)]
mod tests;

const POOL_IDENTIFIER: &str = "sts";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::math::{
		calculate_add_liquidity_shares, calculate_in_given_out, calculate_out_given_in,
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

		type ShareAccountId: ShareAccountIdFor<PoolAssets<Self::AssetId>, AccountId = Self::AccountId>;

		type AssetRegistry: ShareTokenRegistry<Self::AssetId, Vec<u8>, Balance, DispatchError>;

		/// The origin which can create a new pool
		type CreatePoolOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		#[pallet::constant]
		type Precision: Get<Balance>;

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
			assets: PoolAssets<T::AssetId>,
			amplification: Balance,
		},
		/// Liquidity of an asset was added to Omnipool.
		LiquidityAdded {
			id: PoolId<T::AssetId>,
			from: T::AccountId,
			amount: Balance,
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

		/// Balance of an share asset is nto sufficient to withdraw liquiduity.
		InsufficientShares,

		/// Minimum limit has not been reached during trade.
		BuyLimitNotReached,

		/// Maximum limit has been exceeded during trade.
		SellLimitExceeded,

		/// Initial liquidity of asset must be > 0.
		InvalidInitialLiquidity,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
				initial_liquidity.0 >= Balance::zero(),
				Error::<T>::InvalidInitialLiquidity
			);
			ensure!(
				initial_liquidity.1 >= Balance::zero(),
				Error::<T>::InvalidInitialLiquidity
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

			// Add initial liquidity
			let reserves: AssetAmounts<Balance> = initial_liquidity.into();

			let share_amount = calculate_add_liquidity_shares(
				&AssetAmounts::default(),
				&reserves,
				T::Precision::get(),
				pool.amplification,
				Balance::zero(),
			)
			.ok_or(ArithmeticError::Overflow)?;

			let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

			T::Currency::transfer(assets.0, &who, &pool_account, initial_liquidity.0)?;
			T::Currency::transfer(assets.1, &who, &pool_account, initial_liquidity.1)?;

			T::Currency::deposit(pool_id.0, &who, share_amount)?;

			Self::deposit_event(Event::PoolCreated {
				id: pool_id,
				assets: pool_assets,
				amplification,
			});

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolId<T::AssetId>,
			asset: T::AssetId,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(amount > Balance::zero(), Error::<T>::InvalidAssetAmount);

			// Ensure that who has enought balance of each asset
			ensure!(
				T::Currency::free_balance(asset, &who) >= amount,
				Error::<T>::InsufficientBalance
			);

			// NOTE: THIS IS WIP!! The following mess needs refactor. just for POC if math to make sure math is right!.

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;

			ensure!(pool.contains_asset(asset), Error::<T>::AssetNotInPool);

			let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));
			let share_issuance = T::Currency::total_issuance(pool_id.0);

			let reserves = AssetAmounts(
				T::Currency::free_balance(pool.assets.0, &pool_account),
				T::Currency::free_balance(pool.assets.1, &pool_account),
			);

			let (new_reserves, asset_b_amount) = if asset == pool.assets.0 {
				let asset_reserve = reserves.0.checked_add(amount).ok_or(ArithmeticError::Overflow)?;

				let asset_b_reserve = (reserves.1 * asset_reserve) / reserves.0;

				(
					AssetAmounts(asset_reserve, asset_b_reserve),
					asset_b_reserve - reserves.1,
				)
			} else {
				let asset_reserve = reserves.1.checked_add(amount).ok_or(ArithmeticError::Overflow)?;

				let asset_a_reserve = (reserves.0 * asset_reserve) / reserves.1;
				(
					AssetAmounts(asset_a_reserve, asset_reserve),
					asset_a_reserve - reserves.0,
				)
			};

			ensure!(
				T::Currency::free_balance(pool.assets.1, &who) >= asset_b_amount,
				Error::<T>::InsufficientBalance
			);

			let share_amount = calculate_add_liquidity_shares(
				&reserves,
				&new_reserves,
				T::Precision::get(),
				pool.amplification,
				share_issuance,
			)
			.ok_or(ArithmeticError::Overflow)?;

			T::Currency::deposit(pool_id.0, &who, share_amount)?;

			T::Currency::transfer(asset, &who, &pool_account, amount)?;

			if asset == pool.assets.0 {
				T::Currency::transfer(pool.assets.1, &who, &pool_account, asset_b_amount)?;
			} else {
				T::Currency::transfer(pool.assets.0, &who, &pool_account, asset_b_amount)?;
			}

			Self::deposit_event(Event::LiquidityAdded {
				id: pool_id,
				from: who,
				amount,
			});

			Ok(())
		}

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

			let reserve_i = T::Currency::free_balance(pool.assets.0, &pool_account);
			let reserve_j = T::Currency::free_balance(pool.assets.1, &pool_account);

			let share_issuance = T::Currency::total_issuance(pool_id.0);

			let amounts = calculate_remove_liquidity_amounts(&(reserve_i, reserve_j).into(), amount, share_issuance)
				.ok_or(ArithmeticError::Overflow)?;

			T::Currency::withdraw(pool_id.0, &who, amount)?;

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

		#[pallet::weight(<T as Config>::WeightInfo::sell())]
		#[transactional]
		pub fn sell(
			origin: OriginFor<T>,
			pool_id: PoolId<T::AssetId>,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount_in: Balance,
			min_bought: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				T::Currency::free_balance(asset_in, &who) >= amount_in,
				Error::<T>::InsufficientBalance
			);

			Pools::<T>::try_mutate(&pool_id, |maybe_pool| -> DispatchResult {
				let pool = maybe_pool.as_ref().ok_or(Error::<T>::PoolNotFound)?;

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

				ensure!(amount_out >= min_bought, Error::<T>::BuyLimitNotReached);

				T::Currency::transfer(asset_in, &who, &pool_account, amount_in)?;
				T::Currency::transfer(asset_out, &pool_account, &who, amount_out)?;

				Self::deposit_event(Event::SellExecuted {
					who,
					asset_in,
					asset_out,
					amount_in,
					amount_out,
				});

				Ok(())
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		#[transactional]
		pub fn buy(
			origin: OriginFor<T>,
			pool_id: PoolId<T::AssetId>,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount_out: Balance,
			max_sold: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Pools::<T>::try_mutate(&pool_id, |maybe_pool| -> DispatchResult {
				let pool = maybe_pool.as_ref().ok_or(Error::<T>::PoolNotFound)?;

				ensure!(pool.assets.contains(asset_in), Error::<T>::AssetNotInPool);
				ensure!(pool.assets.contains(asset_out), Error::<T>::AssetNotInPool);

				let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

				let reserve_in = T::Currency::free_balance(asset_in, &pool_account);
				let reserve_out = T::Currency::free_balance(asset_out, &pool_account);

				let amount_in = calculate_in_given_out(
					reserve_in,
					reserve_out,
					amount_out,
					T::Precision::get(),
					pool.amplification,
				)
				.ok_or(ArithmeticError::Overflow)?;

				ensure!(amount_in <= max_sold, Error::<T>::BuyLimitNotReached);

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
			})
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}
