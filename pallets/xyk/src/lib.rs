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

//! # XYK Pallet
//!
//! ## Overview
//!
//! XYK pallet provides functionality for managing liquidity pool and executing trades.
//!
//! This pallet implements AMM Api trait therefore it is possible to plug this pool implementation
//! into the exchange pallet.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]

use frame_support::sp_runtime::{
	traits::{CheckedAdd, CheckedDiv, CheckedSub, Hash, Zero},
	DispatchError,
};
use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::Encode, traits::Get, transactional};
use frame_system::ensure_signed;
use primitives::{
	asset::AssetPairT, fee, traits::AMM, Price, MAX_IN_RATIO, MAX_OUT_RATIO, MIN_POOL_LIQUIDITY, MIN_TRADING_LIMIT,
};
use sp_std::{convert::TryInto, marker::PhantomData, vec, vec::Vec};

use frame_support::sp_runtime::app_crypto::sp_core::crypto::UncheckedFrom;
use frame_support::sp_runtime::FixedPointNumber;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::fee::WithFee;
use primitives::traits::AMMTransfer;
use primitives::Amount;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;

use weights::WeightInfo;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[macro_export]
macro_rules! to_u128_result {
	 ($($x:expr),+) => (
		 {($(TryInto::<u128>::try_into($x).map_err(|_| Error::<T>::Overflow)),+)}
	 );
}

