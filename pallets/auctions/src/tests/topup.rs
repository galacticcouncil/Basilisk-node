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

// -------------- TopUp auction tests -------------- //
///
/// Creating a TopUp auction
///
/// Happy path
///
#[test]
fn create_topup_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction.clone()));

		expect_events(vec![mock::Event::Auctions(pallet::Event::<Test>::AuctionCreated {
			auction_id: 0,
			auction,
		})]);

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::TopUp(data) => {
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

/// Error InvalidTimeConfiguration
#[test]
fn create_topup_auction_without_end_time_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.end = 0u64;

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);
	});
}

/// Error InvalidTimeConfiguration (duration too short)
#[test]
fn create_topup_auction_with_duration_shorter_than_minimum_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.end = 20u64;

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);
	});
}

/// Error InvalidNextBidMin
#[test]
fn create_topup_auction_with_invalid_next_bid_min_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);

		common_auction_data.next_bid_min = 0;
		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		// next_bid_min is below BidMinAmount
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);
	});
}

/// Error EmptyAuctionName
#[test]
fn create_topup_auction_with_empty_name_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.name = to_bounded_name(b"".to_vec()).unwrap();

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::EmptyAuctionName
		);
	});
}

/// Error NotATokenOwner
#[test]
fn create_topup_auction_when_sender_not_token_owner_should_not_work() {
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
fn create_topup_auction_when_not_token_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.owner = BOB;

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::NotAuctionOwner
		);
	});
}

/// Error CannotSetAuctionClosed
#[test]
fn create_topup_auction_with_closed_true_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.closed = true;

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::CannotSetAuctionClosed
		);
	});
}

/// Error TokenFrozen
#[test]
fn create_topup_auction_with_frozen_token_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::TokenFrozen
		);
	});
	// TODO test frozen NFT transfer
}

///
/// Updating a TopUp auction
///
/// Happy path
///
#[test]
fn update_topup_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(3);

		let mut updated_common_data = mocked_topup_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction_data = TopUpAuction {
			common_data: updated_common_data,
			specific_data: mocked_topup_specific_data::<Test>(),
		};
		let auction = Auction::TopUp(auction_data);

		assert_ok!(AuctionsModule::update(Origin::signed(ALICE), 0, auction.clone()));

		expect_events(vec![mock::Event::Auctions(pallet::Event::<Test>::AuctionUpdated {
			auction_id: 0,
			auction,
		})]);

		let auction_result = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction_result {
			Auction::TopUp(data) => {
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
fn update_topup_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::AuctionDoesNotExist,
		);
	});
}

/// Error InvalidNextBidMin
#[test]
fn update_topup_auction_with_invalid_next_bid_min_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_topup_common_data::<Test>(ALICE);
		updated_common_data.next_bid_min = 0;

		let auction = mocked_topup_auction_object::<Test>(updated_common_data, mocked_topup_specific_data::<Test>());

		// next_bid_min is below BidMinAmount
		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::InvalidNextBidMin
		);
	});
}

/// Error CannotSetAuctionClosed
#[test]
fn update_topup_auction_with_closed_true_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_topup_common_data::<Test>(ALICE);
		updated_common_data.closed = true;

		let auction = mocked_topup_auction_object::<Test>(updated_common_data, mocked_topup_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::CannotSetAuctionClosed,
		);
	});
}

/// Error NotAuctionOwner
#[test]
fn update_topup_auction_by_non_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_topup_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = mocked_topup_auction_object::<Test>(updated_common_data, mocked_topup_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(BOB), 0, auction),
			Error::<Test>::NotAuctionOwner,
		);
	});
}

/// Error AuctionAlreadyStarted
#[test]
fn update_topup_auction_after_auction_start_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_topup_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = mocked_topup_auction_object::<Test>(updated_common_data, mocked_topup_specific_data::<Test>());

		set_block_number::<Test>(10);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

/// Error NoChangeOfAuctionType
#[test]
fn update_topup_auction_with_mismatching_types_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_english_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction =
			mocked_english_auction_object::<Test>(updated_common_data, mocked_english_specific_data::<Test>());

		set_block_number::<Test>(5);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::NoChangeOfAuctionType,
		);
	});
}

