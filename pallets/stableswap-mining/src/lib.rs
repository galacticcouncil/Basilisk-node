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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;
//pub mod migration;
pub mod weights;

pub use pallet::*;

use frame_support::{
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, BlockNumberProvider},
	traits::tokens::nonfungibles::{Create, Inspect, Mutate},
	transactional, PalletId,
};
use frame_system::ensure_signed;
use hydradx_traits::liquidity_mining::Mutate as LiquidityMiningMutate;
use orml_traits::MultiCurrency;
use pallet_stableswap::{traits::ShareAccountIdFor, types::PoolId, POOL_IDENTIFIER};
use primitives::{AssetId, Balance, InstanceId as NftInstanceId};
use sp_arithmetic::{FixedU128, Permill};
use sp_std::convert::{From, Into};
use warehouse_liquidity_mining::{DepositId, FarmMultiplier, GlobalFarmId, LoyaltyCurve, YieldFarmId};

/// NFT class id type of provided nft implementation
type NFTClassIdOf<T> = <<T as Config>::NFTHandler as Inspect<<T as frame_system::Config>::AccountId>>::ClassId;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type PeriodOf<T> = <T as frame_system::Config>::BlockNumber;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_stableswap::Config<AssetId = AssetId, Currency = <Self as pallet::Config>::MultiCurrency>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Currency for transfers.
		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetId, Balance = Balance>;

		/// The origin account that can create new liquidity mining program.
		type CreateOrigin: EnsureOrigin<Self::Origin>;

		/// Pallet id.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The block number provider
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = Self::BlockNumber>;

		/// NFT class id for liq. mining deposit nfts. Has to be within the range of reserved NFT class IDs.
		type NFTClassId: Get<NFTClassIdOf<Self>>;

		/// Non fungible handling - mint,burn, check owner
		type NFTHandler: Mutate<Self::AccountId>
			+ Create<Self::AccountId>
			+ Inspect<Self::AccountId, InstanceId = DepositId, ClassId = Self::NFTClassId>;

		type LiquidityMiningHandler: LiquidityMiningMutate<
			Self::AccountId,
			AssetId,
			BlockNumberFor<Self>,
			Error = DispatchError,
			AmmPoolId = PoolId<AssetId>,
			Balance = Balance,
			LoyaltyCurve = LoyaltyCurve,
			Period = PeriodOf<Self>,
		>;

		/// Weight information for extrinsic in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		/// A pool with given assets does not exist.
		StableswapPoolNotFound,

		/// Account is not deposit owner.
		NotDepositOwner,

		/// Deposit does not exists.
		DepositNotFound,

		/// Yield farm entry doesn't exist for given deposit.
		YieldFarmEntryNotFound,

		/// Provided asset is not in stableswap pool
		AssetNotInStableswapPool,

		/// Balance of LP tokens if not sufficient to create deposit.
		InsufficientLpShares,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[allow(clippy::too_many_arguments)]
		#[pallet::weight(<T as Config>::WeightInfo::create_farm())]
		#[transactional]
		pub fn create_global_farm(
			origin: OriginFor<T>,
			total_rewards: Balance,
			planned_yielding_periods: PeriodOf<T>,
			blocks_per_period: BlockNumberFor<T>,
			incentivized_asset: AssetId,
			reward_currency: AssetId,
			owner: AccountIdOf<T>,
			yield_per_period: Permill,
			min_deposit: Balance,
			price_adujustment: FixedU128,
		) -> DispatchResult {
			T::CreateOrigin::ensure_origin(origin)?;

			let (_global_farm_id, _max_reward_per_period) = T::LiquidityMiningHandler::create_global_farm(
				total_rewards,
				planned_yielding_periods,
				blocks_per_period,
				incentivized_asset,
				reward_currency,
				owner,
				yield_per_period,
				min_deposit,
				price_adujustment,
			)?;

			//TODO: deposit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::destroy_farm())]
		#[transactional]
		pub fn destroy_global_farm(origin: OriginFor<T>, farm_id: GlobalFarmId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let (_reward_currency, _undistributed_rewards, _who) =
				T::LiquidityMiningHandler::destroy_global_farm(who, farm_id)?;

			//TODO: deposit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity_pool())]
		#[transactional]
		pub fn create_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			pool_id: PoolId<AssetId>,
			multiplier: FarmMultiplier,
			loyalty_curve: Option<LoyaltyCurve>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let pool = pallet_stableswap::Pallet::<T>::pools(pool_id).ok_or(Error::<T>::StableswapPoolNotFound)?;

			let _yield_farm_id = T::LiquidityMiningHandler::create_yield_farm(
				who,
				global_farm_id,
				multiplier,
				loyalty_curve,
				pool_id,
				pool.assets.0,
				pool.assets.1,
			)?;

			//TODO: emit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_liquidity_pool())]
		#[transactional]
		pub fn update_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			pool_id: PoolId<AssetId>,
			multiplier: FarmMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let _yield_farm_id =
				T::LiquidityMiningHandler::update_yield_farm_multiplier(who, global_farm_id, pool_id, multiplier)?;

			//TODO: emit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::cancel_liquidity_pool())]
		#[transactional]
		pub fn stop_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			pool_id: PoolId<AssetId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let _stopped_yield_farm_id = T::LiquidityMiningHandler::stop_yield_farm(who, global_farm_id, pool_id)?;

			//TODO: emit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::resume_liquidity_pool())]
		#[transactional]
		pub fn resume_liquidity_pool(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: PoolId<AssetId>,
			multiplier: FarmMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				pallet_stableswap::Pools::<T>::contains_key(pool_id),
				Error::<T>::StableswapPoolNotFound
			);

			let _ =
				T::LiquidityMiningHandler::resume_yield_farm(who, global_farm_id, yield_farm_id, pool_id, multiplier)?;

			//TODO: emit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::remove_liquidity_pool())]
		#[transactional]
		pub fn destroy_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: PoolId<AssetId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::LiquidityMiningHandler::destroy_yield_farm(who, global_farm_id, yield_farm_id, pool_id)?;

			//TODO: emit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::deposit_shares())]
		#[transactional]
		pub fn deposit_lp_shares(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: PoolId<AssetId>,
			amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let lp_token = Self::get_lp_token(pool_id)?;

			//Check LP shares balance
			ensure!(
				T::MultiCurrency::free_balance(lp_token, &who) >= amount,
				Error::<T>::InsufficientLpShares
			);

			let deposit_id = T::LiquidityMiningHandler::deposit_lp_shares(
				global_farm_id,
				yield_farm_id,
				pool_id,
				amount,
				Self::get_asset_balance_in_stableswap_pool,
			)?;

			//Lock LP shares.
			T::MultiCurrency::transfer(lp_token, &who, &Self::account_id(), amount)?;

			//Create NFT
			T::NFTHandler::mint_into(&T::NFTClassId::get(), &deposit_id, &who)?;

			//TODO:
			// * emit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::deposit_shares())]
		#[transactional]
		pub fn redeposit_lp_shares(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			nft_id: NftInstanceId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				T::NFTHandler::owner(&T::NFTClassId::get(), &nft_id) == Some(who.clone()),
				Error::<T>::NotDepositOwner
			);

			let _redeposited_shares = T::LiquidityMiningHandler::redeposit_lp_shares(
				global_farm_id,
				yield_farm_id,
				nft_id,
				Self::get_asset_balance_in_stableswap_pool,
			)?;

			//TODO: emit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::claim_rewards())]
		#[transactional]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			nft_id: NftInstanceId,
			yield_farm_id: YieldFarmId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				T::NFTHandler::owner(&T::NFTClassId::get(), &nft_id) == Some(who.clone()),
				Error::<T>::NotDepositOwner
			);

			const FAIL_ON_DOUBLE_CLAIM: bool = true;
			let (_global_farm_id, _reward_currency, _claimed, _) =
				T::LiquidityMiningHandler::claim_rewards(who, nft_id, yield_farm_id, FAIL_ON_DOUBLE_CLAIM)?;

			//TODO: emit event

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::withdraw_shares())]
		#[transactional]
		pub fn withdraw_shares(
			origin: OriginFor<T>,
			nft_id: NftInstanceId,
			yield_farm_id: YieldFarmId,
			pool_id: PoolId<AssetId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				T::NFTHandler::owner(&T::NFTClassId::get(), &nft_id) == Some(who.clone()),
				Error::<T>::NotDepositOwner
			);

			let lp_token = Self::get_lp_token(pool_id)?;

			let global_farm_id = T::LiquidityMiningHandler::get_global_farm_id(nft_id, yield_farm_id)
				.ok_or(Error::<T>::DepositNotFound)?;

			let mut unclaimable_rewards = 0;
			if T::LiquidityMiningHandler::is_yield_farm_claimable(global_farm_id, yield_farm_id, pool_id) {
				let (_, _, _claimed, unclaimable) =
					T::LiquidityMiningHandler::claim_rewards(who.clone(), nft_id, yield_farm_id, false)?;

				unclaimable_rewards = unclaimable;

				//TODO: emit claimed event
			}

			let (_global_farm_id, withdrawn_amount, deposit_destroyed) =
				T::LiquidityMiningHandler::withdraw_lp_shares(nft_id, yield_farm_id, unclaimable_rewards)?;

			if deposit_destroyed {
				//Unlock LP tokens
				T::MultiCurrency::transfer(lp_token, &Self::account_id(), &who, withdrawn_amount)?;

				//Destroy NFT
				T::NFTHandler::burn_from(&T::NFTClassId::get(), &nft_id)?;
			}

			//TODO:
			// * emit event

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Account id of pot holding all the shares
	fn account_id() -> AccountIdOf<T> {
		<T as pallet::Config>::PalletId::get().into_account()
	}

	fn get_lp_token(id: PoolId<AssetId>) -> Result<AssetId, Error<T>> {
		let pool = pallet_stableswap::Pallet::<T>::pools(id).ok_or(Error::<T>::StableswapPoolNotFound)?;

		Ok(pool.assets.0)
	}

	fn get_asset_balance_in_stableswap_pool(
		asset: AssetId,
		pool_id: PoolId<AssetId>,
	) -> Result<Balance, DispatchError> {
		let pool = pallet_stableswap::Pallet::<T>::pools(pool_id).ok_or(Error::<T>::StableswapPoolNotFound)?;

		ensure!(pool.assets.contains(asset), Error::<T>::AssetNotInStableswapPool);

		let pool_account =
			<T as pallet_stableswap::Config>::ShareAccountId::from_assets(&pool.assets, Some(POOL_IDENTIFIER));

		Ok(T::MultiCurrency::total_balance(asset, &pool_account))
	}
}
