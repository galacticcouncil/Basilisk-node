#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

//TODO:
// * add assetId validation to weights manipulation by user - reason: change from (w,w) to ((assetId, w), (assetid, w))

use frame_support::sp_runtime::{
	app_crypto::sp_core::crypto::UncheckedFrom,
	traits::{Hash, Zero},
	DispatchError, RuntimeDebug,
};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, transactional};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
use primitives::traits::{AMMTransfer, AMM};
use primitives::{
	asset::AssetPair,
	fee::{self, WithFee},
	Amount, AssetId, Balance, MAX_IN_RATIO, MAX_OUT_RATIO,
};
use sp_std::{marker::PhantomData, vec, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;
use weights::WeightInfo;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[derive(Debug, Clone, PartialEq)]
pub enum MathError {
	Overflow,
	ZeroDuration,
}
use crate::MathError::{Overflow, ZeroDuration};

// TODO: move this function to the hydradx-math crate
// Linear interpolation
fn calculate_linear_weights<BlockNumber: AtLeast32BitUnsigned>(
	a_x: BlockNumber,
	b_x: BlockNumber,
	a_y: LBPWeight,
	b_y: LBPWeight,
	at: BlockNumber,
) -> Result<LBPWeight, MathError> {
	let d1 = b_x.checked_sub(&at).ok_or(Overflow)?;
	let d2 = at.checked_sub(&a_x).ok_or(Overflow)?;
	let dx = b_x.checked_sub(&a_x).ok_or(Overflow)?;

	let dx: u32 = dx.try_into().map_err(|_| Overflow)?;
	// if dx fits into u32, d1 and d2 fit into u128
	let d1: u128 = d1.try_into().map_err(|_| Overflow)?;
	let d2: u128 = d2.try_into().map_err(|_| Overflow)?;

	ensure!(dx != 0, ZeroDuration);

	let left = a_y.checked_mul(d1).ok_or(Overflow)?; // TODO: change to u256
	let right = b_y.checked_mul(d2).ok_or(Overflow)?; // TODO: change to u256
	let result = (left.checked_add(right).ok_or(Overflow)?)
		.checked_div(dx.into())
		.ok_or(Overflow)?;

	Ok(result)
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum CurveType {
	Constant,
	Linear,
}

impl Default for CurveType {
	fn default() -> Self {
		CurveType::Linear
	}
}

type PoolId<T> = <T as frame_system::Config>::AccountId;
type LBPWeight = u128;

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::AtLeast32BitUnsigned;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct Pool<BlockNumber: AtLeast32BitUnsigned + Copy> {
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub initial_weights: ((AssetId, LBPWeight), (AssetId, LBPWeight)),
	pub final_weights: ((AssetId, LBPWeight), (AssetId, LBPWeight)),
	pub last_weight_update: BlockNumber,
	pub last_weights: ((AssetId, LBPWeight), (AssetId, LBPWeight)),
	pub curve: CurveType,
	pub pausable: bool,
	pub paused: bool,
}

type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

pub trait LBPWeightCalculation<BlockNumber: AtLeast32BitUnsigned> {
	fn calculate_weight(
		curve_type: CurveType,
		a_x: BlockNumber,
		b_x: BlockNumber,
		a_y: LBPWeight,
		b_y: LBPWeight,
		at: BlockNumber,
	) -> Result<LBPWeight, MathError>;
}

pub struct LBPWeightFunction;
impl<BlockNumber: AtLeast32BitUnsigned> LBPWeightCalculation<BlockNumber> for LBPWeightFunction {
	fn calculate_weight(
		curve_type: CurveType,
		a_x: BlockNumber,
		b_x: BlockNumber,
		a_y: LBPWeight,
		b_y: LBPWeight,
		at: BlockNumber,
	) -> Result<LBPWeight, MathError> {
		match curve_type {
			CurveType::Linear => calculate_linear_weights(a_x, b_x, a_y, b_y, at),
			CurveType::Constant => Ok(a_y),
		}
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multi currency for transfer of currencies
		type MultiCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId = AssetId, Amount = Amount, Balance = Balance>
			+ MultiReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		/// Native Asset Id
		type NativeAssetId: Get<AssetId>;

		/// Function for calculation of LBP weights
		type LBPWeightFunction: LBPWeightCalculation<Self::BlockNumber>;

		/// Mapping of asset pairs to unique pool identities
		type AssetPairPoolId: AssetPairPoolIdFor<AssetId, PoolId<Self>>;

		#[pallet::constant]
		/// Storage fee
		type PoolDeposit: Get<BalanceOf<Self>>;

		/// Trading fee rate
		type ExchangeFee: Get<fee::Fee>;

		/// Weight information for the extrinsics.
		type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Create pool errors
		CannotCreatePoolWithSameAssets,
		CannotCreatePoolWithZeroLiquidity,

		/// Update pool errors
		NotOwner,
		SaleStarted,
		SaleNotEnded,
		InvalidData,
		CannotPauseEndedPool,
		CannotPausePausedPool,
		PoolIsNotPausable,
		CannotUnpauseEndedPool,
		PoolIsNotPaused,
		MaxSaleDurationExceeded,

		/// Add / Remove liquidity errors
		CannotAddZeroLiquidity,
		CannotRemoveZeroLiquidity,
		LiquidityOverflow,
		LiquidityUnderflow, // TODO: do we need it? Can we use LiquidityOverflow instead?

		/// Balance errors
		InsufficientAssetBalance,

		/// Pool existence errors
		TokenPoolNotFound,
		TokenPoolAlreadyExists,

		/// Calculation errors
		InvalidBlockNumber,
		CalculationError, // TODO: replace me with something more meaningful?

		BlockNumberInvalid,

		ZeroAmount,
		SaleIsNotRunning,

		// Trading limit errors
		MaxInRatioExceeded,
		MaxOutRatioExceeded,

		/// Invalid fee
		FeeAmountInvalid,

		// Asset limit exceeded
		AssetBalanceLimitExceeded,

		Overflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Create new LBP pool
		/// who, asset a, asset b, amount_a, amount_b
		CreatePool(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Update the LBP pool
		/// who, pool id
		UpdatePool(T::AccountId, PoolId<T>),

		/// Add liquidity to the pool
		/// who, asset_a, asset_b, amount_a, amount_b
		AddLiquidity(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Remove liquidity from the pool
		/// who, asset_a, asset_b, shares
		RemoveLiquidity(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Destroy LBP pool
		/// who, asset a, asset b
		PoolDestroyed(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Sell token
		/// who, asset in, asset out, amount, sale price
		SellExecuted(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Buy token
		/// who, asset out, asset in, amount, buy price
		BuyExecuted(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		Paused(T::AccountId),
		Unpaused(T::AccountId),
		WeightsUpdated(PoolId<T>, LBPWeight, LBPWeight),
	}

	#[pallet::storage]
	#[pallet::getter(fn pool_owner)]
	pub type PoolOwner<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_deposit)]
	pub type PoolDeposit<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, BalanceOf<T>, ValueQuery>;

	/// Asset pair for each pool. Assets are stored unordered.
	#[pallet::storage]
	#[pallet::getter(fn pool_assets)]
	pub type PoolAssets<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, (AssetId, AssetId), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_data)]
	pub type PoolData<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, Pool<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_balances)]
	pub type PoolBalances<T: Config> =
		StorageMap<_, Blake2_128Concat, PoolId<T>, (BalanceOf<T>, BalanceOf<T>), ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			asset_a: AssetId,
			amount_a: BalanceOf<T>,
			asset_b: AssetId,
			amount_b: BalanceOf<T>,
			pool_data: Pool<T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				!amount_a.is_zero() || !amount_b.is_zero(),
				Error::<T>::CannotCreatePoolWithZeroLiquidity
			);

			ensure!(asset_a != asset_b, Error::<T>::CannotCreatePoolWithSameAssets);

			let asset_pair = AssetPair {
				asset_in: asset_a,
				asset_out: asset_b,
			};

			ensure!(!Self::exists(asset_pair), Error::<T>::TokenPoolAlreadyExists);

			ensure!(
				T::MultiCurrency::free_balance(asset_a, &who) >= amount_a,
				Error::<T>::InsufficientAssetBalance
			);

			ensure!(
				T::MultiCurrency::free_balance(asset_b, &who) >= amount_b,
				Error::<T>::InsufficientAssetBalance
			);

			Self::validate_pool_data(&pool_data)?;

			let pool_id = Self::get_pair_id(asset_pair);

			let deposit = T::PoolDeposit::get();

			T::MultiCurrency::reserve(T::NativeAssetId::get(), &who, deposit)?;
			<PoolDeposit<T>>::insert(&pool_id, &deposit);

			<PoolOwner<T>>::insert(&pool_id, &who);
			<PoolAssets<T>>::insert(&pool_id, &(asset_a, asset_b));
			<PoolData<T>>::insert(&pool_id, &pool_data);

			T::MultiCurrency::transfer(asset_a, &who, &pool_id, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &who, &pool_id, amount_b)?;
			<PoolBalances<T>>::insert(&pool_id, &(amount_a, amount_b));

			Self::deposit_event(Event::CreatePool(who, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_pool_data())]
		#[transactional]
		pub fn update_pool_data(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			start: Option<T::BlockNumber>,
			end: Option<T::BlockNumber>,
			initial_weights: Option<((AssetId, LBPWeight), (AssetId, LBPWeight))>,
			final_weights: Option<((AssetId, LBPWeight), (AssetId, LBPWeight))>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(<PoolOwner<T>>::contains_key(&pool_id), Error::<T>::TokenPoolNotFound);

			Self::ensure_pool_ownership(&who, &pool_id)?;

			let mut pool_data = Self::pool_data(&pool_id);

			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(
				pool_data.start == Zero::zero() || now < pool_data.start,
				Error::<T>::SaleStarted
			);

			if let Some(new_start) = start {
				pool_data.start = new_start;
			}

			if let Some(new_end) = end {
				pool_data.end = new_end;
			}

			if let Some((w1, w2)) = initial_weights {
				pool_data.initial_weights = (w1, w2);
			}

			if let Some((w1, w2)) = final_weights {
				pool_data.final_weights = (w1, w2);
			}

			Self::validate_pool_data(&pool_data)?;

			<PoolData<T>>::insert(&pool_id, &pool_data);
			Self::deposit_event(Event::UpdatePool(who, pool_id));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::pause_pool())]
		pub fn pause_pool(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(<PoolOwner<T>>::contains_key(&pool_id), Error::<T>::TokenPoolNotFound);

			Self::ensure_pool_ownership(&who, &pool_id)?;

			let mut pool_data = Self::pool_data(&pool_id);

			ensure!(pool_data.pausable, Error::<T>::PoolIsNotPausable);

			ensure!(!pool_data.paused, Error::<T>::CannotPausePausedPool);

			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(pool_data.end > now, Error::<T>::CannotPauseEndedPool);

			pool_data.paused = true;
			<PoolData<T>>::insert(&pool_id, &pool_data);

			Self::deposit_event(Event::Paused(who));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::unpause_pool())]
		pub fn unpause_pool(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(<PoolOwner<T>>::contains_key(&pool_id), Error::<T>::TokenPoolNotFound);

			Self::ensure_pool_ownership(&who, &pool_id)?;

			let mut pool_data = Self::pool_data(&pool_id);

			ensure!(pool_data.paused, Error::<T>::PoolIsNotPaused);

			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(pool_data.end > now, Error::<T>::CannotUnpauseEndedPool);

			pool_data.paused = false;
			<PoolData<T>>::insert(&pool_id, &pool_data);

			Self::deposit_event(Event::Unpaused(who));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			amount_a: BalanceOf<T>,
			amount_b: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(<PoolOwner<T>>::contains_key(&pool_id), Error::<T>::TokenPoolNotFound);

			Self::ensure_pool_ownership(&who, &pool_id)?;

			ensure!(
				!amount_a.is_zero() || !amount_b.is_zero(),
				Error::<T>::CannotAddZeroLiquidity
			);

			let (mut reserve_a, mut reserve_b) = Self::pool_balances(&pool_id);
			let (asset_a, asset_b) = Self::pool_assets(&pool_id);

			let pool_data = Self::pool_data(&pool_id);

			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(
				pool_data.start == Zero::zero() || now < pool_data.start,
				Error::<T>::SaleStarted
			);

			if !amount_a.is_zero() {
				ensure!(
					T::MultiCurrency::free_balance(asset_a, &who) >= amount_a,
					Error::<T>::InsufficientAssetBalance
				);

				reserve_a = reserve_a.checked_add(amount_a).ok_or(Error::<T>::LiquidityOverflow)?;
			}

			if !amount_b.is_zero() {
				ensure!(
					T::MultiCurrency::free_balance(asset_b, &who) >= amount_b,
					Error::<T>::InsufficientAssetBalance
				);

				reserve_b = reserve_b.checked_add(amount_b).ok_or(Error::<T>::LiquidityOverflow)?;
			}

			T::MultiCurrency::transfer(asset_a, &who, &pool_id, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &who, &pool_id, amount_b)?;

			<PoolBalances<T>>::insert(&pool_id, (reserve_a, reserve_b));
			Self::deposit_event(Event::AddLiquidity(pool_id, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			amount_a: BalanceOf<T>,
			amount_b: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(<PoolOwner<T>>::contains_key(&pool_id), Error::<T>::TokenPoolNotFound);

			Self::ensure_pool_ownership(&who, &pool_id)?;

			ensure!(
				!amount_a.is_zero() || !amount_b.is_zero(),
				Error::<T>::CannotRemoveZeroLiquidity
			);

			let (mut reserve_a, mut reserve_b) = Self::pool_balances(&pool_id);
			let (asset_a, asset_b) = Self::pool_assets(&pool_id);

			let pool_data = Self::pool_data(&pool_id);

			ensure!(!Self::is_pool_running(&pool_data), Error::<T>::SaleNotEnded);

			if !amount_a.is_zero() {
				reserve_a = reserve_a.checked_sub(amount_a).ok_or(Error::<T>::LiquidityUnderflow)?;
			}

			if !amount_b.is_zero() {
				reserve_b = reserve_b.checked_sub(amount_b).ok_or(Error::<T>::LiquidityUnderflow)?;
			}

			T::MultiCurrency::transfer(asset_a, &pool_id, &who, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &pool_id, &who, amount_b)?;

			<PoolBalances<T>>::insert(&pool_id, (reserve_a, reserve_b));
			Self::deposit_event(Event::RemoveLiquidity(pool_id, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::destroy_pool())]
		#[transactional]
		pub fn destroy_pool(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(<PoolOwner<T>>::contains_key(&pool_id), Error::<T>::TokenPoolNotFound);

			Self::ensure_pool_ownership(&who, &pool_id)?;

			let pool_data = Self::pool_data(&pool_id);
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(
				pool_data.start.is_zero() || pool_data.end < now,
				Error::<T>::SaleNotEnded
			);

			let (amount_a, amount_b) = Self::pool_balances(&pool_id);
			let (asset_a, asset_b) = Self::pool_assets(&pool_id);

			T::MultiCurrency::transfer(asset_a, &pool_id, &who, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &pool_id, &who, amount_b)?;

			let deposit = Self::pool_deposit(&pool_id);
			T::MultiCurrency::unreserve(T::NativeAssetId::get(), &who, deposit);

			<PoolOwner<T>>::remove(&pool_id);
			<PoolDeposit<T>>::remove(&pool_id);
			<PoolAssets<T>>::remove(&pool_id);
			<PoolData<T>>::remove(&pool_id);
			<PoolBalances<T>>::remove(&pool_id);

			Self::deposit_event(Event::PoolDestroyed(pool_id, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::sell())]
		#[transactional]
		pub fn sell(
			origin: OriginFor<T>,
			asset_in: AssetId,
			asset_out: AssetId,
			amount: BalanceOf<T>,
			max_limit: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<Self as AMM<_, _, _, _>>::sell(&who, AssetPair { asset_in, asset_out }, amount, max_limit, false)?;

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		#[transactional]
		pub fn buy(
			origin: OriginFor<T>,
			asset_out: AssetId,
			asset_in: AssetId,
			amount: BalanceOf<T>,
			max_limit: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<Self as AMM<_, _, _, _>>::buy(&who, AssetPair { asset_in, asset_out }, amount, max_limit, false)?;

			Ok(().into())
		}
	}
}

/// Trade type used in validation to determine how to perform certain checks
#[derive(Debug, Clone, PartialEq, Eq)]
enum TradeType {
	Sell,
	Buy,
}

impl<T: Config> Pallet<T> {
	fn update_weights(
		pool_data: &mut Pool<T::BlockNumber>,
	) -> Result<((AssetId, LBPWeight), (AssetId, LBPWeight)), DispatchError> {
		let now = <frame_system::Pallet<T>>::block_number();

		if now != pool_data.last_weight_update {
			pool_data.last_weights.0 .1 = T::LBPWeightFunction::calculate_weight(
				pool_data.curve,
				pool_data.start,
				pool_data.end,
				pool_data.initial_weights.0 .1,
				pool_data.final_weights.0 .1,
				now,
			)
			.map_err(|_| Error::<T>::CalculationError)?;

			pool_data.last_weights.1 .1 = T::LBPWeightFunction::calculate_weight(
				pool_data.curve,
				pool_data.start,
				pool_data.end,
				pool_data.initial_weights.1 .1,
				pool_data.final_weights.1 .1,
				now,
			)
			.map_err(|_| Error::<T>::CalculationError)?;
			pool_data.last_weight_update = now;
		}

		Ok(pool_data.last_weights)
	}

	fn validate_pool_data(pool_data: &Pool<T::BlockNumber>) -> DispatchResult {
		let now = <frame_system::Pallet<T>>::block_number();
		ensure!(
			pool_data.start == Zero::zero() || now <= pool_data.start,
			Error::<T>::InvalidBlockNumber
		);
		ensure!(
			pool_data.end == Zero::zero() || pool_data.start < pool_data.end,
			Error::<T>::InvalidBlockNumber
		);
		// this restriction is based on the AtLeast32Bit trait of the frame_system::Balance type
		// and is expected by the calculate_linear_weights function
		ensure!(
			pool_data.end - pool_data.start <= u32::MAX.into(),
			Error::<T>::MaxSaleDurationExceeded
		);

		Ok(())
	}

	fn ensure_pool_ownership(who: &T::AccountId, pool_id: &PoolId<T>) -> DispatchResult {
		let pool_owner = Self::pool_owner(&pool_id);
		ensure!(who == &pool_owner, Error::<T>::NotOwner);

		Ok(())
	}

	/// return true if now is in interval <pool.start, pool.end> WARN: pool.paused DOESN'T MATTER
	fn is_pool_running(pool_data: &Pool<T::BlockNumber>) -> bool {
		let now = <frame_system::Pallet<T>>::block_number();
		pool_data.start <= now && now <= pool_data.end
	}

	/// return true if now is in interval <pool.start, pool.end> and POOL IS NOT PAUSED
	fn is_sale_running(pool_data: &Pool<T::BlockNumber>) -> bool {
		let now = <frame_system::Pallet<T>>::block_number();
		pool_data.start <= now && now <= pool_data.end && !pool_data.paused
	}

	fn update_weights_and_validate_trade(
		who: &T::AccountId,
		assets: AssetPair,
		amount: BalanceOf<T>,
		limit: BalanceOf<T>,
		trade_type: TradeType,
	) -> Result<AMMTransfer<T::AccountId, AssetPair, Balance>, DispatchError> {
		ensure!(!amount.is_zero(), Error::<T>::ZeroAmount);

		ensure!(Self::exists(assets), Error::<T>::TokenPoolNotFound);

		ensure!(
			T::MultiCurrency::free_balance(assets.asset_in, &who) >= amount,
			Error::<T>::InsufficientAssetBalance
		);

		let pool_id = Self::get_pair_id(assets);
		let mut pool_data = Self::pool_data(&pool_id);
		ensure!(Self::is_sale_running(&pool_data), Error::<T>::SaleIsNotRunning);

		let asset_in_weight: LBPWeight;
		let asset_out_weight: LBPWeight;
		// update weight or reuse last if update is not necessary
		if pool_data.last_weight_update == <frame_system::Pallet<T>>::block_number() {
			let w = Self::get_weights_in_order(&pool_data, assets.asset_in);

			asset_in_weight = w.0;
			asset_out_weight = w.1;
		} else {
			match Self::update_weights(&mut pool_data) {
				Ok(weights) => {
					if weights.0 .0 == assets.asset_in {
						asset_in_weight = weights.0 .1;
						asset_out_weight = weights.1 .1;
					} else {
						asset_in_weight = weights.1 .1;
						asset_out_weight = weights.0 .1;
					}
				}
				Err(_) => return Err(<Error<T>>::Overflow.into()),
			};

			<PoolData<T>>::insert(&pool_id, &pool_data);
		}

		let asset_in_reserve = T::MultiCurrency::free_balance(assets.asset_in, &pool_id);
		let asset_out_reserve = T::MultiCurrency::free_balance(assets.asset_out, &pool_id);
		if trade_type == TradeType::Sell {
			ensure!(
				amount <= asset_in_reserve / MAX_IN_RATIO,
				Error::<T>::MaxInRatioExceeded
			);
		} else {
			ensure!(
				amount <= asset_out_reserve / MAX_OUT_RATIO,
				Error::<T>::MaxOutRatioExceeded
			);
		}

		let transfer_fee = Self::calculate_fees(amount)?;
		let (amount_in, amount_out) = if trade_type == TradeType::Sell {
			let token_amount_out = hydra_dx_math::lbp::calculate_out_given_in(
				asset_in_reserve,
				asset_out_reserve,
				asset_in_weight,
				asset_out_weight,
				amount.checked_sub(transfer_fee).ok_or(Error::<T>::Overflow)?,
			)
			.map_err(|_| Error::<T>::Overflow)?;

			ensure!(limit <= token_amount_out, Error::<T>::AssetBalanceLimitExceeded);
			(amount, token_amount_out)
		} else {
			let token_amount_in = hydra_dx_math::lbp::calculate_in_given_out(
				asset_in_reserve,
				asset_out_reserve,
				asset_in_weight,
				asset_out_weight,
				amount.checked_add(transfer_fee).ok_or(Error::<T>::Overflow)?,
			)
			.map_err(|_| Error::<T>::Overflow)?;

			ensure!(limit >= token_amount_in, Error::<T>::AssetBalanceLimitExceeded);

			(token_amount_in, amount)
		};

		ensure!(
			T::MultiCurrency::free_balance(assets.asset_out, &pool_id) >= amount_out,
			Error::<T>::InsufficientAssetBalance
		);

		Ok(AMMTransfer {
			origin: who.clone(),
			assets,
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
		})
	}

	#[transactional]
	fn execute_trade(transfer: &AMMTransfer<T::AccountId, AssetPair, Balance>) -> DispatchResult {
		let pool_id = Self::get_pair_id(transfer.assets);

		T::MultiCurrency::transfer(transfer.assets.asset_in, &transfer.origin, &pool_id, transfer.amount)?;
		T::MultiCurrency::transfer(
			transfer.assets.asset_out,
			&pool_id,
			&transfer.origin,
			transfer.amount_out,
		)?;

		Ok(())
	}

	fn calculate_fees(amount: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
		Ok(amount
			.just_fee(T::ExchangeFee::get())
			.ok_or::<Error<T>>(Error::<T>::FeeAmountInvalid)?)
	}

	/// WARN: unsafe function - make sure asset_a is in pool. This function return weigth in order based on asset_a id
	fn get_weights_in_order(pool_data: &Pool<T::BlockNumber>, asset_a: AssetId) -> (LBPWeight, LBPWeight) {
		if pool_data.last_weights.0 .0 == asset_a {
			(pool_data.last_weights.0 .1, pool_data.last_weights.1 .1)
		} else {
			(pool_data.last_weights.1 .1, pool_data.last_weights.0 .1)
		}
	}
}

pub trait AssetPairPoolIdFor<AssetId: Sized, PoolId: Sized> {
	fn from_assets(asset_a: AssetId, asset_b: AssetId) -> PoolId;
}

pub struct AssetPairPoolId<T: Config>(PhantomData<T>);

impl<T: Config> AssetPairPoolIdFor<AssetId, PoolId<T>> for AssetPairPoolId<T>
where
	PoolId<T>: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn from_assets(asset_a: AssetId, asset_b: AssetId) -> PoolId<T> {
		let mut buf: Vec<u8> = b"lbp".to_vec();

		if asset_a < asset_b {
			buf.extend_from_slice(&asset_a.to_le_bytes());
			buf.extend_from_slice(&asset_b.to_le_bytes());
		} else {
			buf.extend_from_slice(&asset_b.to_le_bytes());
			buf.extend_from_slice(&asset_a.to_le_bytes());
		}
		PoolId::<T>::unchecked_from(T::Hashing::hash(&buf[..]))
	}
}

impl<T: Config> AMM<T::AccountId, AssetId, AssetPair, BalanceOf<T>> for Pallet<T> {
	fn exists(assets: AssetPair) -> bool {
		let pair_account = T::AssetPairPoolId::from_assets(assets.asset_in, assets.asset_out);
		<PoolAssets<T>>::contains_key(&pair_account)
	}

	fn get_pair_id(assets: AssetPair) -> T::AccountId {
		T::AssetPairPoolId::from_assets(assets.asset_in, assets.asset_out)
	}

	fn get_pool_assets(pool_account_id: &T::AccountId) -> Option<Vec<AssetId>> {
		match <PoolAssets<T>>::contains_key(pool_account_id) {
			true => {
				let assets = Self::pool_assets(pool_account_id);
				Some(vec![assets.0, assets.1])
			}
			false => None,
		}
	}

	/// Calculate spot price for given assets and amount and update pool weights if necessary
	///
	/// Provided assets must exist in the pool. Panic if an asset does not exist in the pool.
	///
	/// Return 0 if calculation overflows or weights calculation overflows.
	fn get_spot_price_unchecked(asset_a: AssetId, asset_b: AssetId, amount: BalanceOf<T>) -> BalanceOf<T> {
		let pool_id = Self::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let asset_a_reserve = T::MultiCurrency::free_balance(asset_a, &pool_id);
		let asset_b_reserve = T::MultiCurrency::free_balance(asset_b, &pool_id);

		// Note : unchecked function; assumes that existence was checked prior to this call
		let mut pool_data = Self::pool_data(&pool_id);

		let weight_a: LBPWeight;
		let weight_b: LBPWeight;
		// update weight or reuse last if update is not necessary
		if pool_data.last_weight_update == <frame_system::Pallet<T>>::block_number() {
			let w = Self::get_weights_in_order(&pool_data, asset_a);

			weight_a = w.0;
			weight_b = w.1;
		} else {
			match Self::update_weights(&mut pool_data) {
				Ok(weights) => {
					if weights.0 .0 == asset_a {
						weight_a = weights.0 .1;
						weight_b = weights.1 .1;
					} else {
						weight_a = weights.1 .1;
						weight_b = weights.0 .1;
					}

					<PoolData<T>>::insert(&pool_id, &pool_data);
				}
				Err(_) => {
					return BalanceOf::<T>::zero();
				}
			};
		}
		hydra_dx_math::lbp::calculate_spot_price(asset_a_reserve, asset_b_reserve, weight_a, weight_b, amount)
			.unwrap_or_else(|_| BalanceOf::<T>::zero())
	}

	/// Validate sell trade and update pool weights
	fn validate_sell(
		who: &T::AccountId,
		assets: AssetPair,
		amount: BalanceOf<T>,
		min_bought: BalanceOf<T>,
		_discount: bool,
	) -> Result<AMMTransfer<T::AccountId, AssetPair, BalanceOf<T>>, sp_runtime::DispatchError> {
		Self::update_weights_and_validate_trade(who, assets, amount, min_bought, TradeType::Sell)
	}

	fn execute_sell(transfer: &AMMTransfer<T::AccountId, AssetPair, Balance>) -> DispatchResult {
		Self::execute_trade(transfer)?;

		Self::deposit_event(Event::<T>::SellExecuted(
			transfer.origin.clone(),
			transfer.assets.asset_in,
			transfer.assets.asset_out,
			transfer.amount,
			transfer.amount_out,
		));

		Ok(().into())
	}

	/// Validate buy trade and update pool weights
	fn validate_buy(
		who: &T::AccountId,
		assets: AssetPair,
		amount: BalanceOf<T>,
		max_limit: BalanceOf<T>,
		_discount: bool,
	) -> Result<AMMTransfer<T::AccountId, AssetPair, BalanceOf<T>>, frame_support::sp_runtime::DispatchError> {
		Self::update_weights_and_validate_trade(who, assets, amount, max_limit, TradeType::Buy)
	}

	fn execute_buy(transfer: &AMMTransfer<T::AccountId, AssetPair, BalanceOf<T>>) -> DispatchResult {
		Self::execute_trade(transfer)?;

		Self::deposit_event(Event::<T>::BuyExecuted(
			transfer.origin.clone(),
			transfer.assets.asset_out,
			transfer.assets.asset_in,
			transfer.amount,
			transfer.amount_out,
		));
		Ok(().into())
	}
}
