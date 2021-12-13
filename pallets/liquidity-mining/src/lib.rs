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
//  rps - reward per share
//  rpz - reward per share in global pool

// Notion spec naming map:
// * shares                 -> s
// * total_shares           -> S
// * valued_shares          -> s'
// * total_valued_shares    -> S'
// * stake_in_global_pool   -> z
// * total_shares_z         -> Z
// * multiplier             -> m

//NOTE:
//`LiquidityPoolYieldFarm.stake_in_global_pool` - can be removed/replaces with:
//`LiquidityPoolYieldFarm.total_valued_shares * LiquidityPoolYieldFarm.multiplier`

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;
pub mod weights;

pub use pallet::*;

type PoolId = u32;
type PoolMultiplier = u32;

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
use frame_support::sp_runtime::{traits::AccountIdConversion, RuntimeDebug};
use hydradx_traits::AMM;
use scale_info::TypeInfo;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AssetIdOf<T> = <T as pallet::Config>::CurrencyId;
type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;
type PeriodOf<T> = <T as frame_system::Config>::BlockNumber;
type NftClassIdOf<T> = <T as pallet_nft::Config>::NftClassId;
type NftInstanceIfOf<T> = <T as pallet_nft::Config>::NftInstanceId;

use pallet_nft::types::ClassType::PoolShare;

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebugNoBound, TypeInfo)]
pub struct GlobalPool<T: Config> {
	id: PoolId,
	owner: AccountIdOf<T>,
	updated_at: PeriodOf<T>,
	total_shares_z: Balance,
	accumulated_rpz: Balance,
	reward_currency: AssetIdOf<T>,
	accumulated_rewards: Balance,
	paid_accumulated_rewards: Balance,
	yield_per_period: Permill,
	planned_yielding_periods: PeriodOf<T>,
	blocks_per_period: BlockNumberFor<T>,
	incentivized_token: AssetIdOf<T>,
	max_reward_per_period: Balance,
	liq_pools_count: u32,
}

impl<T: Config> GlobalPool<T> {
	fn new(
		id: PoolId,
		updated_at: PeriodOf<T>,
		reward_currency: T::CurrencyId,
		yield_per_period: Permill,
		planned_yielding_periods: PeriodOf<T>,
		blocks_per_period: T::BlockNumber,
		owner: AccountIdOf<T>,
		incentivized_token: T::CurrencyId,
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
			incentivized_token,
			max_reward_per_period,
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebugNoBound, TypeInfo)]
pub struct LiquidityPoolYieldFarm<T: Config> {
	id: PoolId,
	updated_at: PeriodOf<T>,
	total_shares: Balance,
	total_valued_shares: Balance,
	accumulated_rps: Balance,
	accumulated_rpz: Balance,
	loyalty_curve: Option<LoyaltyCurve>,
	stake_in_global_pool: Balance,
	multiplier: PoolMultiplier,
	nft_class: NftClassIdOf<T>,
	canceled: bool,
}

