use crate::AssetRegistry;
use frame_support::dispatch::DispatchResult;
use pallet_asset_registry::traits::InspectRegistry;
use pallet_ema_oracle::BenchmarkHelper as EmaOracleBenchmarkHelper;
use pallet_treasury::ArgumentsFactory;
use primitives::{AccountId, AssetId};

pub struct BenchmarkHelper;

// Support for pallet treasury benchmarking
impl ArgumentsFactory<(), AccountId> for BenchmarkHelper {
	fn create_asset_kind(_seed: u32) {	}

	fn create_beneficiary(seed: [u8; 32]) -> AccountId {
		AccountId::from(seed)
	}
}

// Ema oracle helper
impl EmaOracleBenchmarkHelper<AssetId> for BenchmarkHelper {
	fn register_asset(asset_id: AssetId) -> DispatchResult {
		if AssetRegistry::exists(asset_id) {
			return Ok(());
		}

		let name = asset_id.to_ne_bytes().to_vec();
		let _ = AssetRegistry::register_asset(
			AssetRegistry::to_bounded_name(name)?,
			pallet_asset_registry::AssetType::<AssetId>::Token,
			1u128,
			Some(asset_id),
			None,
		)?;
		Ok(())
	}
}
