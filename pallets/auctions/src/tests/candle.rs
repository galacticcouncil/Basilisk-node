use super::*;
use frame_support::{assert_noop, assert_ok};

// -------------- Candle auction tests -------------- //
///
/// Creating a Candle auction
///
/// Happy path
///
#[test]
fn create_candle_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		expect_event(crate::Event::<Test>::AuctionCreated(ALICE, 0));

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::Candle(data) => {
				assert_eq!(data.common_data.reserve_price, None);
				assert_eq!(data.common_data.last_bid, None);
				assert_eq!(data.common_data.start, 10u64);
				assert_eq!(data.common_data.end, 99_366u64);
				assert_eq!(data.common_data.owner, ALICE);
				assert_eq!(data.common_data.token, mocked_nft_token::<Test>());
				assert_eq!(data.common_data.next_bid_min, 1);
				assert_eq!(data.specific_data.closing_start, 27_366);

				Ok(())
			},
			_ => Err(())
		};

		assert_ok!(auction_check);

		assert_eq!(AuctionsModule::auction_owner_by_id(0), Some(ALICE));
	});
}

/// Error InvalidTimeConfiguration
#[test]
fn create_candle_auction_without_end_time_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_candle_common_data::<Test>(ALICE);
		common_auction_data.end = 0u64;

		let auction = mocked_candle_auction_object(common_auction_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);
	});
}

/// Error InvalidTimeConfiguration (duration too short)
#[test]
fn create_candle_auction_with_duration_shorter_than_minimum_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_candle_common_data::<Test>(ALICE);
		common_auction_data.end = 20u64;

		let auction = mocked_candle_auction_object(common_auction_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);
	});
}

/// Error InvalidNextBidMin
#[test]
fn create_candle_auction_with_invalid_next_bid_min_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_candle_common_data::<Test>(ALICE);

		common_auction_data.next_bid_min = 0;
		let auction = mocked_candle_auction_object(common_auction_data, candle_specific_data::<Test>());

		// next_bid_min is below BidMinAmount
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);
	});
}

/// Error EmptyAuctionName
#[test]
fn create_candle_auction_with_empty_name_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_candle_common_data::<Test>(ALICE);
		common_auction_data.name = to_bounded_name(b"".to_vec()).unwrap();

		let auction = mocked_candle_auction_object(common_auction_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::EmptyAuctionName
		);
	});
}

/// Error NotATokenOwner
#[test]
fn create_candle_auction_when_not_token_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_candle_common_data::<Test>(ALICE);
		common_auction_data.owner = BOB;

		let auction = mocked_candle_auction_object(common_auction_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::NotATokenOwner
		);
	});
}

/// Error CannotSetAuctionClosed
#[test]
fn create_candle_auction_with_closed_true_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_candle_common_data::<Test>(ALICE);
		common_auction_data.closed = true;

		let auction = mocked_candle_auction_object(common_auction_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::CannotSetAuctionClosed
		);
	});
}

/// Error TokenFrozen
#[test]
fn create_candle_auction_with_frozen_token_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::TokenFrozen
		);
	});
	// TODO test frozen NFT transfer
}

/// Error CandleAuctionMustHaveDefaultDuration
#[test]
fn create_candle_auction_with_different_than_default_duration_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_candle_common_data::<Test>(ALICE);
		common_auction_data.end = 99_367;

		let auction = mocked_candle_auction_object(common_auction_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::CandleAuctionMustHaveDefaultDuration
		);
	});
}

/// Error CandleAuctionMustHaveDefaultClosingPeriodDuration
#[test]
fn create_candle_auction_with_different_than_default_closing_period_duration_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut specific_data = candle_specific_data::<Test>();
		specific_data.closing_start = 27_367;

		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), specific_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::CandleAuctionMustHaveDefaultClosingPeriodDuration
		);
	});
}

/// Error CandleAuctionMustHaveDefaultClosingPeriodDuration
#[test]
fn create_candle_auction_with_reserve_price_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut common_auction_data = mocked_candle_common_data::<Test>(ALICE);
		common_auction_data.reserve_price = Some(100);

		let auction = mocked_candle_auction_object(common_auction_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::CandleAuctionDoesNotSupportReservePrice
		);
	});
}

