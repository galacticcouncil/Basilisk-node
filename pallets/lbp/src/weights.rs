// #![allow(unused_parens)]
// #![allow(unused_imports)]
// #![allow(clippy::unnecessary_cast)]

use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

/// Weight functions needed for LBP pallet.
pub trait WeightInfo {
	fn create_pool() -> Weight;
	fn update_pool_data() -> Weight;
	fn add_liquidity() -> Weight;
	fn remove_liquidity() -> Weight;
	fn pause_pool() -> Weight;
	fn unpause_pool() -> Weight;
	fn destroy_pool() -> Weight;
	fn sell() -> Weight;
	fn buy() -> Weight;
}

/// Weights for LBP pallet using the hydraDX node and recommended hardware.
pub struct HydraWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for HydraWeight<T> {
	fn create_pool() -> u64 {
		0
	}

	fn update_pool_data() -> u64 {
		0
	}

	fn add_liquidity() -> u64 {
		0
	}

	fn remove_liquidity() -> u64 {
		0
	}

	fn pause_pool() -> u64 {
		0
	}

	fn unpause_pool() -> u64 {
		0
	}

	fn destroy_pool() -> u64 {
		0
	}

	fn sell() -> u64 {
		0
	}

	fn buy() -> Weight {
		0
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_pool() -> u64 {
		0
	}

	fn update_pool_data() -> u64 {
		0
	}

	fn add_liquidity() -> u64 {
		0
	}

	fn remove_liquidity() -> u64 {
		0
	}

	fn pause_pool() -> u64 {
		0
	}

	fn destroy_pool() -> u64 {
		0
	}

	fn unpause_pool() -> Weight {
		0
	}

	fn sell() -> Weight {
		0
	}

	fn buy() -> Weight {
		0
	}
}