impl<T: Config> LiquidityPoolYieldFarm<T> {
	fn new(
		id: PoolId,
		updated_at: PeriodOf<T>,
		loyalty_curve: Option<LoyaltyCurve>,
		multiplier: u32,
		nft_class: NftClassIdOf<T>,
	) -> Self {
		Self {
			accumulated_rps: Default::default(),
			accumulated_rpz: Default::default(),
			stake_in_global_pool: Default::default(),
			total_shares: Default::default(),
			total_valued_shares: Default::default(),
			canceled: false,
			id,
			updated_at,
			loyalty_curve,
			multiplier,
			nft_class,
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

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Deposit<T: Config> {
	shares: Balance,
	valued_shares: Balance,
	accumulated_rps: Balance,
	accumulated_claimed_rewards: Balance,
	entered_period: PeriodOf<T>,
}

impl<T: Config> Deposit<T> {
	fn new(shares: Balance, valued_shares: Balance, accumulated_rps: Balance, entered_period: PeriodOf<T>) -> Self {
		Self {
			entered_period,
			shares,
			valued_shares,
			accumulated_rps,
			accumulated_claimed_rewards: Default::default(),
		}
	}
}

use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::FixedPointNumber;
use sp_std::convert::{From, Into, TryInto};

use primitives::{asset::AssetPair, Balance};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config + TypeInfo + pallet_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Asset type
		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + From<u32>;

		/// Currency for transfers
		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = Self::CurrencyId, Balance = Balance>;

		/// AMM helper functions
		type AMM: AMM<Self::AccountId, Self::CurrencyId, AssetPair, Balance>;

		/// The origin account that cat create new liquidity mining program
		type CreateOrigin: EnsureOrigin<Self::Origin>;

		type PalletId: Get<PalletId>;

		/// Minimum anout of total rewards to create new farm
		type MinTotalFarmRewards: Get<Balance>;

		/// Minimalnumber of periods to distribute farm rewards
		type MinPlannedYieldingPeriods: Get<Self::BlockNumber>;

		/// The block number provider
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = Self::BlockNumber>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		/// Math computation overflow
		Overflow,

		/// Insufficient balance in global pool to transfer rewards to pool
		InsuffcientBalanceInGlobalPool,

		/// Provide id is not valid. Valid range is [1, u32::MAX)
		InvalidPoolId,

		/// Provided planed_yielding_periods is below limit
		InvalidPlannedYieldingPeriods,

		/// Provided blocks_per_period can't be 0
		InvalidBlocksPerPeriod,

		/// Yield per period can't be 0
		InvalidYieldPerPeriod,

		/// Provided total_rewards for farming is bellow min limit
		InvalidTotalRewards,

		/// Reward currency balance too low
		InsufficientRewardCurrencyBalance,

		/// Account is not allowed to perform action
		Forbidden,

		/// Farm does not exist
		FarmNotFound,

		/// Liquidity pool already exist in farm
		LiquidityPoolAlreadyExists,

		/// Pool multiplier can't be 0
		InvalidMultiplier,

		/// Loyalty curver b param should be from [0, 1)
		InvalidLoyaltyCurverParamB,

		/// Account balance of amm pool shares is not sufficient
		InsufficientAmmSharesBalance,

		/// AMM pool does not exist
		AmmPoolDoesNotExist,

		/// Liq. pool for provided assets was not found in farm
		LiquidityPoolNotFound,

		/// Account already have stake in liq. pool in farm
		DuplicateDeposit,

		/// One or moe liq. pools exist in farm. Only farm without liq. pools can be destroyed.
		FarmIsNotEmpty,

		/// Balance on rewards account is not 0. Only farm with 0 raward balance can be destroyed.
		RewardBalanceIsNotZero,

		/// NFT class for liq. pool does not exists.
		NftClassDoesNotExists,

		/// NFT does not exist.
		NftDoesNotExist,

		/// Provided nft doesn't belong to provided asset pair.
		NftAssetsMismatch,

		/// Liquidity mining for porovided pool is canceled.
		LiquidityMiningCanceled,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New farm was creaated by `CreateOrigin` origin. [pool_id, global_pool]
		FarmCreated(PoolId, GlobalPool<T>),

		/// New liquidity(AMM) pool was added to farm [farm_id, amm_pool_id, liquidity_pool]
		LiquidityPoolAdded(PoolId, AccountIdOf<T>, LiquidityPoolYieldFarm<T>),

		/// Liq. mining farm was destroyed [farm_id, origin]
		FarmDestroyed(PoolId, AccountIdOf<T>),

		/// Amm shares was deposited into liq. mining pool [farm_id, liq_pool_id, who, shares,
		/// share_token, nft_instance_id]
		SharesDeposited(
			PoolId,
			PoolId,
			AccountIdOf<T>,
			Balance,
			T::CurrencyId,
			NftInstanceIfOf<T>,
		),

		/// Liquidity provider claimed rewards. [who, farm_id, liq_pooo_id, claimed_amount, reward_token]
		RewardClaimed(AccountIdOf<T>, PoolId, PoolId, Balance, T::CurrencyId),
	}

	#[pallet::storage]
	#[pallet::getter(fn pool_id)]
	pub type PoolIdSeq<T: Config> = StorageValue<_, PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn global_pool)]
	type GlobalPoolData<T: Config> = StorageMap<_, Twox64Concat, PoolId, GlobalPool<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	type LiquidityPoolData<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		PoolId, //global_pool_id
		Twox64Concat,
		AccountIdOf<T>, //amm_pool_id
		LiquidityPoolYieldFarm<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn deposit)]
	type DepositData<T: Config> =
		StorageDoubleMap<_, Twox64Concat, NftClassIdOf<T>, Twox64Concat, NftInstanceIfOf<T>, Deposit<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nft_class)] //(asset_pair, amount of existing nfts, globalPoolId)
	type NftClassData<T: Config> = StorageMap<_, Twox64Concat, NftClassIdOf<T>, (AssetPair, u64, PoolId), OptionQuery>;

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
		/// - `incentivized_token`
		/// - `reward_currency`
		/// - `admin_account`
		/// - `yield_per_period`
		#[pallet::weight(1000)]
		#[transactional]
		pub fn create_farm(
			origin: OriginFor<T>,
			total_rewards: Balance,
			planned_yielding_periods: PeriodOf<T>,
			blocks_per_period: BlockNumberFor<T>,
			incentivized_token: AssetIdOf<T>,
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
				Balance::from(TryInto::<u128>::try_into(planned_yielding_periods).map_err(|_e| Error::<T>::Overflow)?);
			let max_reward_per_period = total_rewards.checked_div(planned_periods).ok_or(Error::<T>::Overflow)?;
			let now_period = Self::get_now_period(blocks_per_period)?;
			let pool_id = Self::get_next_id()?;

			let pool = GlobalPool::new(
				pool_id,
				now_period,
				reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			<GlobalPoolData<T>>::insert(&pool.id, &pool);

			let pool_account = Self::pool_account_id(pool.id)?;
			T::MultiCurrency::transfer(reward_currency, &pool.owner, &pool_account, total_rewards)?;

			Self::deposit_event(Event::FarmCreated(pool.id, pool));

			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn destroy_farm(origin: OriginFor<T>, farm_id: PoolId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<GlobalPoolData<T>>::try_mutate_exists(farm_id, |maybe_g_pool| -> DispatchResult {
				let g_pool = maybe_g_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

				ensure!(who == g_pool.owner, Error::<T>::Forbidden);

				ensure!(g_pool.liq_pools_count.is_zero(), Error::<T>::FarmIsNotEmpty);

				let g_pool_account = Self::pool_account_id(g_pool.id)?;
				//Note: should this be 0?
				ensure!(
					T::MultiCurrency::free_balance(g_pool.reward_currency, &g_pool_account).is_zero(),
					Error::<T>::RewardBalanceIsNotZero
				);

				//TODO: handle/check unclaimable rewards form user withdrawn

				*maybe_g_pool = None;

				Self::deposit_event(Event::FarmDestroyed(farm_id, who));
				Ok(())
			})
		}

		#[pallet::weight(1000)]
		pub fn withdraw_undistributed_rewards(_origin: OriginFor<T>, _farm_id: PoolId) -> DispatchResult {
			//NOTE: don't forget to check if amm stil exists otherwise get_share_token return
			//0(native token) and I will transfer native token instead of shares
			todo!()
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn add_liquidity_pool(
			origin: OriginFor<T>,
			farm_id: PoolId,
			asset_pair: AssetPair,
			multiplier: PoolMultiplier,
			loyalty_curve: Option<LoyaltyCurve>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!multiplier.is_zero(), Error::<T>::InvalidMultiplier);

			if loyalty_curve.is_some() {
				let c = loyalty_curve.as_ref().unwrap();
				ensure!(
					c.initial_reward_percentage.lt(&FixedU128::one()),
					Error::<T>::InvalidLoyaltyCurverParamB
				);
			}

			ensure!(T::AMM::exists(asset_pair), Error::<T>::AmmPoolDoesNotExist);

			<GlobalPoolData<T>>::try_mutate(farm_id, |maybe_pool| -> DispatchResult {
				let g_pool = maybe_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

				ensure!(who == g_pool.owner, Error::<T>::Forbidden);

				let amm_pool_id = T::AMM::get_pair_id(asset_pair);
				ensure!(
					!<LiquidityPoolData<T>>::contains_key(farm_id, &amm_pool_id),
					Error::<T>::LiquidityPoolAlreadyExists
				);

				let now_period = Self::get_now_period(g_pool.blocks_per_period)?;
				if !g_pool.total_shares_z.is_zero() && g_pool.updated_at != now_period {
					let reward_per_period = Self::get_reward_per_period(
						g_pool.yield_per_period.into(),
						g_pool.total_shares_z,
						g_pool.max_reward_per_period,
					)?;
					Self::update_global_pool(g_pool, now_period, reward_per_period)?;
				}
				g_pool.liq_pools_count = g_pool.liq_pools_count.checked_add(1).ok_or(Error::<T>::Overflow)?;

				let liq_pool_id = Self::get_next_id()?;
				let pallet_account = Self::account_id();
				let nft_class = pallet_nft::Pallet::<T>::do_create_class(Some(pallet_account), PoolShare, vec![])?;
				<NftClassData<T>>::insert(nft_class, (asset_pair, 0, g_pool.id));

				let pool = LiquidityPoolYieldFarm::new(liq_pool_id, now_period, loyalty_curve, multiplier, nft_class);

				<LiquidityPoolData<T>>::insert(g_pool.id, &amm_pool_id, &pool);

				Self::deposit_event(Event::LiquidityPoolAdded(g_pool.id, amm_pool_id, pool));

				Ok(())
			})
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn update_liquidity_pool(
			origin: OriginFor<T>,
			farm_id: PoolId,
			asset_pair: AssetPair,
			multiplier: PoolMultiplier,
		) -> DispatchResult {
			//TODO: this fn is WIP
			let who = ensure_signed(origin)?;

			ensure!(!multiplier.is_zero(), Error::<T>::InvalidMultiplier);

			let amm_pool_id = T::AMM::get_pair_id(asset_pair);
			<LiquidityPoolData<T>>::try_mutate(farm_id, &amm_pool_id, |liq_pool| {
				let liq_pool = liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

				<GlobalPoolData<T>>::try_mutate(farm_id, |g_pool| {
					let g_pool = g_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

					ensure!(who == g_pool.owner, Error::<T>::Forbidden);

					let now_period = Self::get_now_period(g_pool.blocks_per_period)?;
					let reward_per_period = Self::get_reward_per_period(
						g_pool.yield_per_period.into(),
						g_pool.total_shares_z,
						g_pool.max_reward_per_period,
					)?;
					Self::update_global_pool(g_pool, now_period, reward_per_period)?;

					let pool_reward = Self::claim_from_global_pool(g_pool, liq_pool, liq_pool.stake_in_global_pool)?;
					Self::update_pool(liq_pool, pool_reward, now_period, g_pool.id, g_pool.reward_currency)?;

					let incentivized_token_balance_in_amm =
						T::MultiCurrency::free_balance(g_pool.reward_currency, &amm_pool_id);

					let new_stake_in_global_pool = incentivized_token_balance_in_amm
						.checked_mul(liq_pool.total_valued_shares)
						.ok_or(Error::<T>::Overflow)?
						.checked_mul(multiplier.into())
						.ok_or(Error::<T>::Overflow)?;

					g_pool
						.total_shares_z
						.checked_sub(liq_pool.stake_in_global_pool)
						.ok_or(Error::<T>::Overflow)?;
					g_pool
						.total_shares_z
						.checked_add(new_stake_in_global_pool)
						.ok_or(Error::<T>::Overflow)?;

					liq_pool.stake_in_global_pool = new_stake_in_global_pool;
					liq_pool.multiplier = multiplier;

					Ok(())
				})
			})
		}

		#[pallet::weight(1000)]
		pub fn cancel_liqudity_pool(_origin: OriginFor<T>, _farm_id: PoolId) -> DispatchResult {
			todo!()
		}

		#[pallet::weight(1000)]
		pub fn remove_liqudity_pool(_origin: OriginFor<T>, _farm_id: PoolId) -> DispatchResult {
			todo!()
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn deposit_shares(
			origin: OriginFor<T>,
			farm_id: PoolId,
			asset_pair: AssetPair,
			shares: Balance,
		) -> DispatchResult {
			//TODO: ..._should_not_work tests
			let who = ensure_signed(origin)?;

			let amm_share_token = T::AMM::get_share_token(asset_pair);

			ensure!(
				T::MultiCurrency::free_balance(amm_share_token, &who) >= shares,
				Error::<T>::InsufficientAmmSharesBalance
			);

			let amm_account = T::AMM::get_pair_id(asset_pair);
			<LiquidityPoolData<T>>::try_mutate(farm_id, amm_account.clone(), |liq_pool| {
				let liq_pool = liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

				//TODO: add test
				ensure!(!liq_pool.canceled, Error::<T>::LiquidityMiningCanceled);

				<GlobalPoolData<T>>::try_mutate(farm_id, |g_pool| {
					let g_pool = g_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

					let now_period = Self::get_now_period(g_pool.blocks_per_period)?;

					Self::maybe_update_pools(g_pool, liq_pool, now_period)?;

					let valued_shares = Self::get_valued_shares(shares, amm_account, g_pool.reward_currency)?;
					let global_pool_shares = Self::get_global_pool_shares(valued_shares, liq_pool.multiplier)?;

					liq_pool.total_shares = liq_pool.total_shares.checked_add(shares).ok_or(Error::<T>::Overflow)?;

					liq_pool.total_valued_shares = liq_pool
						.total_valued_shares
						.checked_add(valued_shares)
						.ok_or(Error::<T>::Overflow)?;

					liq_pool.stake_in_global_pool = liq_pool
						.stake_in_global_pool
						.checked_add(global_pool_shares)
						.ok_or(Error::<T>::Overflow)?;

					g_pool.total_shares_z = g_pool
						.total_shares_z
						.checked_add(global_pool_shares)
						.ok_or(Error::<T>::Overflow)?;

					let pallet_account = Self::account_id();
					T::MultiCurrency::transfer(amm_share_token, &who, &pallet_account, shares)?;

					//TODO/NOTE: This will change after NFT pallet refactor
					let (_, nft_id) = pallet_nft::Pallet::<T>::do_mint_for_liquidity_mining(
						liq_pool.nft_class,
						who.clone(),
						vec![],
						shares,
						valued_shares,
						liq_pool.accumulated_rps,
						0_u128,
						T::BlockNumberProvider::current_block_number(),
					)?;

					let d = Deposit::new(shares, valued_shares, liq_pool.accumulated_rps, now_period);
					<DepositData<T>>::insert(&liq_pool.nft_class, &nft_id, d);
					<NftClassData<T>>::try_mutate(liq_pool.nft_class, |maybe_class| -> DispatchResult {
						let class = maybe_class.as_mut().ok_or(Error::<T>::NftClassDoesNotExists)?;
						class.1 = class.1.checked_add(1).ok_or(Error::<T>::Overflow)?;
						Ok(())
					})?;

					Self::deposit_event(Event::SharesDeposited(
						g_pool.id,
						liq_pool.id,
						who,
						shares,
						amm_share_token,
						nft_id,
					));

					Ok(())
				})
			})
		}

		/// use `withdraw_shares()` if nft is valid but pool/farm wasn't found
		#[pallet::weight(1000)]
		#[transactional]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			nft_class_id: NftClassIdOf<T>,
			nft_id: NftInstanceIfOf<T>,
		) -> DispatchResult {
			//NOTE: WIP
			let who = ensure_signed(origin)?;

			let (asset_pair, _, farm_id) =
				<NftClassData<T>>::get(nft_class_id).ok_or(Error::<T>::NftClassDoesNotExists)?;

			<DepositData<T>>::try_mutate(nft_class_id, nft_id, |maybe_nft| {
				let deposit = maybe_nft.as_mut().ok_or(Error::<T>::NftDoesNotExist)?;

				//TODO: check nft owner. Waiting for nft pallet refactor

				let amm_account = T::AMM::get_pair_id(asset_pair);
				<LiquidityPoolData<T>>::try_mutate(farm_id, amm_account, |maybe_liq_pool| {
					let liq_pool = maybe_liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

					//NOTE: do we want to force him to use withdraw?
					ensure!(!liq_pool.canceled, Error::<T>::LiquidityMiningCanceled);

					<GlobalPoolData<T>>::try_mutate(farm_id, |g_pool| {
						let g_pool = g_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

						let now_period = Self::get_now_period(g_pool.blocks_per_period)?;

						if !liq_pool.stake_in_global_pool.is_zero() {
							Self::maybe_update_pools(g_pool, liq_pool, now_period)?;
						}

						let (reward, _) =
							Self::do_claim_rewards(who.clone(), deposit, liq_pool, now_period, g_pool.reward_currency)?;

						Self::deposit_event(Event::RewardClaimed(
							who,
							g_pool.id,
							liq_pool.id,
							reward,
							g_pool.reward_currency,
						));

						Ok(())
					})
				})
			})
		}

		#[pallet::weight(1000)]
		#[transactional]
		pub fn withdraw_shares(
			origin: OriginFor<T>,
			nft_class_id: NftClassIdOf<T>,
			nft_id: NftInstanceIfOf<T>,
		) -> DispatchResult {
			//NOTE: WIP
			let who = ensure_signed(origin)?;

			//TODO: check nft owner. Waiting for nft refactor

			<NftClassData<T>>::try_mutate_exists(nft_class_id, |maybe_nft_class| {
				let (asset_pair, nfts_in_class, farm_id) = maybe_nft_class.ok_or(Error::<T>::NftClassDoesNotExists)?;

				<DepositData<T>>::try_mutate_exists(nft_class_id, nft_id, |maybe_nft| {
					let deposit = maybe_nft.as_mut().ok_or(Error::<T>::NftDoesNotExist)?;

					let amm_account = T::AMM::get_pair_id(asset_pair);
					let mut liq_pool_canceled = false;
					<LiquidityPoolData<T>>::try_mutate(
						farm_id,
						amm_account,
						|maybe_liq_pool| -> Result<(), DispatchError> {
							if maybe_liq_pool.is_some() {
								//This is intetional. This fn should not fail if liq. pool does not
								//exist, it should only update different things if do exist.
								let liq_pool = maybe_liq_pool.as_mut().ok_or(Error::<T>::LiquidityPoolNotFound)?;

								liq_pool_canceled = liq_pool.canceled;

								<GlobalPoolData<T>>::try_mutate(
									farm_id,
									|maybe_g_pool| -> Result<(), DispatchError> {
										//This should never happen. If this happen somethin is very broken.
										let g_pool = maybe_g_pool.as_mut().ok_or(Error::<T>::FarmNotFound)?;

										let now_period = Self::get_now_period(g_pool.blocks_per_period)?;

										if !liq_pool.stake_in_global_pool.is_zero() {
											Self::maybe_update_pools(g_pool, liq_pool, now_period)?;
										}
										let (reward, unclaimable_rewards) = Self::do_claim_rewards(
											who.clone(),
											deposit,
											liq_pool,
											now_period,
											g_pool.reward_currency,
										)?;

										let g_pool_account = Self::pool_account_id(g_pool.id)?;
										let liq_pool_account = Self::pool_account_id(liq_pool.id)?;

										liq_pool.total_shares = liq_pool
											.total_shares
											.checked_sub(deposit.shares)
											.ok_or(Error::<T>::Overflow)?
											.min(0);

										liq_pool.total_valued_shares = liq_pool
											.total_valued_shares
											.checked_sub(deposit.valued_shares)
											.ok_or(Error::<T>::Overflow)?
											.min(0);

										let global_pool_shares =
											Self::get_global_pool_shares(deposit.valued_shares, liq_pool.multiplier)?;

										liq_pool.stake_in_global_pool = liq_pool
											.stake_in_global_pool
											.checked_sub(global_pool_shares)
											.ok_or(Error::<T>::Overflow)?
											.min(0);

										g_pool.total_shares_z = g_pool
											.total_shares_z
											.checked_sub(global_pool_shares)
											.ok_or(Error::<T>::Overflow)?
											.min(0);

										T::MultiCurrency::transfer(
											g_pool.reward_currency,
											&liq_pool_account,
											&g_pool_account,
											unclaimable_rewards,
										)?;

										//Note: do we want this event?
										Self::deposit_event(Event::RewardClaimed(
											who.clone(),
											g_pool.id,
											liq_pool.id,
											reward,
											g_pool.reward_currency,
										));
										Ok(())
									},
								)?;
							}

							//Note: how to communicate this to user - no shares will be transferes?
							if T::AMM::exists(asset_pair) {
								let amm_token = T::AMM::get_share_token(asset_pair);

								let pallet_account = Self::account_id();
								T::MultiCurrency::transfer(amm_token, &pallet_account, &who, deposit.shares)?;
							}

							if nfts_in_class.is_one() && (maybe_liq_pool.is_none() || liq_pool_canceled) {
								//TODO: Burn NFT
								*maybe_nft_class = None;
							} else {
								*maybe_nft_class = Some((
									asset_pair,
									nfts_in_class.checked_sub(1).ok_or(Error::<T>::Overflow)?,
									farm_id,
								));
							}
							Ok(())
						},
					)?;
					Ok(())
				})
			})
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_id() -> Result<PoolId, Error<T>> {
		PoolIdSeq::<T>::try_mutate(|current_id| {
			*current_id = current_id.checked_add(1).ok_or(Error::<T>::Overflow)?;

			Ok(*current_id)
		})
	}

	/// Account id of pot holding all the shares
	fn account_id() -> AccountIdOf<T> {
		T::PalletId::get().into_account()
	}

	/// Return pallet account or pool acocunt from PoolId
	///
	/// WARN: pool_id = 0 is same as `T::PalletId::get().into_account()`. 0 is not valid value
	fn pool_account_id(pool_id: PoolId) -> Result<AccountIdOf<T>, Error<T>> {
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
	// theta = periods/[(b + 1) * scale_coef];
	//
	// loyalty-multiplier = [theta + (theta * b) + b]/[theta + (theta * b) + 1]
	//
	fn get_loyalty_multiplier(periods: PeriodOf<T>, curve: Option<LoyaltyCurve>) -> Result<FixedU128, Error<T>> {
		let curve = match curve {
			Some(v) => v,
			None => return Ok(FixedU128::one()),
		};

		//b.is_one() is special case - this case is prevented by validate_loyalty_curve()
		if FixedPointNumber::is_one(&curve.initial_reward_percentage) {
			return Ok(FixedU128::one());
		}

		let denom = curve
			.initial_reward_percentage
			.checked_add(&1.into())
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(&FixedU128::from(Into::<u128>::into(curve.scale_coef)))
			.ok_or(Error::<T>::Overflow)?;

		let periods = FixedU128::from(TryInto::<u128>::try_into(periods).map_err(|_e| Error::<T>::Overflow)?);
		let theta = periods.checked_div(&denom).ok_or(Error::<T>::Overflow)?;

		let theta_mul_b = theta
			.checked_mul(&curve.initial_reward_percentage)
			.ok_or(Error::<T>::Overflow)?;

		let theta_add_theta_mul_b = theta.checked_add(&theta_mul_b).ok_or(Error::<T>::Overflow)?;

		let num = theta_add_theta_mul_b
			.checked_add(&curve.initial_reward_percentage)
			.ok_or(Error::<T>::Overflow)?;

		let denom = theta_add_theta_mul_b
			.checked_add(&1.into())
			.ok_or(Error::<T>::Overflow)?;

		num.checked_div(&denom).ok_or(Error::<T>::Overflow)
	}

	fn get_reward_per_period(
		yield_per_period: FixedU128,
		total_global_pool_shares_z: Balance,
		max_reward_per_period: Balance,
	) -> Result<Balance, Error<T>> {
		Ok(yield_per_period
			.checked_mul_int(total_global_pool_shares_z)
			.ok_or(Error::<T>::Overflow)?
			.min(max_reward_per_period))
	}

	fn update_global_pool(
		pool: &mut GlobalPool<T>,
		now_period: PeriodOf<T>,
		reward_per_period: Balance,
	) -> Result<(), Error<T>> {
		if pool.updated_at == now_period {
			return Ok(());
		}

		if pool.total_shares_z.is_zero() {
			return Ok(());
		}

		let periods_since_last_update: Balance =
			TryInto::<u128>::try_into(now_period.checked_sub(&pool.updated_at).ok_or(Error::<T>::Overflow)?)
				.map_err(|_e| Error::<T>::Overflow)?
				.into();

		let pool_account = Self::pool_account_id(pool.id)?;
		let reward = periods_since_last_update
			.checked_mul(reward_per_period)
			.ok_or(Error::<T>::Overflow)?
			.min(T::MultiCurrency::free_balance(
				pool.reward_currency,
				&pool_account.clone(),
			));

		if !reward.is_zero() {
			pool.accumulated_rpz = Self::get_accumulated_rps(pool.accumulated_rpz, pool.total_shares_z, reward)?;
			pool.accumulated_rewards = pool
				.accumulated_rewards
				.checked_add(reward)
				.ok_or(Error::<T>::Overflow)?;
		}

		pool.updated_at = now_period;

		return Ok(());
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
		accumulated_rps: Balance,
		valued_shares: Balance, // Value of shares at the time of entry in incentivized tokens.
		accumulated_claimed_rewards: Balance,
		accumulated_rps_now: Balance,
		loyalty_multiplier: FixedU128,
	) -> Result<(Balance, Balance), Error<T>> {
		let max_rewards = accumulated_rps_now
			.checked_sub(accumulated_rps)
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(valued_shares)
			.ok_or(Error::<T>::Overflow)?;

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
		g_pool: &mut GlobalPool<T>,
		liq_pool: &mut LiquidityPoolYieldFarm<T>,
		shares: Balance,
	) -> Result<Balance, Error<T>> {
		let reward = g_pool
			.accumulated_rpz
			.checked_sub(liq_pool.accumulated_rpz)
			.ok_or(Error::<T>::Overflow)?
			.checked_mul(shares)
			.ok_or(Error::<T>::Overflow)?
			.min(g_pool.accumulated_rewards);

		liq_pool.accumulated_rpz = g_pool.accumulated_rpz;

		g_pool.paid_accumulated_rewards = g_pool
			.paid_accumulated_rewards
			.checked_add(reward)
			.ok_or(Error::<T>::Overflow)?;

		g_pool.accumulated_rewards = g_pool
			.accumulated_rewards
			.checked_sub(reward)
			.ok_or(Error::<T>::Overflow)?;

		return Ok(reward);
	}

	fn update_pool(
		pool: &mut LiquidityPoolYieldFarm<T>,
		rewards: Balance,
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

		pool.accumulated_rps = Self::get_accumulated_rps(pool.accumulated_rps, pool.total_valued_shares, rewards)?;
		pool.updated_at = period_now;

		let global_pool_balance =
			T::MultiCurrency::free_balance(reward_currency, &Self::pool_account_id(global_pool_id)?);

		ensure!(
			global_pool_balance >= rewards,
			Error::<T>::InsuffcientBalanceInGlobalPool
		);

		let global_pool_account = Self::pool_account_id(global_pool_id)?;
		let pool_account = Self::pool_account_id(pool.id)?;
		T::MultiCurrency::transfer(reward_currency, &global_pool_account, &pool_account, rewards)
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
		valued_shares.checked_mul(multiplier.into()).ok_or(Error::<T>::Overflow)
	}

	fn get_valued_shares(
		shares: Balance,
		amm: AccountIdOf<T>,
		reward_currency: T::CurrencyId,
	) -> Result<Balance, Error<T>> {
		let reward_currency_balance = T::MultiCurrency::free_balance(reward_currency, &amm);

		shares.checked_mul(reward_currency_balance).ok_or(Error::<T>::Overflow)
	}

	//TODO: add tests
	fn do_claim_rewards(
		who: AccountIdOf<T>,
		d: &mut Deposit<T>,
		pool: &LiquidityPoolYieldFarm<T>,
		now_period: PeriodOf<T>,
		reward_currency: T::CurrencyId,
	) -> Result<(Balance, Balance), DispatchError> {
		let periods = now_period.checked_sub(&d.entered_period).ok_or(Error::<T>::Overflow)?;
		let loyalty_multiplier = Self::get_loyalty_multiplier(periods, pool.loyalty_curve.clone())?;
		let (rewards, unclaimable_rewards) = Self::get_user_reward(
			d.accumulated_rps,
			d.valued_shares,
			d.accumulated_claimed_rewards,
			pool.accumulated_rps,
			loyalty_multiplier,
		)?;

		d.accumulated_claimed_rewards = d
			.accumulated_claimed_rewards
			.checked_add(rewards)
			.ok_or(Error::<T>::Overflow)?;

		let pool_account = Self::pool_account_id(pool.id)?;
		T::MultiCurrency::transfer(reward_currency, &pool_account, &who, rewards)?;

		Ok((rewards, unclaimable_rewards))
	}

	/// This function update pools if conditions are met
	fn maybe_update_pools(
		g_pool: &mut GlobalPool<T>,
		liq_pool: &mut LiquidityPoolYieldFarm<T>,
		now_period: PeriodOf<T>,
	) -> DispatchResult {
		if liq_pool.canceled {
			return Ok(());
		}

		if !liq_pool.total_shares.is_zero() && liq_pool.updated_at != now_period {
			if !g_pool.total_shares_z.is_zero() && g_pool.updated_at != now_period {
				let rewards = Self::get_reward_per_period(
					g_pool.yield_per_period.into(),
					g_pool.total_shares_z,
					g_pool.max_reward_per_period,
				)?;

				Self::update_global_pool(g_pool, now_period, rewards)?;
			}

			let rewards = Self::claim_from_global_pool(g_pool, liq_pool, liq_pool.stake_in_global_pool)?;
			Self::update_pool(liq_pool, rewards, now_period, g_pool.id, g_pool.reward_currency)?;
		}

		Ok(())
	}
}
