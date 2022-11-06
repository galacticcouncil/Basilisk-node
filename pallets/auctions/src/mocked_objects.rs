//      ---_ ......._-_--.        ,adPPYba, 8b,dPPYba,    ,adPPYba,  88   ,d8
//     (|\ /      / /| \  \       I8[    "" 88P'   `"8a  a8P_____88  88 ,a8"
//     /  /     .'  -=-'   `.      `"Y8ba,  88       88  8PP"""""""  8888[
//    /  /    .'             )    aa    ]8I 88       88  "8b,   ,aa  88`"Yba,
//  _/  /   .'        _.)   /     `"YbbdP"' 88       88   `"Ybbd8"'  88   `Y8a
//  / o   o        _.-' /  .'
//  \          _.-'    / .'*|
//  \______.-'//    .'.' \*|      This file is part of Basilisk-node.
//   \|  \ | //   .'.' _ |*|      Built with <3 for decentralisation.
//    `   \|//  .'.'_ _ _|*|
//     .  .// .'.' | _ _ \*|      Copyright (C) 2021-2022  Intergalactic, Limited (GIB).
//     \`-|\_/ /    \ _ _ \*\     SPDX-License-Identifier: Apache-2.0
//      `/'\__/      \ _ _ \*\    Licensed under the Apache License, Version 2.0 (the "License");
//     /^|            \ _ _ \*    you may not use this file except in compliance with the License.
//    '  `             \ _ _ \    http://www.apache.org/licenses/LICENSE-2.0
//     '  `             \ _ _ \

#![allow(clippy::unused_unit)]

use super::*;

// NFT mocks
pub fn mocked_nft_collection_id_1<T: Config>() -> <T as pallet_nft::Config>::NftCollectionId {
	<T as pallet_nft::Config>::NftCollectionId::from(1_000_000u32)
}

pub fn mocked_nft_item_id_1<T: Config>() -> <T as pallet_nft::Config>::NftItemId {
	<T as pallet_nft::Config>::NftItemId::from(1u16)
}

pub fn mocked_nft_instance_id_2<T: Config>() -> <T as pallet_nft::Config>::NftItemId {
	<T as pallet_nft::Config>::NftItemId::from(2u16)
}

pub fn mocked_nft_token<T: Config>() -> (
	<T as pallet_nft::Config>::NftCollectionId,
	<T as pallet_nft::Config>::NftItemId,
) {
	(mocked_nft_collection_id_1::<T>(), mocked_nft_item_id_1::<T>())
}

pub fn mocked_nft_token_2<T: Config>() -> (
	<T as pallet_nft::Config>::NftCollectionId,
	<T as pallet_nft::Config>::NftItemId,
) {
	(mocked_nft_collection_id_1::<T>(), mocked_nft_instance_id_2::<T>())
}

// English Auction object mocks
pub fn mocked_english_auction_object<T: Config>(
	common_data: CommonAuctionData<T>,
	specific_data: EnglishAuctionData,
) -> Auction<T> {
	let auction_data = EnglishAuction {
		common_data,
		specific_data,
	};

	Auction::English(auction_data)
}

pub fn mocked_english_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T> {
	CommonAuctionData {
		name: sp_std::vec![0; <T as pallet::Config>::AuctionsStringLimit::get() as usize]
			.try_into()
			.unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u32.into(),
		end: 21u32.into(),
		closed: false,
		owner,
		token: mocked_nft_token::<T>(),
		next_bid_min: BalanceOf::<T>::from(1u32),
	}
}

pub fn mocked_english_specific_data<T: Config>() -> EnglishAuctionData {
	EnglishAuctionData {}
}

// Candle Auction object mocks
pub fn mocked_candle_auction_object<T: Config>(
	common_data: CommonAuctionData<T>,
	specific_data: CandleAuctionData<T>,
) -> Auction<T> {
	let auction_data = CandleAuction {
		common_data,
		specific_data,
	};

	Auction::Candle(auction_data)
}

pub fn mocked_candle_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T> {
	CommonAuctionData {
		name: sp_std::vec![0; <T as pallet::Config>::AuctionsStringLimit::get() as usize]
			.try_into()
			.unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u32.into(),
		end: 99_366u32.into(),
		closed: false,
		owner,
		token: mocked_nft_token::<T>(),
		next_bid_min: BalanceOf::<T>::from(1u32),
	}
}

pub fn mocked_candle_specific_data<T: Config>() -> CandleAuctionData<T> {
	CandleAuctionData {
		closing_start: 27_366u32.into(),
		winner: None,
		winning_closing_range: None,
	}
}

// TopUp Auction object mocks
pub fn mocked_topup_auction_object<T: Config>(
	common_data: CommonAuctionData<T>,
	specific_data: TopUpAuctionData,
) -> Auction<T> {
	let auction_data = TopUpAuction {
		common_data,
		specific_data,
	};

	Auction::TopUp(auction_data)
}

pub fn mocked_topup_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T> {
	CommonAuctionData {
		name: sp_std::vec![0; <T as pallet::Config>::AuctionsStringLimit::get() as usize]
			.try_into()
			.unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u32.into(),
		end: 21u32.into(),
		closed: false,
		owner,
		token: mocked_nft_token::<T>(),
		next_bid_min: BalanceOf::<T>::from(1u32),
	}
}

pub fn mocked_topup_specific_data<T: Config>() -> TopUpAuctionData {
	TopUpAuctionData {}
}
