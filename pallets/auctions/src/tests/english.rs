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
use frame_support::{assert_noop, assert_ok};

// -------------- English auction tests -------------- //
/// Creating an English auction
///
/// Happy path
#[test]
fn create_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction.clone()));

		expect_events(vec![mock::Event::Auctions(
			pallet::Event::<Test>::AuctionCreated {
				auction_id: 0,
				auction: auction,
			}
			.into(),
		)]);

		let auction = AuctionsModule::auctions(0).unwrap();

		let auction_check = match auction {
			Auction::English(data) => {
				assert_eq!(data.common_data.reserve_price, None);
				assert_eq!(data.common_data.last_bid, None);
				assert_eq!(data.common_data.start, 10u64);
				assert_eq!(data.common_data.end, 21u64);
				assert_eq!(data.common_data.owner, ALICE);
				assert_eq!(data.common_data.token, mocked_nft_token::<Test>());
				assert_eq!(data.common_data.next_bid_min, 1);

				Ok(())
			}
			_ => Err(()),
		};

		assert_ok!(auction_check);

		assert_eq!(AuctionsModule::auction_owner_by_id(0), Some(ALICE));
	});
}

/// Error AuctionStartTimeAlreadyPassed
#[test]
fn create_english_auction_starting_in_the_past_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_english_common_data::<Test>(ALICE);
		common_auction_data.start = 0u64;

		let auction =
			mocked_english_auction_object::<Test>(common_auction_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::AuctionStartTimeAlreadyPassed
		);
	});
}

/// Error InvalidTimeConfiguration
#[test]
fn create_english_auction_without_end_time_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_english_common_data::<Test>(ALICE);
		common_auction_data.end = 0u64;

		let auction =
			mocked_english_auction_object::<Test>(common_auction_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);
	});
}

/// Error InvalidTimeConfiguration (duration too short)
#[test]
fn create_english_auction_with_duration_shorter_than_minimum_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_english_common_data::<Test>(ALICE);
		common_auction_data.end = 20u64;

		let auction =
			mocked_english_auction_object::<Test>(common_auction_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);
	});
}

/// Error InvalidNextBidMin
#[test]
fn create_english_auction_with_invalid_next_bid_min_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_english_common_data::<Test>(ALICE);

		common_auction_data.next_bid_min = 0;
		let auction =
			mocked_english_auction_object::<Test>(common_auction_data.clone(), mocked_english_specific_data::<Test>());

		// next_bid_min is below BidMinAmount
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);

		common_auction_data.next_bid_min = 10;
		let auction =
			mocked_english_auction_object::<Test>(common_auction_data.clone(), mocked_english_specific_data::<Test>());

		// next_bid_min cannot be set when reserve_price is None
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);

		common_auction_data.reserve_price = Some(20);
		let auction =
			mocked_english_auction_object::<Test>(common_auction_data, mocked_english_specific_data::<Test>());

		// next_bid_min cannot be different from reserve_price
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);
	});
}

/// Error EmptyAuctionName
#[test]
fn create_english_auction_with_empty_name_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_english_common_data::<Test>(ALICE);
		common_auction_data.name = to_bounded_name(b"".to_vec()).unwrap();

		let auction =
			mocked_english_auction_object::<Test>(common_auction_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::EmptyAuctionName
		);
	});
}

/// Error NotATokenOwner
#[test]
fn create_english_auction_when_sender_not_token_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let common_auction_data = mocked_english_common_data::<Test>(ALICE);

		let auction =
			mocked_english_auction_object::<Test>(common_auction_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(BOB), auction),
			Error::<Test>::NotATokenOwner
		);
	});
}

/// Error NotATokenOwner
#[test]
fn create_english_auction_when_not_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_english_common_data::<Test>(ALICE);
		common_auction_data.owner = BOB;

		let auction =
			mocked_english_auction_object::<Test>(common_auction_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::NotAuctionOwner
		);
	});
}

/// Error CannotSetAuctionClosed
#[test]
fn create_english_auction_with_closed_true_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_english_common_data::<Test>(ALICE);
		common_auction_data.closed = true;

		let auction =
			mocked_english_auction_object::<Test>(common_auction_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::CannotSetAuctionClosed
		);
	});
}

