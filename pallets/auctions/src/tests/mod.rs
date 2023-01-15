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

use super::*;

use crate::mock::*;
use crate::mocked_objects::*;

use frame_support::BoundedVec;
use sp_core::crypto::AccountId32;
use sp_std::convert::TryInto;

pub type AuctionsModule = Pallet<Test>;

#[cfg(test)]
mod english;

pub fn expect_events(e: Vec<TestEvent>) {
	e.into_iter().for_each(frame_system::Pallet::<Test>::assert_has_event);
}

pub fn set_block_number<T: frame_system::Config<BlockNumber = u64>>(n: u64) {
	frame_system::Pallet::<T>::set_block_number(n);
}

pub fn to_bounded_name(name: Vec<u8>) -> Result<BoundedVec<u8, AuctionsStringLimit>, Error<Test>> {
	name.try_into().map_err(|_| Error::<Test>::TooLong)
}

pub fn bid_object(amount: BalanceOf<Test>, block_number: <Test as frame_system::Config>::BlockNumber) -> Bid<Test> {
	Bid { amount, block_number }
}

pub fn get_auction_subaccount_id(auction_id: <Test as pallet::Config>::AuctionId) -> AccountId32 {
	<Test as pallet::Config>::PalletId::get()
		.try_into_sub_account(auction_id)
		.unwrap()
}

fn predefined_test_ext() -> sp_io::TestExternalities {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, 1_000_000 * BSX),
			(BOB, 1_000_000 * BSX),
			(CHARLIE, 1_000_000 * BSX),
		])
		.with_minted_nft((
			ALICE,
			mocked_nft_collection_id_1::<Test>(),
			mocked_nft_item_id_1::<Test>(),
		))
		.build()
}
