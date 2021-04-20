#![cfg_attr(not(feature = "std"), no_std)]

use primitives::{
	Amount, AssetId, Balance, BlockNumber,
	{asset::AssetPair},
};
use frame_support::{
	traits::Get,
};
use orml_traits::MultiCurrencyExtended;

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
}