///
/// Updating a Candle auction
///
/// Happy path
///
#[test]
fn update_candle_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(3);

		let mut updated_common_data = mocked_candle_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction_data = CandleAuction {
			common_data: updated_common_data,
			specific_data: candle_specific_data::<Test>(),
		};
		let auction = Auction::Candle(auction_data);

		assert_ok!(AuctionsModule::update(Origin::signed(ALICE), 0, auction));

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::Candle(data) => {
				assert_eq!(
					String::from_utf8(data.common_data.name.to_vec()).unwrap(),
					"Auction renamed"
				);

				Ok(())
			},
			_ => Err(())
		};

		assert_ok!(auction_check);
	});
}

/// Error AuctionDoesNotExist
#[test]
fn update_candle_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::AuctionDoesNotExist,
		);
	});
}

/// Error InvalidNextBidMin
#[test]
fn update_candle_auction_with_invalid_next_bid_min_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_candle_common_data::<Test>(ALICE);
		updated_common_data.next_bid_min = 0;

		let auction = mocked_candle_auction_object(updated_common_data, candle_specific_data::<Test>());

		// next_bid_min is below BidMinAmount
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);
	});
}

/// Error CannotSetAuctionClosed
#[test]
fn update_candle_auction_with_closed_true_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_candle_common_data::<Test>(ALICE);
		updated_common_data.closed = true;

		let auction = mocked_candle_auction_object(updated_common_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::CannotSetAuctionClosed,
		);
	});
}

/// Error NotAuctionOwner
#[test]
fn update_candle_auction_by_non_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_candle_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = mocked_candle_auction_object(updated_common_data, candle_specific_data::<Test>());

		assert_noop!(
			AuctionsModule::update(Origin::signed(BOB), 0, auction),
			Error::<Test>::NotAuctionOwner,
		);
	});
}

/// Error AuctionAlreadyStarted
#[test]
fn update_candle_auction_after_auction_start_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_candle_common_data::<Test>(ALICE);
		updated_common_data.start = 18;
		updated_common_data.end = 99_374;

		let mut updated_specific_data = candle_specific_data::<Test>();
		updated_specific_data.closing_start = 27_374;

		let auction = mocked_candle_auction_object(updated_common_data, updated_specific_data);

		set_block_number::<Test>(15);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

/// Error NoChangeOfAuctionType
#[test]
fn update_candle_auction_with_mismatching_types_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(valid_common_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_common_data = mocked_candle_common_data::<Test>(ALICE);
		updated_common_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = mocked_candle_auction_object(updated_common_data, candle_specific_data::<Test>());

		set_block_number::<Test>(5);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::NoChangeOfAuctionType,
		);
	});
}

///
/// Destroying a Candle auction
///
/// Happy path
///
#[test]
fn destroy_candle_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(3);

		assert_ok!(AuctionsModule::destroy(Origin::signed(ALICE), 0));

		assert_eq!(AuctionsModule::auctions(0), None);
		assert_eq!(AuctionsModule::auction_owner_by_id(0), None);

		expect_event(crate::Event::<Test>::AuctionDestroyed(0));

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
fn destroy_candle_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionDoesNotExist,
		);
	});
}

/// Error NotAuctionOwner
#[test]
fn destroy_candle_auction_by_non_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		assert_noop!(
			AuctionsModule::destroy(Origin::signed(BOB), 0),
			Error::<Test>::NotAuctionOwner,
		);
	});
}

/// Error AuctionAlreadyStarted
#[test]
fn destroy_candle_auction_after_auction_started_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(10);

		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

