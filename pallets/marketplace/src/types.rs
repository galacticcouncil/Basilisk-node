use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenInfo<AccountId, Balance> {
	/// The user account which receives the royalty
	pub(super) author: AccountId,
	/// Royalty in percent in range 0-99
	pub(super) royalty: u8,
	/// Listing price, None = not for sale
	pub(super) price: Option<Balance>,
	/// Highest offer
	pub(super) offer: Option<(AccountId, Balance)>,
}
