//      ---_ ......._-_--.        ,adPPYba, 8b,dPPYba,    ,adPPYba,  88   ,d8
//     (|\ /      / /| \  \       I8[    "" 88P'   `"8a  a8P_____88  88 ,a8"
// 		 /  /     .'  -=-'   `.      `"Y8ba,  88       88  8PP"""""""  8888[
// 		/  /    .'             )    aa    ]8I 88       88  "8b,   ,aa  88`"Yba,
// 	_/  /   .'        _.)   /     `"YbbdP"' 88       88   `"Ybbd8"'  88   `Y8a
//  / o   o        _.-' /  .'
//  \          _.-'    / .'*|
// 	\______.-'//    .'.' \*|      This file is part of Basilisk-node.
// 	 \|  \ | //   .'.' _ |*|      Built with <3 for decentralisation.
// 		`   \|//  .'.'_ _ _|*|
// 		 .  .// .'.' | _ _ \*|      Copyright (C) 2021-2022  Intergalactic, Limited (GIB).
// 		 \`-|\_/ /    \ _ _ \*\     SPDX-License-Identifier: Apache-2.0
// 			`/'\__/      \ _ _ \*\    Licensed under the Apache License, Version 2.0 (the "License");
// 		 /^|            \ _ _ \*    you may not use this file except in compliance with the License.
// 		'  `             \ _ _ \    http://www.apache.org/licenses/LICENSE-2.0
//     '  `             \ _ _ \

pub use crate::Config;
use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, pallet_prelude::DispatchError, traits::Currency, BoundedVec};
use scale_info::TypeInfo;

pub trait NftAuction<AccountId, AuctionId, BalanceOf, NftAuction, Bid> {
	fn create(&self, sender: AccountId, auction: &NftAuction) -> DispatchResult;

	fn update(self, sender: AccountId, auction_id: AuctionId) -> DispatchResult;

	fn bid(&mut self, auction_id: AuctionId, bidder: AccountId, bid: &Bid) -> DispatchResult;

	fn close(&mut self, auction_id: AuctionId) -> Result<bool, DispatchError>;

	fn claim(&self, auction_id: AuctionId, bidder: AccountId, amount: BalanceOf) -> Result<bool, DispatchError>;

	fn validate_data(&self) -> DispatchResult;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub enum Auction<T: Config> {
	English(EnglishAuction<T>),
	TopUp(TopUpAuction<T>),
	Candle(CandleAuction<T>),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct EnglishAuction<T: Config> {
	pub common_data: CommonAuctionData<T>,
	pub specific_data: EnglishAuctionData,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct TopUpAuction<T: Config> {
	pub common_data: CommonAuctionData<T>,
	pub specific_data: TopUpAuctionData,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct Bid<T: Config> {
	pub amount: BalanceOf<T>,
	pub block_number: <T as frame_system::Config>::BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct CandleAuction<T: Config> {
	pub common_data: CommonAuctionData<T>,
	pub specific_data: CandleAuctionData<T>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct EnglishAuctionData {}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct TopUpAuctionData {}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct CandleAuctionData<T: Config> {
	pub closing_start: <T as frame_system::Config>::BlockNumber,
	pub winning_closing_range: Option<u32>,
	pub winner: Option<<T as frame_system::Config>::AccountId>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct CommonAuctionData<T: Config> {
	pub name: BoundedVec<u8, <T as crate::Config>::AuctionsStringLimit>,
	pub reserve_price: Option<BalanceOf<T>>,
	pub last_bid: Option<(<T as frame_system::Config>::AccountId, BalanceOf<T>)>,
	pub next_bid_min: BalanceOf<T>,
	pub start: <T as frame_system::Config>::BlockNumber,
	pub end: <T as frame_system::Config>::BlockNumber,
	pub closed: bool,
	pub owner: <T as frame_system::Config>::AccountId,
	pub token: (
		<T as pallet_nft::Config>::NftClassId,
		<T as pallet_nft::Config>::NftInstanceId,
	),
}

/// Define type aliases for better readability
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
