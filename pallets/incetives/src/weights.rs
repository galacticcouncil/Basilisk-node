#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for amm.
pub trait WeightInfo {
	fn add_shares() -> Weight;
}

/// Weights for amm using the hydraDX node and recommended hardware.
pub struct HydraWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for HydraWeight<T> {
	fn add_shares() -> Weight {
        0
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn add_shares() -> Weight {
        0
	}
}