///
/// Destroying a TopUp auction
///
/// Happy path
///
#[test]
fn destroy_topup_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(3);

		assert_ok!(AuctionsModule::destroy(Origin::signed(ALICE), 0));

		assert_eq!(AuctionsModule::auctions(0), None);
		assert_eq!(AuctionsModule::auction_owner_by_id(0), None);

		expect_events(vec![mock::Event::Auctions(pallet::Event::<Test>::AuctionDestroyed {
			auction_id: 0,
		})]);

		// NFT can be transferred
		assert_ok!(Nft::transfer(
			Origin::signed(ALICE),
			mocked_nft_collection_id_1::<Test>(),
			mocked_nft_item_id_1::<Test>(),
			CHARLIE
		));
		assert_ok!(Nft::transfer(
			Origin::signed(CHARLIE),
			mocked_nft_collection_id_1::<Test>(),
			mocked_nft_item_id_1::<Test>(),
			ALICE
		));
	});
}

/// Error AuctionDoesNotExist
#[test]
fn destroy_topup_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionDoesNotExist,
		);
	});
}

/// Error NotAuctionOwner
#[test]
fn destroy_topup_auction_by_non_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		assert_noop!(
			AuctionsModule::destroy(Origin::signed(BOB), 0),
			Error::<Test>::NotAuctionOwner,
		);
	});
}

/// Error CannotChangeForbiddenAttribute
#[test]
fn update_topup_auction_change_token_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_topup_common_data::<Test>(ALICE);
		updated_common_data.token = mocked_nft_token_2::<Test>();

		let auction = mocked_topup_auction_object::<Test>(updated_common_data, mocked_topup_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::CannotChangeForbiddenAttribute,
		);
	});
}

/// Error AuctionAlreadyStarted
#[test]
fn destroy_topup_auction_after_auction_started_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(10);

		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

/// Bidding on a TopUp auction
///
/// Happy path with 2 bidders
#[test]
fn bid_topup_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_before = Balances::free_balance(&BOB);
		let charlie_balance_before = Balances::free_balance(&CHARLIE);

		// First bidder
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
			mock::Event::Auctions(pallet::Event::<Test>::BidAmountReserved {
				auction_id: 0,
				bidder: BOB,
				amount: BalanceOf::<Test>::from(1_000_u32),
			}),
			mock::Event::Auctions(pallet::Event::<Test>::BidPlaced {
				auction_id: 0,
				bidder: BOB,
				bid: bid_object(1_000, 11),
			}),
		]);

		// Second bidder
		set_block_number::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let charlie_balance_after = Balances::free_balance(&CHARLIE);

		// The second bid is reserved
		assert_eq!(
			AuctionsModule::reserved_amounts(CHARLIE, 0),
			BalanceOf::<Test>::from(1_100_u32)
		);
		assert_eq!(
			auction_subaccount_balance_before.saturating_add(2_100),
			auction_subaccount_balance_after
		);
		assert_eq!(charlie_balance_before.saturating_sub(1_100), charlie_balance_after);

		expect_events(vec![
			mock::Event::Auctions(pallet::Event::<Test>::BidAmountReserved {
				auction_id: 0,
				bidder: CHARLIE,
				amount: BalanceOf::<Test>::from(1_100_u32),
			}),
			mock::Event::Auctions(pallet::Event::<Test>::BidPlaced {
				auction_id: 0,
				bidder: CHARLIE,
				bid: bid_object(1_100, 12),
			}),
		]);

		// Third bidder = First bidder
		set_block_number::<Test>(14);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_500_u32)
		));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_after = Balances::free_balance(&BOB);

		// The updated bid is reserved
		assert_eq!(
			AuctionsModule::reserved_amounts(BOB, 0),
			BalanceOf::<Test>::from(1_500_u32)
		);
		assert_eq!(
			auction_subaccount_balance_before.saturating_add(2_600),
			auction_subaccount_balance_after
		);
		assert_eq!(bob_balance_before.saturating_sub(1_500), bob_balance_after);

		expect_events(vec![
			mock::Event::Auctions(pallet::Event::<Test>::BidAmountReserved {
				auction_id: 0,
				bidder: BOB,
				amount: BalanceOf::<Test>::from(500_u32),
			}),
			mock::Event::Auctions(pallet::Event::<Test>::BidPlaced {
				auction_id: 0,
				bidder: BOB,
				bid: bid_object(1_500, 14),
			}),
		]);

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::TopUp(data) => {
				// Next bid step is updated
				assert_eq!(data.common_data.next_bid_min, 1650);

				// Auction time is extended with 1 block when end time is less than 10 blocks away
				assert_eq!(data.common_data.end, 24u64);

				Ok(())
			}
			_ => Err(()),
		};

		assert_ok!(auction_check);
	});
}

