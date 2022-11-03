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
		pallet_marketplace::migration::v2::pre_migrate::<Runtime>();

		frame_support::log::info!("PreMigrate NFT Pallet start");
		pallet_nft::migration::v1::pre_migrate::<Runtime>();
		frame_support::log::info!("PreMigrate NFT Pallet end");

		frame_support::log::info!("PreMigrate XYK liquidity mining start");
		pallet_xyk_liquidity_mining::migration::v1::pre_migrate::<Runtime>();
		frame_support::log::info!("PreMigrate XYK liquidity mining end");

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		let mut weight: Weight = Weight::zero();

		frame_support::log::info!("Migrate Uniques Pallet start");
		weight = weight.saturating_add(<MigrateUniquesPallet as OnRuntimeUpgrade>::on_runtime_upgrade());
		frame_support::log::info!("Migrate Uniques Pallet end");

		weight = weight.saturating_add(pallet_marketplace::migration::v2::migrate::<Runtime>());

		frame_support::log::info!("Migrate NFT Pallet start");
		weight = weight.saturating_add(pallet_nft::migration::v1::migrate::<Runtime>());
		frame_support::log::info!("Migrate NFT Pallet end");

		frame_support::log::info!("Migrate XYK Liquidity Mining Pallet start");
		weight = weight.saturating_add(pallet_xyk_liquidity_mining::migration::v1::migrate::<Runtime>());
		frame_support::log::info!("Migrate XYK Liquidity Mining Pallet end");

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		pallet_marketplace::migration::v2::post_migrate::<Runtime>();

		frame_support::log::info!("PostMigrate NFT Pallet start");
		pallet_nft::migration::v1::post_migrate::<Runtime>();
		frame_support::log::info!("PostMigrate NFT Pallet end");

		frame_support::log::info!("PostMigrate XYK Liquidity Mining Pallet start");
		pallet_nft::migration::v1::post_migrate::<Runtime>();
		frame_support::log::info!("PostMigrate XYK Liquidity Mining Pallet end");

		Ok(())
	}
}
