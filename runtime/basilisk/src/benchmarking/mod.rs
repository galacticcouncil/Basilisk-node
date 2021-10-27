#![cfg(feature = "runtime-benchmarks")]

pub mod currencies;
pub mod tokens;
pub mod vesting;

use crate::AssetRegistry;

use frame_system::RawOrigin;
use primitives::{AssetId, Balance};
use sp_std::vec::Vec;

pub const BSX: Balance = primitives::constants::currency::UNITS;

pub fn register_asset(name: Vec<u8>, deposit: Balance) {
	let _ = AssetRegistry::register(
		RawOrigin::Root.into(),
		name,
		pallet_asset_registry::AssetType::<AssetId>::Token,
		deposit,
	);
}