#[macro_export]
macro_rules! to_balance_result {
	 ($($x:expr),+) => (
		 {($($x.try_into().map_err(|_| Error::<T>::Overflow)),+)}
	 );
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;
	use primitives::traits::ShareTokenRegistry;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	pub type BalanceOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Registry support
		type AssetRegistry: ShareTokenRegistry<Self::AssetId, Vec<u8>, BalanceOf<Self>, DispatchError>;

		/// Share token support
		type AssetPairAccountId: AssetPairAccountIdFor<Self::AssetId, Self::AccountId>;

		/// Multi currency for transfer of currencies
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = Self::AssetId, Amount = Amount>;

		/// Asset type
		type AssetId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + Default + From<u32>;

		/// Native Asset Id
		#[pallet::constant]
		type NativeAssetId: Get<Self::AssetId>;

		/// Weight information for the extrinsics.
		type WeightInfo: WeightInfo;

		/// Trading fee rate
		#[pallet::constant]
		type GetExchangeFee: Get<fee::Fee>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// It is not allowed to create a pool between same assets.
		CannotCreatePoolWithSameAssets,

		/// Liquidity has not reached the required minimum.
		InsufficientLiquidity,

		/// Amount is less than min trading limit.
		InsufficientTradingAmount,

		/// Liquidity is zero.
		ZeroLiquidity,

		/// It is not allowed to create a pool with zero initial price.
		ZeroInitialPrice,

		/// Overflow
		CreatePoolAssetAmountInvalid,

		/// Overflow
		InvalidMintedLiquidity, // No tests - but it is currently not possible this error to occur due to previous checks in the code.

		/// Overflow
		InvalidLiquidityAmount, // no tests - it is currently not possible this error to occur due to previous checks in the code.

		/// Asset amount has exceeded given limit.
		AssetAmountExceededLimit,

		/// Asset amount has not reached given limit.
		AssetAmountNotReachedLimit,

		/// Asset balance is not sufficient.
		InsufficientAssetBalance,

		/// Not enough asset liquidity in the pool.
		InsufficientPoolAssetBalance,

		/// Not enough core asset liquidity in the pool.
		InsufficientNativeCurrencyBalance,

		/// Liquidity pool for given assets does not exist.
		TokenPoolNotFound,

		/// Liquidity pool for given assets already exists.
		TokenPoolAlreadyExists,

		/// Overflow
		AddAssetAmountInvalid, // no tests
		/// Overflow
		RemoveAssetAmountInvalid, // no tests
		/// Overflow
		SellAssetAmountInvalid, // no tests
		/// Overflow
		BuyAssetAmountInvalid, // no tests

		/// Overflow
		FeeAmountInvalid,

		/// Overflow
		CannotApplyDiscount,

		/// Max fraction of pool to buy in single transaction has been exceeded.
		MaxOutRatioExceeded,
		/// Max fraction of pool to sell in single transaction has been exceeded.
		MaxInRatioExceeded,

		/// Overflow
		Overflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New liquidity was provided to the pool. [who, asset a, asset b, amount a, amount b]
		LiquidityAdded(T::AccountId, T::AssetId, T::AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Liquidity was removed from the pool. [who, asset a, asset b, shares]
		LiquidityRemoved(T::AccountId, T::AssetId, T::AssetId, BalanceOf<T>),

		/// Pool was created. [who, asset a, asset b, initial shares amount, share token, pool account id]
		PoolCreated(
			T::AccountId,
			T::AssetId,
			T::AssetId,
			BalanceOf<T>,
			T::AssetId,
			T::AccountId,
		),

		/// Pool was destroyed. [who, asset a, asset b, share token, pool account id]
		PoolDestroyed(T::AccountId, T::AssetId, T::AssetId, T::AssetId, T::AccountId),

		/// Asset sale executed. [who, asset in, asset out, amount, sale price, fee asset, fee amount]
		SellExecuted(
			T::AccountId,
			T::AssetId,
			T::AssetId,
			BalanceOf<T>,
			BalanceOf<T>,
			T::AssetId,
			BalanceOf<T>,
		),

		/// Asset purchase executed. [who, asset out, asset in, amount, buy price, fee asset, fee amount]
		BuyExecuted(
			T::AccountId,
			T::AssetId,
			T::AssetId,
			BalanceOf<T>,
			BalanceOf<T>,
			T::AssetId,
			BalanceOf<T>,
		),
	}

	/// Asset id storage for shared pool tokens
	#[pallet::storage]
	#[pallet::getter(fn share_token)]
	pub type ShareToken<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AssetId, ValueQuery>;

	/// Total liquidity in a pool.
	#[pallet::storage]
	#[pallet::getter(fn total_liquidity)]
	pub type TotalLiquidity<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Asset pair in a pool.
	#[pallet::storage]
	#[pallet::getter(fn pool_assets)]
	pub type PoolAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (T::AssetId, T::AssetId), ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create new pool for given asset pair.
		///
		/// Registers new pool for given asset pair (`asset a` and `asset b`) in asset registry.
		/// Asset registry creates new id or returns previously created one if such pool existed before.
		///
		/// Pool is created with initial liquidity provided by `origin`.
		/// Shares are issued with specified initial price and represents proportion of asset in the pool.
		///
		/// Emits `PoolCreated` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			asset_a: T::AssetId,
			asset_b: T::AssetId,
			amount: BalanceOf<T>,
			initial_price: Price,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(amount >= MIN_POOL_LIQUIDITY.into(), Error::<T>::InsufficientLiquidity);

			ensure!(!(initial_price == Price::zero()), Error::<T>::ZeroInitialPrice);

			ensure!(asset_a != asset_b, Error::<T>::CannotCreatePoolWithSameAssets);

			let asset_pair = AssetPairT::<T::AssetId> {
				asset_in: asset_a,
				asset_out: asset_b,
			};

			ensure!(!Self::exists(asset_pair), Error::<T>::TokenPoolAlreadyExists);

			let amount_u128 = to_u128_result!(amount)?;

			let asset_b_amount = initial_price
				.checked_mul_int(amount_u128)
				.ok_or(Error::<T>::CreatePoolAssetAmountInvalid)?
				.try_into()
				.map_err(|_| Error::<T>::Overflow)?;

			ensure!(
				asset_b_amount >= MIN_POOL_LIQUIDITY.into(),
				Error::<T>::InsufficientLiquidity
			);

			let shares_added = if asset_a < asset_b { amount } else { asset_b_amount };

			ensure!(
				T::Currency::free_balance(asset_a, &who) >= amount,
				Error::<T>::InsufficientAssetBalance
			);

			ensure!(
				T::Currency::free_balance(asset_b, &who) >= asset_b_amount,
				Error::<T>::InsufficientAssetBalance
			);

			let pair_account = Self::get_pair_id(asset_pair);

			let token_name = asset_pair.name();

			let share_token = T::AssetRegistry::get_or_create_shared_asset(
				token_name,
				vec![asset_a, asset_b],
				MIN_POOL_LIQUIDITY.into(),
			)?;

			<ShareToken<T>>::insert(&pair_account, &share_token);
			<PoolAssets<T>>::insert(&pair_account, (asset_a, asset_b));

			T::Currency::transfer(asset_a, &who, &pair_account, amount)?;
			T::Currency::transfer(asset_b, &who, &pair_account, asset_b_amount)?;

			T::Currency::deposit(share_token, &who, shares_added)?;

			<TotalLiquidity<T>>::insert(&pair_account, shares_added);

			Self::deposit_event(Event::PoolCreated(
				who,
				asset_a,
				asset_b,
				shares_added,
				share_token,
				pair_account,
			));

			Ok(().into())
		}

		/// Add liquidity to previously created asset pair pool.
		///
		/// Shares are issued with current price.
		///
		/// Emits `LiquidityAdded` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset_a: T::AssetId,
			asset_b: T::AssetId,
			amount_a: BalanceOf<T>,
			amount_b_max_limit: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let asset_pair = AssetPairT::<T::AssetId> {
				asset_in: asset_a,
				asset_out: asset_b,
			};

			ensure!(Self::exists(asset_pair), Error::<T>::TokenPoolNotFound);

			ensure!(
				amount_a >= MIN_TRADING_LIMIT.into(),
				Error::<T>::InsufficientTradingAmount
			);

			ensure!(!amount_b_max_limit.is_zero(), Error::<T>::ZeroLiquidity);

			ensure!(
				T::Currency::free_balance(asset_a, &who) >= amount_a,
				Error::<T>::InsufficientAssetBalance
			);

			ensure!(
				T::Currency::free_balance(asset_b, &who) >= amount_b_max_limit,
				Error::<T>::InsufficientAssetBalance
			);

			let pair_account = Self::get_pair_id(asset_pair);

			let share_token = Self::share_token(&pair_account);

			let account_shares = T::Currency::free_balance(share_token, &who);

			let asset_a_reserve = T::Currency::free_balance(asset_a, &pair_account);
			let asset_a_reserve = to_u128_result!(asset_a_reserve)?;

			let asset_b_reserve = T::Currency::free_balance(asset_b, &pair_account);
			let asset_b_reserve = to_u128_result!(asset_b_reserve)?;

			let total_liquidity = Self::total_liquidity(&pair_account);

			let amount_a_u128 = to_u128_result!(amount_a).map_err(|_| Error::<T>::CreatePoolAssetAmountInvalid)?;

			let amount_b_required: BalanceOf<T> =
				hydra_dx_math::xyk::calculate_liquidity_in(asset_a_reserve, asset_b_reserve, amount_a_u128)
					.map_err(|_| Error::<T>::AddAssetAmountInvalid)?
					.try_into()
					.map_err(|_| Error::<T>::Overflow)?;

			let shares_added = if asset_a < asset_b { amount_a } else { amount_b_required };

			ensure!(
				amount_b_required <= amount_b_max_limit,
				Error::<T>::AssetAmountExceededLimit
			);

			ensure!(!shares_added.is_zero(), Error::<T>::InvalidMintedLiquidity);

			// Make sure that account share liquidity is at least MIN_POOL_LIQUIDITY
			ensure!(
				account_shares
					.checked_add(&shares_added)
					.ok_or(Error::<T>::InvalidMintedLiquidity)?
					>= MIN_POOL_LIQUIDITY.into(),
				Error::<T>::InsufficientLiquidity
			);

			let liquidity_amount = total_liquidity
				.checked_add(&shares_added)
				.ok_or(Error::<T>::InvalidLiquidityAmount)?;

			T::Currency::transfer(asset_a, &who, &pair_account, amount_a)?;
			T::Currency::transfer(asset_b, &who, &pair_account, amount_b_required)?;

			T::Currency::deposit(share_token, &who, shares_added)?;

			<TotalLiquidity<T>>::insert(&pair_account, liquidity_amount);

			Self::deposit_event(Event::LiquidityAdded(
				who,
				asset_a,
				asset_b,
				amount_a,
				amount_b_required,
			));

			Ok(().into())
		}

		/// Remove liquidity from specific liquidity pool in the form of burning shares.
		///
		/// If liquidity in the pool reaches 0, it is destroyed.
		///
		/// Emits 'LiquidityRemoved' when successful.
		/// Emits 'PoolDestroyed' when pool is destroyed.
		#[pallet::weight(<T as Config>::WeightInfo::remove_liquidity())]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			asset_a: T::AssetId,
			asset_b: T::AssetId,
			liquidity_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let asset_pair = AssetPairT::<T::AssetId> {
				asset_in: asset_a,
				asset_out: asset_b,
			};

			ensure!(!liquidity_amount.is_zero(), Error::<T>::ZeroLiquidity);

			ensure!(Self::exists(asset_pair), Error::<T>::TokenPoolNotFound);

			let pair_account = Self::get_pair_id(asset_pair);

			let share_token = Self::share_token(&pair_account);

			let total_shares = Self::total_liquidity(&pair_account);

			let account_shares = T::Currency::free_balance(share_token, &who);

			ensure!(total_shares >= liquidity_amount, Error::<T>::InsufficientAssetBalance);

			ensure!(account_shares >= liquidity_amount, Error::<T>::InsufficientAssetBalance);

			// Account's liquidity left should be either 0 or at least MIN_POOL_LIQUIDITY
			ensure!(
				(account_shares - liquidity_amount) >= MIN_POOL_LIQUIDITY.into()
					|| (account_shares - liquidity_amount).is_zero(),
				Error::<T>::InsufficientLiquidity
			);

			let asset_a_reserve = T::Currency::free_balance(asset_a, &pair_account);
			let asset_b_reserve = T::Currency::free_balance(asset_b, &pair_account);

			let liquidity_out = hydra_dx_math::xyk::calculate_liquidity_out(
				to_u128_result!(asset_a_reserve)?,
				to_u128_result!(asset_b_reserve)?,
				to_u128_result!(liquidity_amount)?,
				to_u128_result!(total_shares)?,
			)
			.map_err(|_| Error::<T>::RemoveAssetAmountInvalid)?;

			let remove_amount_a = to_balance_result!(liquidity_out.0)?;
			let remove_amount_b = to_balance_result!(liquidity_out.1)?;

			ensure!(
				T::Currency::free_balance(asset_a, &pair_account) >= remove_amount_a,
				Error::<T>::InsufficientPoolAssetBalance
			);
			ensure!(
				T::Currency::free_balance(asset_b, &pair_account) >= remove_amount_b,
				Error::<T>::InsufficientPoolAssetBalance
			);

			let liquidity_left = total_shares
				.checked_sub(&liquidity_amount)
				.ok_or(Error::<T>::InvalidLiquidityAmount)?;

			T::Currency::transfer(asset_a, &pair_account, &who, remove_amount_a)?;
			T::Currency::transfer(asset_b, &pair_account, &who, remove_amount_b)?;

			T::Currency::withdraw(share_token, &who, liquidity_amount)?;

			<TotalLiquidity<T>>::insert(&pair_account, liquidity_left);

			Self::deposit_event(Event::LiquidityRemoved(who.clone(), asset_a, asset_b, liquidity_amount));

			if liquidity_left == 0u32.into() {
				<ShareToken<T>>::remove(&pair_account);
				<PoolAssets<T>>::remove(&pair_account);

				Self::deposit_event(Event::PoolDestroyed(who, asset_a, asset_b, share_token, pair_account));
			}

			Ok(().into())
		}

		/// Trade asset in for asset out.
		///
		/// Executes a swap of `asset_in` for `asset_out`. Price is determined by the liquidity pool.
		///
		/// `max_limit` - minimum amount of `asset_out` / amount of asset_out to be obtained from the pool in exchange for `asset_in`.
		///
		/// Emits `SellExecuted` when successful.
		#[pallet::weight(<T as Config>::WeightInfo::sell())]
		pub fn sell(
			origin: OriginFor<T>,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			amount: BalanceOf<T>,
			max_limit: BalanceOf<T>,
			discount: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<Self as AMM<_, _, _, _>>::sell(
				&who,
				AssetPairT::<T::AssetId> { asset_in, asset_out },
				amount,
				max_limit,
				discount,
			)?;

			Ok(().into())
		}

		/// Trade asset in for asset out.
		///
		/// Executes a swap of `asset_in` for `asset_out`. Price is determined by the liquidity pool.
		///
		/// `max_limit` - maximum amount of `asset_in` to be sold in exchange for `asset_out`.
		///
		/// Emits `BuyExecuted` when successful.
		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		pub fn buy(
			origin: OriginFor<T>,
			asset_out: T::AssetId,
			asset_in: T::AssetId,
			amount: BalanceOf<T>,
			max_limit: BalanceOf<T>,
			discount: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<Self as AMM<_, _, _, _>>::buy(
				&who,
				AssetPairT::<T::AssetId> { asset_in, asset_out },
				amount,
				max_limit,
				discount,
			)?;

			Ok(().into())
		}
	}
}