/// Error TokenFrozen
#[test]
fn create_english_auction_with_frozen_token_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::TokenFrozen
		);
	});
	// TODO test frozen NFT transfer
}

/// Updating an English auction
///
/// Happy path
#[test]
fn update_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(3);

		let mut updated_common_data = mocked_english_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction_data = EnglishAuction {
			common_data: updated_common_data,
			specific_data: mocked_english_specific_data::<Test>(),
		};
		let auction = Auction::English(auction_data);

		assert_ok!(AuctionsModule::update(Origin::signed(ALICE), 0, auction.clone()));

		expect_events(vec![mock::Event::Auctions(
			pallet::Event::<Test>::AuctionUpdated {
				auction_id: 0,
				auction: auction,
			}
			.into(),
		)]);

		let auction_result = AuctionsModule::auctions(0).unwrap();

		let auction_check = match auction_result {
			Auction::English(data) => {
				assert_eq!(
					String::from_utf8(data.common_data.name.to_vec()).unwrap(),
					"Auction renamed"
				);

				Ok(())
			}
			_ => Err(()),
		};

		assert_ok!(auction_check);
	});
}

/// Error AuctionDoesNotExist
#[test]
fn update_english_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::AuctionDoesNotExist,
		);
	});
}

/// Error InvalidNextBidMin
#[test]
fn update_english_auction_with_invalid_next_bid_min_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_english_common_data::<Test>(ALICE);
		updated_common_data.next_bid_min = 0;

		let auction =
			mocked_english_auction_object::<Test>(updated_common_data.clone(), mocked_english_specific_data::<Test>());

		// next_bid_min is below BidMinAmount
		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::InvalidNextBidMin
		);

		updated_common_data.next_bid_min = 10;

		let auction =
			mocked_english_auction_object::<Test>(updated_common_data.clone(), mocked_english_specific_data::<Test>());

		// next_bid_min is set while reserve_price is None
		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::InvalidNextBidMin
		);

		updated_common_data.reserve_price = Some(20);
		let auction =
			mocked_english_auction_object::<Test>(updated_common_data, mocked_english_specific_data::<Test>());

		// next_bid_min != reserve_price
		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::InvalidNextBidMin
		);
	});
}

/// Error CannotSetAuctionClosed
#[test]
fn update_english_auction_with_closed_true_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_english_common_data::<Test>(ALICE);
		updated_common_data.closed = true;

		let auction =
			mocked_english_auction_object::<Test>(updated_common_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::CannotSetAuctionClosed,
		);
	});
}

/// Error NotAuctionOwner
#[test]
fn update_english_auction_by_non_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_english_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction =
			mocked_english_auction_object::<Test>(updated_common_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(BOB), 0, auction),
			Error::<Test>::NotAuctionOwner,
		);
	});
}

/// Error CannotChangeForbiddenAttribute
#[test]
fn update_english_auction_change_token_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_english_common_data::<Test>(ALICE);
		updated_common_data.token = mocked_nft_token_2::<Test>();

		let auction =
			mocked_english_auction_object::<Test>(updated_common_data, mocked_english_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::CannotChangeForbiddenAttribute,
		);
	});
}

/// Error AuctionAlreadyStarted
#[test]
fn update_english_auction_after_auction_start_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_english_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction =
			mocked_english_auction_object::<Test>(updated_common_data, mocked_english_specific_data::<Test>());

		set_block_number::<Test>(10);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

/// Error NoChangeOfAuctionType
#[test]
fn update_english_auction_with_mismatching_types_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_topup_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = mocked_topup_auction_object::<Test>(updated_common_data, mocked_topup_specific_data::<Test>());

		set_block_number::<Test>(5);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::NoChangeOfAuctionType,
		);
	});
}