/// Bidding on a Candle auction
///
/// Happy path with 2 bidders
#[test]
fn bid_candle_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(20);

		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_before = Balances::free_balance(&BOB);
		let charlie_balance_before = Balances::free_balance(&CHARLIE);

		// First bidder, bid before start of closing period
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_after = Balances::free_balance(&BOB);

		// The bid amount is transferred to the auction subaccount
		assert_eq!(
			auction_subaccount_balance_before.saturating_add(1_000),
			auction_subaccount_balance_after
		);
		assert_eq!(bob_balance_before.saturating_sub(1_000), bob_balance_after);

		// The bidder is set as highest bidder for the current range
		assert_eq!(
			AuctionsModule::highest_bidders_by_auction_closing_range(0, 1).unwrap(),
			BOB
		);

		// Second bidder, in the beginning of the closing period
		set_block_number::<Test>(27_368);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));
		expect_event(crate::Event::<Test>::BidPlaced(0, CHARLIE, bid_object(1100, 27_368)));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let charlie_balance_after = Balances::free_balance(&CHARLIE);

		// The difference between bid amount and last bid is transferred to the auction subaccount
		assert_eq!(
			auction_subaccount_balance_before.saturating_add(2_100),
			auction_subaccount_balance_after
		);
		assert_eq!(charlie_balance_before.saturating_sub(1_100), charlie_balance_after);

		// The bidder is set as highest bidder for the current range
		assert_eq!(
			AuctionsModule::highest_bidders_by_auction_closing_range(0, 1).unwrap(),
			CHARLIE
		);

		// Third bidder = First bidder (repeated bid), at the end of the closing period
		set_block_number::<Test>(99_356);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_500_u32)
		));
		expect_event(crate::Event::<Test>::BidPlaced(0, BOB, bid_object(1500, 99_356)));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let bob_balance_after = Balances::free_balance(&BOB);

		// The difference between bid amount and last bid is transferred to the auction subaccount
		assert_eq!(
			auction_subaccount_balance_before.saturating_add(3_600),
			auction_subaccount_balance_after
		);
		assert_eq!(bob_balance_before.saturating_sub(2_500), bob_balance_after);

		// The bidder is set as highest bidder for the current range
		assert_eq!(
			AuctionsModule::highest_bidders_by_auction_closing_range(0, 99).unwrap(),
			BOB
		);

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::Candle(data) => {
				// Next bid step is updated
				assert_eq!(data.common_data.next_bid_min, 1650);

				// Auction time is extended with 1 block when end time is less than 10 blocks away
				assert_eq!(data.common_data.end, 99_366);

				Ok(())
			},
			_ => Err(())
		};

		assert_ok!(auction_check);

		// Bids on TopUp auctions are stored in order to validate the claim_bids extrinsic
		let locked_amount_bob = AuctionsModule::reserved_amounts(BOB, 0);
		assert_eq!(locked_amount_bob, 2500);

		let locked_amount_charlie = AuctionsModule::reserved_amounts(CHARLIE, 0);
		assert_eq!(locked_amount_charlie, 1_100);
	});
}

/// Closing a Candle auction
///
/// Happy path
#[test]
fn close_candle_auction_with_winner_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(20);

		let auction_subaccount_balance_before = Balances::free_balance(&get_auction_subaccount_id(0));
		let alice_balance_before = Balances::free_balance(&ALICE);
		let bob_balance_before = Balances::free_balance(&BOB);
		let charlie_balance_before = Balances::free_balance(&CHARLIE);

		// First bidder, bid before start of closing period
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		// Second bidder, in the beginning of the closing period
		set_block_number::<Test>(27_368);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		// Third bidder = First bidder (repeated bid), at the end of the closing period
		set_block_number::<Test>(99_356);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_500_u32)
		));

		set_block_number::<Test>(99_376);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		set_block_number::<Test>(99_377);

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let alice_balance_after = Balances::free_balance(&ALICE);
		let bob_balance_after = Balances::free_balance(&BOB);
		let charlie_balance_after = Balances::free_balance(&CHARLIE);

		// Alice receives all bids from winner Charlie
		assert_eq!(alice_balance_before.saturating_add(1_100), alice_balance_after);
		assert_eq!(charlie_balance_before.saturating_sub(1_100), charlie_balance_after);

		// Funds of Bob remain reserved in the auction subaccount, available to claim
		assert_eq!(auction_subaccount_balance_before.saturating_add(2_500), auction_subaccount_balance_after);
		assert_eq!(bob_balance_before.saturating_sub(2_500), bob_balance_after);

		// The auction winner is the new owner of the NFT
		assert_eq!(Nft::owner(mocked_nft_class_id_1::<Test>(), mocked_nft_instance_id_1::<Test>()), Some(CHARLIE));

		let auction = AuctionsModule::auctions(0).unwrap();

		let auction_check = match auction {
			Auction::Candle(data) => {
				assert!(data.common_data.closed);
				assert_eq!(data.specific_data.winner.unwrap(), CHARLIE);

				// In the tests the winning closing range is deterministic because the same blocks will always return same randomness
				assert_eq!(data.specific_data.winning_closing_range.unwrap(), 69);

				Ok(())
			},
			_ => Err(())
		};

		assert_ok!(auction_check);
	});
}