/// Closing a TopUp auction
///
/// Happy path
#[test]
fn close_topup_auction_with_winner_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		let alice_balance_before = Balances::free_balance(&ALICE);
		let bob_balance_before = Balances::free_balance(&BOB);
		let charlie_balance_before = Balances::free_balance(&CHARLIE);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		set_block_number::<Test>(22);

		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		let alice_balance_after = Balances::free_balance(&ALICE);
		let bob_balance_after = Balances::free_balance(&BOB);
		let charlie_balance_after = Balances::free_balance(&CHARLIE);
		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));

		// transfer all funds from bids to the seller
		assert_eq!(alice_balance_before.saturating_add(2_100), alice_balance_after);
		assert_eq!(bob_balance_before.saturating_sub(1000), bob_balance_after);
		assert_eq!(charlie_balance_before.saturating_sub(1_100), charlie_balance_after);
		assert_eq!(
			auction_subaccount_balance_before.saturating_sub(2_100),
			auction_subaccount_balance_after
		);

		// The auction winner is the new owner of the NFT
		assert_eq!(
			Nft::owner(&mocked_nft_collection_id_1::<Test>(), &mocked_nft_item_id_1::<Test>()),
			Some(CHARLIE)
		);

		expect_events(vec![mock::Event::Auctions(pallet::Event::<Test>::AuctionClosed {
			auction_id: 0,
			auction_winner: Some(CHARLIE),
		})]);

		// Auction data is destroyed
		assert!(matches!(AuctionsModule::auctions(0), None));
		assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
	});
}

#[test]
fn close_topup_auction_without_winner_should_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.reserve_price = Some(1_500);

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		let alice_balance_before = Balances::free_balance(&ALICE);
		let bob_balance_before = Balances::free_balance(&BOB);
		let charlie_balance_before = Balances::free_balance(&CHARLIE);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		set_block_number::<Test>(22);

		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		let alice_balance_after = Balances::free_balance(&ALICE);
		let bob_balance_after = Balances::free_balance(&BOB);
		let charlie_balance_after = Balances::free_balance(&CHARLIE);
		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));

		// the funds are placed in the subaccount of the auction, available to be claimed
		assert_eq!(alice_balance_before, alice_balance_after);
		assert_eq!(bob_balance_before.saturating_sub(1000), bob_balance_after);
		assert_eq!(charlie_balance_before.saturating_sub(1_100), charlie_balance_after);
		assert_eq!(auction_subaccount_balance_before, auction_subaccount_balance_after);

		// The auction winner is the new owner of the NFT
		assert_eq!(
			Nft::owner(&mocked_nft_collection_id_1::<Test>(), &mocked_nft_item_id_1::<Test>()),
			Some(ALICE)
		);

		expect_events(vec![mock::Event::Auctions(pallet::Event::<Test>::AuctionClosed {
			auction_id: 0,
			auction_winner: None,
		})]);

		// Auction data is not destroyed
		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::TopUp(data) => {
				assert!(data.common_data.closed);

				Ok(())
			}
			_ => Err(()),
		};

		assert_ok!(auction_check);
	});
}

#[test]
fn close_topup_auction_without_bidders_should_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.reserve_price = Some(1_500);

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(22);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		// Auction data is destroyed
		assert!(matches!(AuctionsModule::auctions(0), None));
		assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
	});
}