pub trait AssetPairAccountIdFor<AssetId: Sized, AccountId: Sized> {
	fn from_assets(asset_a: AssetId, asset_b: AssetId) -> AccountId;
}

pub struct AssetPairAccountId<T: Config>(PhantomData<T>);

impl<T: Config> AssetPairAccountIdFor<T::AssetId, T::AccountId> for AssetPairAccountId<T>
where
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn from_assets(asset_a: T::AssetId, asset_b: T::AssetId) -> T::AccountId {
		let mut buf = Vec::new();
		buf.extend_from_slice(b"hydradx");
		if asset_a < asset_b {
			buf.extend_from_slice(&asset_a.encode());
			buf.extend_from_slice(&asset_b.encode());
		} else {
			buf.extend_from_slice(&asset_b.encode());
			buf.extend_from_slice(&asset_a.encode());
		}
		T::AccountId::unchecked_from(T::Hashing::hash(&buf[..]))
	}
}

impl<T: Config> Pallet<T> {
	/// Return balance of each asset in selected liquidity pool.
	pub fn get_pool_balances(pool_address: T::AccountId) -> Option<Vec<(T::AssetId, BalanceOf<T>)>> {
		let mut balances = Vec::new();

		if let Some(assets) = Self::get_pool_assets(&pool_address) {
			for item in &assets {
				let reserve = T::Currency::free_balance(*item, &pool_address);
				balances.push((*item, reserve));
			}
		}
		Some(balances)
	}
	/// Calculate discounted trade fee
	fn calculate_discounted_fee(amount: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
		Ok(amount
			.discounted_fee()
			.ok_or::<Error<T>>(Error::<T>::FeeAmountInvalid)?)
	}

