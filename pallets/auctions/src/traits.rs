pub use crate::Config;
use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, traits::Currency, BoundedVec};
use primitives::BlockNumber;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

pub trait NftAuction<AccountId, AuctionId, BalanceOf, NftAuction, Bid> {
	fn create(&self, sender: AccountId, auction: &NftAuction) -> DispatchResult;

	fn update(&self, sender: AccountId, auction_id: AuctionId, auction: NftAuction) -> DispatchResult;

	fn bid(&mut self, auction_id: &AuctionId, bidder: AccountId, bid: &Bid) -> DispatchResult;

	fn close(&mut self, auction_id: &AuctionId) -> DispatchResult;

	fn validate_general_data(&self) -> DispatchResult;
}

pub trait Randomness<BlockNumber> {
	fn get_random_block_from_range(from: BlockNumber, to: BlockNumber) -> Result<BlockNumber, DispatchError>;
}

pub struct Random;
impl Randomness<BlockNumber> for Random {
	fn get_random_block_from_range(_from: BlockNumber, _to: BlockNumber) -> Result<BlockNumber, DispatchError> {
		Ok(42)
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub enum Auction<T: Config> {
	English(EnglishAuction<T>),
	TopUp(TopUpAuction<T>),
	Candle(CandleAuction<T>),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct EnglishAuction<T: Config> {
	pub general_data: GeneralAuctionData<T>,
	pub specific_data: EnglishAuctionData,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct TopUpAuction<T: Config> {
	pub general_data: GeneralAuctionData<T>,
	pub specific_data: TopUpAuctionData,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct Bid<T: Config> {
	pub amount: BalanceOf<T>,
	pub block_number: T::BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct CandleAuction<T: Config> {
	pub general_data: GeneralAuctionData<T>,
	pub specific_data: CandleAuctionData,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct Winner<AccountId, Balance> {
	pub bidder: AccountId,
	pub amount: Balance,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct EnglishAuctionData {}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct TopUpAuctionData {}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct CandleAuctionData {}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct GeneralAuctionData<T: Config> {
	pub name: BoundedVec<u8, <T as crate::Config>::AuctionsStringLimit>,
	pub reserve_price: Option<BalanceOf<T>>,
	pub last_bid: Option<(<T as frame_system::Config>::AccountId, BalanceOf<T>)>,
	pub next_bid_min: BalanceOf<T>,
	pub start: <T as frame_system::Config>::BlockNumber,
	pub end: <T as frame_system::Config>::BlockNumber,
	pub closed: bool,
	pub owner: <T as frame_system::Config>::AccountId,
	pub token: (
		<T as pallet_uniques::Config>::ClassId,
		<T as pallet_uniques::Config>::InstanceId,
	),
}

/// Define type aliases for better readability
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type WinnerOf<T> = Winner<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

impl<T: Config> sp_std::fmt::Debug for Bid<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "Bid")
	}
}

impl<T: Config> sp_std::fmt::Debug for Auction<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "Auction")
	}
}
