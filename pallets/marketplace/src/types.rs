use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenInfo<AccountId, Balance, BlockNumber> {
	/// Listing price, None = not for sale
	pub(super) price: Option<Balance>,
	/// Highest offer \[bidder, amount, until\]
	pub(super) offer: Option<(AccountId, Balance, BlockNumber)>,
}
