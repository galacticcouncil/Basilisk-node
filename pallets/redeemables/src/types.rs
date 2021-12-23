use frame_support::pallet_prelude::*;

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
    /// Price of previous token (sell to pool)
	pub fn price(&self) -> Balance {
        self.curve.slope / (self.max_supply - self.issued).pow(self.curve.exponent) as Balance
    }
}