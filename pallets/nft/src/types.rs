use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ClassType {
	Unknown = 0,
	Art = 1,
	PoolShare = 2,
}

impl Default for ClassType {
	fn default() -> Self {
		ClassType::Unknown
	}
}
