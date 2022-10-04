use super::*;

/// Migrate the Uniques pallet storage to v1
pub struct MigrateUniquesPallet;
impl OnRuntimeUpgrade for MigrateUniquesPallet {
	fn on_runtime_upgrade() -> Weight {
		pallet_uniques::migration::migrate_to_v1::<Runtime, _, Uniques>()
	}
}

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
pub struct OnRuntimeUpgradeMigration;
impl OnRuntimeUpgrade for OnRuntimeUpgradeMigration {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		frame_support::log::info!("PreMigrate Marketplace Pallet start");
		pallet_marketplace::migration::v1::pre_migrate::<Runtime>();
		frame_support::log::info!("PreMigrate Marketplace Pallet end");

		frame_support::log::info!("PreMigrate NFT Pallet start");
		pallet_nft::migration::v1::pre_migrate::<Runtime>();
		frame_support::log::info!("PreMigrate NFT Pallet end");

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		let mut weight: Weight = 0;

		frame_support::log::info!("Migrate Uniques Pallet start");
		weight = weight.saturating_add(<MigrateUniquesPallet as OnRuntimeUpgrade>::on_runtime_upgrade());
		frame_support::log::info!("Migrate Uniques Pallet end");

		frame_support::log::info!("Migrate Marketplace Pallet start");
		weight = weight.saturating_add(pallet_marketplace::migration::v1::migrate::<Runtime>());
		frame_support::log::info!("Migrate Marketplace Pallet end");

		frame_support::log::info!("Migrate NFT Pallet start");
		weight = weight.saturating_add(pallet_nft::migration::v1::migrate::<Runtime>());
		frame_support::log::info!("Migrate NFT Pallet end");

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		frame_support::log::info!("PostMigrate Marketplace Pallet start");
		pallet_marketplace::migration::v1::post_migrate::<Runtime>();
		frame_support::log::info!("PostMigrate Marketplace Pallet end");

		frame_support::log::info!("PostMigrate NFT Pallet start");
		pallet_nft::migration::v1::post_migrate::<Runtime>();
		frame_support::log::info!("PostMigrate NFT Pallet end");

		Ok(())
	}
}
