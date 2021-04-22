// #![allow(unused_parens)]
// #![allow(unused_imports)]
// #![allow(clippy::unnecessary_cast)]

use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

/// Weight functions needed for LBP pallet.
pub trait WeightInfo {
	fn create_pool() -> Weight;
}

/// Weights for LBP pallet using the hydraDX node and recommended hardware.
pub struct HydraWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for HydraWeight<T> {
	fn create_pool() -> u64 {
		0
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_pool() -> u64 {
		0
	}
}