#[test]
fn close_candle_auction_with_single_bidder_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(20);

		let alice_balance_before = Balances::free_balance(&ALICE);
		let bob_balance_before = Balances::free_balance(&BOB);

		// First bidder, bid before start of closing period
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(99_377);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
		let alice_balance_after = Balances::free_balance(&ALICE);
		let bob_balance_after = Balances::free_balance(&BOB);

		// Alice receives all auction funds from single bidder BOB
		assert_eq!(alice_balance_before.saturating_add(1_000), alice_balance_after);
		assert_eq!(bob_balance_before.saturating_sub(1_000), bob_balance_after);
		assert!(auction_subaccount_balance_after.is_zero());

		// Auction data is destroyed
		assert!(matches!(AuctionsModule::auctions(0), None));
		assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
		assert!(matches!(AuctionsModule::highest_bidders_by_auction_closing_range(0, 1), None));
	});
}

#[test]
fn close_candle_auction_without_bidders_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(99_377);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		// Auction data is destroyed
		assert!(matches!(AuctionsModule::auctions(0), None));
		assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
		assert!(matches!(AuctionsModule::highest_bidders_by_auction_closing_range(0, 1), None));
	});
}

/// Error AuctionEndTimeNotReached
#[test]
fn close_candle_auction_before_auction_end_time_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(21);

		assert_noop!(
			AuctionsModule::close(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionEndTimeNotReached,
		);
	});
}

/// Error AuctionClosed
#[test]
fn close_candle_auction_which_is_already_closed_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(99_366);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		set_block_number::<Test>(99_367);

		assert_noop!(
			AuctionsModule::close(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionDoesNotExist,
		);
	});
}

/// Claiming reserved amounts from a Candle auction
///
/// Happy path
///
///
#[test]
fn claim_candle_auction_by_losing_bidder_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(20);

		let bob_balance_before = Balances::free_balance(&BOB);

		// First bidder, bid before start of closing period
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		// Second bidder, in the beginning of the closing period
		set_block_number::<Test>(27_368);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		// Third bidder = First bidder (repeated bid), at the end of the closing period
		set_block_number::<Test>(99_356);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_500_u32)
		));

		set_block_number::<Test>(99_376);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		set_block_number::<Test>(99_377);
		
		// Alice claims for Bob
		assert_ok!(AuctionsModule::claim(Origin::signed(ALICE), BOB, 0));

		let bob_balance_after = Balances::free_balance(&BOB);
		let auction_subaccount_balance = Balances::free_balance(&get_auction_subaccount_id(0));

		assert_eq!(bob_balance_before, bob_balance_after);
		assert_eq!(auction_subaccount_balance, Zero::zero());

		// Bob cannot claim twice
		assert_noop!(
			AuctionsModule::claim(Origin::signed(BOB), BOB, 0),
			Error::<Test>::NoReservedAmountAvailableToClaim
		);

		// Auction data is destroyed
		assert!(matches!(AuctionsModule::auctions(0), None));
		assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
		assert!(matches!(AuctionsModule::highest_bidders_by_auction_closing_range(0, 1), None));
	});
}

#[test]
fn claim_candle_auction_by_winning_bidder_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(20);

		// First bidder, bid before start of closing period
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		// Second bidder, in the beginning of the closing period
		set_block_number::<Test>(27_368);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		// Third bidder = First bidder (repeated bid), at the end of the closing period
		set_block_number::<Test>(99_356);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_500_u32)
		));

		set_block_number::<Test>(99_376);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		set_block_number::<Test>(99_377);
		
		// Winner of the auction tries to claim, not possible because the reserved amount was transferred at close
		assert_noop!(
			AuctionsModule::claim(Origin::signed(CHARLIE), CHARLIE, 0),
			Error::<Test>::NoReservedAmountAvailableToClaim
		);
	});
}

#[test]
fn claim_running_candle_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(20);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		set_block_number::<Test>(202);

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
fn claim_candle_not_closed_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = mocked_candle_auction_object(mocked_candle_common_data::<Test>(ALICE), candle_specific_data::<Test>());
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(20);

		// First bidder, bid before start of closing period
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		// Second bidder, in the beginning of the closing period
		set_block_number::<Test>(27_368);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));

		set_block_number::<Test>(99_377);
		
		assert_noop!(
			AuctionsModule::claim(Origin::signed(BOB), BOB, 0),
			Error::<Test>::CloseAuctionBeforeClaimingReservedAmounts
		);
	});
}
