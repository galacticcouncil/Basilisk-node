use frame_support::pallet_prelude::*;

use primitives::{nft::ClassType, Balance};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct RedeemablesClassInfo {
	/// Class type
	pub class_type: ClassType,
	/// Max pool issuance
	pub max_supply: u32,
	/// Count of currently issued (minted) in circulation
	pub issued: u32,
	/// Count of redeemed (burned) from circulation
	pub redeemed: u32,
	/// Curve id
	pub curve: BondingCurve,
}

impl RedeemablesClassInfo {
	/// Calculate price of current buy/sell
	pub fn price(&self) -> Balance {
		self.curve.slope / (self.max_supply - self.issued).pow(self.curve.exponent) as Balance
	}
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BondingCurve {
	/// The exponent of the curve.
	pub exponent: u32,
	/// The slope of the curve.
	pub slope: u128,
}
