// This file is part of HydraDX

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

//for audit:
// * check if nft is manipulable only by this pallet

// * make sure nft minted event happen
// * make sure in the docs: 1-th reward for next whole period

// add test for 0 multiplier (liq. pool)
// add to docs blocks are related to relay chain blocks

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod migration;
pub mod weights;

pub use pallet::*;

type PoolId = u32;
type GlobalPoolId = PoolId;
type PoolMultiplier = FixedU128;

//This value is result of: u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0])
//This is necessary because first 4 bytes of NftInstanceIdOf(u128) is reserved to encode liq_pool_id(u32) into NftInstanceIdOf.
const MAX_NFT_INSTANCE_SEQUENCER: u128 = 79_228_162_514_264_337_593_543_950_335;

use frame_support::{
	ensure,
	sp_runtime::traits::{BlockNumberProvider, One, Zero},
	transactional, PalletId,
};
use frame_system::ensure_signed;

use sp_arithmetic::{
	traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
	FixedU128, Permill,
};

use orml_traits::MultiCurrency;

use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::*,
	sp_runtime::{traits::AccountIdConversion, FixedPointNumber, RuntimeDebug},
};
use hydradx_traits::AMM;
use scale_info::TypeInfo;
use sp_std::{
	convert::{From, Into, TryInto},
	vec,
};

use primitives::nft::ClassType;
use primitives::{asset::AssetPair, Balance};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AssetIdOf<T> = <T as pallet::Config>::CurrencyId;
type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;
type PeriodOf<T> = <T as frame_system::Config>::BlockNumber;
pub type NftClassIdOf<T> = <T as pallet_nft::Config>::NftClassId;
pub type NftInstanceIdOf<T> = <T as pallet_nft::Config>::NftInstanceId;

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebugNoBound, TypeInfo)]
pub struct GlobalPool<T: Config> {
	id: GlobalPoolId,
	owner: AccountIdOf<T>,
	pub updated_at: PeriodOf<T>,
	total_shares_z: Balance,
	accumulated_rpz: Balance,
	reward_currency: AssetIdOf<T>,
	accumulated_rewards: Balance,
	paid_accumulated_rewards: Balance,
	yield_per_period: Permill,
	planned_yielding_periods: PeriodOf<T>,
	blocks_per_period: BlockNumberFor<T>,
	incentivized_asset: AssetIdOf<T>,
	max_reward_per_period: Balance,
	pub liq_pools_count: u32,
}