///
/// Destroying an English auction
///
/// Happy path
///
#[test]
fn destroy_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(3);

		assert_ok!(AuctionsModule::destroy(Origin::signed(ALICE), 0));

		assert_eq!(AuctionsModule::auctions(0), None);
		assert_eq!(AuctionsModule::auction_owner_by_id(0), None);

		expect_events(vec![mock::Event::Auctions(
			pallet::Event::<Test>::AuctionDestroyed { auction_id: 0 }.into(),
		)]);

		// NFT can be transferred
		assert_ok!(Nft::transfer(
			Origin::signed(ALICE),
			mocked_nft_class_id_1::<Test>(),
			mocked_nft_instance_id_1::<Test>(),
			CHARLIE
		));
		assert_ok!(Nft::transfer(
			Origin::signed(CHARLIE),
			mocked_nft_class_id_1::<Test>(),
			mocked_nft_instance_id_1::<Test>(),
			ALICE
		));
	});
}

/// Error AuctionDoesNotExist
#[test]
fn destroy_english_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionDoesNotExist,
		);
	});
}

/// Error NotAuctionOwner
#[test]
fn destroy_english_auction_by_non_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		assert_noop!(
			AuctionsModule::destroy(Origin::signed(BOB), 0),
			Error::<Test>::NotAuctionOwner,
		);
	});
}

/// Error AuctionAlreadyStarted
#[test]
fn destroy_english_auction_after_auction_started_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(10);

		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

/// Bidding on an English auction
///
/// Happy path with 2 bidders
#[test]
fn bid_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		// First highest bidder
		set_block_number::<Test>(11);

		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_before = Balances::free_balance(&BOB);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_after = Balances::free_balance(&BOB);

		// The bid amount is reserved
		assert_eq!(
			AuctionsModule::reserved_amounts(BOB, 0),
			BalanceOf::<Test>::from(1_000_u32)
		);
		assert_eq!(
			auction_subaccount_balance_before.saturating_add(1_000),
			auction_subaccount_balance_after
		);
		assert_eq!(bob_balance_before.saturating_sub(1_000), bob_balance_after);

		expect_events(vec![
			mock::Event::Auctions(
				pallet::Event::<Test>::BidPlaced {
					auction_id: 0,
					bidder: BOB,
					bid: bid_object(1_000, 11),
				}
				.into(),
			),
			mock::Event::Auctions(
				pallet::Event::<Test>::BidAmountReserved {
					auction_id: 0,
					bidder: BOB,
					amount: BalanceOf::<Test>::from(1_000_u32),
				}
				.into(),
			),
		]);

		// Second highest bidder
		set_block_number::<Test>(12);

		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_before = Balances::free_balance(&BOB);
		let charlie_balance_before = Balances::free_balance(&CHARLIE);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_after = Balances::free_balance(&BOB);
		let charlie_balance_after = Balances::free_balance(&CHARLIE);

		// Previous bid amount is unreserved
		assert_eq!(AuctionsModule::reserved_amounts(BOB, 0), BalanceOf::<Test>::from(0_u32));
		assert_eq!(bob_balance_before.saturating_add(1_000), bob_balance_after);

		// New bid amount is reserved
		assert_eq!(
			AuctionsModule::reserved_amounts(CHARLIE, 0),
			BalanceOf::<Test>::from(1_100_u32)
		);
		assert_eq!(
			auction_subaccount_balance_before.saturating_add(100),
			auction_subaccount_balance_after
		);
		assert_eq!(charlie_balance_before.saturating_sub(1_100), charlie_balance_after);

		expect_events(vec![
			mock::Event::Auctions(
				pallet::Event::<Test>::BidAmountUnreserved {
					auction_id: 0,
					bidder: BOB,
					amount: BalanceOf::<Test>::from(1_000_u32),
					beneficiary: BOB,
				}
				.into(),
			),
			mock::Event::Auctions(
				pallet::Event::<Test>::BidAmountReserved {
					auction_id: 0,
					bidder: CHARLIE,
					amount: BalanceOf::<Test>::from(1_100_u32),
				}
				.into(),
			),
			mock::Event::Auctions(
				pallet::Event::<Test>::BidPlaced {
					auction_id: 0,
					bidder: CHARLIE,
					bid: bid_object(1100, 12),
				}
				.into(),
			),
		]);

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::English(data) => {
				// Next bid step is updated
				assert_eq!(data.common_data.next_bid_min, 1210);

				// Auction time is extended with 1 block when end time is less than 10 blocks away
				assert_eq!(data.common_data.end, 22u64);

				Ok(())
			}
			_ => Err(()),
		};

		assert_ok!(auction_check);
	});
}