	/// Calculate trade fee
	fn calculate_fee(amount: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
		Ok(amount
			.just_fee(T::GetExchangeFee::get())
			.ok_or::<Error<T>>(Error::<T>::FeeAmountInvalid)?)
	}
}

// Implementation of AMM API which makes possible to plug the AMM pool into the exchange pallet.
impl<T: Config> AMM<T::AccountId, T::AssetId, AssetPairT<T::AssetId>, BalanceOf<T>> for Pallet<T> {
	fn exists(assets: AssetPairT<T::AssetId>) -> bool {
		let pair_account = T::AssetPairAccountId::from_assets(assets.asset_in, assets.asset_out);
		<ShareToken<T>>::contains_key(&pair_account)
	}

	fn get_pair_id(assets: AssetPairT<T::AssetId>) -> T::AccountId {
		T::AssetPairAccountId::from_assets(assets.asset_in, assets.asset_out)
	}

	fn get_pool_assets(pool_account_id: &T::AccountId) -> Option<Vec<T::AssetId>> {
		match <PoolAssets<T>>::contains_key(pool_account_id) {
			true => {
				let assets = Self::pool_assets(pool_account_id);
				Some(vec![assets.0, assets.1])
			}
			false => None,
		}
	}

	fn get_spot_price_unchecked(asset_a: T::AssetId, asset_b: T::AssetId, amount: BalanceOf<T>) -> BalanceOf<T> {
		let pair_account = Self::get_pair_id(AssetPairT::<T::AssetId> {
			asset_out: asset_a,
			asset_in: asset_b,
		});

		let asset_a_reserve = to_u128_result!(T::Currency::free_balance(asset_a, &pair_account)).unwrap_or(0u128);
		let asset_b_reserve = to_u128_result!(T::Currency::free_balance(asset_b, &pair_account)).unwrap_or(0u128);

		hydra_dx_math::xyk::calculate_spot_price(
			asset_a_reserve,
			asset_b_reserve,
			TryInto::<u128>::try_into(amount).unwrap_or(0u128),
		)
		.map_or_else(
			|_| BalanceOf::<T>::zero(),
			|value| value.try_into().unwrap_or_else(|_| BalanceOf::<T>::zero()),
		)
	}

