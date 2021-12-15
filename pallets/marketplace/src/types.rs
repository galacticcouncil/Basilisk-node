use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use pallet_nft::NftPermission;
use primitives::ClassType;
use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Offer<AccountId, Balance, BlockNumber> {
	/// User who made the offer
	pub(super) maker: AccountId,
	/// Offered amount
	pub(super) amount: Balance,
	/// After this block the offer can't be accepted
	pub(super) expires: BlockNumber,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MarketInstance<AccountId> {
	/// The user account which receives the royalty
	pub author: AccountId,
	/// Royalty in percent in range 0-99
	pub royalty: u8,
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MarketPlacePermissions;

impl NftPermission<ClassType> for MarketPlacePermissions {
	fn can_create(class_type: &ClassType) -> bool {
		*class_type == ClassType::Marketplace
	}

	fn can_mint(class_type: &ClassType) -> bool {
		*class_type == ClassType::Marketplace
	}

	fn can_transfer(class_type: &ClassType) -> bool {
		*class_type == ClassType::Marketplace
	}

	fn can_burn(class_type: &ClassType) -> bool {
		*class_type == ClassType::Marketplace
	}

	fn can_destroy(class_type: &ClassType) -> bool {
		*class_type == ClassType::Marketplace
	}

	fn has_deposit(class_type: &ClassType) -> bool {
		*class_type == ClassType::Marketplace
	}
}
