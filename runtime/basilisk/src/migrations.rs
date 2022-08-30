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
	fn on_runtime_upgrade() -> Weight {
		let mut weight = 0;

		frame_support::log::info!("MigrateUniquesPallet start");
		weight += <MigrateUniquesPallet as OnRuntimeUpgrade>::on_runtime_upgrade();
		frame_support::log::info!("MigrateUniquesPallet end");

		weight
	}
}