/// Error AuctionNotStarted
#[test]
fn bid_english_auction_before_auction_start_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(9);

		assert_noop!(
			AuctionsModule::bid(Origin::signed(BOB), 0, BalanceOf::<Test>::from(2_000_u32)),
			Error::<Test>::AuctionNotStarted,
		);
	});
}

/// Error CannotBidOnOwnAuction
#[test]
fn bid_english_auction_by_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		assert_noop!(
			AuctionsModule::bid(Origin::signed(ALICE), 0, BalanceOf::<Test>::from(2_000_u32)),
			Error::<Test>::CannotBidOnOwnAuction,
		);
	});
}

/// Error InvalidBidPrice when bid is zero
#[test]
fn bid_english_auction_with_bid_amount_zero_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		assert_noop!(
			AuctionsModule::bid(Origin::signed(BOB), 0, BalanceOf::<Test>::zero()),
			Error::<Test>::InvalidBidPrice,
		);
	});
}

/// Error InvalidBidPrice when second bid <= last_bid
#[test]
fn bid_english_auction_with_second_bid_below_first_bid_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		// First bid
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(12);

		assert_noop!(
			AuctionsModule::bid(Origin::signed(CHARLIE), 0, BalanceOf::<Test>::from(1_099_u32)),
			Error::<Test>::InvalidBidPrice,
		);
	});
}

/// Error AuctionEndTimeReached
#[test]
fn bid_english_auction_after_auction_end_time_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(22);

		assert_noop!(
			AuctionsModule::bid(Origin::signed(BOB), 0, BalanceOf::<Test>::from(2_000_u32)),
			Error::<Test>::AuctionEndTimeReached,
		);
	});
}

/// Closing an English auction
///
/// Happy path
#[test]
fn close_english_auction_with_winner_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		let bid = BalanceOf::<Test>::from(1_000_u32);
		assert_ok!(AuctionsModule::bid(Origin::signed(BOB), 0, bid));

		set_block_number::<Test>(21);

		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));
		let alice_balance_before = Balances::free_balance(&ALICE);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let alice_balance_after = Balances::free_balance(&ALICE);

		// The auction winner is the new owner of the NFT
		assert_eq!(
			Nft::owner(mocked_nft_class_id_1::<Test>(), mocked_nft_instance_id_1::<Test>()),
			Some(BOB)
		);

		assert_eq!(alice_balance_before.saturating_add(bid), alice_balance_after);
		assert_eq!(
			auction_subaccount_balance_before.saturating_sub(bid),
			auction_subaccount_balance_after
		);

		expect_events(vec![mock::Event::Auctions(
			pallet::Event::<Test>::AuctionClosed {
				auction_id: 0,
				auction_winner: Some(BOB),
			}
			.into(),
		)]);

		set_block_number::<Test>(22);

		// Auction data is destroyed
		assert!(matches!(AuctionsModule::auctions(0), None));
		assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
	});
}

#[test]
fn close_english_auction_without_winner_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(21);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		expect_events(vec![mock::Event::Auctions(
			pallet::Event::<Test>::AuctionClosed {
				auction_id: 0,
				auction_winner: None,
			}
			.into(),
		)]);

		set_block_number::<Test>(22);

		// Auction data is destroyed
		assert!(matches!(AuctionsModule::auctions(0), None));
		assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
	});
}

/// Error AuctionEndTimeNotReached
#[test]
fn close_english_auction_before_auction_end_time_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		assert_noop!(
			AuctionsModule::close(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionEndTimeNotReached,
		);
	});
}

/// Error AuctionClosed
#[test]
fn close_english_auction_which_is_already_closed_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_english_auction_object::<Test>(
			mocked_english_common_data::<Test>(ALICE),
			mocked_english_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(21);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		set_block_number::<Test>(23);

		assert_noop!(
			AuctionsModule::close(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionDoesNotExist,
		);
	});
}
