use super::*;

use frame_support::{
	log, migration::storage_key_iter, pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight, StoragePrefixedMap,
};
use pallet_asset_registry::{AssetLocations, LocationAssets};
use polkadot_xcm::v3::MultiLocation;

pub struct OnRuntimeUpgradeMigration;
impl OnRuntimeUpgrade for OnRuntimeUpgradeMigration {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {

		Ok(vec![])
	}

	fn on_runtime_upgrade() -> Weight {
		let mut weight: Weight = Weight::zero();

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		Ok(())
	}
}
