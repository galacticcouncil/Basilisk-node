#![cfg_attr(not(feature = "std"), no_std)]

use primitives::{
	Amount, AssetId, Balance, BlockNumber, CORE_ASSET_ID,
	asset::AssetPair,
};
use frame_support::{
	ensure, transactional,
	traits::Get,
};
use frame_support::sp_runtime::{
	traits::{Hash, Zero},
	app_crypto::sp_core::crypto::UncheckedFrom,
};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
use sp_std::{marker::PhantomData, vec, vec::Vec};
use sp_runtime::RuntimeDebug;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum CurveType {
	Constant,
	Linear,
}

impl Default for CurveType {
	fn default() -> Self { CurveType::Linear }
}

type PoolId<T> = <T as frame_system::Config>::AccountId;
type Weight = Balance;
// type BalanceInfo = (Balance, AssetId);
// type AssetParams = (AssetId, Weight, Balance);
type AssetWeights = (Weight, Weight);
type AssetBalances = (Balance, Balance);

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct Pool<BlockNumber> {
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub end_weights: (Weight, Weight),
	pub curve: CurveType,
	pub pausable: bool,
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
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = AssetId, Balance = Balance, Amount = Amount>
			+ MultiReservableCurrency<Self::AccountId>;

		/// Mapping of asset pairs to unique pool identities
		type AssetPairPoolId: AssetPairPoolIdFor<AssetId, PoolId<Self>>;

		/// Trading fee rate
		type PoolDeposit: Get<Balance>;

		/// Trading fee rate
		type SwapFee: Get<Balance>;

		// /// Weight information for the extrinsics.
		// type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Create pool errors
		CannotCreatePoolWithSameAssets,
		CannotCreatePoolWithZeroLiquidity,

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
		/// Add liquidity to the pool
		/// who, asset_a, asset_b, amount_a, amount_b
		AddLiquidity(T::AccountId, AssetId, AssetId, Balance, Balance),

		/// Remove liquidity from the pool
		/// who, asset_a, asset_b, shares
		RemoveLiquidity(T::AccountId, AssetId, AssetId, Balance),

		/// Create new LBP pool
		/// who, asset a, asset b, amount_a, amount_b
		CreatePool(T::AccountId, AssetId, AssetId, Balance, Balance),

		/// Destroy LBP pool
		/// who, asset a, asset b
		PoolDestroyed(T::AccountId, AssetId, AssetId),

		/// Sell token
		/// who, asset in, asset out, amount, sale price
		Sell(T::AccountId, AssetId, AssetId, Balance, Balance),

		/// Buy token
		/// who, asset out, asset in, amount, buy price
		Buy(T::AccountId, AssetId, AssetId, Balance, Balance),

		Paused,
		Unpaused,
	}

	#[pallet::storage]
	#[pallet::getter(fn pool_owner)]
	pub type PoolOwner<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, T::AccountId, ValueQuery>;

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
		#[pallet::weight(0)]	// TODO: update the weight
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

			ensure!(!amount_a.is_zero() || !amount_b.is_zero(), Error::<T>::CannotCreatePoolWithZeroLiquidity);

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
				pool_data.end_weights.0 <= 100 && pool_data.end_weights.1 <= 100,
				Error::<T>::MaxWeightExceeded
			);

			T::Currency::reserve(CORE_ASSET_ID, &who, T::PoolDeposit::get())?;

			let pool_account = Self::get_pair_id(asset_pair);

			<PoolOwner<T>>::insert(&pool_account, &who);
			<PoolAssets<T>>::insert(&pool_account, &(asset_a, asset_b));
			<PoolData<T>>::insert(&pool_account, &pool_data);

			T::Currency::transfer(asset_a, &who, &pool_account, amount_a)?;
			T::Currency::transfer(asset_b, &who, &pool_account, amount_b)?;

			<PoolBalances<T>>::insert(&pool_account, &(amount_a, amount_b));

			Self::deposit_event(Event::CreatePool(who, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {

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