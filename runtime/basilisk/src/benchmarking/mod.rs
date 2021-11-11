#![cfg(feature = "runtime-benchmarks")]

pub mod currencies;
pub mod tokens;
pub mod vesting;

use crate::AssetRegistry;

use primitives::{AssetId, Balance};
use sp_std::vec::Vec;

pub const BSX: Balance = primitives::constants::currency::UNITS;

pub fn register_asset(name: Vec<u8>, deposit: Balance) -> Result<AssetId, ()> {
	AssetRegistry::register_asset(
		AssetRegistry::to_bounded_name(name).map_err(|_| ())?,
		pallet_asset_registry::AssetType::<AssetId>::Token,
		deposit,
	).map_err(|_| ())
}
