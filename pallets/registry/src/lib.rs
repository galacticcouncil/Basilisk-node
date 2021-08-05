// This file is part of Basilisk-node.

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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchError;
use frame_support::sp_runtime::traits::{CheckedAdd, One};
use frame_support::transactional;
use frame_system::pallet_prelude::*;
use sp_arithmetic::traits::BaseArithmetic;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod types;

// Re-export pallet items so that they can be accessed from the crate namespace.
use frame_support::BoundedVec;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::types::{AssetDetails, AssetMetadata, AssetType};
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Asset type
		type AssetId: Parameter + Member + Default + Copy + BaseArithmetic + MaybeSerializeDeserialize + MaxEncodedLen;

		/// Asset origin type
		type AssetNativeLocation: Parameter + Member;

		/// The maximum length of a name or symbol stored on-chain.
		type StringLimit: Get<u32>;

		/// Native Asset Id
		#[pallet::constant]
		type NativeAssetId: Get<Self::AssetId>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Asset Id is not available. This only happens when it reaches the MAX value of given id type.
		NoIdAvailable,

		/// Invalid asset name or symbol.
		AsseTNotFound,

		/// Invalid asset name or symbol.
		BadMetadata,

		/// Asset ID is not registered in the registry.
		AssetNotRegistered,

		/// Asset is already registered.
		AssetAlreadyRegistered,

		/// Setting metadata for incorrect asset.
		AssetMismatch,
	}

	/// Details of an asset.
	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub type Assets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		AssetDetails<T::AssetId, BoundedVec<u8, T::StringLimit>>,
		OptionQuery,
	>;

	/// Next available asset id. This is sequential id assigned for each new registered asset.
	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

	/// Created assets
	#[pallet::storage]
	#[pallet::getter(fn asset_ids)]
	pub type AssetIds<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::StringLimit>, T::AssetId, OptionQuery>;

	///
	#[pallet::storage]
	#[pallet::getter(fn locations)]
	pub type AssetLocations<T: Config> = StorageMap<_, Twox64Concat, T::AssetId, T::AssetNativeLocation, OptionQuery>;

	///
	#[pallet::storage]
	#[pallet::getter(fn location_assets)]
	pub type LocationAssets<T: Config> = StorageMap<_, Twox64Concat, T::AssetNativeLocation, T::AssetId, OptionQuery>;

	/// Details of an asset.
	#[pallet::storage]
	#[pallet::getter(fn asset_metadata)]
	pub type AssetMetadataMap<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, AssetMetadata<BoundedVec<u8, T::StringLimit>>, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub asset_ids: Vec<(Vec<u8>, T::AssetId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { asset_ids: vec![] }
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.asset_ids.iter().for_each(|(name, asset_id)| {
				let bounded_name: BoundedVec<u8, T::StringLimit> = name
					.clone()
					.try_into()
					.map_err(|_| panic!("Invalid asset name!"))
					.unwrap();
				AssetIds::<T>::insert(bounded_name, asset_id);
			})
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[transactional]
		pub fn register_asset(
			origin: OriginFor<T>,
			name: Vec<u8>,
			asset_type: AssetType<T::AssetId>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let bounded_name = Self::to_bounded_name(name)?;

			ensure!(
				Self::asset_ids(&bounded_name).is_none(),
				Error::<T>::AssetAlreadyRegistered
			);

			NextAssetId::<T>::mutate(|value| -> DispatchResult {
				// Check if current id does not clash with CORE ASSET ID.
				// If yes, just skip it and use next one, otherwise use it.
				// Note: this way we prevent accidental clashes with native asset id, so no need to set next asset id to be > next asset id
				let asset_id = if *value == T::NativeAssetId::get() {
					value
						.checked_add(&T::AssetId::from(1))
						.ok_or(Error::<T>::NoIdAvailable)?
				} else {
					*value
				};

				// Map name to asset id
				AssetIds::<T>::insert(&bounded_name, asset_id);

				let details = AssetDetails {
					name: bounded_name,
					asset_type,
					locked: false,
				};

				// Store the details
				Assets::<T>::insert(asset_id, details);

				// Increase asset id to be assigned for following asset.
				*value = asset_id
					.checked_add(&T::AssetId::from(1))
					.ok_or(Error::<T>::NoIdAvailable)?;

				//TODO: add event

				Ok(())
			})?;

			Ok(())
		}
		#[pallet::weight(0)]
		#[transactional]
		pub fn update_asset(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			name: Vec<u8>,
			asset_type: AssetType<T::AssetId>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			Assets::<T>::try_mutate(asset_id, |maybe_detail| -> DispatchResult {
				let mut detail = maybe_detail.as_mut().ok_or(Error::<T>::AsseTNotFound)?;

				let bn = Self::to_bounded_name(name)?;

				if bn != detail.name {
					// update also name map
					AssetIds::<T>::remove(&detail.name);
					AssetIds::<T>::insert(&bn, asset_id);
				}

				detail.name = bn;
				detail.asset_type = asset_type;

				//TODO: add event

				Ok(())
			})
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn set_metadata(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(Self::assets(asset_id).is_some(), Error::<T>::AsseTNotFound);

			let metadata = AssetMetadata::<BoundedVec<u8, T::StringLimit>> {
				symbol: Self::to_bounded_name(symbol)?,
				decimals,
			};

			AssetMetadataMap::<T>::insert(asset_id, metadata);

			//TODO: add event

			Ok(())
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn set_asset_location(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			location: T::AssetNativeLocation,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(Self::assets(asset_id).is_some(), Error::<T>::AssetNotRegistered);

			AssetLocations::<T>::insert(asset_id, &location);
			LocationAssets::<T>::insert(location, asset_id);

			//TODO: add event

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn to_bounded_name(name: Vec<u8>) -> Result<BoundedVec<u8, T::StringLimit>, Error<T>> {
		name.try_into().map_err(|_| Error::<T>::BadMetadata)
	}

	/// Create asset for given name or return existing AssetId if such asset already exists.
	pub fn get_or_create_asset(name: Vec<u8>) -> Result<T::AssetId, DispatchError> {
		let bounded_name: BoundedVec<u8, T::StringLimit> = Self::to_bounded_name(name)?;

		if <AssetIds<T>>::contains_key(&bounded_name) {
			Ok(<AssetIds<T>>::get(&bounded_name).unwrap())
		} else {
			let asset_id = Self::next_asset_id();
			let next_id = asset_id.checked_add(&One::one()).ok_or(Error::<T>::NoIdAvailable)?;
			<NextAssetId<T>>::put(next_id);
			<AssetIds<T>>::insert(bounded_name, asset_id);
			Ok(asset_id)
		}
	}

	pub fn asset_to_location(asset_id: T::AssetId) -> Option<T::AssetNativeLocation> {
		Self::locations(asset_id)
	}

	pub fn location_to_asset(location: T::AssetNativeLocation) -> Option<T::AssetId> {
		Self::location_assets(location)
	}
}
