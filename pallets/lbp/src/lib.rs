#![cfg_attr(not(feature = "std"), no_std)]

use primitives::{
	Amount, AssetId, Balance, BlockNumber,
	asset::AssetPair,
};
use frame_support::{
	traits::Get,
	sp_runtime::traits::Hash,
};
use orml_traits::MultiCurrencyExtended;
use sp_std::{marker::PhantomData, vec, vec::Vec};
use frame_support::sp_runtime::app_crypto::sp_core::crypto::UncheckedFrom;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum CurveType {
	Constant,
	Linear,
}

impl Default for CurveType {
	fn default() -> Self { CurveType::Linear }
}

type PoolId<T> = <T as frame_system::Config>::AccountId;
type Weight = Balance;
// type Asset = (AssetId, Balance);
// type AssetParams = (AssetId, Weight, Balance);
type AssetWeights = (Weight, Weight);
type AssetBalances = (Balance, Balance);

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct Pool {
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub end_weights: AssetWeights,
	pub curve: CurveType,
	pub pausable: bool,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multi currency for transfer of currencies
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = AssetId, Balance = Balance, Amount = Amount>;

		/// Mapping of asset pairs to unique pool identities
		type AssetPairPoolId: AssetPairPoolIdFor<AssetId, PoolId<Self>>;

		// /// Weight information for the extrinsics.
		// type WeightInfo: WeightInfo;

		/// Trading fee rate
		type SwapFee: Get<Balance>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
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
		/// who, asset a, asset b, liquidity
		CreatePool(T::AccountId, AssetId, AssetId, Balance),

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
	pub type PoolAssets<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, AssetPair, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_data)]
	pub type PoolData<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, Pool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_balances)]
	pub type PoolBalances<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, AssetBalances, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
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
				Some(vec![assets.asset_in, assets.asset_out])
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