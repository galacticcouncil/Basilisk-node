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

mod math;
mod traits;
mod types;
pub mod weights;

pub use pallet::*;
use weights::WeightInfo;

const POOL_IDENTIFIER: &str = "sts";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::math::{calculate_buy_changes, calculate_sell_changes};
	use crate::traits::ShareAccountIdFor;
	use crate::types::{Balance, PoolAssets, PoolId, PoolInfo};
	use codec::HasCompact;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use hydradx_traits::{Registry, ShareTokenRegistry};
	use math::calculate_add_liquidity_changes;
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
		type CreatePoolOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: weights::WeightInfo;
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
			asset: T::AssetId,
			amount: Balance,
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

		/// Minimum limit has not been reached during trade.
		BuyLimitNotReached,

		/// Maximum limit has been exceeded during trade.
		SellLimitExceeded,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			assets: (T::AssetId, T::AssetId),
			amplification: Balance,
			fee: Permill,
		) -> DispatchResult {
			T::CreatePoolOrigin::ensure_origin(origin)?;

			let pool_assets: PoolAssets<T::AssetId> = assets.into();

			ensure!(pool_assets.is_valid(), Error::<T>::SameAssets);

			for asset in (&pool_assets).into_iter() {
				ensure!(T::AssetRegistry::exists(asset), Error::<T>::AssetNotRegistered);
			}

			let share_asset_ident = T::ShareAccountId::name(&pool_assets, Some(POOL_IDENTIFIER));

			let share_asset = T::AssetRegistry::get_or_create_shared_asset(
				share_asset_ident,
				(&pool_assets).into(),
				Balance::zero(),
			)?;

			let pool_id = PoolId(share_asset);

			Pools::<T>::try_mutate(&pool_id, |maybe_pool| -> DispatchResult {
				ensure!(maybe_pool.is_none(), Error::<T>::PoolExists);

				*maybe_pool = Some(PoolInfo {
					assets: pool_assets.clone(),
					amplification,
					fee,
				});

				Ok(())
			})?;

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

			ensure!(
				T::Currency::free_balance(asset, &who) >= amount,
				Error::<T>::InsufficientBalance
			);

			Pools::<T>::try_mutate(&pool_id, |maybe_pool| -> DispatchResult {
				let pool = maybe_pool.as_ref().ok_or(Error::<T>::PoolNotFound)?;

				ensure!(pool.assets.contains(asset), Error::<T>::AssetNotInPool);

				let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

				let asset_reserve = T::Currency::free_balance(asset, &pool_account);

				let delta_changes = calculate_add_liquidity_changes(pool, asset, asset_reserve, amount)
					.ok_or(ArithmeticError::Overflow)?;

				T::Currency::deposit(pool_id.0, &who, delta_changes.share_amount)?;
				T::Currency::transfer(asset, &who, &pool_account, amount)?;

				Ok(())
			})?;

			Self::deposit_event(Event::LiquidityAdded {
				id: pool_id,
				from: who,
				asset,
				amount,
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
			amount: Balance,
			min_bought: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				T::Currency::free_balance(asset_in, &who) >= amount,
				Error::<T>::InsufficientBalance
			);

			Pools::<T>::try_mutate(&pool_id, |maybe_pool| -> DispatchResult {
				let pool = maybe_pool.as_ref().ok_or(Error::<T>::PoolNotFound)?;

				let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

				//TODO: retrieve asset reserves

				let updated_state =
					calculate_sell_changes(pool, asset_in, asset_out, amount).ok_or(ArithmeticError::Overflow)?;

				ensure!(
					updated_state.delta_amount_out >= min_bought,
					Error::<T>::BuyLimitNotReached
				);

				T::Currency::transfer(asset_in, &who, &pool_account, amount)?;
				T::Currency::transfer(asset_out, &pool_account, &who, updated_state.delta_amount_out)?;

				Self::deposit_event(Event::SellExecuted {
					who,
					asset_in,
					asset_out,
					amount_in: amount,
					amount_out: updated_state.delta_amount_out,
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
			amount: Balance,
			max_sold: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Pools::<T>::try_mutate(&pool_id, |maybe_pool| -> DispatchResult {
				let pool = maybe_pool.as_ref().ok_or(Error::<T>::PoolNotFound)?;

				let pool_account = T::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

				let updated_state =
					calculate_buy_changes(pool, asset_in, asset_out, amount).ok_or(ArithmeticError::Overflow)?;

				let amount_in = updated_state.delta_amount_out;

				ensure!(amount_in <= max_sold, Error::<T>::BuyLimitNotReached);
				ensure!(
					T::Currency::free_balance(asset_in, &who) >= amount_in,
					Error::<T>::InsufficientBalance
				);

				T::Currency::transfer(asset_in, &who, &pool_account, amount_in)?;
				T::Currency::transfer(asset_out, &pool_account, &who, amount)?;

				Self::deposit_event(Event::BuyExecuted {
					who,
					asset_in,
					asset_out,
					amount_in,
					amount_out: amount,
				});

				Ok(())
			})
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}