	/// Validate a sell. Perform all necessary checks and calculations.
	/// No storage changes are performed yet.
	///
	/// Return `AMMTransfer` with all info needed to execute the transaction.
	fn validate_sell(
		who: &T::AccountId,
		assets: AssetPairT<T::AssetId>,
		amount: BalanceOf<T>,
		min_bought: BalanceOf<T>,
		discount: bool,
	) -> Result<AMMTransfer<T::AccountId, T::AssetId, AssetPairT<T::AssetId>, BalanceOf<T>>, sp_runtime::DispatchError>
	{
		ensure!(
			amount >= MIN_TRADING_LIMIT.into(),
			Error::<T>::InsufficientTradingAmount
		);

		ensure!(Self::exists(assets), Error::<T>::TokenPoolNotFound);

		ensure!(
			T::Currency::free_balance(assets.asset_in, who) >= amount,
			Error::<T>::InsufficientAssetBalance
		);

		// If discount, pool for Sell asset and native asset must exist
		if discount {
			ensure!(
				Self::exists(AssetPairT::<T::AssetId> {
					asset_in: assets.asset_in,
					asset_out: T::NativeAssetId::get()
				}),
				Error::<T>::CannotApplyDiscount
			);
		}

		let pair_account = Self::get_pair_id(assets);

		let asset_in_reserve: u128 = to_u128_result!(T::Currency::free_balance(assets.asset_in, &pair_account))?;
		let asset_out_reserve: u128 = to_u128_result!(T::Currency::free_balance(assets.asset_out, &pair_account))?;
		let amount_u128: u128 = to_u128_result!(amount)?;

		ensure!(
			amount_u128 <= asset_in_reserve.checked_div(MAX_IN_RATIO).ok_or(Error::<T>::Overflow)?,
			Error::<T>::MaxInRatioExceeded
		);

		let amount_out_u128 =
			hydra_dx_math::xyk::calculate_out_given_in(asset_in_reserve, asset_out_reserve, amount_u128)
				.map_err(|_| Error::<T>::SellAssetAmountInvalid)?;

		let amount_out = amount_out_u128.try_into().map_err(|_| Error::<T>::Overflow)?;

		let transfer_fee = if discount {
			Self::calculate_discounted_fee(amount_out)?
		} else {
			Self::calculate_fee(amount_out)?
		};

		let amount_out_without_fee = amount_out
			.checked_sub(&transfer_fee)
			.ok_or(Error::<T>::SellAssetAmountInvalid)?;

		ensure!(
			asset_out_reserve > amount_out_u128,
			Error::<T>::InsufficientAssetBalance
		);

		ensure!(
			min_bought <= amount_out_without_fee,
			Error::<T>::AssetAmountNotReachedLimit
		);

		let discount_fee = if discount {
			let native_asset = T::NativeAssetId::get();

			let native_pair_account = Self::get_pair_id(AssetPairT::<T::AssetId> {
				asset_in: assets.asset_in,
				asset_out: native_asset,
			});

			let native_reserve = T::Currency::free_balance(native_asset, &native_pair_account)
				.try_into()
				.map_err(|_| Error::<T>::Overflow)?;
			let asset_reserve = T::Currency::free_balance(assets.asset_in, &native_pair_account)
				.try_into()
				.map_err(|_| Error::<T>::Overflow)?;

			let native_fee_spot_price =
				hydra_dx_math::xyk::calculate_spot_price(asset_reserve, native_reserve, to_u128_result!(transfer_fee)?)
					.map_err(|_| Error::<T>::CannotApplyDiscount)?
					.try_into()
					.map_err(|_| Error::<T>::Overflow)?;

			ensure!(
				T::Currency::free_balance(native_asset, who) >= native_fee_spot_price,
				Error::<T>::InsufficientNativeCurrencyBalance
			);

			native_fee_spot_price
		} else {
			BalanceOf::<T>::zero()
		};

		let transfer = AMMTransfer {
			origin: who.clone(),
			assets,
			amount,
			amount_out: amount_out_without_fee,
			discount,
			discount_amount: discount_fee,
			fee: (assets.asset_out, transfer_fee),
		};

		Ok(transfer)
	}

