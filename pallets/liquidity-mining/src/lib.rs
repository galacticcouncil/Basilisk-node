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
//
// Abbr:
//  rpvs - reward per valued share
//  rpz - reward per share in global pool

// Notion spec naming map:
// * shares                 -> s
// * total_shares           -> S
// * valued_shares          -> s'
// * total_valued_shares    -> S'
// * stake_in_global_pool   -> z
// * total_shares_z         -> Z
// * multiplier             -> m

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
mod mock;
#[cfg(test)]
mod tests;

pub mod migration;
pub mod weights;

pub use pallet::*;

use frame_support::traits::tokens::nonfungibles::{Create, Inspect, Mutate};
use frame_support::{
	ensure,
	sp_runtime::traits::{BlockNumberProvider, Zero},
	transactional, PalletId,
};
use hydradx_traits::liquidity_mining::Mutate as LiquidityMiningMutate;
use warehouse_liquidity_mining::{GlobalFarmId, LoyaltyCurve};

use frame_support::{pallet_prelude::*, sp_runtime::traits::AccountIdConversion};
use frame_system::ensure_signed;
use hydradx_traits_amm::AMM;
use orml_traits::MultiCurrency;
use primitives::{asset::AssetPair, AssetId, Balance};
use scale_info::TypeInfo;
use sp_arithmetic::{FixedU128, Permill};
use sp_std::convert::{From, Into};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type PeriodOf<T> = <T as frame_system::Config>::BlockNumber;
type MultiCurrencyOf<T> = <T as pallet::Config>::MultiCurrency;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
	use warehouse_liquidity_mining::{FarmMultiplier, YieldFarmId};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			migration::init_nft_class::<T>()
		}

		fn integrity_test() {
			assert!(
				T::NftClassId::get() <= T::ReserveClassIdUpTo::get(),
				"`NftClassId` must be within the range of reserved NFT class IDs"
			);
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

		/// Minimum total rewards to distribute from global pool during liquidity mining.
		type MinTotalFarmRewards: Get<Balance>;

		/// Minimum number of periods to run liquidity mining program.
		type MinPlannedYieldingPeriods: Get<Self::BlockNumber>;

		/// The block number provider
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = Self::BlockNumber>;

		/// NFT class id for liq. mining deposit nfts. Has to be within the range of reserved NFT class IDs.
		type NftClassId: Get<primitives::ClassId>;

		/// Class IDs reserved for runtime up to the following constant
		#[pallet::constant]
		type ReserveClassIdUpTo: Get<primitives::ClassId>;

		/// Non fungible handling
		type NFTHandler: Mutate<Self::AccountId>
			+ Create<Self::AccountId>
			+ Inspect<Self::AccountId, ClassId = primitives::ClassId, InstanceId = primitives::InstanceId>;

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

		/// Weight information for extrinsic in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		/// Nft pallet didn't return an owner.
		CantFindDepositOwner,

		/// Nft pallet didn't return an owner.
		DepositMetadataNotFound,

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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New global farm was created.
		GlobalFarmCreated {
			id: GlobalFarmId,
			owner: AccountIdOf<T>,
			reward_currency: AssetId,
			yield_per_period: Permill,
			planned_yielding_periods: PeriodOf<T>,
			blocks_per_period: BlockNumberFor<T>,
			incentivized_asset: AssetId,
			max_reward_per_period: Balance,
		},

		/// New yield farm was added into the farm.
		YieldFarmCreated {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			multiplier: FarmMultiplier,
			nft_class: primitives::ClassId,
			asset_pair: AssetPair,
			loyalty_curve: Option<warehouse_liquidity_mining::LoyaltyCurve>,
		},

		/// Global farm was destroyed.
		GlobalFarmDestroyed { id: YieldFarmId, who: AccountIdOf<T> },

		/// New LP tokens was deposited.
		SharesDeposited {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: AccountIdOf<T>,
			amount: Balance,
			lp_token: AssetId,
			nft_class_id: primitives::ClassId,
		},

		/// LP token was redeposited for a new yield farm entry
		SharesRedeposited {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: AccountIdOf<T>,
			amount: Balance,
			lp_token: AssetId,
			nft_class_id: primitives::ClassId,
		},

		/// Rewards was claimed.
		RewardClaimed {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: AccountIdOf<T>,
			claimed: Balance,
			reward_currency: AssetId,
		},

		/// LP tokens was withdrawn.
		SharesWithdrawn {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: AccountIdOf<T>,
			lp_token: AssetId,
			amount: Balance,
		},

		/// Liquidity mining for asset pair was canceled.
		LiquidityMiningCanceled {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: AccountIdOf<T>,
			asset_pair: AssetPair,
		},

		/// Liquidity mining for asset pair was resumed.
		LiquidityMiningResumed {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: AccountIdOf<T>,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
		},

		/// Yield farm was removed from farm.
		YieldFarmRemoved {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: AccountIdOf<T>,
			asset_pair: AssetPair,
		},

		/// Yield farm multiplier was updated.
		YieldFarmUpdated {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			who: AccountIdOf<T>,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
		},

		/// Farm's(`GlobalFarm`) accumulated reward per share was updated.
		FarmAccRPZUpdated {
			farm_id: GlobalFarmId,
			accumulated_rpz: Balance,
			total_shares_z: Balance,
		},

		/// Yield farm's `accumulated_rpvs` was updated.
		YieldFarmAccRPVSUpdated {
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			accumulated_rpvs: Balance,
			total_valued_shares: Balance,
		},

		/// NFT representing deposit has been destroyed
		DepositDestroyed {
			who: AccountIdOf<T>,
			nft_instance_id: primitives::InstanceId,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create new liquidity mining program with proved parameters.
		///
		/// `owner` account have to have at least `total_rewards` balance. This fund will be
		/// transferred from `owner` to farm account.
		///
		/// The dispatch origin for this call must be `T::CreateOrigin`.
		///
		/// Parameters:
		/// - `origin`: account allowed to create new liq. mining program.
		/// - `total_rewards`: total rewards planned to distribute. This rewards will be
		/// distributed between all liq. pools in liq. mining program.
		/// - `planned_yielding_periods`: planned number of periods to distribute `total_rewards`.
		/// WARN: THIS IS NOT HARD DEADLINE. Not all rewards have to be distributed in
		/// `planned_yielding_periods`. Rewards are distributed based on the situation in the liq.
		/// pools and can be distributed in a longer time frame but never in the shorter time frame.
		/// - `blocks_per_period`:  number of blocks in a single period. Min. number of blocks per
		/// period is 1.
		/// - `incentivized_asset`: asset to be incentivized in AMM pools. All liq. pools added into
		/// liq. mining program have to have `incentivized_asset` in their pair.
		/// - `reward_currency`: payoff currency of rewards.
		/// - `owner`: liq. mining farm owner.
		/// - `yield_per_period`: percentage return on `reward_currency` of all pools p.a.
		/// - `min_deposit`: minimum amount which can be deposited to the farm
		/// - `price_adjustment`:
		/// Emits `FarmCreated` event when successful.
		//TODO: Ask Martin what price_adjustment is
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
			});

			Ok(())
		}

		/// Destroy existing liq. mining program.
		///
		/// Only farm owner can perform this action.
		///
		/// WARN: To successfully destroy a farm, farm have to be empty(all liq. pools have to be
		/// removed from the farm) and all undistributed rewards have to be withdrawn.
		///
		/// Parameters:
		/// - `origin`: account allowed to create new liq. mining program.
		/// - `farm_id`: id of global farm to be destroyed.
		///
		/// Emits `FarmDestroyed` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::destroy_farm())]
		#[transactional]
		pub fn destroy_global_farm(origin: OriginFor<T>, farm_id: GlobalFarmId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::LiquidityMiningHandler::destroy_global_farm(who.clone(), farm_id)?;

			Self::deposit_event(Event::GlobalFarmDestroyed { id: farm_id, who });
			Ok(())
		}

		/// Add yield farm for given `asset_pair` amm.
		///  
		/// Only farm owner can perform this action.
		///
		/// Only AMMs with `asset_pair` with `incentivized_asset` can be added into the farm. AMM
		/// for `asset_pair` has to exist to successfully add yield farm. Same AMM can
		/// in the same farm only once.
		///
		/// Parameters:
		/// - `origin`: account allowed to create new liq. mining program.
		/// - `farm_id`: farm id to which a liq. pool will be added.
		/// - `asset_pair`: asset pair identifying liq. pool. Liq. mining will be allowed for this
		/// `asset_pair` and one of the assets in the pair must be `incentivized_asset`.
		/// - `multiplier`: liq. pool multiplier in the farm.
		/// - `loyalty_curve`: curve to calculate loyalty multiplier to distribute rewards to users
		/// with time incentive. `None` means no loyalty multiplier.
		///
		/// Emits `LiquidityPoolAdded` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity_pool())]
		#[transactional]
		pub fn create_yield_farm(
			origin: OriginFor<T>,
			farm_id: GlobalFarmId,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
			loyalty_curve: Option<warehouse_liquidity_mining::LoyaltyCurve>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);
			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let yield_farm_id = T::LiquidityMiningHandler::create_yield_farm(
				who,
				farm_id,
				multiplier,
				loyalty_curve.clone(),
				amm_pool_id,
				asset_pair.asset_in,
				asset_pair.asset_out,
			)?;

			Self::deposit_event(Event::YieldFarmCreated {
				farm_id,
				yield_farm_id,
				nft_class: T::NftClassId::get(),
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
		/// - `origin`: account allowed to create new liq. mining program.
		/// - `farm_id`: farm id in which liq. pool will be updated.
		/// - `asset_pair`: asset pair identifying liq. pool in farm.
		/// - `multiplier`: new liq. pool multiplier in the farm.
		///
		/// Emits `LiquidityPoolUpdated` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::update_liquidity_pool())]
		#[transactional]
		pub fn update_yield_farm(
			origin: OriginFor<T>,
			farm_id: GlobalFarmId,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let yield_farm_id =
				T::LiquidityMiningHandler::update_yield_farm_multiplier(who.clone(), farm_id, amm_pool_id, multiplier)?;

			Self::deposit_event(Event::YieldFarmUpdated {
				farm_id,
				yield_farm_id,
				multiplier,
				who,
				asset_pair,
			});

			Ok(())
		}

		/// Stop liq. miming for specific yield farm.
		///
		/// This function claims rewards from `GlobalFarm` last time and stops liq. pool
		/// incentivization from a `GlobalFarm`. Users will be able to only withdraw
		/// shares(with claiming) after calling this function.
		/// `deposit_shares()` and `claim_rewards()` are not allowed on canceled liq. pool.
		///  
		/// Only farm owner can perform this action.
		///
		/// Parameters:
		/// - `origin`: account allowed to create new liq. mining program.
		/// - `farm_id`: farm id in which liq. pool will be canceled.
		/// - `asset_pair`: asset pair identifying liq. pool in the farm.
		///
		/// Emits `LiquidityMiningCanceled` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::cancel_liquidity_pool())]
		#[transactional]
		pub fn stop_yield_farm(origin: OriginFor<T>, farm_id: GlobalFarmId, asset_pair: AssetPair) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let yield_farm_id = T::LiquidityMiningHandler::stop_yield_farm(who.clone(), farm_id, amm_pool_id)?;

			Self::deposit_event(Event::LiquidityMiningCanceled {
				farm_id,
				yield_farm_id,
				who,
				asset_pair,
			});

			Ok(())
		}

		/// Resume liq. miming for sopped yield farm.
		///
		/// This function resume incentivization from `GlobalFarm` and restore full functionality
		/// for yield farm. Users will be able to deposit, claim and withdraw again.
		///
		/// WARN: Yield farm is NOT rewarded for time it was stopped.
		///
		/// Only farm owner can perform this action.
		///
		/// Parameters:
		/// - `origin`: account allowed to create new liq. mining program.
		/// - `farm_id`: farm id in which liq. pool will be resumed.
		/// - `yield_farm_id`: id of yield farm to be resumed.
		/// - `asset_pair`: asset pair identifying liq. pool in the farm.
		/// - `multiplier`: liq. pool multiplier in the farm.
		///
		/// Emits `LiquidityMiningResumed` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::resume_liquidity_pool())]
		#[transactional]
		pub fn resume_yield_farm(
			origin: OriginFor<T>,
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
			multiplier: FarmMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			T::LiquidityMiningHandler::resume_yield_farm(who.clone(), farm_id, yield_farm_id, amm_pool_id, multiplier)?;

			Self::deposit_event(Event::<T>::LiquidityMiningResumed {
				farm_id,
				yield_farm_id,
				who,
				asset_pair,
				multiplier,
			});

			Ok(())
		}

		/// Remove yield farm
		///
		/// This function removes yield farm and also from storage. Users will be able to
		/// only withdraw shares(without claiming rewards from liq. mining). Unpaid rewards will be
		/// transferred back to farm(`GlobalFarm`) account and will be used to distribute to other
		/// yield farms in the farm.
		///
		/// Yield farm must be canceled before calling this function.
		///
		/// Only farm owner can perform this action.
		///
		/// Parameters:
		/// - `origin`: account allowed to create new liq. mining program.
		/// - `farm_id`: farm id from which liq. pool should be removed.
		/// - `yield_farm_id`: id of yield farm to be removed.
		/// - `asset_pair`: asset pair identifying liq. pool in the farm.
		///
		/// Emits `YieldFarmRemoved` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::remove_liquidity_pool())]
		#[transactional]
		pub fn destroy_yield_farm(
			origin: OriginFor<T>,
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			T::LiquidityMiningHandler::destroy_yield_farm(who.clone(), farm_id, yield_farm_id, amm_pool_id)?;

			Self::deposit_event(Event::YieldFarmRemoved {
				farm_id,
				yield_farm_id,
				who,
				asset_pair,
			});

			Ok(().into())
		}

		/// Deposit LP shares to a liq. mining.
		///
		/// This function transfer LP shares from `origin` to pallet's account and mint nft for
		/// `origin` account. Minted nft represent deposit in the liq. mining.
		///
		/// Parameters:
		/// - `origin`: account depositing LP shares. This account have to have at least
		/// `shares_amount` of LP shares.
		/// - `farm_id`: id of farm to which user want to deposit LP shares.
		/// - `yield_farm_id`: id of yield farm to deposit to.
		/// - `asset_pair`: asset pair identifying LP shares user want to deposit.
		/// - `shares_amount`: amount of LP shares user want to deposit.
		///
		/// Emits `SharesDeposited` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::deposit_shares())]
		#[transactional]
		pub fn deposit_shares(
			origin: OriginFor<T>,
			farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
			shares_amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let amm_share_token = T::AMM::get_share_token(asset_pair);

			ensure!(
				MultiCurrencyOf::<T>::free_balance(amm_share_token, &who) >= shares_amount,
				Error::<T>::InsufficientAmmSharesBalance
			);

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let deposit_id = T::LiquidityMiningHandler::deposit_lp_shares(
				farm_id,
				yield_farm_id,
				amm_pool_id.clone(),
				shares_amount,
				Self::get_asset_balance_in_liquidity_mining_pool,
			)?;

			Self::lock_lp_tokens(amm_pool_id, who.clone(), shares_amount)?;
			Self::mint_nft_representing_deposit(who.clone(), &deposit_id)?;

			Self::deposit_event(Event::SharesDeposited {
				farm_id,
				yield_farm_id,
				who,
				amount: shares_amount,
				lp_token: amm_share_token,
				nft_class_id: T::NftClassId::get(),
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
		#[pallet::weight(<T as Config>::WeightInfo::deposit_shares())]
		pub fn redeposit_lp_shares(
			origin: OriginFor<T>,
			global_farm_id: GlobalFarmId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
			deposit_id: primitives::InstanceId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nft_class_id = T::NftClassId::get();
			let nft_owner = T::NFTHandler::owner(&nft_class_id, &deposit_id).ok_or(Error::<T>::CantFindDepositOwner)?;

			ensure!(nft_owner == who, Error::<T>::NotDepositOwner);

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);
			let amm_share_token = T::AMM::get_share_token(asset_pair);

			let shares_amount = T::LiquidityMiningHandler::redeposit_lp_shares(
				global_farm_id,
				yield_farm_id,
				deposit_id,
				Self::get_asset_balance_in_liquidity_mining_pool,
			)?;

			Self::deposit_event(Event::SharesRedeposited {
				farm_id: global_farm_id,
				yield_farm_id,
				who,
				amount: shares_amount,
				lp_token: amm_share_token,
				nft_class_id,
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
		/// - `deposit_id`: nft id representing deposit in the liq. pool.
		/// - `yield_farm_id`: yield farm identifier to claim rewards from.
		///
		/// Emits `RewardClaimed` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::claim_rewards())]
		#[transactional]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			deposit_id: primitives::InstanceId,
			yield_farm_id: YieldFarmId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let deposit_owner =
				T::NFTHandler::owner(&T::NftClassId::get(), &deposit_id).ok_or(Error::<T>::CantFindDepositOwner)?;

			ensure!(deposit_owner == who, Error::<T>::NotDepositOwner);

			let fail_on_double_claim = true;
			let (farm_id, reward_currency, claimed, _) =
				T::LiquidityMiningHandler::claim_rewards(who.clone(), deposit_id, yield_farm_id, fail_on_double_claim)?;

			if !claimed.is_zero() {
				Self::deposit_event(Event::RewardClaimed {
					farm_id,
					yield_farm_id,
					who,
					claimed,
					reward_currency,
				});
			}

			Ok(())
		}

		/// Withdraw LP shares from liq. mining. with reward claiming if possible.
		///
		/// Cases for transfer LP shares and claimed rewards:
		///
		/// * liq. mining is active(liq. pool is not canceled) - claim and transfer rewards(if it
		/// wasn't claimed in this period) and transfer LP shares.
		/// * liq. mining is stopped - claim and transfer rewards(if it
		/// wasn't claimed in this period) and transfer LP shares.
		/// * liq. pool was destroyed - only LP shares will be transferred.
		/// * farm was destroyed - only LP shares will be transferred.
		/// * SPECIAL CASE: AMM pool does not exist - claim may happen if liq. pool is still active, LP
		/// shares will not be transferred.
		///
		/// User's unclaimable rewards will be transferred back to global pool's account.
		///
		/// Parameters:
		/// - `origin`: account owner of deposit(nft).
		/// - `deposit_id`: nft id representing deposit in the liq. pool.
		/// - `yield_farm_id`: yield farm identifier to dithdraw shares from.
		/// - `asset_pair`: asset pair identifying liq. pool in farm.
		///
		/// Emits:
		/// * `RewardClaimed` if claim happen
		/// * `SharesWithdrawn` event when successful
		#[pallet::weight(<T as Config>::WeightInfo::withdraw_shares())]
		#[transactional]
		pub fn withdraw_shares(
			origin: OriginFor<T>,
			deposit_id: primitives::InstanceId,
			yield_farm_id: YieldFarmId,
			asset_pair: AssetPair,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let nft_owner =
				T::NFTHandler::owner(&T::NftClassId::get(), &deposit_id).ok_or(Error::<T>::CantFindDepositOwner)?;

			ensure!(nft_owner == who, Error::<T>::NotDepositOwner);

			let global_farm_id = T::LiquidityMiningHandler::get_global_farm_id(deposit_id, yield_farm_id)
				.ok_or(Error::<T>::DepositDataNotFound)?;

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);

			let mut unclaimable_rewards: Balance = 0;
			if T::LiquidityMiningHandler::is_yield_farm_claimable(global_farm_id, yield_farm_id, amm_pool_id.clone()) {
				//This should not fail on double claim, we need unclaimable_rewards
				let fail_on_double_claim = false;
				let (farm_id, reward_currency, claimed, unclaimable) = T::LiquidityMiningHandler::claim_rewards(
					who.clone(),
					deposit_id,
					yield_farm_id,
					fail_on_double_claim,
				)?;

				if !claimed.is_zero() {
					Self::deposit_event(Event::RewardClaimed {
						farm_id,
						yield_farm_id,
						who: who.clone(),
						claimed,
						reward_currency,
					});
				}

				unclaimable_rewards = unclaimable;
			}

			let (farm_id, withdrawn_amount, is_destroyed) =
				T::LiquidityMiningHandler::withdraw_lp_shares(deposit_id, yield_farm_id, unclaimable_rewards)?;

			let lp_token = Self::get_lp_token(&amm_pool_id)?;
			if !withdrawn_amount.is_zero() {
				Self::deposit_event(Event::SharesWithdrawn {
					farm_id,
					yield_farm_id,
					who: who.clone(),
					lp_token,
					amount: withdrawn_amount,
				});
			}

			if is_destroyed {
				Self::unlock_lp_tokens(amm_pool_id, who.clone(), withdrawn_amount)?;
				T::NFTHandler::burn_from(&T::NftClassId::get(), &deposit_id)?;

				Self::deposit_event(Event::DepositDestroyed {
					who,
					nft_instance_id: deposit_id,
				});
			}

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn account_id_for_all_lp_shares() -> AccountIdOf<T> {
		<T as pallet::Config>::PalletId::get().into_account()
	}

	fn get_lp_token(amm_pool_id: &AccountIdOf<T>) -> Result<AssetId, Error<T>> {
		let assets = T::AMM::get_pool_assets(amm_pool_id).ok_or(Error::<T>::CantGetAmmAssets)?;
		let asset_pair = AssetPair::new(assets[0], assets[1]);

		//NOTE: this check is important AMM:get_share_token() return `0` if amm doesn't exist
		if !T::AMM::exists(asset_pair) {
			return Err(Error::<T>::AmmPoolDoesNotExist);
		}

		Ok(T::AMM::get_share_token(asset_pair))
	}

	fn lock_lp_tokens(amm_pool_id: T::AccountId, who: T::AccountId, amount: Balance) -> Result<(), DispatchError> {
		let lp_token = Self::get_lp_token(&amm_pool_id)?;

		let service_account_for_lp_shares = Self::account_id_for_all_lp_shares();
		MultiCurrencyOf::<T>::transfer(lp_token, &who, &service_account_for_lp_shares, amount)?;

		Ok(())
	}

	fn unlock_lp_tokens(amm_pool_id: T::AccountId, who: T::AccountId, amount: Balance) -> Result<(), DispatchError> {
		let lp_token = Self::get_lp_token(&amm_pool_id)?;

		let service_account_for_lp_shares = Self::account_id_for_all_lp_shares();
		MultiCurrencyOf::<T>::transfer(lp_token, &service_account_for_lp_shares, &who, amount)?;

		Ok(())
	}

	fn mint_nft_representing_deposit(who: T::AccountId, deposit_id: &u128) -> Result<(), DispatchError> {
		T::NFTHandler::mint_into(&T::NftClassId::get(), deposit_id, &who)?;

		Ok(())
	}

	fn get_asset_balance_in_liquidity_mining_pool(
		asset: AssetId,
		amm_pool_id: AccountIdOf<T>,
	) -> Result<Balance, DispatchError> {
		Ok(T::MultiCurrency::total_balance(asset, &amm_pool_id))
	}
}
