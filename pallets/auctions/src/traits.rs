use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	Parameter,
};
use sp_runtime::{
	traits::{AtLeast32Bit, Bounded, MaybeDisplay, MaybeSerializeDeserialize, Member},
	RuntimeDebug,
};
use sp_std::{
	fmt::{Debug, Display, Formatter},
	result,
	vec::Vec,
};

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuctionType {
	English,
	Candle,
	Dutch,
	TopUp,
	FixedSwap,
}

impl Display for AuctionType {
	fn fmt(&self, f: &mut Formatter) -> sp_std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Default for AuctionType {
	fn default() -> Self {
		AuctionType::English
	}
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct AuctionInfo<AccountId, Balance, BlockNumber, NftClassId, NftTokenId> {
	// TODO: Replace Vec with BoundedVec
	pub name: Vec<u8>,
	pub last_bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub owner: AccountId,
	pub auction_type: AuctionType,
	pub token: (NftClassId, NftTokenId),
	pub minimal_bid: Balance,
	// pub no_identity_allowed: bool,
	// pub starting_price: Balance,
	// pub private: bool,
	// pub max_participants: u32,
}

/// Abstraction over a NFT auction system.
pub trait Auction<AccountId, BlockNumber, NftClassId, NftTokenId> {
	/// The id of an AuctionInfo
	type AuctionId: Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to bid.
	type Balance: AtLeast32Bit + Copy + MaybeSerializeDeserialize + Debug + Default;
	/// Account id
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;

	/// Create new auction with specific startblock and endblock, return the id
	fn new_auction(
		info: AuctionInfo<Self::AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>,
	) -> result::Result<Self::AuctionId, DispatchError>;
	/// Update the auction info of `id` with `info`
	fn update_auction(
		id: Self::AuctionId,
		info: AuctionInfo<Self::AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>,
	) -> DispatchResult;
	/// Remove auction by `id`
	// fn remove_auction(id: Self::AuctionId) -> DispatchResult;
	/// Bid
	fn bid(bidder: Self::AccountId, id: Self::AuctionId, value: Self::Balance) -> DispatchResult;
}