	/// Execute sell. validate_sell must be called first.
	/// Perform necessary storage/state changes.
	/// Note : the execution should not return error as everything was previously verified and validated.
	#[transactional]
	fn execute_sell(
		transfer: &AMMTransfer<T::AccountId, T::AssetId, AssetPairT<T::AssetId>, BalanceOf<T>>,
	) -> DispatchResult {
		let pair_account = Self::get_pair_id(transfer.assets);

		if transfer.discount && transfer.discount_amount > BalanceOf::<T>::zero() {
			let native_asset = T::NativeAssetId::get();
			T::Currency::withdraw(native_asset, &transfer.origin, transfer.discount_amount)?;
		}

		T::Currency::transfer(
			transfer.assets.asset_in,
			&transfer.origin,
			&pair_account,
			transfer.amount,
		)?;
		T::Currency::transfer(
			transfer.assets.asset_out,
			&pair_account,
			&transfer.origin,
			transfer.amount_out,
		)?;

		Self::deposit_event(Event::<T>::SellExecuted(
			transfer.origin.clone(),
			transfer.assets.asset_in,
			transfer.assets.asset_out,
			transfer.amount,
			transfer.amount_out,
			transfer.fee.0,
			transfer.fee.1,
		));

		Ok(())
	}

	/// Validate a buy. Perform all necessary checks and calculations.
	/// No storage changes are performed yet.
	///
	/// Return `AMMTransfer` with all info needed to execute the transaction.
	fn validate_buy(
		who: &T::AccountId,
		assets: AssetPairT<T::AssetId>,
		amount: BalanceOf<T>,
		max_limit: BalanceOf<T>,
		discount: bool,
	) -> Result<AMMTransfer<T::AccountId, T::AssetId, AssetPairT<T::AssetId>, BalanceOf<T>>, DispatchError> {
		ensure!(
			amount >= MIN_TRADING_LIMIT.into(),
			Error::<T>::InsufficientTradingAmount
		);

		ensure!(Self::exists(assets), Error::<T>::TokenPoolNotFound);

		let pair_account = Self::get_pair_id(assets);

		let asset_out_reserve = T::Currency::free_balance(assets.asset_out, &pair_account);
		let asset_in_reserve = T::Currency::free_balance(assets.asset_in, &pair_account);

		ensure!(asset_out_reserve > amount, Error::<T>::InsufficientPoolAssetBalance);

		ensure!(
			amount
				<= asset_out_reserve
					.checked_div(&to_balance_result!(MAX_OUT_RATIO)?)
					.ok_or(Error::<T>::Overflow)?,
			Error::<T>::MaxOutRatioExceeded
		);

		// If discount, pool for Sell asset and native asset must exist
		if discount {
			ensure!(
				Self::exists(AssetPairT::<T::AssetId> {
					asset_in: assets.asset_out,
					asset_out: T::NativeAssetId::get()
				}),
				Error::<T>::CannotApplyDiscount
			);
		}

		let buy_price = hydra_dx_math::xyk::calculate_in_given_out(
			to_u128_result!(asset_out_reserve)?,
			to_u128_result!(asset_in_reserve)?,
			to_u128_result!(amount)?,
		)
		.map_err(|_| Error::<T>::BuyAssetAmountInvalid)?
		.try_into()
		.map_err(|_| Error::<T>::Overflow)?;

		let transfer_fee = if discount {
			Self::calculate_discounted_fee(buy_price)?
		} else {
			Self::calculate_fee(buy_price)?
		};

		let buy_price_with_fee = buy_price
			.checked_add(&transfer_fee)
			.ok_or(Error::<T>::BuyAssetAmountInvalid)?;

		ensure!(max_limit >= buy_price_with_fee, Error::<T>::AssetAmountExceededLimit);

		ensure!(
			T::Currency::free_balance(assets.asset_in, who) >= buy_price_with_fee,
			Error::<T>::InsufficientAssetBalance
		);

		let discount_fee = if discount {
			let native_asset = T::NativeAssetId::get();

			let native_pair_account = Self::get_pair_id(AssetPairT::<T::AssetId> {
				asset_in: assets.asset_out,
				asset_out: native_asset,
			});

			let native_reserve = T::Currency::free_balance(native_asset, &native_pair_account);
			let asset_reserve = T::Currency::free_balance(assets.asset_out, &native_pair_account);

			let native_fee_spot_price = hydra_dx_math::xyk::calculate_spot_price(
				to_u128_result!(asset_reserve)?,
				to_u128_result!(native_reserve)?,
				to_u128_result!(transfer_fee)?,
			)
			.map_err(|_| Error::<T>::CannotApplyDiscount)?
			.try_into()
			.map_err(|_| Error::<T>::Overflow)?;

			ensure!(
				T::Currency::free_balance(native_asset, who) >= native_fee_spot_price,
				Error::<T>::InsufficientNativeCurrencyBalance
			);
			native_fee_spot_price
		} else {
			BalanceOf::<T>::zero()
		};

		let transfer = AMMTransfer {
			origin: who.clone(),
			assets,
			amount,
			amount_out: buy_price,
			discount,
			discount_amount: discount_fee,
			fee: (assets.asset_in, transfer_fee),
		};

		Ok(transfer)
	}

