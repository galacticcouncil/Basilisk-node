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

#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarking;

#[cfg(test)]
mod tests;

pub mod migration;
pub mod weights;

pub use pallet::*;

use frame_support::{
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, BlockNumberProvider, Zero},
	traits::tokens::nonfungibles::{Create, Inspect, Mutate},
	PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use hydradx_traits::{
	liquidity_mining::Mutate as LiquidityMiningMutate,
	nft::{CreateTypedCollection, ReserveCollectionId},
};
use orml_traits::MultiCurrency;
use pallet_liquidity_mining::{FarmMultiplier, GlobalFarmId, LoyaltyCurve, YieldFarmId};
use pallet_nft::CollectionType;
use primitives::{AssetId, Balance, ItemId as DepositId};
use sp_arithmetic::{FixedU128, Perquintill};
use sp_std::convert::{From, Into};

type PeriodOf<T> = <T as frame_system::Config>::BlockNumber;

#[frame_support::pallet]
#[allow(clippy::too_many_arguments)]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use frame_system::pallet_prelude::BlockNumberFor;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
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

		/// NFT collection id for liq. mining deposit nfts. Has to be within the range of reserved NFT class IDs.
		#[pallet::constant]
		type NftCollectionId: Get<primitives::CollectionId>;

		/// Non fungible handling
		type NFTHandler: Mutate<Self::AccountId>
			+ Create<Self::AccountId>
			+ Inspect<Self::AccountId, CollectionId = primitives::CollectionId, ItemId = DepositId>
			+ CreateTypedCollection<Self::AccountId, primitives::CollectionId, CollectionType>
			+ ReserveCollectionId<primitives::CollectionId>;

		type LiquidityMiningHandler: LiquidityMiningMutate<
			Self::AccountId,
			AssetId,
			BlockNumberFor<Self>,
			Error = DispatchError,
			AmmPoolId = AssetId,
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

        /// 
        CantFindDepositOwner,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		GlobalFarmCreated {
			id: GlobalFarmId,
			owner: T::AccountId,
			total_rewards: Balance,
			reward_currency: AssetId,
			yield_per_period: Perquintill,
			planned_yielding_periods: PeriodOf<T>,
			blocks_per_period: BlockNumberFor<T>,
			incentivized_asset: AssetId,
			max_reward_per_period: Balance,
			min_deposit: Balance,
			price_adjustment: FixedU128,
		},

		GlobalFarmUpdated {
			who: T::AccountId,
			id: GlobalFarmId,
			price_adjustment: FixedU128,
		},

		GlobalFarmDestroyed {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			reward_currency: AssetId,
			undistributed_rewards: Balance,
		},

		YieldFarmCreated {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
			multiplier: FarmMultiplier,
			loyalty_curve: Option<LoyaltyCurve>,
		},

		YieldFarmUpdated {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
			multiplier: FarmMultiplier,
		},

		YieldFarmStopped {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
		},

		YieldFarmResumed {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
			multiplier: FarmMultiplier,
		},

		YieldFarmDestroyed {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
		},

		SharesDeposited {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			deposit_id: DepositId,
			lp_token: AssetId,
			amount: Balance,
		},

		SharesRedeposited {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			deposit_id: DepositId,
		},

		RewardsClaimed {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			deposit_id: DepositId,
			reward_currency: AssetId,
			claimed: Balance,
		},

		SharesWithdrawn {
			who: T::AccountId,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			lp_token: AssetId,
			amount: Balance,
		},

		DepositDestroyed {
			who: T::AccountId,
			deposit_id: DepositId,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn create_global_farm(
			origin: OriginFor<T>,
			total_rewards: Balance,
			planned_yielding_periods: PeriodOf<T>,
			blocks_per_period: BlockNumberFor<T>,
			incentivized_asset: AssetId,
			reward_currency: AssetId,
			owner: T::AccountId,
			yield_per_period: Perquintill,
			min_deposit: Balance,
			price_adjustment: FixedU128,
		) -> DispatchResult {
			T::CreateOrigin::ensure_origin(origin)?;

			let (id, max_reward_per_period) = T::LiquidityMiningHandler::create_global_farm(
				total_rewards,
				planned_yielding_periods,
				blocks_per_period,
				incentivized_asset,
				reward_currency,
				owner.clone(),
				yield_per_period,
				min_deposit,
				price_adjustment,
			)?;

			Self::deposit_event(Event::GlobalFarmCreated {
				id,
				owner,
				reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				incentivized_asset,
				max_reward_per_period,
				min_deposit,
				price_adjustment,
				total_rewards,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn update_global_farm_price_adjustment(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			price_adjustment: FixedU128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::LiquidityMiningHandler::update_global_farm_price_adjustment(
				who.clone(),
				global_farm_id,
				price_adjustment,
			)?;

			Self::deposit_event(Event::GlobalFarmUpdated {
				who,
				id: global_farm_id,
				price_adjustment,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn destroy_global_farm(origin: OriginFor<T>, global_farm_id: GlobalFarmId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let (reward_currency, undistributed_rewards, _who) =
				T::LiquidityMiningHandler::destroy_global_farm(who.clone(), global_farm_id)?;

			Self::deposit_event(Event::GlobalFarmDestroyed {
				who,
				global_farm_id,
				reward_currency,
				undistributed_rewards,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn create_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			pool_id: AssetId,
			multiplier: FarmMultiplier,
			loyalty_curve: Option<LoyaltyCurve>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let pool = pallet_stableswap::Pallet::<T>::pools(pool_id).ok_or(Error::<T>::StableswapPoolNotFound)?;

			let yield_farm_id = T::LiquidityMiningHandler::create_yield_farm(
				who,
				global_farm_id,
				multiplier,
				loyalty_curve.clone(),
				pool_id,
				pool.assets.to_vec(),
			)?;

			Self::deposit_event(Event::YieldFarmCreated {
				global_farm_id,
				yield_farm_id,
				pool_id,
				multiplier,
				loyalty_curve,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn update_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			pool_id: AssetId,
			multiplier: FarmMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				pallet_stableswap::Pools::<T>::contains_key(pool_id),
				Error::<T>::StableswapPoolNotFound
			);

			let yield_farm_id = T::LiquidityMiningHandler::update_yield_farm_multiplier(
				who.clone(),
				global_farm_id,
				pool_id,
				multiplier,
			)?;

			Self::deposit_event(Event::YieldFarmUpdated {
				who,
				global_farm_id,
				yield_farm_id,
				pool_id,
				multiplier,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn stop_yield_farm(origin: OriginFor<T>, global_farm_id: GlobalFarmId, pool_id: AssetId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//NOTE: don't check pool existance, owner must be able to stop yield farm.
			let yield_farm_id = T::LiquidityMiningHandler::stop_yield_farm(who.clone(), global_farm_id, pool_id)?;

			Self::deposit_event(Event::YieldFarmStopped {
				who,
				global_farm_id,
				yield_farm_id,
				pool_id,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn resume_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
			multiplier: FarmMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				pallet_stableswap::Pools::<T>::contains_key(pool_id),
				Error::<T>::StableswapPoolNotFound
			);

			T::LiquidityMiningHandler::resume_yield_farm(
				who.clone(),
				global_farm_id,
				yield_farm_id,
				pool_id,
				multiplier,
			)?;

			Self::deposit_event(Event::YieldFarmResumed {
				who,
				global_farm_id,
				yield_farm_id,
				pool_id,
				multiplier,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn destroy_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::LiquidityMiningHandler::destroy_yield_farm(who.clone(), global_farm_id, yield_farm_id, pool_id)?;

			Self::deposit_event(Event::YieldFarmDestroyed {
				who,
				global_farm_id,
				yield_farm_id,
				pool_id,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn deposit_shares(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
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
				Self::get_token_value_of_lp_shares,
			)?;

			//Lock LP shares.
			T::MultiCurrency::transfer(lp_token, &who, &Self::account_id(), amount)?;

			//Create NFT
			T::NFTHandler::mint_into(&T::NftCollectionId::get(), &deposit_id, &who)?;

			Self::deposit_event(Event::SharesDeposited {
				who,
				global_farm_id,
				yield_farm_id,
				deposit_id,
				lp_token,
				amount,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn redeposit_shares(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			deposit_id: DepositId,
		) -> DispatchResult {
            let owner = Self::ensure_nft_owner(origin, deposit_id)?;

			let _ = T::LiquidityMiningHandler::redeposit_lp_shares(
				global_farm_id,
				yield_farm_id,
				deposit_id,
				Self::get_token_value_of_lp_shares,
			)?;

			Self::deposit_event(Event::SharesRedeposited {
				who: owner,
				global_farm_id,
				yield_farm_id,
				deposit_id,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			deposit_id: DepositId,
			yield_farm_id: YieldFarmId,
		) -> DispatchResult {
			let owner = Self::ensure_nft_owner(origin, deposit_id)?;

			const FAIL_ON_DOUBLE_CLAIM: bool = true;
			let (global_farm_id, reward_currency, claimed, _) =
				T::LiquidityMiningHandler::claim_rewards(owner.clone(), deposit_id, yield_farm_id, FAIL_ON_DOUBLE_CLAIM)?;

			Self::deposit_event(Event::RewardsClaimed {
				who: owner,
				global_farm_id,
				yield_farm_id,
				deposit_id,
				reward_currency,
				claimed,
			});

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn withdraw_shares(
			origin: OriginFor<T>,
			deposit_id: DepositId,
			yield_farm_id: YieldFarmId,
			pool_id: AssetId,
		) -> DispatchResult {
            let owner = Self::ensure_nft_owner(origin, deposit_id)?;

			let lp_token = Self::get_lp_token(pool_id)?;

			let global_farm_id = T::LiquidityMiningHandler::get_global_farm_id(deposit_id, yield_farm_id)
				.ok_or(Error::<T>::DepositNotFound)?;

			let mut unclaimable_rewards = 0;
			if T::LiquidityMiningHandler::is_yield_farm_claimable(global_farm_id, yield_farm_id, pool_id) {
				let (global_farm_id, reward_currency, claimed, unclaimable) =
					T::LiquidityMiningHandler::claim_rewards(owner.clone(), deposit_id, yield_farm_id, false)?;

				unclaimable_rewards = unclaimable;

				if claimed.gt(&Balance::zero()) {
					Self::deposit_event(Event::RewardsClaimed {
						who: owner.clone(),
						global_farm_id,
						yield_farm_id,
						deposit_id,
						reward_currency,
						claimed,
					});
				}
			}

			let (global_farm_id, withdrawn_amount, deposit_destroyed) =
				T::LiquidityMiningHandler::withdraw_lp_shares(deposit_id, yield_farm_id, unclaimable_rewards)?;

			Self::deposit_event(Event::SharesWithdrawn {
				who: owner.clone(),
				global_farm_id,
				yield_farm_id,
				lp_token,
				amount: withdrawn_amount,
			});

			if deposit_destroyed {
				//Unlock LP tokens
				T::MultiCurrency::transfer(lp_token, &Self::account_id(), &owner, withdrawn_amount)?;

				//Destroy NFT
				T::NFTHandler::burn(&T::NftCollectionId::get(), &deposit_id, Some(&owner))?;

				Self::deposit_event(Event::DepositDestroyed { who: owner, deposit_id });
			}

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Account id of pot holding all the shares
	fn account_id() -> T::AccountId {
		<T as pallet::Config>::PalletId::get().into_account_truncating()
	}

	fn get_lp_token(pool_id: AssetId) -> Result<AssetId, Error<T>> {
		ensure!(
			pallet_stableswap::Pools::<T>::contains_key(pool_id),
			Error::<T>::StableswapPoolNotFound
		);

		Ok(pool_id)
	}

	fn get_token_value_of_lp_shares(
		_asset: AssetId,
		_pool_id: AssetId,
		_lp_shares_amount: Balance,
	) -> Result<Balance, DispatchError> {
		todo!()
	}
    
    //TODO: create generic function, this is coppied from xyk-lm-pallet
    fn ensure_nft_owner(origin: OriginFor<T>, deposit_id: DepositId) -> Result<T::AccountId, DispatchError> {
		let who = ensure_signed(origin)?;

		let nft_owner =
			T::NFTHandler::owner(&T::NftCollectionId::get(), &deposit_id).ok_or(Error::<T>::CantFindDepositOwner)?;

		ensure!(nft_owner == who, Error::<T>::NotDepositOwner);

		Ok(who)
	}
}