/// Error AuctionEndTimeNotReached
#[test]
fn close_topup_auction_before_auction_end_time_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
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
fn close_topup_auction_which_is_already_closed_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_topup_auction_object::<Test>(
			mocked_topup_common_data::<Test>(ALICE),
			mocked_topup_specific_data::<Test>(),
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

/// Claiming reserved amounts from a TopUp auction
///
/// Happy path
///
///
#[test]
fn claim_topup_auction_without_winner_should_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.reserve_price = Some(1_500);

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		set_block_number::<Test>(22);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		// Bob claims
		let bob_balance_before = Balances::free_balance(&BOB);
		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));

		assert_ok!(AuctionsModule::claim(Origin::signed(BOB), BOB, 0));

		let bob_balance_after = Balances::free_balance(&BOB);
		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));

		assert_eq!(bob_balance_before.saturating_add(1000), bob_balance_after);
		assert_eq!(
			auction_subaccount_balance_before.saturating_sub(1000),
			auction_subaccount_balance_after
		);

		expect_events(vec![
			mock::Event::Auctions(pallet::Event::<Test>::BidAmountUnreserved {
				auction_id: 0,
				bidder: BOB,
				amount: BalanceOf::<Test>::from(1_000_u32),
				beneficiary: BOB,
			}),
			mock::Event::Auctions(pallet::Event::<Test>::BidAmountClaimed {
				auction_id: 0,
				bidder: BOB,
				amount: BalanceOf::<Test>::from(1_000_u32),
			}),
		]);

		// Bob cannot claim twice
		let bob_balance_before = Balances::free_balance(&BOB);
		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));

		assert_noop!(
			AuctionsModule::claim(Origin::signed(BOB), BOB, 0),
			Error::<Test>::NoReservedAmountAvailableToClaim
		);

		let bob_balance_after = Balances::free_balance(&BOB);
		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));

		assert_eq!(bob_balance_before, bob_balance_after);
		assert_eq!(auction_subaccount_balance_before, auction_subaccount_balance_after);

		// Bob claims for Charlie
		let charlie_balance_before = Balances::free_balance(&CHARLIE);

		assert_ok!(AuctionsModule::claim(Origin::signed(BOB), CHARLIE, 0));

		let charlie_balance_after = Balances::free_balance(&CHARLIE);
		assert_eq!(charlie_balance_before.saturating_add(1100), charlie_balance_after);

		let final_auction_account_balance = Balances::free_balance(&get_auction_subaccount_id(0));
		assert_eq!(final_auction_account_balance, Zero::zero());

		expect_events(vec![
			mock::Event::Auctions(pallet::Event::<Test>::BidAmountUnreserved {
				auction_id: 0,
				bidder: CHARLIE,
				amount: BalanceOf::<Test>::from(1_100_u32),
				beneficiary: CHARLIE,
			}),
			mock::Event::Auctions(pallet::Event::<Test>::BidAmountClaimed {
				auction_id: 0,
				bidder: CHARLIE,
				amount: BalanceOf::<Test>::from(1_100_u32),
			}),
		]);

		// Auction data is destroyed
		assert!(matches!(AuctionsModule::auctions(0), None));
		assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
	});
}

#[test]
fn claim_topup_auction_with_winner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.reserve_price = Some(1_500);

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(2_000_u32)
		));

		set_block_number::<Test>(22);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		assert_noop!(
			AuctionsModule::claim(Origin::signed(BOB), BOB, 0),
			Error::<Test>::AuctionDoesNotExist
		);
	});
}

#[test]
fn claim_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			AuctionsModule::claim(Origin::signed(BOB), BOB, 0),
			Error::<Test>::NoReservedAmountAvailableToClaim
		);
	});
}

#[test]
fn claim_running_topup_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.reserve_price = Some(1_500);

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		assert_noop!(
			AuctionsModule::claim(Origin::signed(BOB), BOB, 0),
			Error::<Test>::AuctionEndTimeNotReached
		);
	});
}

#[test]
fn claim_topup_not_closed_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_topup_common_data::<Test>(ALICE);
		common_auction_data.reserve_price = Some(1_500);

		let auction = mocked_topup_auction_object::<Test>(common_auction_data, mocked_topup_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		set_block_number::<Test>(22);

		assert_noop!(
			AuctionsModule::claim(Origin::signed(BOB), BOB, 0),
			Error::<Test>::CloseAuctionBeforeClaimingReservedAmounts
		);
	});
}
