use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetType<AssetId> {
	Token,
	PoolShare(AssetId, AssetId),
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetDetails<AssetId, Balance, BoundedString> {
	/// The name of this asset. Limited in length by `StringLimit`.
	pub(super) name: BoundedString,

	pub(super) asset_type: AssetType<AssetId>,

	pub(super) existential_deposit: Balance,

	pub(super) locked: bool,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, TypeInfo)]
pub struct AssetMetadata<BoundedString> {
	/// The ticker symbol for this asset. Limited in length by `StringLimit`.
	pub(super) symbol: BoundedString,
	/// The number of decimals this asset uses to represent one unit.
	pub(super) decimals: u8,
}
