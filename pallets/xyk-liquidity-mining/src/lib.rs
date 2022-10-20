// This file is part of HydraDX

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
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

//! ## Overview
//!
//! This pallet provide functionality for liquidity mining program with time incentive(loyalty
//! factor) and multiple incentives scheme for basilisk.
//! Users are rewarded for each period they stay in liq. mining program.
//!
//! Reward per one period is derived from the user's loyalty factor which grows with time(periods)
//! the user is in the liq. mining and amount of LP shares user locked into deposit.
//! User's loyalty factor is reset if the user exits and reenters liquidity mining.
//! User can claim rewards without resetting loyalty factor, only withdrawing shares
//! is penalized by loyalty factor reset.
//! User is rewarded from the next period after he enters.
//!
//! Multiple Incentives
//!
//! This feature allow users to redeposit already deposited LP shares to multiple yield farms and
//! receive incentives from this farms.
//! Deposit in yield farm is called "farm entry".
//! Maximal number of redepositing same LP shares is configured by variable: `MaxFarmEntriesPerDeposit`.
//! Set `MaxFarmEntriesPerDeposit` to `1` to disable multiple incentives scheme. !!!NEVER set this
//! variable to `0`.
//! LP shares can be redeposited only to different yield farms running liquidity mining for same
//! pair of assets.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

pub mod migration;
pub mod weights;

pub use pallet::*;

use frame_support::traits::tokens::nonfungibles::{Create, Inspect, Mutate};
use frame_support::{
	ensure,
	sp_runtime::traits::{BlockNumberProvider, Zero},
	PalletId,
};
use hydradx_traits_lm::liquidity_mining::{GlobalFarmId, Mutate as LiquidityMiningMutate, YieldFarmId};
use warehouse_liquidity_mining::{FarmMultiplier, LoyaltyCurve};

use frame_support::{pallet_prelude::*, sp_runtime::traits::AccountIdConversion};
use frame_system::ensure_signed;
use hydradx_traits::{nft::CreateTypedCollection, AMM};
use orml_traits::MultiCurrency;
use pallet_nft::CollectionType;
use primitives::{asset::AssetPair, AssetId, Balance, CollectionId as DepositId};
use scale_info::TypeInfo;
use sp_arithmetic::{FixedU128, Perquintill};
use sp_std::{
	convert::{From, Into, TryInto},
	vec,
};

type PeriodOf<T> = <T as frame_system::Config>::BlockNumber;

