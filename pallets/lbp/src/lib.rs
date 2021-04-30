#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::sp_runtime::{
	app_crypto::sp_core::crypto::UncheckedFrom,
	traits::{Hash, Zero},
};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, transactional};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
use primitives::{asset::AssetPair, Amount, AssetId, Balance, CORE_ASSET_ID};
use sp_runtime::RuntimeDebug;
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
type Weight = Balance;
// type BalanceInfo = (Balance, AssetId);
// type AssetParams = (AssetId, Weight, Balance);
// type AssetWeights = (Weight, Weight);
// type AssetBalances = (Balance, Balance);

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct Pool<BlockNumber> {
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub final_weights: (Weight, Weight),
	pub curve: CurveType,
	pub pausable: bool,
	pub paused: bool,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, print};
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multi currency for transfer of currencies
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = AssetId, Balance = Balance, Amount = Amount>
			+ MultiReservableCurrency<Self::AccountId>;

		/// Mapping of asset pairs to unique pool identities
		type AssetPairPoolId: AssetPairPoolIdFor<AssetId, PoolId<Self>>;

		/// Trading fee rate
		type PoolDeposit: Get<Balance>;

		/// Trading fee rate
		type SwapFee: Get<Balance>;

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
		InvalidData,
		CannotPauseEndedPool,
		CannotPausePausedPool,
		PoolIsNotPausable,
		CannotUnpauseEndedPool,
		PoolIsNotPaused,

		/// Balance errors
		InsufficientAssetBalance,

		/// Pool existence errors
		TokenPoolNotFound,
		TokenPoolAlreadyExists,

		/// Calculation errors
		BlockNumberInvalid,
		MaxWeightExceeded,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Create new LBP pool
		/// who, asset a, asset b, amount_a, amount_b
		CreatePool(T::AccountId, AssetId, AssetId, Balance, Balance),

		/// Update the LBP pool
		/// who, pool id
		UpdatePool(T::AccountId, PoolId<T>),

		/// Add liquidity to the pool
		/// who, asset_a, asset_b, amount_a, amount_b
		AddLiquidity(T::AccountId, AssetId, AssetId, Balance, Balance),

		/// Remove liquidity from the pool
		/// who, asset_a, asset_b, shares
		RemoveLiquidity(T::AccountId, AssetId, AssetId, Balance),

		/// Destroy LBP pool
		/// who, asset a, asset b
		PoolDestroyed(T::AccountId, AssetId, AssetId),

		/// Sell token
		/// who, asset in, asset out, amount, sale price
		Sell(T::AccountId, AssetId, AssetId, Balance, Balance),

		/// Buy token
		/// who, asset out, asset in, amount, buy price
		Buy(T::AccountId, AssetId, AssetId, Balance, Balance),

		Paused(T::AccountId),
		Unpaused(T::AccountId),
	}

	#[pallet::storage]
	#[pallet::getter(fn pool_owner)]
	pub type PoolOwner<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_deposit)]
	pub type PoolDeposit<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, Balance, ValueQuery>;

	/// Asset pair for each pool. Assets are stored unordered.
	#[pallet::storage]
	#[pallet::getter(fn pool_assets)]
	pub type PoolAssets<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, (AssetId, AssetId), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_data)]
	pub type PoolData<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, Pool<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_balances)]
	pub type PoolBalances<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, (Balance, Balance), ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			asset_a: AssetId,
			amount_a: Balance,
			asset_b: AssetId,
			amount_b: Balance,
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
				T::Currency::free_balance(asset_a, &who) >= amount_a,
				Error::<T>::InsufficientAssetBalance
			);

			ensure!(
				T::Currency::free_balance(asset_b, &who) >= amount_b,
				Error::<T>::InsufficientAssetBalance
			);

			Self::validate_pool_data(&pool_data)?;

			let pool_id = Self::get_pair_id(asset_pair);

			let deposit = T::PoolDeposit::get();

			T::Currency::reserve(CORE_ASSET_ID, &who, deposit)?;
			<PoolDeposit<T>>::insert(&pool_id, &deposit);

			<PoolOwner<T>>::insert(&pool_id, &who);
			<PoolAssets<T>>::insert(&pool_id, &(asset_a, asset_b));
			<PoolData<T>>::insert(&pool_id, &pool_data);

			T::Currency::transfer(asset_a, &who, &pool_id, amount_a)?;
			T::Currency::transfer(asset_b, &who, &pool_id, amount_b)?;

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
			final_weights: Option<(Balance, Balance)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(<PoolOwner<T>>::contains_key(&pool_id), Error::<T>::TokenPoolNotFound);

			let pool_owner = Self::pool_owner(&pool_id);
			ensure!(who == pool_owner, Error::<T>::NotOwner);

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

			let pool_owner = Self::pool_owner(&pool_id);
			ensure!(who == pool_owner, Error::<T>::NotOwner);

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

			let pool_owner = Self::pool_owner(&pool_id);
			ensure!(who == pool_owner, Error::<T>::NotOwner);

			let mut pool_data = Self::pool_data(&pool_id);

			ensure!(pool_data.paused, Error::<T>::PoolIsNotPaused);

			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(pool_data.end > now, Error::<T>::CannotUnpauseEndedPool);

			pool_data.paused = false;
			<PoolData<T>>::insert(&pool_id, &pool_data);

			Self::deposit_event(Event::Unpaused(who));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn validate_pool_data(pool_data: &Pool<T::BlockNumber>) -> DispatchResult {
		let now = <frame_system::Pallet<T>>::block_number();
		ensure!(
			pool_data.start == Zero::zero() || now <= pool_data.start,
			Error::<T>::BlockNumberInvalid
		);
		ensure!(
			pool_data.end == Zero::zero() || pool_data.start < pool_data.end,
			Error::<T>::BlockNumberInvalid
		);
		ensure!(
			pool_data.final_weights.0 <= 100 && pool_data.final_weights.1 <= 100,
			Error::<T>::MaxWeightExceeded
		);

		Ok(())
	}

	fn exists(assets: AssetPair) -> bool {
		let pair_account = T::AssetPairPoolId::from_assets(assets.asset_in, assets.asset_out);
		<PoolAssets<T>>::contains_key(&pair_account)
	}

	fn get_pair_id(assets: AssetPair) -> PoolId<T> {
		T::AssetPairPoolId::from_assets(assets.asset_in, assets.asset_out)
	}

	fn get_pool_assets(pool_id: &T::AccountId) -> Option<Vec<AssetId>> {
		match <PoolAssets<T>>::contains_key(pool_id) {
			true => {
				let assets = Self::pool_assets(pool_id);
				Some(vec![assets.0, assets.1])
			}
			false => None,
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
		let mut buf = Vec::new();
		buf.extend_from_slice(b"lbp");
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
