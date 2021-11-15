#![cfg(feature = "runtime-benchmarks")]

pub mod currencies;
pub mod duster;
pub mod tokens;
pub mod vesting;

use crate::AssetRegistry;
use frame_system::RawOrigin;

use primitives::{AssetId, Balance};
use sp_std::vec::Vec;

pub const BSX: Balance = primitives::constants::currency::UNITS;

pub fn register_asset(name: Vec<u8>, deposit: Balance) -> Result<AssetId, ()> {
	AssetRegistry::register_asset(
		AssetRegistry::to_bounded_name(name).map_err(|_| ())?,
		pallet_asset_registry::AssetType::<AssetId>::Token,
		deposit,
	)
	.map_err(|_| ())
}

pub fn update_asset(asset_id: AssetId, name: Vec<u8>, deposit: Balance) -> Result<(), ()> {
	AssetRegistry::update(
		RawOrigin::Root.into(),
		asset_id,
		name,
		pallet_asset_registry::AssetType::<AssetId>::Token,
		Some(deposit),
	)
	.map_err(|_| ())
}