#[frame_support::pallet]
#[allow(clippy::too_many_arguments)]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
	use hydradx_traits::pools::DustRemovalAccountWhitelist;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::genesis_config]
	#[cfg_attr(feature = "std", derive(Default))]
	pub struct GenesisConfig {}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			let pallet_account = <Pallet<T>>::account_id();

			T::NonDustableWhitelistHandler::add_account(&pallet_account).unwrap();

			T::NFTHandler::create_typed_collection(
				pallet_account,
				T::NftCollectionId::get(),
				CollectionType::LiquidityMining,
			)
			.unwrap();
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + TypeInfo {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Currency for transfers.
		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetId, Balance = Balance>;

		/// AMM helper functions.
		type AMM: AMM<Self::AccountId, AssetId, AssetPair, Balance>;

		/// The origin account that can create new liquidity mining program.
		type CreateOrigin: EnsureOrigin<Self::Origin>;

		/// Pallet id.
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
			+ CreateTypedCollection<Self::AccountId, primitives::CollectionId, CollectionType>;

		/// Liquidity mining handler for managing liquidity mining functionalities
		type LiquidityMiningHandler: LiquidityMiningMutate<
			Self::AccountId,
			AssetId,
			BlockNumberFor<Self>,
			Error = DispatchError,
			AmmPoolId = Self::AccountId,
			Balance = Balance,
			LoyaltyCurve = LoyaltyCurve,
			Period = PeriodOf<Self>,
		>;

		/// Account whitelist manager to exclude pool accounts from dusting mechanism.
		type NonDustableWhitelistHandler: DustRemovalAccountWhitelist<Self::AccountId, Error = DispatchError>;

		/// Weight information for extrinsic in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq, Eq))]
	pub enum Error<T> {
		/// Nft pallet didn't return an owner.
		CantFindDepositOwner,

		/// Account balance of amm pool shares is not sufficient.
		InsufficientAmmSharesBalance,

		/// AMM pool does not exist
		AmmPoolDoesNotExist,

		/// Account is not deposit owner.
		NotDepositOwner,

		/// AMM did not return assets for given `amm_pool_id`
		CantGetAmmAssets,

		/// Yield farm can not be found
		YieldFarmNotFound,

		///Deposit data not found
		DepositDataNotFound,

		/// Calculated reward to claim is 0.
		ZeroClaimedRewards,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New global farm was created.
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

		/// Global farm's `price_adjustment` was updated.
		GlobalFarmUpdated {
			id: GlobalFarmId,
			price_adjustment: FixedU128,
		},

		/// New yield farm was added into the farm.
		YieldFarmCreated {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			multiplier: FarmMultiplier,
			asset_pair: AssetPair,
			loyalty_curve: Option<LoyaltyCurve>,
		},

		/// Global farm was destroyed.
		GlobalFarmDestroyed {
			global_farm_id: GlobalFarmId,
			who: T::AccountId,
			reward_currency: AssetId,
			undistributed_rewards: Balance,
		},

		/// New LP tokens was deposited.
		SharesDeposited {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: T::AccountId,
			amount: Balance,
			lp_token: AssetId,
			deposit_id: DepositId,
		},

		/// LP token was redeposited for a new yield farm entry
		SharesRedeposited {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: T::AccountId,
			amount: Balance,
			lp_token: AssetId,
			deposit_id: DepositId,
		},

		/// Rewards was claimed.
		RewardClaimed {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: T::AccountId,
			claimed: Balance,
			reward_currency: AssetId,
			deposit_id: DepositId,
		},

		/// LP tokens was withdrawn.
		SharesWithdrawn {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: T::AccountId,
			lp_token: AssetId,
			amount: Balance,
			deposit_id: DepositId,
		},

		/// Yield farm for asset pair was stopped.
		YieldFarmStopped {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: T::AccountId,
			asset_pair: AssetPair,
		},

		/// Yield farm for asset pair was resumed.
		YieldFarmResumed {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: T::AccountId,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
		},

		/// Yield farm was destroyed from global farm.
		YieldFarmDestroyed {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: T::AccountId,
			asset_pair: AssetPair,
		},

		/// Yield farm multiplier was updated.
		YieldFarmUpdated {
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: T::AccountId,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
		},

		/// NFT representing deposit has been destroyed
		DepositDestroyed { who: T::AccountId, deposit_id: DepositId },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create new liquidity mining program with provided parameters.
		///
		/// `owner` account have to have at least `total_rewards` balance. This fund will be
		/// transferred from `owner` to farm account.
		///
		/// The dispatch origin for this call must be `T::CreateOrigin`.
		///
		/// Parameters:
		/// - `origin`: global farm's owner.
		/// - `total_rewards`: total rewards planned to distribute. This rewards will be
		/// distributed between all yield farms in the global farm.
		/// - `planned_yielding_periods`: planned number of periods to distribute `total_rewards`.
		/// WARN: THIS IS NOT HARD DEADLINE. Not all rewards have to be distributed in
		/// `planned_yielding_periods`. Rewards are distributed based on the situation in the yield
		/// farms and can be distributed in a longer time frame but never in the shorter time frame.
		/// - `blocks_per_period`:  number of blocks in a single period. Min. number of blocks per
		/// period is 1.
		/// - `incentivized_asset`: asset to be incentivized in AMM pools. All yield farms added into
		/// liq. mining program have to have `incentivized_asset` in their pair.
		/// - `reward_currency`: payoff currency of rewards.
		/// - `owner`: liq. mining program owner.
		/// - `yield_per_period`: percentage return on `reward_currency` of all farms p.a.
		/// - `min_deposit`: minimum amount which can be deposited to the farm
		/// - `price_adjustment`:
		/// Emits `GlobalFarmCreated` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::create_global_farm())]
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
				total_rewards,
				reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				incentivized_asset,
				max_reward_per_period,
				min_deposit,
				price_adjustment,
			});

			Ok(())
		}

		/// Update global farm's prices adjustment.
		///
		/// Only farm's owner can perform this action.
		///
		/// Parameters:
		/// - `origin`: global farm's owner.
		/// - `global_farm_id`: id of the global farm to update
		/// - `price_adjustment`: new value for price adjustment
		///
		/// Emits `GlobalFarmUpdated` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::update_global_farm())]
		pub fn update_global_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			price_adjustment: FixedU128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::LiquidityMiningHandler::update_global_farm_price_adjustment(who, global_farm_id, price_adjustment)?;

			Self::deposit_event(Event::GlobalFarmUpdated {
				id: global_farm_id,
				price_adjustment,
			});

			Ok(())
		}

		/// Destroy existing liq. mining program.
		///
		/// Only farm owner can perform this action.
		///
		/// WARN: To successfully destroy a farm, farm have to be empty(all yield farms in he global farm must be destroyed).
		///
		/// Parameters:
		/// - `origin`: global farm's owner.
		/// - `global_farm_id`: id of global farm to be destroyed.
		///
		/// Emits `FarmDestroyed` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::destroy_global_farm())]
		pub fn destroy_global_farm(origin: OriginFor<T>, global_farm_id: GlobalFarmId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let (reward_currency, undistributed_rewards, who) =
				T::LiquidityMiningHandler::destroy_global_farm(who, global_farm_id)?;

			Self::deposit_event(Event::GlobalFarmDestroyed {
				global_farm_id,
				who,
				reward_currency,
				undistributed_rewards,
			});
			Ok(())
		}

		/// Add yield farm for given `asset_pair` amm.
		///  
		/// Only farm owner can perform this action.
		///
		/// Only AMMs with `asset_pair` with `incentivized_asset` can be added into the farm. AMM
		/// for `asset_pair` has to exist to successfully create yield farm. Yield farm for same `asset_pair` can exist only once in the global farm.
		///
		/// Parameters:
		/// - `origin`: global farm's owner.
		/// - `farm_id`: global farm id to which a yield farm will be added.
		/// - `asset_pair`: asset pair identifying yield farm. Liq. mining will be allowed for this
		/// `asset_pair` and one of the assets in the pair must be `incentivized_asset`.
		/// - `multiplier`: yield farm multiplier.
		/// - `loyalty_curve`: curve to calculate loyalty multiplier to distribute rewards to users
		/// with time incentive. `None` means no loyalty multiplier.
		///
		/// Emits `YieldFarmCreated` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::create_yield_farm())]
		pub fn create_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
			loyalty_curve: Option<LoyaltyCurve>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);
			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let yield_farm_id = T::LiquidityMiningHandler::create_yield_farm(
				who,
				global_farm_id,
				multiplier,
				loyalty_curve.clone(),
				amm_pool_id,
				vec![asset_pair.asset_in, asset_pair.asset_out],
			)?;

			Self::deposit_event(Event::YieldFarmCreated {
				global_farm_id,
				yield_farm_id,
				multiplier,
				loyalty_curve,
				asset_pair,
			});

			Ok(())
		}

		/// Update yield farm multiplier.
		///  
		/// Only farm owner can perform this action.
		///
		/// Parameters:
		/// - `origin`: global farm's owner.
		/// - `global_farm_id`: global farm id in which yield farm will be updated.
		/// - `asset_pair`: asset pair identifying yield farm in global farm.
		/// - `multiplier`: new yield farm multiplier.
		///
		/// Emits `YieldFarmUpdated` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::update_yield_farm())]
		pub fn update_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);
			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let yield_farm_id = T::LiquidityMiningHandler::update_yield_farm_multiplier(
				who.clone(),
				global_farm_id,
				amm_pool_id,
				multiplier,
			)?;

			Self::deposit_event(Event::YieldFarmUpdated {
				global_farm_id,
				yield_farm_id,
				multiplier,
				who,
				asset_pair,
			});

			Ok(())
		}

		/// Stop liq. miming for specific yield farm.
		///
		/// This function claims rewards from `GlobalFarm` last time and stops yield farm
		/// incentivization from a `GlobalFarm`. Users will be able to only withdraw
		/// shares(with claiming) after calling this function.
		/// `deposit_shares()` and `claim_rewards()` are not allowed on canceled yield farm.
		///  
		/// Only farm owner can perform this action.
		///
		/// Parameters:
		/// - `origin`: global farm's owner.
		/// - `global_farm_id`: farm id in which yield farm will be canceled.
		/// - `asset_pair`: asset pair identifying yield farm in the farm.
		///
		/// Emits `YieldFarmStopped` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::stop_yield_farm())]
		pub fn stop_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			asset_pair: AssetPair,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);
			let yield_farm_id = T::LiquidityMiningHandler::stop_yield_farm(who.clone(), global_farm_id, amm_pool_id)?;

			Self::deposit_event(Event::YieldFarmStopped {
				global_farm_id,
				yield_farm_id,
				who,
				asset_pair,
			});

			Ok(())
		}

		/// Resume yield farm for stopped yield farm.
		///
		/// This function resume incentivization from `GlobalFarm` and restore full functionality
		/// for yield farm. Users will be able to deposit, claim and withdraw again.
		///
		/// WARN: Yield farm is NOT rewarded for time it was stopped.
		///
		/// Only farm owner can perform this action.
		///
		/// Parameters:
		/// - `origin`: global farm's owner.
		/// - `global_farm_id`: global farm id in which yield farm will be resumed.
		/// - `yield_farm_id`: id of yield farm to be resumed.
		/// - `asset_pair`: asset pair identifying yield farm in global farm.
		/// - `multiplier`: yield farm multiplier in the farm.
		///
		/// Emits `YieldFarmResumed` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::resume_yield_farm())]
		pub fn resume_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);
			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			T::LiquidityMiningHandler::resume_yield_farm(
				who.clone(),
				global_farm_id,
				yield_farm_id,
				amm_pool_id,
				multiplier,
			)?;

			Self::deposit_event(Event::<T>::YieldFarmResumed {
				global_farm_id,
				yield_farm_id,
				who,
				asset_pair,
				multiplier,
			});

			Ok(())
		}

		/// Remove yield farm
		///
		/// This function marks a yield farm as ready to be removed from storage when it's empty. Users will
		/// be able to only withdraw shares(without claiming rewards from yield farm). Unpaid rewards
		/// will be transferred back to global farm and will be used to distribute to other yield farms.
		///
		/// Yield farm must be stopped before calling this function.
		///
		/// Only global farm's owner can perform this action. Yield farm stays in the storage until it's
		/// empty(all farm entries are withdrawn). Last withdrawn from yield farm trigger removing from
		/// the storage.
		///
		/// Parameters:
		/// - `origin`: global farm's owner.
		/// - `global_farm_id`: farm id from which yield farm should be destroyed.
		/// - `yield_farm_id`: id of yield farm to be destroyed.
		/// - `asset_pair`: asset pair identifying yield farm in the global farm.
		///
		/// Emits `YieldFarmDestroyed` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::destroy_yield_farm())]
		pub fn destroy_yield_farm(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			T::LiquidityMiningHandler::destroy_yield_farm(who.clone(), global_farm_id, yield_farm_id, amm_pool_id)?;

			Self::deposit_event(Event::YieldFarmDestroyed {
				global_farm_id,
				yield_farm_id,
				who,
				asset_pair,
			});

			Ok(().into())
		}

		/// Deposit LP shares to a liq. mining.
		///
		/// This function transfers LP shares from `origin` to pallet's account and mint nft for
		/// `origin` account. Minted nft represents deposit in the liq. mining.
		///
		/// Parameters:
		/// - `origin`: account depositing LP shares. This account has to have at least
		/// `shares_amount` of LP shares.
		/// - `global_farm_id`: id of global farm to which user wants to deposit LP shares.
		/// - `yield_farm_id`: id of yield farm to deposit to.
		/// - `asset_pair`: asset pair identifying LP shares user wants to deposit.
		/// - `shares_amount`: amount of LP shares user wants to deposit.
		///
		/// Emits `SharesDeposited` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::deposit_shares())]
		pub fn deposit_shares(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
			shares_amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);
			let amm_share_token = T::AMM::get_share_token(asset_pair);

			ensure!(
				T::MultiCurrency::ensure_can_withdraw(amm_share_token, &who, shares_amount).is_ok(),
				Error::<T>::InsufficientAmmSharesBalance
			);

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let deposit_id = T::LiquidityMiningHandler::deposit_lp_shares(
				global_farm_id,
				yield_farm_id,
				amm_pool_id,
				shares_amount,
				Self::get_asset_balance_in_amm_pool,
			)?;

			Self::lock_lp_tokens(amm_share_token, &who, shares_amount)?;
			T::NFTHandler::mint_into(&T::NftCollectionId::get(), &deposit_id, &who)?;

			Self::deposit_event(Event::SharesDeposited {
				global_farm_id,
				yield_farm_id,
				who,
				amount: shares_amount,
				lp_token: amm_share_token,
				deposit_id,
			});

			Ok(())
		}

		/// This function create yield farm entry for existing deposit. LP shares are not transferred
		/// and amount of LP shares is based on existing deposit.
		///
		/// This function DOESN'T create new deposit.
		///
		/// Parameters:
		/// - `origin`: account depositing LP shares. This account have to have at least
		/// - `global_farm_id`: global farm identifier.
		/// - `yield_farm_id`: yield farm identifier redepositing to.
		/// - `asset_pair`: asset pair identifying LP shares user want to deposit.
		/// - `deposit_id`: identifier of the AMM pool.
		///
		/// Emits `SharesRedeposited` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::redeposit_lp_shares())]
		pub fn redeposit_lp_shares(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
			deposit_id: DepositId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nft_class_id = T::NftCollectionId::get();
			let nft_owner = T::NFTHandler::owner(&nft_class_id, &deposit_id).ok_or(Error::<T>::CantFindDepositOwner)?;

			ensure!(nft_owner == who, Error::<T>::NotDepositOwner);

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);
			let amm_share_token = T::AMM::get_share_token(asset_pair);

			let shares_amount = T::LiquidityMiningHandler::redeposit_lp_shares(
				global_farm_id,
				yield_farm_id,
				deposit_id,
				Self::get_asset_balance_in_amm_pool,
			)?;

			Self::deposit_event(Event::SharesRedeposited {
				global_farm_id,
				yield_farm_id,
				who,
				amount: shares_amount,
				lp_token: amm_share_token,
				deposit_id,
			});

			Ok(())
		}

		/// Claim rewards from liq. mining for deposit represented by `nft_id`.
		///
		/// This function calculate user rewards from liq. mining and transfer rewards to `origin`
		/// account. Claiming in the same period is allowed only once.
		///
		/// Parameters:
		/// - `origin`: account owner of deposit(nft).
		/// - `deposit_id`: nft id representing deposit in the yield farm.
		/// - `yield_farm_id`: yield farm identifier to claim rewards from.
		///
		/// Emits `RewardClaimed` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::claim_rewards())]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			deposit_id: DepositId,
			yield_farm_id: YieldFarmId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let deposit_owner = T::NFTHandler::owner(&T::NftCollectionId::get(), &deposit_id)
				.ok_or(Error::<T>::CantFindDepositOwner)?;

			ensure!(deposit_owner == who, Error::<T>::NotDepositOwner);

			let fail_on_double_claim = true;
			let (global_farm_id, reward_currency, claimed, _) =
				T::LiquidityMiningHandler::claim_rewards(who.clone(), deposit_id, yield_farm_id, fail_on_double_claim)?;

			ensure!(!claimed.is_zero(), Error::<T>::ZeroClaimedRewards);

			Self::deposit_event(Event::RewardClaimed {
				global_farm_id,
				yield_farm_id,
				who,
				claimed,
				reward_currency,
				deposit_id,
			});

			Ok(())
		}

		/// Withdraw LP shares from liq. mining. with reward claiming if possible.
		///
		/// Cases for transfer LP shares and claimed rewards:
		///
		/// * yield farm is active(yield farm is not stopped) - claim and transfer rewards(if it
		/// wasn't claimed in this period) and transfer LP shares.
		/// * liq. mining is stopped - claim and transfer rewards(if it
		/// wasn't claimed in this period) and transfer LP shares.
		/// * yield farm was destroyed - only LP shares will be transferred.
		/// * farm was destroyed - only LP shares will be transferred.
		/// * SPECIAL CASE: AMM pool does not exist - claim may happen if yield farm is still active, LP
		/// shares will not be transferred.
		///
		/// User's unclaimable rewards will be transferred back to global farm's account.
		///
		/// Parameters:
		/// - `origin`: account owner of deposit(nft).
		/// - `deposit_id`: nft id representing deposit in the yield farm.
		/// - `yield_farm_id`: yield farm identifier to dithdraw shares from.
		/// - `asset_pair`: asset pair identifying yield farm in global farm.
		///
		/// Emits:
		/// * `RewardClaimed` if claim happen
		/// * `SharesWithdrawn` event when successful
		#[pallet::weight(<T as Config>::WeightInfo::withdraw_shares())]
		pub fn withdraw_shares(
			origin: OriginFor<T>,
			deposit_id: DepositId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nft_owner = T::NFTHandler::owner(&T::NftCollectionId::get(), &deposit_id)
				.ok_or(Error::<T>::CantFindDepositOwner)?;

			ensure!(nft_owner == who, Error::<T>::NotDepositOwner);

			let global_farm_id = T::LiquidityMiningHandler::get_global_farm_id(deposit_id, yield_farm_id)
				.ok_or(Error::<T>::DepositDataNotFound)?;

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let unclaimable_rewards: Balance = if T::LiquidityMiningHandler::is_yield_farm_claimable(
				global_farm_id,
				yield_farm_id,
				amm_pool_id.clone(),
			) {
				//This should not fail on double claim, we need unclaimable_rewards
				let fail_on_double_claim = false;
				let (global_farm_id, reward_currency, claimed, unclaimable) = T::LiquidityMiningHandler::claim_rewards(
					who.clone(),
					deposit_id,
					yield_farm_id,
					fail_on_double_claim,
				)?;

				if !claimed.is_zero() {
					Self::deposit_event(Event::RewardClaimed {
						global_farm_id,
						yield_farm_id,
						who: who.clone(),
						claimed,
						reward_currency,
						deposit_id,
					});
				}

				unclaimable
			} else {
				0
			};

			let (global_farm_id, withdrawn_amount, is_destroyed) =
				T::LiquidityMiningHandler::withdraw_lp_shares(deposit_id, yield_farm_id, unclaimable_rewards)?;

			let lp_token = Self::get_lp_token(&amm_pool_id)?;
			if !withdrawn_amount.is_zero() {
				Self::deposit_event(Event::SharesWithdrawn {
					global_farm_id,
					yield_farm_id,
					who: who.clone(),
					lp_token,
					amount: withdrawn_amount,
					deposit_id,
				});
			}

			if is_destroyed {
				Self::unlock_lp_tokens(lp_token, &who, withdrawn_amount)?;
				T::NFTHandler::burn(&T::NftCollectionId::get(), &deposit_id, Some(&nft_owner))?;

				Self::deposit_event(Event::DepositDestroyed { who, deposit_id });
			}

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Account ID of the pot holding locked LP shares. This account is also owner of NFT class
	/// for all the NFTs minted by this pallet.
	fn account_id() -> T::AccountId {
		<T as pallet::Config>::PalletId::get().into_account_truncating()
	}

	fn get_lp_token(amm_pool_id: &T::AccountId) -> Result<AssetId, Error<T>> {
		let assets = T::AMM::get_pool_assets(amm_pool_id).ok_or(Error::<T>::CantGetAmmAssets)?;
		let asset_pair = AssetPair::new(assets[0], assets[1]);

		//NOTE: this check is important AMM:get_share_token() return `0` if amm doesn't exist
		if !T::AMM::exists(asset_pair) {
			return Err(Error::<T>::AmmPoolDoesNotExist);
		}

		Ok(T::AMM::get_share_token(asset_pair))
	}

	fn lock_lp_tokens(lp_token: AssetId, who: &T::AccountId, amount: Balance) -> Result<(), DispatchError> {
		let service_account_for_lp_shares = Self::account_id();

		T::MultiCurrency::transfer(lp_token, who, &service_account_for_lp_shares, amount)
	}

	fn unlock_lp_tokens(lp_token: AssetId, who: &T::AccountId, amount: Balance) -> Result<(), DispatchError> {
		let service_account_for_lp_shares = Self::account_id();

		T::MultiCurrency::transfer(lp_token, &service_account_for_lp_shares, who, amount)
	}

	fn get_asset_balance_in_amm_pool(asset: AssetId, amm_pool_id: T::AccountId) -> Result<Balance, DispatchError> {
		Ok(T::MultiCurrency::total_balance(asset, &amm_pool_id))
	}
}
