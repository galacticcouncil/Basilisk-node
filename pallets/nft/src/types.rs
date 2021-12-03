use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;
use primitives::Balance;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ClassType {
	Marketplace = 0_isize,
	PoolShare = 1_isize,
	Redeemable = 2_isize,
}

impl Default for ClassType {
	fn default() -> Self {
		ClassType::Marketplace
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassInfo<BoundedString> {
	/// Class type
	pub class_type: ClassType,
	/// Arbitrary data about a class, e.g. IPFS hash
	pub metadata: BoundedString,
}

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

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MarketInstance<AccountId, BoundedString> {
	/// The user account which receives the royalty
	pub author: AccountId,
	/// Royalty in percent in range 0-99
	pub royalty: u8,
	/// Arbitrary data about an instance, e.g. IPFS hash
	pub metadata: BoundedString,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LiqMinInstance<Balance, BoundedString> {
	/// Number of shares in a liquidity mining pool
	pub shares: Balance,
	/// Accumulated reward per share
	pub accrps: Balance,
	/// Arbitrary data about an instance, e.g. IPFS hash
	pub metadata: BoundedString,
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BondingCurve {
    /// The exponent of the curve.
    pub exponent: u32,
    /// The slope of the curve.
    pub slope: u128,
}