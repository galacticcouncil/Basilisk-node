pub use crate::Config;
use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, traits::Currency, BoundedVec};
use scale_info::TypeInfo;

pub trait NftAuction<AccountId, AuctionId, BalanceOf, NftAuction> {
	fn bid(&mut self, bidder: AccountId, value: BalanceOf) -> DispatchResult;

	fn close(&mut self) -> DispatchResult;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub enum Auction<T: Config> {
	English(EnglishAuction<T>),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct EnglishAuction<T: Config> {
	pub general_data: GeneralAuctionData<T>,
	pub specific_data: EnglishAuctionData<T>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct EnglishAuctionData<T: Config> {
	pub reserve_price: BalanceOf<T>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct GeneralAuctionData<T: Config> {
	pub name: BoundedVec<u8, <T as crate::Config>::AuctionsStringLimit>,
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

impl<T: Config> sp_std::fmt::Debug for Auction<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "Auction")
	}
}
