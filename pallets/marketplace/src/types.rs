use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenInfo<Balance> {
	/// Listing price, None = not for sale
	pub(super) price: Option<Balance>,
}

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
