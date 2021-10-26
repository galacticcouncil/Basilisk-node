use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenInfo<AccountId, Balance, BlockNumber> {
	/// The user account which receives the royalty
	pub(super) author: AccountId,
	/// Royalty in percent in range 0-99
	pub(super) royalty: u8,
	/// Listing price, None = not for sale
	pub(super) price: Option<Balance>,
	/// Highest offer \[bidder, amount, until\]
	pub(super) offer: Option<(AccountId, Balance, BlockNumber)>,
	/// If currently listed on marketplace
	pub(super) is_listed: bool,
}
