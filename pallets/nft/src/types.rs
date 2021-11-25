use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ClassType {
	Marketplace = 0_isize,
	PoolShare = 1_isize,
}

impl Default for ClassType {
	fn default() -> Self {
		ClassType::Marketplace
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassInfo<BoundedString> {
	/// The user account which receives the royalty
	pub class_type: ClassType,
	/// Arbitrary data about a class, e.g. IPFS hash
	pub metadata: BoundedString,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct InstanceInfo<AccountId, Balance, BoundedString> {
	/// The user account which receives the royalty
	pub author: AccountId,
	/// Royalty in percent in range 0-99
	pub royalty: u8,
	/// Arbitrary data about an instance, e.g. IPFS hash
	pub metadata: BoundedString,
	/// Number of shares in a liquidity mining pool
	pub shares: Balance,
	/// Accumulated reward per share
	pub accrps: Balance,
}
