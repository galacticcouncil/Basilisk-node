#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]

use codec::{Decode, Encode};
use frame_support::sp_runtime::{
	app_crypto::sp_core::crypto::UncheckedFrom,
	traits::{AtLeast32BitUnsigned, Hash, Zero},
	DispatchError, RuntimeDebug,
};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{EnsureOrigin, Get},
	transactional,
};
use frame_system::ensure_signed;
use hydra_dx_math::lbp::LBPWeight;
use orml_traits::{MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
use primitives::traits::{AMMTransfer, AMM};
use primitives::{
	asset::AssetPair,
	fee::{self, WithFee},
	Amount, AssetId, Balance, MAX_IN_RATIO, MAX_OUT_RATIO,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::{fmt::Debug, marker::PhantomData, vec, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;
pub mod weights;
use weights::WeightInfo;
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
type PoolId<T> = <T as frame_system::Config>::AccountId;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum WeightCurveType {
	Constant,
	Linear,
}

impl Default for WeightCurveType {
	fn default() -> Self {
		WeightCurveType::Linear
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct Pool<AccountId, BlockNumber: AtLeast32BitUnsigned + Copy> {
	pub owner: AccountId,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub assets: (AssetId, AssetId),
	pub initial_weights: (LBPWeight, LBPWeight),
	pub final_weights: (LBPWeight, LBPWeight),
	pub last_weight_update: BlockNumber,
	pub last_weights: (LBPWeight, LBPWeight),
	pub weight_curve: WeightCurveType,
	pub pausable: bool,
	pub paused: bool,
}

impl<AccountId, BlockNumber: AtLeast32BitUnsigned + Copy> Pool<AccountId, BlockNumber>
where
	BlockNumber: AtLeast32BitUnsigned + Copy,
	Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq,
{
	fn new(
		pool_owner: AccountId,
		asset_a: LBPAssetInfo<Balance>,
		asset_b: LBPAssetInfo<Balance>,
		sale_duration: (BlockNumber, BlockNumber),
		weight_curve: WeightCurveType,
		pausable: bool,
	) -> Self {
		let ordered_assets = if asset_a.id < asset_b.id {
			(asset_a, asset_b)
		} else {
			(asset_b, asset_a)
		};

		Pool {
			owner: pool_owner,
			start: sale_duration.0,
			end: sale_duration.1,
			assets: (ordered_assets.0.id, ordered_assets.1.id),
			initial_weights: (ordered_assets.0.initial_weight, ordered_assets.1.initial_weight),
			final_weights: (ordered_assets.0.final_weight, ordered_assets.1.final_weight),
			last_weight_update: Zero::zero(),
			last_weights: (ordered_assets.0.initial_weight, ordered_assets.1.initial_weight),
			weight_curve,
			pausable,
			paused: true,
		}
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct LBPAssetInfo<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> {
	pub id: AssetId,
	pub amount: Balance,
	pub initial_weight: LBPWeight,
	pub final_weight: LBPWeight,
}

pub trait LBPWeightCalculation<BlockNumber: AtLeast32BitUnsigned> {
	fn calculate_weight(
		weight_curve: WeightCurveType,
		start: BlockNumber,
		end: BlockNumber,
		initial_weight: LBPWeight,
		final_weight: LBPWeight,
		at: BlockNumber,
	) -> Result<LBPWeight, ()>;
}

pub struct LBPWeightFunction;
impl<BlockNumber: AtLeast32BitUnsigned> LBPWeightCalculation<BlockNumber> for LBPWeightFunction {
	fn calculate_weight(
		weight_curve: WeightCurveType,
		start: BlockNumber,
		end: BlockNumber,
		initial_weight: LBPWeight,
		final_weight: LBPWeight,
		at: BlockNumber,
	) -> Result<LBPWeight, ()> {
		match weight_curve {
			WeightCurveType::Linear => {
				hydra_dx_math::lbp::calculate_linear_weights(start, end, initial_weight, final_weight, at)
					.map_err(|_| ())
			}
			WeightCurveType::Constant => Ok(initial_weight),
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

		/// The origin which can create a new pool
		type CreatePoolOrigin: EnsureOrigin<Self::Origin>;

		/// Function for calculation of LBP weights
		type LBPWeightFunction: LBPWeightCalculation<Self::BlockNumber>;

		/// Mapping of asset pairs to unique pool identities
		type AssetPairPoolId: AssetPairPoolIdFor<AssetId, PoolId<Self>>;

		#[pallet::constant]
		/// Storage fee
		type PoolDeposit: Get<BalanceOf<Self>>;

		/// Trading fee rate
		type ExchangeFee: Get<fee::Fee>;

		/// Weight information for the extrinsics
		type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Pool assets can not be the same
		CannotCreatePoolWithSameAssets,
		/// Initial liquidity should be non-zero
		CannotCreatePoolWithZeroLiquidity,
		/// Account is not a pool owner
		NotOwner,
		/// Sale already started
		SaleStarted,
		/// Sale is still in progress
		SaleNotEnded,
		/// Sale is not running
		SaleIsNotRunning,
		/// Sale already ended
		CannotPauseEndedPool,
		/// Sale already ended
		CannotUnpauseEndedPool,
		/// Sale is already paused
		CannotPausePausedPool,
		/// Pool cannot be paused
		PoolIsNotPausable,
		/// Pool is not paused
		PoolIsNotPaused,
		/// Sale duration is too long
		MaxSaleDurationExceeded,
		/// Liquidity being added should not be zero
		CannotAddZeroLiquidity,
		/// Can not remove zero liquidity
		CannotRemoveZeroLiquidity,
		/// Overflow after adding liquidity
		LiquidityOverflow,
		/// Asset balance too low
		InsufficientAssetBalance,
		/// Pool does not exist
		PoolNotFound,
		/// Pool has been already created
		PoolAlreadyExists,
		/// Pool does not contain the asset
		InvalidAsset,
		/// Invalid block number
		InvalidBlockNumber,
		/// Calculation error
		WeightCalculationError,
		/// Invalid block number
		BlockNumberInvalid,
		/// Can not perform a trade with zero amount
		ZeroAmount,
		/// Trade amount is too high
		MaxInRatioExceeded,
		/// Trade amount is too high
		MaxOutRatioExceeded,
		/// Invalid fee amount
		FeeAmountInvalid,
		/// Trading limit reached
		AssetBalanceLimitExceeded,
		/// An unexpected integer overflow occurred
		Overflow,
		/// Nothing to update
		NothingToUpdate,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Pool was created by the `CreatePool` origin. [who, pool_id, asset_a, asset_b, amount_a, amount_b]
		PoolCreated(T::AccountId, PoolId<T>, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Pool data were updated. [who, pool_id]
		PoolUpdated(T::AccountId, PoolId<T>),

		/// New liquidity was provided to the pool. [who, asset_a, asset_b, amount_a, amount_b]
		LiquidityAdded(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Liquidity was removed from the pool. [who, asset_a, asset_b, amount_a, amount_b]
		LiquidityRemoved(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Pool was destroyed. [who, asset_a, asset_b, balance_a, balance_b]
		PoolDestroyed(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Sale executed. [who, asset_in, asset_out, amount, sale_price]
		SellExecuted(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Purchase executed. [who, asset_out, asset_in, amount, buy_price]
		BuyExecuted(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Pool was paused. [who, pool_id]
		Paused(T::AccountId, PoolId<T>),

		/// Pool was unpaused. [who, pool_id]
		Unpaused(T::AccountId, PoolId<T>),
	}

	/// Paid deposit. Returned when a pool is destroyed and removed from the storage.
	#[pallet::storage]
	#[pallet::getter(fn pool_deposit)]
	pub type PoolDeposit<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, BalanceOf<T>, ValueQuery>;

	/// Details of a pool.
	#[pallet::storage]
	#[pallet::getter(fn pool_data)]
	pub type PoolData<T: Config> =
		StorageMap<_, Blake2_128Concat, PoolId<T>, Pool<T::AccountId, T::BlockNumber>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			pool_owner: T::AccountId,
			asset_a: LBPAssetInfo<BalanceOf<T>>,
			asset_b: LBPAssetInfo<BalanceOf<T>>,
			sale_duration: (T::BlockNumber, T::BlockNumber),
			weight_curve: WeightCurveType,
			pausable: bool,
		) -> DispatchResultWithPostInfo {
			T::CreatePoolOrigin::ensure_origin(origin)?;

			ensure!(
				!asset_a.amount.is_zero() && !asset_b.amount.is_zero(),
				Error::<T>::CannotCreatePoolWithZeroLiquidity
			);

			ensure!(asset_a.id != asset_b.id, Error::<T>::CannotCreatePoolWithSameAssets);

			let asset_pair = AssetPair {
				asset_in: asset_a.id,
				asset_out: asset_b.id,
			};

			ensure!(!Self::exists(asset_pair), Error::<T>::PoolAlreadyExists);

			ensure!(
				T::MultiCurrency::free_balance(asset_a.id, &pool_owner) >= asset_a.amount,
				Error::<T>::InsufficientAssetBalance
			);

			ensure!(
				T::MultiCurrency::free_balance(asset_b.id, &pool_owner) >= asset_b.amount,
				Error::<T>::InsufficientAssetBalance
			);

			let pool_data = Pool::new(
				pool_owner.clone(),
				asset_a,
				asset_b,
				sale_duration,
				weight_curve,
				pausable,
			);
			Self::validate_pool_data(&pool_data)?;

			let deposit = T::PoolDeposit::get();

			T::MultiCurrency::reserve(T::NativeAssetId::get(), &pool_owner, deposit)?;
			let pool_id = Self::get_pair_id(asset_pair);
			<PoolDeposit<T>>::insert(&pool_id, &deposit);

			<PoolData<T>>::insert(&pool_id, &pool_data);

			T::MultiCurrency::transfer(asset_a.id, &pool_owner, &pool_id, asset_a.amount)?;
			T::MultiCurrency::transfer(asset_b.id, &pool_owner, &pool_id, asset_b.amount)?;

			Self::deposit_event(Event::PoolCreated(
				pool_owner,
				pool_id,
				asset_a.id,
				asset_b.id,
				asset_a.amount,
				asset_b.amount,
			));

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

			<PoolData<T>>::try_mutate_exists(pool_id.clone(), |maybe_pool| -> DispatchResultWithPostInfo {
				// check existence of the pool
				let mut pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound)?;

				ensure!(
					start.is_some() || end.is_some() || initial_weights.is_some() || final_weights.is_some(),
					Error::<T>::NothingToUpdate
				);

				Self::ensure_pool_ownership(&who, &pool_id)?;

				let now = <frame_system::Pallet<T>>::block_number();

				ensure!(pool.start == Zero::zero() || now < pool.start, Error::<T>::SaleStarted);

				pool.start = start.unwrap_or(pool.start);
				pool.end = end.unwrap_or(pool.end);
				if let Some(initial_weights) = initial_weights {
					pool.initial_weights = Self::get_weights_in_order(pool, initial_weights)?;
				}
				if let Some(final_weights) = final_weights {
					pool.final_weights = Self::get_weights_in_order(pool, final_weights)?;
				}

				Self::validate_pool_data(&pool)?;

				Self::deposit_event(Event::PoolUpdated(who, pool_id));
				Ok(().into())
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::pause_pool())]
		pub fn pause_pool(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<PoolData<T>>::try_mutate_exists(pool_id.clone(), |maybe_pool| -> DispatchResultWithPostInfo {
				// check existence of the pool
				let mut pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound)?;

				Self::ensure_pool_ownership(&who, &pool_id)?;

				ensure!(pool.pausable, Error::<T>::PoolIsNotPausable);
				ensure!(!pool.paused, Error::<T>::CannotPausePausedPool);

				let now = <frame_system::Pallet<T>>::block_number();
				ensure!(pool.end > now, Error::<T>::CannotPauseEndedPool);

				pool.paused = true;

				Self::deposit_event(Event::Paused(who, pool_id));
				Ok(().into())
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::unpause_pool())]
		pub fn unpause_pool(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<PoolData<T>>::try_mutate_exists(pool_id.clone(), |maybe_pool| -> DispatchResultWithPostInfo {
				// check existence of the pool
				let mut pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound)?;

				Self::ensure_pool_ownership(&who, &pool_id)?;

				ensure!(pool.paused, Error::<T>::PoolIsNotPaused);

				let now = <frame_system::Pallet<T>>::block_number();
				ensure!(pool.end > now, Error::<T>::CannotUnpauseEndedPool);

				pool.paused = false;

				Self::deposit_event(Event::Unpaused(who, pool_id));
				Ok(().into())
			})
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			amount_a: (AssetId, BalanceOf<T>),
			amount_b: (AssetId, BalanceOf<T>),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let (asset_a, asset_b) = (amount_a.0, amount_b.0);
			let (amount_a, amount_b) = (amount_a.1, amount_b.1);

			let asset_pair_id = T::AssetPairPoolId::from_assets(asset_a, asset_b);
			ensure!(pool_id == asset_pair_id, Error::<T>::InvalidAsset);

			ensure!(
				!amount_a.is_zero() || !amount_b.is_zero(),
				Error::<T>::CannotAddZeroLiquidity
			);

			let pool_data = <PoolData<T>>::try_get(&pool_id).map_err(|_| Error::<T>::PoolNotFound)?;

			ensure!(who == pool_data.owner, Error::<T>::NotOwner);

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

				T::MultiCurrency::free_balance(asset_a, &pool_id)
					.checked_add(amount_a)
					.ok_or(Error::<T>::LiquidityOverflow)?;
			}

			if !amount_b.is_zero() {
				let reserve_b = T::MultiCurrency::free_balance(asset_b, &who);
				ensure!(reserve_b >= amount_b, Error::<T>::InsufficientAssetBalance);

				T::MultiCurrency::free_balance(asset_b, &pool_id)
					.checked_add(amount_b)
					.ok_or(Error::<T>::LiquidityOverflow)?;
			}

			T::MultiCurrency::transfer(asset_a, &who, &pool_id, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &who, &pool_id, amount_b)?;

			Self::deposit_event(Event::LiquidityAdded(pool_id, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			amount_a: (AssetId, BalanceOf<T>),
			amount_b: (AssetId, BalanceOf<T>),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let (asset_a, asset_b) = (amount_a.0, amount_b.0);
			let (amount_a, amount_b) = (amount_a.1, amount_b.1);

			let asset_pair_id = T::AssetPairPoolId::from_assets(asset_a, asset_b);
			ensure!(pool_id == asset_pair_id, Error::<T>::InvalidAsset);

			ensure!(
				!amount_a.is_zero() || !amount_b.is_zero(),
				Error::<T>::CannotRemoveZeroLiquidity
			);

			let pool_data = <PoolData<T>>::try_get(&pool_id).map_err(|_| Error::<T>::PoolNotFound)?;

			ensure!(who == pool_data.owner, Error::<T>::NotOwner);

			ensure!(!Self::is_pool_running(&pool_data), Error::<T>::SaleNotEnded);

			if !amount_a.is_zero() {
				ensure!(
					T::MultiCurrency::free_balance(asset_a, &pool_id) >= amount_a,
					Error::<T>::InsufficientAssetBalance
				);

				T::MultiCurrency::free_balance(asset_a, &who)
					.checked_add(amount_a)
					.ok_or(Error::<T>::Overflow)?;
			}

			if !amount_b.is_zero() {
				ensure!(
					T::MultiCurrency::free_balance(asset_b, &pool_id) >= amount_b,
					Error::<T>::InsufficientAssetBalance
				);

				T::MultiCurrency::free_balance(asset_b, &who)
					.checked_add(amount_b)
					.ok_or(Error::<T>::Overflow)?;
			}

			T::MultiCurrency::transfer(asset_a, &pool_id, &who, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &pool_id, &who, amount_b)?;

			Self::deposit_event(Event::LiquidityRemoved(pool_id, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::destroy_pool())]
		#[transactional]
		pub fn destroy_pool(origin: OriginFor<T>, pool_id: PoolId<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let pool_data = <PoolData<T>>::try_get(&pool_id).map_err(|_| Error::<T>::PoolNotFound)?;

			ensure!(who == pool_data.owner, Error::<T>::NotOwner);

			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(
				pool_data.start.is_zero() || pool_data.end < now,
				Error::<T>::SaleNotEnded
			);

			let (asset_a, asset_b) = pool_data.assets;

			let amount_a = T::MultiCurrency::free_balance(asset_a, &pool_id);
			let amount_b = T::MultiCurrency::free_balance(asset_b, &pool_id);

			T::MultiCurrency::transfer(asset_a, &pool_id, &who, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &pool_id, &who, amount_b)?;

			let deposit = Self::pool_deposit(&pool_id);
			T::MultiCurrency::unreserve(T::NativeAssetId::get(), &who, deposit);

			<PoolDeposit<T>>::remove(&pool_id);
			<PoolData<T>>::remove(&pool_id);

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
#[derive(RuntimeDebug, Clone, PartialEq, Eq)]
enum TradeType {
	Sell,
	Buy,
}

impl<T: Config> Pallet<T> {
	fn update_weights(
		pool_id: &PoolId<T>,
		pool_data: &mut Pool<T::AccountId, T::BlockNumber>,
	) -> Result<(LBPWeight, LBPWeight), DispatchError> {
		let now = <frame_system::Pallet<T>>::block_number();

		if now != pool_data.last_weight_update {
			pool_data.last_weights.0 = T::LBPWeightFunction::calculate_weight(
				pool_data.weight_curve,
				pool_data.start,
				pool_data.end,
				pool_data.initial_weights.0,
				pool_data.final_weights.0,
				now,
			)
			.map_err(|_| Error::<T>::WeightCalculationError)?;

			pool_data.last_weights.1 = T::LBPWeightFunction::calculate_weight(
				pool_data.weight_curve,
				pool_data.start,
				pool_data.end,
				pool_data.initial_weights.1,
				pool_data.final_weights.1,
				now,
			)
			.map_err(|_| Error::<T>::WeightCalculationError)?;
			pool_data.last_weight_update = now;

			let pool_data = &*pool_data;
			<PoolData<T>>::insert(&pool_id, &pool_data);
		}

		Ok(pool_data.last_weights)
	}

	fn validate_pool_data(pool_data: &Pool<T::AccountId, T::BlockNumber>) -> DispatchResult {
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
		let pool_owner = Self::pool_data(&pool_id).owner;
		ensure!(who == &pool_owner, Error::<T>::NotOwner);

		Ok(())
	}

	/// return true if now is in interval <pool.start, pool.end> WARN: pool.paused DOESN'T MATTER
	fn is_pool_running(pool_data: &Pool<T::AccountId, T::BlockNumber>) -> bool {
		let now = <frame_system::Pallet<T>>::block_number();
		pool_data.start <= now && now <= pool_data.end
	}

	/// return true if now is in interval <pool.start, pool.end> and POOL IS NOT PAUSED
	fn is_sale_running(pool_data: &Pool<T::AccountId, T::BlockNumber>) -> bool {
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

		// check existence of the pool
		ensure!(Self::exists(assets), Error::<T>::PoolNotFound);

		ensure!(
			T::MultiCurrency::free_balance(assets.asset_in, &who) >= amount,
			Error::<T>::InsufficientAssetBalance
		);

		let pool_id = Self::get_pair_id(assets);
		let mut pool_data = Self::pool_data(&pool_id);
		ensure!(Self::is_sale_running(&pool_data), Error::<T>::SaleIsNotRunning);

		// update weight or reuse last if update is not necessary
		let (weight_in, weight_out) = if pool_data.last_weight_update == <frame_system::Pallet<T>>::block_number() {
			let last_weights = Self::get_last_weights_in_order(&pool_data, assets.asset_in);
			(last_weights.0, last_weights.1)
		} else {
			match Self::update_weights(&pool_id, &mut pool_data) {
				Ok(weights) => {
					if pool_data.assets.0 == assets.asset_in {
						(weights.0, weights.1)
					} else {
						(weights.1, weights.0)
					}
				}
				Err(_) => return Err(<Error<T>>::Overflow.into()),
			}
		};

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
				weight_in,
				weight_out,
				amount.checked_sub(transfer_fee).ok_or(Error::<T>::Overflow)?,
			)
			.map_err(|_| Error::<T>::Overflow)?;

			ensure!(limit <= token_amount_out, Error::<T>::AssetBalanceLimitExceeded);
			(amount, token_amount_out)
		} else {
			let token_amount_in = hydra_dx_math::lbp::calculate_in_given_out(
				asset_in_reserve,
				asset_out_reserve,
				weight_in,
				weight_out,
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

	/// WARN: unsafe function - make sure asset_a is in pool. This function return weight in order based on asset_a id
	fn get_last_weights_in_order(
		pool_data: &Pool<T::AccountId, T::BlockNumber>,
		asset_a: AssetId,
	) -> (LBPWeight, LBPWeight) {
		if pool_data.assets.0 == asset_a {
			(pool_data.last_weights.0, pool_data.last_weights.1)
		} else {
			(pool_data.last_weights.1, pool_data.last_weights.0)
		}
	}

	fn get_weights_in_order(
		pool_data: &Pool<T::AccountId, T::BlockNumber>,
		weight_data: ((AssetId, LBPWeight), (AssetId, LBPWeight)),
	) -> Result<(LBPWeight, LBPWeight), DispatchError> {
		if pool_data.assets == (weight_data.0 .0, weight_data.1 .0) {
			Ok((weight_data.0 .1, weight_data.1 .1))
		} else if pool_data.assets == (weight_data.1 .0, weight_data.0 .0) {
			Ok((weight_data.1 .1, weight_data.0 .1))
		} else {
			Err(Error::<T>::InvalidAsset.into())
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
		<PoolData<T>>::contains_key(&pair_account)
	}

	fn get_pair_id(assets: AssetPair) -> T::AccountId {
		T::AssetPairPoolId::from_assets(assets.asset_in, assets.asset_out)
	}

	fn get_pool_assets(pool_account_id: &T::AccountId) -> Option<Vec<AssetId>> {
		let maybe_pool = <PoolData<T>>::try_get(pool_account_id);
		if let Ok(pool_data) = maybe_pool {
			Some(vec![pool_data.assets.0, pool_data.assets.1])
		} else {
			None
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

		let maybe_pool_data = <PoolData<T>>::try_get(&pool_id);
		if maybe_pool_data.is_err() {
			return BalanceOf::<T>::zero()
		}
		let mut pool_data = maybe_pool_data.unwrap();

		// update weight or reuse last if update is not necessary
		let (weight_in, weight_out) = if pool_data.last_weight_update == <frame_system::Pallet<T>>::block_number() {
			let last_weights = Self::get_last_weights_in_order(&pool_data, asset_a);
			(last_weights.0, last_weights.1)
		} else {
			match Self::update_weights(&pool_id, &mut pool_data) {
				Ok(weights) => {
					if pool_data.assets.0 == asset_a {
						(weights.0, weights.1)
					} else {
						(weights.1, weights.0)
					}
				}
				Err(_) => {
					return BalanceOf::<T>::zero();
				}
			}
		};

		hydra_dx_math::lbp::calculate_spot_price(asset_a_reserve, asset_b_reserve, weight_in, weight_out, amount)
			.unwrap_or_else(|_| BalanceOf::<T>::zero())
	}

	/// Validate sell trade and update pool weights
	fn validate_sell(
		who: &T::AccountId,
		assets: AssetPair,
		amount: BalanceOf<T>,
		min_bought: BalanceOf<T>,
		_discount: bool,
	) -> Result<AMMTransfer<T::AccountId, AssetPair, BalanceOf<T>>, DispatchError> {
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

		Ok(())
	}

	/// Validate buy trade and update pool weights
	fn validate_buy(
		who: &T::AccountId,
		assets: AssetPair,
		amount: BalanceOf<T>,
		max_limit: BalanceOf<T>,
		_discount: bool,
	) -> Result<AMMTransfer<T::AccountId, AssetPair, BalanceOf<T>>, DispatchError> {
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
		Ok(())
	}
}
