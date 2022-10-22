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

use frame_support::{assert_ok, BoundedVec};
use primitives::nft::ClassType;
use sp_core::crypto::AccountId32;
use sp_std::convert::TryInto;

pub type AuctionsModule = Pallet<Test>;

#[cfg(test)]
mod english;

#[cfg(test)]
mod topup;

#[cfg(test)]
mod candle;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn last_event() -> crate::mock::Event {
	frame_system::Pallet::<Test>::events()
		.pop()
		.expect("An event expected")
		.event
}

pub fn expect_event<E: Into<TestEvent>>(e: E) {
	assert_eq!(last_event(), e.into());
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
	<Test as pallet::Config>::PalletId::get().into_sub_account(("ac", auction_id))
}

fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| set_block_number::<Test>(1));
	ext
}

fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			mocked_nft_class_id_1::<Test>(),
			ClassType::Marketplace,
			bvec![0]
		));
		assert_ok!(Nft::mint(
			Origin::signed(ALICE),
			mocked_nft_class_id_1::<Test>(),
			mocked_nft_instance_id_1::<Test>(),
			bvec![0]
		));
		assert_ok!(Nft::mint(
			Origin::signed(ALICE),
			mocked_nft_class_id_1::<Test>(),
			mocked_nft_instance_id_2::<Test>(),
			bvec![0]
		));

	});

	ext
}