	/// Execute buy. validate_buy must be called first.
	/// Perform necessary storage/state changes.
	/// Note : the execution should not return error as everything was previously verified and validated.
	#[transactional]
	fn execute_buy(
		transfer: &AMMTransfer<T::AccountId, T::AssetId, AssetPairT<T::AssetId>, BalanceOf<T>>,
	) -> DispatchResult {
		let pair_account = Self::get_pair_id(transfer.assets);

		if transfer.discount && transfer.discount_amount > BalanceOf::<T>::zero() {
			let native_asset = T::NativeAssetId::get();
			T::Currency::withdraw(native_asset, &transfer.origin, transfer.discount_amount)?;
		}

		T::Currency::transfer(
			transfer.assets.asset_out,
			&pair_account,
			&transfer.origin,
			transfer.amount,
		)?;
		T::Currency::transfer(
			transfer.assets.asset_in,
			&transfer.origin,
			&pair_account,
			transfer.amount_out + transfer.fee.1,
		)?;

		Self::deposit_event(Event::<T>::BuyExecuted(
			transfer.origin.clone(),
			transfer.assets.asset_out,
			transfer.assets.asset_in,
			transfer.amount,
			transfer.amount_out,
			transfer.fee.0,
			transfer.fee.1,
		));

		Ok(())
	}
}
