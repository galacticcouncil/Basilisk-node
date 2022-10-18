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
		pallet_marketplace::migration::v1::move_old_storage::pre_migrate::<Runtime>();
		frame_support::log::info!("PreMigrate Marketplace Pallet end");

		frame_support::log::info!("PreMigrate NFT Pallet start");
		pallet_nft::migration::v1::pre_migrate::<Runtime>();
		frame_support::log::info!("PreMigrate NFT Pallet end");

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		frame_support::log::info!("Migrate Uniques Pallet start");
		<MigrateUniquesPallet as OnRuntimeUpgrade>::on_runtime_upgrade();
		frame_support::log::info!("Migrate Uniques Pallet end");

		frame_support::log::info!("Migrate Marketplace Pallet start");
		pallet_marketplace::migration::v1::move_old_storage::migrate::<Runtime>();
		frame_support::log::info!("Migrate Marketplace Pallet end");

		frame_support::log::info!("Migrate NFT Pallet start");
		pallet_nft::migration::v1::migrate::<Runtime>();
		frame_support::log::info!("Migrate NFT Pallet end");

		// Both marketplace and nft pallets return max_block weight. Test that it fits into a block before executing the migration.
		<Runtime as frame_system::Config>::BlockWeights::get().max_block
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		frame_support::log::info!("PostMigrate Marketplace Pallet start");
		pallet_marketplace::migration::v1::move_old_storage::post_migrate::<Runtime>();
		frame_support::log::info!("PostMigrate Marketplace Pallet end");

		frame_support::log::info!("PostMigrate NFT Pallet start");
		pallet_nft::migration::v1::post_migrate::<Runtime>();
		frame_support::log::info!("PostMigrate NFT Pallet end");

		Ok(())
	}
}