impl<T: Config> GlobalPool<T> {
	fn new(
		id: GlobalPoolId,
		updated_at: PeriodOf<T>,
		reward_currency: T::CurrencyId,
		yield_per_period: Permill,
		planned_yielding_periods: PeriodOf<T>,
		blocks_per_period: T::BlockNumber,
		owner: AccountIdOf<T>,
		incentivized_asset: T::CurrencyId,
		max_reward_per_period: Balance,
	) -> Self {
		Self {
			accumulated_rewards: Default::default(),
			accumulated_rpz: Default::default(),
			paid_accumulated_rewards: Default::default(),
			total_shares_z: Default::default(),
			liq_pools_count: Default::default(),
			id,
			updated_at,
			reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_asset,
			max_reward_per_period,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebugNoBound, TypeInfo)]
pub struct LiquidityPoolYieldFarm<T: Config> {
	id: PoolId,
	pub updated_at: PeriodOf<T>,
	total_shares: Balance,
	total_valued_shares: Balance,
	accumulated_rpvs: Balance,
	accumulated_rpz: Balance,
	loyalty_curve: Option<LoyaltyCurve>,
	stake_in_global_pool: Balance, //NOTE: may be replaced with: total_valued_shares * multiplier
	pub multiplier: PoolMultiplier,
	pub canceled: bool,
}

impl<T: Config> LiquidityPoolYieldFarm<T> {
	fn new(
		id: PoolId,
		updated_at: PeriodOf<T>,
		loyalty_curve: Option<LoyaltyCurve>,
		multiplier: PoolMultiplier,
	) -> Self {
		Self {
			accumulated_rpvs: Default::default(),
			accumulated_rpz: Default::default(),
			stake_in_global_pool: Default::default(),
			total_shares: Default::default(),
			total_valued_shares: Default::default(),
			canceled: false,
			id,
			updated_at,
			loyalty_curve,
			multiplier,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct LoyaltyCurve {
	initial_reward_percentage: FixedU128,
	scale_coef: u32,
}

impl Default for LoyaltyCurve {
	fn default() -> Self {
		Self {
			initial_reward_percentage: FixedU128::from_inner(500_000_000_000_000_000), // 0.5
			scale_coef: 100,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebugNoBound, TypeInfo)]
pub struct Deposit<T: Config> {
	shares: Balance,
	valued_shares: Balance,
	accumulated_rpvs: Balance,
	accumulated_claimed_rewards: Balance,
	entered_at: PeriodOf<T>,
	updated_at: PeriodOf<T>,
}

impl<T: Config> Deposit<T> {
	fn new(shares: Balance, valued_shares: Balance, accumulated_rpvs: Balance, entered_at: PeriodOf<T>) -> Self {
		Self {
			updated_at: entered_at,
			entered_at,
			shares,
			valued_shares,
			accumulated_rpvs,
			accumulated_claimed_rewards: Default::default(),
		}
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			migration::init_nft_class::<T>()
		}
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ TypeInfo
		+ pallet_nft::Config<
			ClassType = ClassType,
			NftClassId = primitives::ClassId,
			NftInstanceId = primitives::InstanceId,
		>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Asset type
		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + From<u32>;

		/// Currency for transfers
		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = Self::CurrencyId, Balance = Balance>;

		/// AMM helper functions
		type AMM: AMM<Self::AccountId, Self::CurrencyId, AssetPair, Balance>;

		/// The origin account that cat create new liquidity mining program
		type CreateOrigin: EnsureOrigin<Self::Origin>;

		/// Pallet id
		type PalletId: Get<PalletId>;

		/// Minimum amount of total rewards to create new farm
		type MinTotalFarmRewards: Get<Balance>;

		/// Minimal number of periods to distribute farm rewards
		type MinPlannedYieldingPeriods: Get<Self::BlockNumber>;

		/// The block number provider
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = Self::BlockNumber>;

		type NftClass: Get<primitives::ClassId>;

		/// Weight information for extrinsic in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		/// Math computation overflow
		Overflow,

		/// Insufficient balance in global pool to transfer rewards to pool
		InsufficientBalanceInGlobalPool,

		/// Provide id is not valid. Valid range is [1, u32::MAX)
		InvalidPoolId,

		/// Provided `planed_yielding_periods` is below min. limit
		InvalidPlannedYieldingPeriods,

		/// Blocks per period can't be 0
		InvalidBlocksPerPeriod,

		/// Yield per period can't be 0
		InvalidYieldPerPeriod,

		/// Provided `total_rewards` for farming is bellow min limit
		InvalidTotalRewards,

		/// Reward currency balance is too low
		InsufficientRewardCurrencyBalance,

		/// Account is not allowed to perform action
		Forbidden,

		/// Farm does not exist
		FarmNotFound,

		/// Liquidity pool already exist in farm
		LiquidityPoolAlreadyExists,

		/// Pool multiplier can't be 0
		InvalidMultiplier,

		/// Initial reward percentage of Loyalty curve should be from interval: [0, 1)
		InvalidInitialRewardPercentage,

		/// Account balance of amm pool shares is not sufficient
		InsufficientAmmSharesBalance,

		/// AMM pool does not exist
		AmmPoolDoesNotExist,

		/// Liq. pool for provided assets was not found in farm
		LiquidityPoolNotFound,

		/// One or more liq. pools exist in farm. Only farm without liq. pools can be destroyed
		FarmIsNotEmpty,

		/// Balance on rewards account is not 0. Only farm with 0 reward balance can be destroyed
		RewardBalanceIsNotZero,

		/// Metadata for liquidity pool was not found.
		LiquidityPoolMetadataNotFound,

		/// NFT does not exist.
		NftDoesNotExist,

		/// Max number of ntf id was reached.
		NftIdOwerflow,

		/// Nft id is not valid.
		InvalidNftId,

		/// Liquidity mining for provided pool is canceled.
		LiquidityMiningCanceled,

		/// Liquidity mining is not canceled yet for provided pool.
		LiquidityMiningIsNotCanceled,

		/// Deposit amount is out of valid range
		InvalidDepositAmount,

		/// Account is not owner of the deposit
		NotDepositOwner,

		/// Nft pallet doesn't returned owner
		CantFindDepositOwner,

		/// Account can't claim multiple times in the same period
		DoubleClaimInThePeriod,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New farm created.
		FarmCreated {
			farm_id: GlobalPoolId,
			owner: AccountIdOf<T>,
			reward_currency: AssetIdOf<T>,
			yield_per_period: Permill,
			planned_yielding_periods: PeriodOf<T>,
			blocks_per_period: BlockNumberFor<T>,
			incentivized_asset: AssetIdOf<T>,
			max_reward_per_period: Balance,
		},

		/// New liquidity mining farm for AMM created.
		LiquidityPoolAdded {
			farm_id: GlobalPoolId,
			liq_pool_farm_id: PoolId,
			multiplier: PoolMultiplier,
			nft_class: NftClassIdOf<T>,
			asset_pair: AssetPair,
			loyalty_curve: Option<LoyaltyCurve>,
		},

		/// Destroy farm.
		FarmDestroyed { id: PoolId, who: AccountIdOf<T> },

		/// Deposit lp tokens.
		SharesDeposited {
			farm_id: GlobalPoolId,
			liq_pool_farm_id: PoolId,
			who: AccountIdOf<T>,
			amount: Balance,
			lp_token: T::CurrencyId,
			nft_class: NftClassIdOf<T>,
			nft_instance_id: NftInstanceIdOf<T>,
		},

		/// Claim rewards.
		RewardClaimed {
			farm_id: GlobalPoolId,
			liq_pool_farm_id: PoolId,
			who: AccountIdOf<T>,
			claimed: Balance,
			reward_currency: T::CurrencyId,
		},

		//TODO: add farm, pool_id, asset_pair
		/// Withdraw shares.
		SharesWithdrawn {
			who: AccountIdOf<T>,
			lp_token: T::CurrencyId,
			amount: Balance,
		},

		/// Cancel liquidity mining for AMM.
		LiquidityMiningCanceled {
			farm_id: GlobalPoolId,
			liq_pool_farm_id: PoolId,
			who: AccountIdOf<T>,
			asset_pair: AssetPair,
		},

		/// Resume liquidity mining for AMM.
		LiquidityMiningResumed {
			farm_id: GlobalPoolId,
			liq_pool_farm_id: PoolId,
			who: AccountIdOf<T>,
			asset_pair: AssetPair,
			multiplier: PoolMultiplier,
		},

		/// Remove liquidity mining pool farm.
		LiquidityPoolRemoved {
			farm_id: GlobalPoolId,
			liq_pool_farm_id: PoolId,
			who: AccountIdOf<T>,
			asset_pair: AssetPair,
		},

		/// Withdraw undistributed rewards from farm.
		UndistributedRewardsWithdrawn {
			farm_id: GlobalPoolId,
			who: AccountIdOf<T>,
			reward_currency: T::CurrencyId,
			amount: Balance,
		},

		/// Update liquidity pool farm multiplier.
		LiquidityPoolUpdated {
			farm_id: GlobalPoolId,
			liq_pool_farm_id: PoolId,
			who: AccountIdOf<T>,
			asset_pair: AssetPair,
			multiplier: PoolMultiplier,
		},

		/// Update of `accumulated_rpz` of farm.
		FarmAccRPZUpdated {
			farm_id: GlobalPoolId,
			accumulated_rpz: Balance,
			total_shares_z: Balance,
		},

		/// Update of `accumulated_rpvs` of liq. pool farm.
		LiquidityPoolAccRPVSUpdated {
			farm_id: GlobalPoolId,
			liq_pool_farm_id: PoolId,
			accumulated_rpvs: Balance,
			total_valued_shares: Balance,
		},
	}

	#[pallet::storage]
	#[pallet::getter(fn pool_id)]
	pub type PoolIdSequencer<T: Config> = StorageValue<_, PoolId, ValueQuery>;

	#[pallet::storage]
	pub type NftInstanceSequencer<T: Config> = StorageValue<_, primitives::InstanceId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn global_pool)]
	type GlobalPoolData<T: Config> = StorageMap<_, Twox64Concat, GlobalPoolId, GlobalPool<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	type LiquidityPoolData<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		GlobalPoolId,
		Twox64Concat,
		AccountIdOf<T>, //amm_pool_id
		LiquidityPoolYieldFarm<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn deposit)]
	type DepositData<T: Config> = StorageMap<_, Twox64Concat, NftInstanceIdOf<T>, Deposit<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liq_pool_meta)] //(asset_pair, amount of existing nfts, globalPoolId)
	type LiquidityPoolMetadata<T: Config> =
		StorageMap<_, Twox64Concat, PoolId, (AssetPair, u64, GlobalPoolId), OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create new liquidity mining program
		///
		/// Parameters:
		/// - `origin`:
		/// - `total_rewards`:
		/// - `planned_yielding_periods`: planned number of periods to distribute rewards. WARN: this is not
		/// how long will farming run.  Owner can destroy farm sooner or liq. mining can run longer
		/// if all the rewards will not distributed.
		/// - `blocks_per_period`:
		/// - `incentivized_asset`
		/// - `reward_currency`
		/// - `admin_account`
		/// - `yield_per_period`
		#[pallet::weight(<T as Config>::WeightInfo::create_farm())]
		#[transactional]
		pub fn create_farm(
			origin: OriginFor<T>,
			total_rewards: Balance,
			planned_yielding_periods: PeriodOf<T>,
			blocks_per_period: BlockNumberFor<T>,
			incentivized_asset: AssetIdOf<T>,
			reward_currency: AssetIdOf<T>,
			owner: AccountIdOf<T>,
			yield_per_period: Permill,
		) -> DispatchResult {
			T::CreateOrigin::ensure_origin(origin)?;

			Self::validate_create_farm_data(
				total_rewards,
				planned_yielding_periods,
				blocks_per_period,
				yield_per_period,
			)?;

			ensure!(
				T::MultiCurrency::free_balance(reward_currency, &owner) >= total_rewards,
				Error::<T>::InsufficientRewardCurrencyBalance
			);

			let planned_periods =
				TryInto::<u128>::try_into(planned_yielding_periods).map_err(|_e| Error::<T>::Overflow)?;
			let max_reward_per_period = total_rewards.checked_div(planned_periods).ok_or(Error::<T>::Overflow)?;
			let now_period = Self::get_now_period(blocks_per_period)?;
			let pool_id = Self::get_next_pool_id()?;

			let global_pool = GlobalPool::new(
				pool_id,
				now_period,
				reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_asset,
				max_reward_per_period,
			);

			<GlobalPoolData<T>>::insert(&global_pool.id, &global_pool);

			let global_pool_account = Self::pool_account_id(global_pool.id)?;
			T::MultiCurrency::transfer(reward_currency, &global_pool.owner, &global_pool_account, total_rewards)?;

			Self::deposit_event(Event::FarmCreated {
				farm_id: global_pool.id,
				owner: global_pool.owner,
				reward_currency: global_pool.reward_currency,
				yield_per_period: global_pool.yield_per_period,
				planned_yielding_periods: global_pool.planned_yielding_periods,
				blocks_per_period: global_pool.blocks_per_period,
				incentivized_asset: global_pool.incentivized_asset,
				max_reward_per_period: global_pool.max_reward_per_period,
			});

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::destroy_farm())]
		#[transactional]
		pub fn destroy_farm(origin: OriginFor<T>, farm_id: GlobalPoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<GlobalPoolData<T>>::try_mutate_exists(farm_id, |maybe_global_pool| -> DispatchResult {
				let global_pool = maybe_global_pool.as_ref().ok_or(Error::<T>::FarmNotFound)?;

				ensure!(who == global_pool.owner, Error::<T>::Forbidden);

				ensure!(global_pool.liq_pools_count.is_zero(), Error::<T>::FarmIsNotEmpty);

				let global_pool_account = Self::pool_account_id(global_pool.id)?;
				ensure!(
					T::MultiCurrency::free_balance(global_pool.reward_currency, &global_pool_account).is_zero(),
					Error::<T>::RewardBalanceIsNotZero
				);

				*maybe_global_pool = None;

				Self::deposit_event(Event::FarmDestroyed { id: farm_id, who });
				Ok(())
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::withdraw_undistributed_rewards())]
		#[transactional]
		pub fn withdraw_undistributed_rewards(origin: OriginFor<T>, farm_id: GlobalPoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let global_pool = Self::global_pool(farm_id).ok_or(Error::<T>::FarmNotFound)?;

			ensure!(global_pool.owner == who, Error::<T>::Forbidden);

			ensure!(global_pool.liq_pools_count.is_zero(), Error::<T>::FarmIsNotEmpty);

			let global_pool_account = Self::pool_account_id(global_pool.id)?;

			let undistributed_reward =
				T::MultiCurrency::total_balance(global_pool.reward_currency, &global_pool_account);

			T::MultiCurrency::transfer(
				global_pool.reward_currency,
				&global_pool_account,
				&who,
				undistributed_reward,
			)?;

			Self::deposit_event(Event::UndistributedRewardsWithdrawn {
				farm_id: global_pool.id,
				who,
				reward_currency: global_pool.reward_currency,
				amount: undistributed_reward,
			});

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity_pool())]
		#[transactional]
		pub fn add_liquidity_pool(
			origin: OriginFor<T>,
			farm_id: GlobalPoolId,
			asset_pair: AssetPair,
			multiplier: PoolMultiplier,
			loyalty_curve: Option<LoyaltyCurve>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!multiplier.is_zero(), Error::<T>::InvalidMultiplier);

			if let Some(ref curve) = loyalty_curve {
				ensure!(
					curve.initial_reward_percentage.lt(&FixedU128::one()),
					Error::<T>::InvalidInitialRewardPercentage
				);
			}

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);

			<GlobalPoolData<T>>::try_mutate(farm_id, |maybe_pool| -> DispatchResult {
				let global_pool = maybe_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

				ensure!(who == global_pool.owner, Error::<T>::Forbidden);

				let amm_pool_id = T::AMM::get_pair_id(asset_pair);
				ensure!(
					!<LiquidityPoolData<T>>::contains_key(farm_id, &amm_pool_id),
					Error::<T>::LiquidityPoolAlreadyExists
				);

				// update accRPZ
				let now_period = Self::get_now_period(global_pool.blocks_per_period)?;
				if !global_pool.total_shares_z.is_zero() && global_pool.updated_at != now_period {
					let reward_per_period = Self::get_global_pool_reward_per_period(
						global_pool.yield_per_period.into(),
						global_pool.total_shares_z,
						global_pool.max_reward_per_period,
					)?;
					Self::update_global_pool(global_pool, now_period, reward_per_period)?;
				}

				let liq_pool_id = Self::get_next_pool_id()?;
				<LiquidityPoolMetadata<T>>::insert(liq_pool_id, (asset_pair, 0, global_pool.id));

				let pool = LiquidityPoolYieldFarm::new(liq_pool_id, now_period, loyalty_curve.clone(), multiplier);

				<LiquidityPoolData<T>>::insert(global_pool.id, &amm_pool_id, &pool);
				global_pool.liq_pools_count = global_pool.liq_pools_count.checked_add(1).ok_or(Error::<T>::Overflow)?;

				Self::deposit_event(Event::LiquidityPoolAdded {
					farm_id: global_pool.id,
					liq_pool_farm_id: pool.id,
					nft_class: T::NftClass::get(),
					multiplier,
					loyalty_curve,
					asset_pair,
				});

				Ok(())
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_liquidity_pool())]
		#[transactional]
		pub fn update_liquidity_pool(
			origin: OriginFor<T>,
			farm_id: GlobalPoolId,
			asset_pair: AssetPair,
			multiplier: PoolMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!multiplier.is_zero(), Error::<T>::InvalidMultiplier);

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);
			<LiquidityPoolData<T>>::try_mutate(farm_id, &amm_pool_id, |liq_pool| {
				let liq_pool = liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

				ensure!(!liq_pool.canceled, Error::<T>::LiquidityMiningCanceled);

				<GlobalPoolData<T>>::try_mutate(farm_id, |maybe_global_pool| {
					let global_pool = maybe_global_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

					ensure!(who == global_pool.owner, Error::<T>::Forbidden);

					//TODO: maybe replace this wich maybe_update_pools() - check spec
					let now_period = Self::get_now_period(global_pool.blocks_per_period)?;
					if !global_pool.total_shares_z.is_zero() {
						let reward_per_period = Self::get_global_pool_reward_per_period(
							global_pool.yield_per_period.into(),
							global_pool.total_shares_z,
							global_pool.max_reward_per_period,
						)?;
						Self::update_global_pool(global_pool, now_period, reward_per_period)?;
					}

					if !liq_pool.total_shares.is_zero() {
						let pool_reward =
							Self::claim_from_global_pool(global_pool, liq_pool, liq_pool.stake_in_global_pool)?;
						Self::update_liq_pool(
							liq_pool,
							pool_reward,
							now_period,
							global_pool.id,
							global_pool.reward_currency,
						)?;
					}

					let new_stake_in_global_pool =
						Self::get_global_pool_shares(liq_pool.total_valued_shares, multiplier)?;

					global_pool.total_shares_z = global_pool
						.total_shares_z
						.checked_sub(liq_pool.stake_in_global_pool)
						.ok_or(Error::<T>::Overflow)?
						.checked_add(new_stake_in_global_pool)
						.ok_or(Error::<T>::Overflow)?;

					liq_pool.stake_in_global_pool = new_stake_in_global_pool;
					liq_pool.multiplier = multiplier;

					Self::deposit_event(Event::LiquidityPoolUpdated {
						farm_id: global_pool.id,
						liq_pool_farm_id: liq_pool.id,
						multiplier: liq_pool.multiplier,
						who,
						asset_pair,
					});

					Ok(())
				})
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::cancel_liquidity_pool())]
		#[transactional]
		pub fn cancel_liquidity_pool(
			origin: OriginFor<T>,
			farm_id: GlobalPoolId,
			asset_pair: AssetPair,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let amm_account = T::AMM::get_pair_id(asset_pair);

			<LiquidityPoolData<T>>::try_mutate(farm_id, amm_account, |maybe_liq_pool| {
				let liq_pool = maybe_liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

				ensure!(!liq_pool.canceled, Error::<T>::LiquidityMiningCanceled);

				<GlobalPoolData<T>>::try_mutate(farm_id, |maybe_global_pool| {
					let global_pool = maybe_global_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

					ensure!(global_pool.owner == who, Error::<T>::Forbidden);

					let now_period = Self::get_now_period(global_pool.blocks_per_period)?;
					Self::maybe_update_pools(global_pool, liq_pool, now_period)?;

					global_pool.total_shares_z = global_pool
						.total_shares_z
						.checked_sub(liq_pool.stake_in_global_pool)
						.ok_or(Error::<T>::Overflow)?;

					liq_pool.canceled = true;
					liq_pool.stake_in_global_pool = 0;
					liq_pool.multiplier = 0.into();

					Self::deposit_event(Event::LiquidityMiningCanceled {
						farm_id,
						liq_pool_farm_id: liq_pool.id,
						who,
						asset_pair,
					});

					Ok(())
				})
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::resume_liquidity_pool())]
		#[transactional]
		pub fn resume_liquidity_pool(
			origin: OriginFor<T>,
			farm_id: GlobalPoolId,
			asset_pair: AssetPair,
			multiplier: PoolMultiplier,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!multiplier.is_zero(), Error::<T>::InvalidMultiplier);

			let amm_account = T::AMM::get_pair_id(asset_pair);

			<LiquidityPoolData<T>>::try_mutate(farm_id, amm_account, |maybe_liq_pool| {
				let liq_pool = maybe_liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

				ensure!(liq_pool.canceled, Error::<T>::LiquidityMiningIsNotCanceled);

				<GlobalPoolData<T>>::try_mutate(farm_id, |maybe_global_pool| {
					// this should never happen, liq. pool can't exist without global_pool
					let global_pool = maybe_global_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

					ensure!(global_pool.owner == who, Error::<T>::Forbidden);

					let now_period = Self::get_now_period(global_pool.blocks_per_period)?;
					if !global_pool.total_shares_z.is_zero() && global_pool.updated_at != now_period {

						let reward_per_period = Self::get_global_pool_reward_per_period(
							global_pool.yield_per_period.into(),
							global_pool.total_shares_z,
							global_pool.max_reward_per_period,
						)?;
						Self::update_global_pool(global_pool, now_period, reward_per_period)?;
					}

					let new_stake_in_global_poll =
						Self::get_global_pool_shares(liq_pool.total_valued_shares, multiplier)?;

					global_pool.total_shares_z = global_pool
						.total_shares_z
						.checked_add(new_stake_in_global_poll)
						.ok_or(Error::<T>::Overflow)?;

					liq_pool.accumulated_rpz = global_pool.accumulated_rpz;
					liq_pool.updated_at = now_period;
					liq_pool.stake_in_global_pool = new_stake_in_global_poll;
					liq_pool.canceled = false;
					liq_pool.multiplier = multiplier;

					Self::deposit_event(Event::<T>::LiquidityMiningResumed {
						farm_id,
						liq_pool_farm_id: liq_pool.id,
						who,
						asset_pair,
						multiplier,
					});

					Ok(())
				})
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::remove_liquidity_pool())]
		#[transactional]
		pub fn remove_liquidity_pool(
			origin: OriginFor<T>,
			farm_id: GlobalPoolId,
			asset_pair: AssetPair,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let amm_account = T::AMM::get_pair_id(asset_pair);

			<LiquidityPoolData<T>>::try_mutate_exists(
				farm_id,
				amm_account,
				|maybe_liq_pool| -> DispatchResultWithPostInfo {
					let liq_pool = maybe_liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

					ensure!(liq_pool.canceled, Error::<T>::LiquidityMiningIsNotCanceled);

					<GlobalPoolData<T>>::try_mutate(farm_id, |maybe_global_pool| -> DispatchResultWithPostInfo {
						let global_pool = maybe_global_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

						ensure!(global_pool.owner == who, Error::<T>::Forbidden);

						global_pool.liq_pools_count =
							global_pool.liq_pools_count.checked_sub(1).ok_or(Error::<T>::Overflow)?;

						//transfer unpaid rewards back to global_pool
						let global_pool_account = Self::pool_account_id(global_pool.id)?;
						let liq_pool_account = Self::pool_account_id(liq_pool.id)?;

						let unpaid_rew =
							T::MultiCurrency::total_balance(global_pool.reward_currency, &liq_pool_account);
						T::MultiCurrency::transfer(
							global_pool.reward_currency,
							&liq_pool_account,
							&global_pool_account,
							unpaid_rew,
						)?;

						if let Some((_, nfts_in_class, _)) = Self::liq_pool_meta(liq_pool.id) {
							if nfts_in_class.is_zero() {
								<LiquidityPoolMetadata<T>>::remove(liq_pool.id);
							}
						};

						Ok(().into())
					})?;

					Self::deposit_event(Event::LiquidityPoolRemoved {
						farm_id,
						liq_pool_farm_id: liq_pool.id,
						who,
						asset_pair,
					});

					*maybe_liq_pool = None;
					Ok(().into())
				},
			)
		}

		#[pallet::weight(<T as Config>::WeightInfo::deposit_shares())]
		#[transactional]
		pub fn deposit_shares(
			origin: OriginFor<T>,
			farm_id: GlobalPoolId,
			asset_pair: AssetPair,
			shares_amount: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!shares_amount.is_zero(), Error::<T>::InvalidDepositAmount);

			let amm_share_token = T::AMM::get_share_token(asset_pair);

			ensure!(
				T::MultiCurrency::free_balance(amm_share_token, &who) >= shares_amount,
				Error::<T>::InsufficientAmmSharesBalance
			);

			let amm_account = T::AMM::get_pair_id(asset_pair);
			<LiquidityPoolData<T>>::try_mutate(farm_id, amm_account.clone(), |liq_pool| {
				let liq_pool = liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

				ensure!(!liq_pool.canceled, Error::<T>::LiquidityMiningCanceled);

				<GlobalPoolData<T>>::try_mutate(farm_id, |maybe_global_pool| {
					//something is very wrong if this fail, liq_pool can't exist without global_pool
					let global_pool = maybe_global_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

					let now_period = Self::get_now_period(global_pool.blocks_per_period)?;

					Self::maybe_update_pools(global_pool, liq_pool, now_period)?;

					let valued_shares =
						Self::get_valued_shares(shares_amount, amm_account, global_pool.reward_currency)?;
					let shares_in_global_pool_for_deposit =
						Self::get_global_pool_shares(valued_shares, liq_pool.multiplier)?;

					liq_pool.total_shares = liq_pool
						.total_shares
						.checked_add(shares_amount)
						.ok_or(Error::<T>::Overflow)?;

					liq_pool.total_valued_shares = liq_pool
						.total_valued_shares
						.checked_add(valued_shares)
						.ok_or(Error::<T>::Overflow)?;

					liq_pool.stake_in_global_pool = liq_pool
						.stake_in_global_pool
						.checked_add(shares_in_global_pool_for_deposit)
						.ok_or(Error::<T>::Overflow)?;

					global_pool.total_shares_z = global_pool
						.total_shares_z
						.checked_add(shares_in_global_pool_for_deposit)
						.ok_or(Error::<T>::Overflow)?;

					let pallet_account = Self::account_id();
					T::MultiCurrency::transfer(amm_share_token, &who, &pallet_account, shares_amount)?;

					let nft_id = Self::get_next_nft_id(liq_pool.id)?;
					let _ = pallet_nft::Pallet::<T>::do_mint(
						who.clone(),
						T::NftClass::get(),
						nft_id,
						vec![].try_into().unwrap(),
					)?;

					let d = Deposit::new(shares_amount, valued_shares, liq_pool.accumulated_rpvs, now_period);
					<DepositData<T>>::insert(&nft_id, d);
					<LiquidityPoolMetadata<T>>::try_mutate(liq_pool.id, |maybe_liq_pool_metadata| -> DispatchResult {
						//Something is very wrong if this fail. Metadata can exist without liq. pool but liq. pool can't
						//exist without metadata.
						let liq_pool_metadata = maybe_liq_pool_metadata
							.as_mut()
							.ok_or(Error::<T>::LiquidityPoolMetadataNotFound)?;
						liq_pool_metadata.1 = liq_pool_metadata.1.checked_add(1).ok_or(Error::<T>::Overflow)?;
						Ok(())
					})?;

					Self::deposit_event(Event::SharesDeposited {
						farm_id,
						liq_pool_farm_id: liq_pool.id,
						who,
						amount: shares_amount,
						lp_token: amm_share_token,
						nft_class: T::NftClass::get(),
						nft_instance_id: nft_id,
					});

					Ok(())
				})
			})
		}

		/// use `withdraw_shares()` if nft is valid but pool/farm doesn't exist or liq. mining is
		/// canceled
		#[pallet::weight(<T as Config>::WeightInfo::claim_rewards())]
		#[transactional]
		pub fn claim_rewards(origin: OriginFor<T>, nft_id: NftInstanceIdOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let liq_pool_id = Self::get_pool_id_from_nft_id(nft_id)?;

			//This is same as liq pool not found in this case. Liq. pool metadata CAN exist
			//without liq. pool but liq. pool CAN'T exist without metadata.
			let (asset_pair, _, farm_id) =
				<LiquidityPoolMetadata<T>>::get(liq_pool_id).ok_or(Error::<T>::LiquidityPoolNotFound)?;

			<DepositData<T>>::try_mutate(nft_id, |maybe_nft| {
				let deposit = maybe_nft.as_mut().ok_or(Error::<T>::NftDoesNotExist)?;

				let nft_owner = pallet_nft::Pallet::<T>::owner(T::NftClass::get(), nft_id)
					.ok_or(Error::<T>::CantFindDepositOwner)?;

				ensure!(nft_owner == who, Error::<T>::NotDepositOwner);

				let amm_account = T::AMM::get_pair_id(asset_pair);
				<LiquidityPoolData<T>>::try_mutate(farm_id, amm_account, |maybe_liq_pool| {
					let liq_pool = maybe_liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

					ensure!(!liq_pool.canceled, Error::<T>::LiquidityMiningCanceled);

					<GlobalPoolData<T>>::try_mutate(farm_id, |maybe_global_pool| {
						// something is very wrong if this fail
						let global_pool = maybe_global_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

						// can't claim multiple times in the same period
						let now_period = Self::get_now_period(global_pool.blocks_per_period)?;
						ensure!(deposit.updated_at != now_period, Error::<T>::DoubleClaimInThePeriod);

						Self::maybe_update_pools(global_pool, liq_pool, now_period)?;

						let (reward, _) = Self::do_claim_rewards(
							who.clone(),
							deposit,
							liq_pool,
							now_period,
							global_pool.reward_currency,
						)?;

						Self::deposit_event(Event::RewardClaimed {
							farm_id,
							liq_pool_farm_id: liq_pool.id,
							who,
							claimed: reward,
							reward_currency: global_pool.reward_currency,
						});

						Ok(())
					})
				})
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::withdraw_shares())]
		#[transactional]
		pub fn withdraw_shares(origin: OriginFor<T>, nft_id: NftInstanceIdOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let nft_owner =
				pallet_nft::Pallet::<T>::owner(T::NftClass::get(), nft_id).ok_or(Error::<T>::CantFindDepositOwner)?;

			ensure!(nft_owner == who, Error::<T>::NotDepositOwner);

			let liq_pool_id = Self::get_pool_id_from_nft_id(nft_id)?;
			<LiquidityPoolMetadata<T>>::try_mutate_exists(liq_pool_id, |maybe_liq_pool_metadata| {
				//This is same same as liq pool not found in this case. Liq. pool metadata CAN exist
				//without liq. pool but liq. pool CAN'T exist without metadata.
				//If metadata doesn't exist, user CAN'T withdraw.
				let (asset_pair, nfts_in_liq_pool, farm_id) =
					maybe_liq_pool_metadata.ok_or(Error::<T>::LiquidityPoolNotFound)?;

				<DepositData<T>>::try_mutate_exists(nft_id, |maybe_deposit| {
					let deposit = maybe_deposit.as_mut().ok_or(Error::<T>::NftDoesNotExist)?;

					let amm_account = T::AMM::get_pair_id(asset_pair);
					//metadata can be removed only if liq pool don't exists. It can be resumed if
					//it's only canceled.
					let mut can_remove_liq_pool_metadata = false;
					<LiquidityPoolData<T>>::try_mutate(
						farm_id,
						amm_account,
						|maybe_liq_pool| -> Result<(), DispatchError> {
							if maybe_liq_pool.is_some() {
								//This is intentional. This fn should not fail if liq. pool does not
								//exist, it should only behave differently.
								let liq_pool = maybe_liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

								<GlobalPoolData<T>>::try_mutate(
									farm_id,
									|maybe_global_pool| -> Result<(), DispatchError> {
										//This should never happen. If this happen something is very broken.
										let global_pool = maybe_global_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

										let now_period = Self::get_now_period(global_pool.blocks_per_period)?;

										if !liq_pool.canceled {
											Self::maybe_update_pools(global_pool, liq_pool, now_period)?;
										}

										let (reward, unclaimable_rewards) = Self::do_claim_rewards(
											who.clone(),
											deposit,
											liq_pool,
											now_period,
											global_pool.reward_currency,
										)?;

										let global_pool_account = Self::pool_account_id(global_pool.id)?;
										let liq_pool_account = Self::pool_account_id(liq_pool.id)?;

										liq_pool.total_shares = liq_pool
											.total_shares
											.checked_sub(deposit.shares)
											.ok_or(Error::<T>::Overflow)?;

										liq_pool.total_valued_shares = liq_pool
											.total_valued_shares
											.checked_sub(deposit.valued_shares)
											.ok_or(Error::<T>::Overflow)?;

										if !liq_pool.canceled {
											let shares_in_global_pool_for_deposit = Self::get_global_pool_shares(
												deposit.valued_shares,
												liq_pool.multiplier,
											)?;

											liq_pool.stake_in_global_pool = liq_pool
												.stake_in_global_pool
												.checked_sub(shares_in_global_pool_for_deposit)
												.ok_or(Error::<T>::Overflow)?;

											global_pool.total_shares_z = global_pool
												.total_shares_z
												.checked_sub(shares_in_global_pool_for_deposit)
												.ok_or(Error::<T>::Overflow)?;
										}

										T::MultiCurrency::transfer(
											global_pool.reward_currency,
											&liq_pool_account,
											&global_pool_account,
											unclaimable_rewards,
										)?;

										//emit this event only if something was claimed
										if !reward.is_zero() {
											Self::deposit_event(Event::RewardClaimed {
												farm_id: global_pool.id,
												liq_pool_farm_id: liq_pool.id,
												who: who.clone(),
												claimed: reward,
												reward_currency: global_pool.reward_currency,
											});
										}

										Ok(())
									},
								)?;
							} else {
								//canceled liq. pool can be resumed so metadata can be removed only
								//if liq pool doesn't exist.
								can_remove_liq_pool_metadata = true;
							}
							Ok(())
						},
					)?;

					//NOTE: no shares or rewards will be transferred to user if AMM no longer exists.
					if T::AMM::exists(asset_pair) {
						let amm_token = T::AMM::get_share_token(asset_pair);

						let pallet_account = Self::account_id();
						T::MultiCurrency::transfer(amm_token, &pallet_account, &who, deposit.shares)?;

						//NOTE: Theoretically neither `GlobalPool` nor
						//`LiquidityPoolYieldFarm` may not exist
						Self::deposit_event(Event::SharesWithdrawn {
							who: who.clone(),
							lp_token: amm_token,
							amount: deposit.shares,
						});
					}

					*maybe_deposit = None;
					pallet_nft::Pallet::<T>::do_burn(who, T::NftClass::get(), nft_id)?;

					if nfts_in_liq_pool.is_one() && can_remove_liq_pool_metadata {
						*maybe_liq_pool_metadata = None;
					} else {
						*maybe_liq_pool_metadata = Some((
							asset_pair,
							nfts_in_liq_pool.checked_sub(1).ok_or(Error::<T>::Overflow)?,
							farm_id,
						));
					}
					Ok(().into())
				})
			})
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_pool_id() -> Result<PoolId, Error<T>> {
		PoolIdSequencer::<T>::try_mutate(|current_id| {
			*current_id = current_id.checked_add(1).ok_or(Error::<T>::Overflow)?;

			Ok(*current_id)
		})
	}

	/// This function encode liq_pool_id into nft id. 4 most significant bytes of nft id are
	/// reserverd for liq pool id.
	fn get_next_nft_id(liq_pool_id: PoolId) -> Result<NftInstanceIdOf<T>, Error<T>> {
		Self::validate_pool_id(liq_pool_id)?;

		NftInstanceSequencer::<T>::try_mutate(|current_id| {
			*current_id = current_id.checked_add(1).ok_or(Error::<T>::Overflow)?;

			ensure!(MAX_NFT_INSTANCE_SEQUENCER.ge(current_id), Error::<T>::NftIdOwerflow);

			let mut id_bytes: [u8; 16] = [0; 16];

			id_bytes[..4].copy_from_slice(&liq_pool_id.to_le_bytes());
			id_bytes[4..].copy_from_slice(&current_id.to_le_bytes()[..12]);

			Ok(u128::from_le_bytes(id_bytes))
		})
	}

	/// This function decode liq. pool id from nft id
	fn get_pool_id_from_nft_id(nft_id: NftInstanceIdOf<T>) -> Result<PoolId, Error<T>> {
		//largets invalid nft id
		ensure!(4_294_967_296_u128.lt(&nft_id), Error::<T>::InvalidNftId);

		let mut pool_id_bytes = [0; 4];

		pool_id_bytes.copy_from_slice(&nft_id.to_le_bytes()[..4]);

		Ok(PoolId::from_le_bytes(pool_id_bytes))
	}

	/// Account id of pot holding all the shares
	fn account_id() -> AccountIdOf<T> {
		T::PalletId::get().into_account()
	}

	/// Return pallet account or pool acocunt from PoolId
	///
	/// WARN: pool_id = 0 is same as `T::PalletId::get().into_account()`. 0 is not valid value
	pub fn pool_account_id(pool_id: PoolId) -> Result<AccountIdOf<T>, Error<T>> {
		Self::validate_pool_id(pool_id)?;

		Ok(T::PalletId::get().into_sub_account(pool_id))
	}

	/// Return now period number
	fn get_now_period(blocks_per_period: BlockNumberFor<T>) -> Result<PeriodOf<T>, Error<T>> {
		Self::get_period_number(T::BlockNumberProvider::current_block_number(), blocks_per_period)
	}

	/// Return period number from block number now and number of blocks in one period
	fn get_period_number(
		block: BlockNumberFor<T>,
		blocks_per_period: BlockNumberFor<T>,
	) -> Result<PeriodOf<T>, Error<T>> {
		block.checked_div(&blocks_per_period).ok_or(Error::<T>::Overflow)
	}

	/// Loyalty multiplier  
	///
	// theta = periods/[(initial_reward_percentage + 1) * scale_coef];
	//
	// loyalty_multiplier = [theta + (theta * initial_reward_percentage) + initial_reward_percentage]/[theta + (theta * initial_reward_percentage) + 1]
	//
	fn get_loyalty_multiplier(periods: PeriodOf<T>, curve: Option<LoyaltyCurve>) -> Result<FixedU128, Error<T>> {
		let curve = match curve {
			Some(v) => v,
			None => return Ok(FixedU128::one()),
		};

		//b.is_one() is special case - this case is prevented by loyalty curve params validation
		if FixedPointNumber::is_one(&curve.initial_reward_percentage) {
			return Ok(FixedU128::one());
		}

		let denom = curve
			.initial_reward_percentage
			.checked_add(&1.into())
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(&FixedU128::from(curve.scale_coef as u128))
			.ok_or(Error::<T>::Overflow)?;

		let periods = FixedU128::from(TryInto::<u128>::try_into(periods).map_err(|_e| Error::<T>::Overflow)?);
		let theta = periods.checked_div(&denom).ok_or(Error::<T>::Overflow)?;

		// theta * initial_reward_percentage
		let theta_mul_initial_reward_percentage = theta
			.checked_mul(&curve.initial_reward_percentage)
			.ok_or(Error::<T>::Overflow)?;

		// theta + (theta * initial_reward_percentage)
		let theta_add_theta_mul_initial_reward_percentage = theta
			.checked_add(&theta_mul_initial_reward_percentage)
			.ok_or(Error::<T>::Overflow)?;

		// theta + (theta * initial_reward_percentage) + initial_reward_percentage
		let num = theta_add_theta_mul_initial_reward_percentage
			.checked_add(&curve.initial_reward_percentage)
			.ok_or(Error::<T>::Overflow)?;

		// theta + (theta * initial_reward_percentage) + 1
		let denom = theta_add_theta_mul_initial_reward_percentage
			.checked_add(&1.into())
			.ok_or(Error::<T>::Overflow)?;

		num.checked_div(&denom).ok_or(Error::<T>::Overflow)
	}

	fn get_global_pool_reward_per_period(
		yield_per_period: FixedU128,
		total_pool_shares_z: Balance,
		max_reward_per_period: Balance,
	) -> Result<Balance, Error<T>> {
		Ok(yield_per_period
			.checked_mul_int(total_pool_shares_z)
			.ok_or(Error::<T>::Overflow)?
			.min(max_reward_per_period))
	}

	fn update_global_pool(
		global_pool: &mut GlobalPool<T>,
		now_period: PeriodOf<T>,
		reward_per_period: Balance,
	) -> Result<(), Error<T>> {
		if global_pool.updated_at == now_period {
			return Ok(());
		}

		if global_pool.total_shares_z.is_zero() {
			return Ok(());
		}

		let periods_since_last_update: Balance = TryInto::<u128>::try_into(
			now_period
				.checked_sub(&global_pool.updated_at)
				.ok_or(Error::<T>::Overflow)?,
		)
		.map_err(|_e| Error::<T>::Overflow)?;

		let global_pool_account = Self::pool_account_id(global_pool.id)?;
		let left_to_distribute = T::MultiCurrency::free_balance(global_pool.reward_currency, &global_pool_account);
		let reward = periods_since_last_update
			.checked_mul(reward_per_period)
			.ok_or(Error::<T>::Overflow)?
			.min(left_to_distribute);

		if !reward.is_zero() {
			global_pool.accumulated_rpz =
				Self::get_accumulated_rps(global_pool.accumulated_rpz, global_pool.total_shares_z, reward)?;
			global_pool.accumulated_rewards = global_pool
				.accumulated_rewards
				.checked_add(reward)
				.ok_or(Error::<T>::Overflow)?;
		}

		global_pool.updated_at = now_period;

		Self::deposit_event(Event::FarmAccRPZUpdated {
			farm_id: global_pool.id,
			accumulated_rpz: global_pool.accumulated_rpz,
			total_shares_z: global_pool.total_shares_z,
		});

		Ok(())
	}

	fn get_accumulated_rps(
		accumulated_rps_now: Balance,
		total_shares: Balance,
		reward: Balance,
	) -> Result<Balance, Error<T>> {
		reward
			.checked_div(total_shares)
			.ok_or(Error::<T>::Overflow)?
			.checked_add(accumulated_rps_now)
			.ok_or(Error::<T>::Overflow)
	}

	/// (user_rewards, unclaimable_rewards)
	/// NOTE: claimable_reward and user_rewards is not the same !!!
	fn get_user_reward(
		accumulated_rpvs: Balance,
		valued_shares: Balance, // Value of shares at the time of entry in incentivized tokens.
		accumulated_claimed_rewards: Balance,
		accumulated_rpvs_now: Balance,
		loyalty_multiplier: FixedU128,
	) -> Result<(Balance, Balance), Error<T>> {
		let max_rewards = accumulated_rpvs_now
			.checked_sub(accumulated_rpvs)
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(valued_shares)
			.ok_or(Error::<T>::Overflow)?;

		if max_rewards.is_zero() {
			return Ok((0, 0));
		}

		let claimable_rewards = loyalty_multiplier
			.checked_mul_int(max_rewards)
			.ok_or(Error::<T>::Overflow)?;

		let unclaimable_rewards = max_rewards.checked_sub(claimable_rewards).ok_or(Error::<T>::Overflow)?;

		let user_rewards = claimable_rewards
			.checked_sub(accumulated_claimed_rewards)
			.ok_or(Error::<T>::Overflow)?;

		Ok((user_rewards, unclaimable_rewards))
	}

	fn claim_from_global_pool(
		global_pool: &mut GlobalPool<T>,
		liq_pool: &mut LiquidityPoolYieldFarm<T>,
		stake_in_global_pool: Balance,
	) -> Result<Balance, Error<T>> {
		let reward = global_pool
			.accumulated_rpz
			.checked_sub(liq_pool.accumulated_rpz)
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(stake_in_global_pool)
			.ok_or(Error::<T>::Overflow)?;

		liq_pool.accumulated_rpz = global_pool.accumulated_rpz;

		global_pool.paid_accumulated_rewards = global_pool
			.paid_accumulated_rewards
			.checked_add(reward)
			.ok_or(Error::<T>::Overflow)?;

		global_pool.accumulated_rewards = global_pool
			.accumulated_rewards
			.checked_sub(reward)
			.ok_or(Error::<T>::Overflow)?;

		Ok(reward)
	}

	fn update_liq_pool(
		pool: &mut LiquidityPoolYieldFarm<T>,
		pool_rewards: Balance,
		period_now: BlockNumberFor<T>,
		global_pool_id: PoolId,
		reward_currency: T::CurrencyId,
	) -> DispatchResult {
		if pool.updated_at == period_now {
			return Ok(());
		}

		if pool.total_valued_shares.is_zero() {
			return Ok(());
		}

		pool.accumulated_rpvs =
			Self::get_accumulated_rps(pool.accumulated_rpvs, pool.total_valued_shares, pool_rewards)?;
		pool.updated_at = period_now;

		let global_pool_balance =
			T::MultiCurrency::free_balance(reward_currency, &Self::pool_account_id(global_pool_id)?);

		ensure!(
			global_pool_balance >= pool_rewards,
			Error::<T>::InsufficientBalanceInGlobalPool
		);

		let global_pool_account = Self::pool_account_id(global_pool_id)?;
		let pool_account = Self::pool_account_id(pool.id)?;

		Self::deposit_event(Event::LiquidityPoolAccRPVSUpdated {
			farm_id: global_pool_id,
			liq_pool_farm_id: pool.id,
			accumulated_rpvs: pool.accumulated_rpvs,
			total_valued_shares: pool.total_valued_shares,
		});

		T::MultiCurrency::transfer(reward_currency, &global_pool_account, &pool_account, pool_rewards)
	}

	fn validate_pool_id(pool_id: PoolId) -> Result<(), Error<T>> {
		if pool_id.is_zero() {
			return Err(Error::<T>::InvalidPoolId);
		}

		Ok(())
	}

	fn validate_create_farm_data(
		total_rewards: Balance,
		planned_yielding_periods: PeriodOf<T>,
		blocks_per_period: BlockNumberFor<T>,
		yield_per_period: Permill,
	) -> DispatchResult {
		ensure!(
			total_rewards >= T::MinTotalFarmRewards::get(),
			Error::<T>::InvalidTotalRewards
		);

		ensure!(
			planned_yielding_periods >= T::MinPlannedYieldingPeriods::get(),
			Error::<T>::InvalidPlannedYieldingPeriods
		);

		ensure!(!blocks_per_period.is_zero(), Error::<T>::InvalidBlocksPerPeriod);

		ensure!(!yield_per_period.is_zero(), Error::<T>::InvalidYieldPerPeriod);

		Ok(())
	}

	fn get_global_pool_shares(valued_shares: Balance, multiplier: PoolMultiplier) -> Result<Balance, Error<T>> {
		multiplier.checked_mul_int(valued_shares).ok_or(Error::<T>::Overflow)
	}

	fn get_valued_shares(
		shares: Balance,
		amm: AccountIdOf<T>,
		reward_currency: T::CurrencyId,
	) -> Result<Balance, Error<T>> {
		let reward_currency_balance = T::MultiCurrency::free_balance(reward_currency, &amm);

		shares.checked_mul(reward_currency_balance).ok_or(Error::<T>::Overflow)
	}

	fn do_claim_rewards(
		who: AccountIdOf<T>,
		d: &mut Deposit<T>,
		liq_pool: &LiquidityPoolYieldFarm<T>,
		now_period: PeriodOf<T>,
		reward_currency: T::CurrencyId,
	) -> Result<(Balance, Balance), DispatchError> {
		let periods = now_period.checked_sub(&d.entered_at).ok_or(Error::<T>::Overflow)?;

		if d.updated_at == now_period {
			return Ok((0, 0));
		}

		let loyalty_multiplier = Self::get_loyalty_multiplier(periods, liq_pool.loyalty_curve.clone())?;

		let (rewards, unclaimable_rewards) = Self::get_user_reward(
			d.accumulated_rpvs,
			d.valued_shares,
			d.accumulated_claimed_rewards,
			liq_pool.accumulated_rpvs,
			loyalty_multiplier,
		)?;

		d.accumulated_claimed_rewards = d
			.accumulated_claimed_rewards
			.checked_add(rewards)
			.ok_or(Error::<T>::Overflow)?;

		d.updated_at = now_period;

		let liq_pool_account = Self::pool_account_id(liq_pool.id)?;
		T::MultiCurrency::transfer(reward_currency, &liq_pool_account, &who, rewards)?;

		Ok((rewards, unclaimable_rewards))
	}

	/// This function update pools if conditions are met
	fn maybe_update_pools(
		global_pool: &mut GlobalPool<T>,
		liq_pool: &mut LiquidityPoolYieldFarm<T>,
		now_period: PeriodOf<T>,
	) -> DispatchResult {
		if liq_pool.canceled {
			return Ok(());
		}

		if !liq_pool.total_shares.is_zero() && liq_pool.updated_at != now_period {
			if !global_pool.total_shares_z.is_zero() && global_pool.updated_at != now_period {
				let rewards = Self::get_global_pool_reward_per_period(
					global_pool.yield_per_period.into(),
					global_pool.total_shares_z,
					global_pool.max_reward_per_period,
				)?;

				Self::update_global_pool(global_pool, now_period, rewards)?;
			}

			let rewards = Self::claim_from_global_pool(global_pool, liq_pool, liq_pool.stake_in_global_pool)?;
			Self::update_liq_pool(
				liq_pool,
				rewards,
				now_period,
				global_pool.id,
				global_pool.reward_currency,
			)?;
		}

		Ok(())
	}
}
