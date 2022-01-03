use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassInfo<ClassType, BoundedVec> {
	/// The user account which receives the royalty
	pub class_type: ClassType,
	/// Arbitrary data about a class, e.g. IPFS hash
	pub metadata: BoundedVec,
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct InstanceInfo<BoundedVec> {
	pub metadata: BoundedVec,
}
