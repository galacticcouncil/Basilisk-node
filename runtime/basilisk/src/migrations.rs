use super::*;

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};

pub struct OnRuntimeUpgradeMigration;
impl OnRuntimeUpgrade for OnRuntimeUpgradeMigration {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// TODO: include migrations for transaction pause and collator rewards pallets in the next release
		// (not in the 106 release, because we don't use the latest pallet versions there).

		Ok(vec![])
	}

	fn on_runtime_upgrade() -> Weight {
		Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		Ok(())
	}
}
