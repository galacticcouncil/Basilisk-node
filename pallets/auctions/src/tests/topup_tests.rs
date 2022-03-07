mod topup_tests {
  use super::super::*;
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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      expect_event(crate::Event::<Test>::AuctionCreated(ALICE, 0));

      let auction = AuctionsModule::auctions(0).unwrap();
      let auction_check = match auction {
        Auction::TopUp(data) => {
          assert_eq!(String::from_utf8(data.general_data.name.to_vec()).unwrap(), "Auction 0");
          assert_eq!(data.general_data.reserve_price, None);
          assert_eq!(data.general_data.last_bid, None);
          assert_eq!(data.general_data.start, 10u64);
          assert_eq!(data.general_data.end, 21u64);
          assert_eq!(data.general_data.owner, ALICE);
          assert_eq!(data.general_data.token, (NFT_CLASS_ID_1, NFT_INSTANCE_ID_1));
          assert_eq!(data.general_data.next_bid_min, 1);

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
  fn create_topup_auction_without_end_time_should_not_work() {
    predefined_test_ext().execute_with(|| {
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.end = 0u64;

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.end = 20u64;

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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
      let mut general_auction_data = valid_general_auction_data();

      general_auction_data.next_bid_min = 0;
      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.name = to_bounded_name(b"".to_vec()).unwrap();

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

      assert_noop!(
        AuctionsModule::create(Origin::signed(ALICE), auction),
        Error::<Test>::EmptyAuctionName
      );
    });
  }

  /// Error NotATokenOwner
  #[test]
  fn create_topup_auction_when_not_token_owner_should_not_work() {
    predefined_test_ext().execute_with(|| {
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.owner = BOB;

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

      assert_noop!(
        AuctionsModule::create(Origin::signed(ALICE), auction),
        Error::<Test>::NotATokenOwner
      );
    });
  }

  /// Error CannotSetAuctionClosed
  #[test]
  fn create_topup_auction_with_closed_true_should_not_work() {
    predefined_test_ext().execute_with(|| {
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.closed = true;

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      set_block_number::<Test>(3);

      let mut updated_general_data = valid_general_auction_data();
      updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

      let auction_data = TopUpAuction {
        general_data: updated_general_data,
        specific_data: valid_topup_specific_data(),
      };
      let auction = Auction::TopUp(auction_data);

      assert_ok!(AuctionsModule::update(Origin::signed(ALICE), 0, auction));

      let auction_result = AuctionsModule::auctions(0).unwrap();
      let auction_check = match auction_result {
        Auction::TopUp(data) => {
          assert_eq!(
            String::from_utf8(data.general_data.name.to_vec()).unwrap(),
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
  fn update_topup_auction_with_nonexisting_auction_should_not_work() {
    predefined_test_ext().execute_with(|| {
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      let mut updated_general_data = valid_general_auction_data();
      updated_general_data.next_bid_min = 0;

      let auction = topup_auction_object(updated_general_data, valid_topup_specific_data());

      // next_bid_min is below BidMinAmount
      assert_noop!(
        AuctionsModule::create(Origin::signed(ALICE), auction),
        Error::<Test>::InvalidNextBidMin
      );
    });
  }

  /// Error CannotSetAuctionClosed
  #[test]
  fn update_topup_auction_with_closed_true_should_not_work() {
    predefined_test_ext().execute_with(|| {
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      let mut updated_general_data = valid_general_auction_data();
      updated_general_data.closed = true;

      let auction = topup_auction_object(updated_general_data, valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      let mut updated_general_data = valid_general_auction_data();
      updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

      let auction = topup_auction_object(updated_general_data, valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      let mut updated_general_data = valid_general_auction_data();
      updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

      let auction = topup_auction_object(updated_general_data, valid_topup_specific_data());

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
      let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      let mut updated_general_data = valid_general_auction_data();
      updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

      let auction = topup_auction_object(updated_general_data, valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      set_block_number::<Test>(3);

      assert_ok!(AuctionsModule::destroy(Origin::signed(ALICE), 0));

      assert_eq!(AuctionsModule::auctions(0), None);
      assert_eq!(AuctionsModule::auction_owner_by_id(0), None);

      expect_event(crate::Event::<Test>::AuctionDestroyed(0));

      // NFT can be transferred
      assert_ok!(Nft::transfer(
        Origin::signed(ALICE),
        NFT_CLASS_ID_1,
        NFT_INSTANCE_ID_1,
        CHARLIE
      ));
      assert_ok!(Nft::transfer(
        Origin::signed(CHARLIE),
        NFT_CLASS_ID_1,
        NFT_INSTANCE_ID_1,
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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

      assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

      assert_noop!(
        AuctionsModule::destroy(Origin::signed(BOB), 0),
        Error::<Test>::NotAuctionOwner,
      );
    });
  }

  /// Error AuctionAlreadyStarted
  #[test]
  fn destroy_topup_auction_after_auction_started_should_not_work() {
    predefined_test_ext().execute_with(|| {
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

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

      // The bid amount is transferred to the auction subaccount
      assert_eq!(
        auction_subaccount_balance_before.saturating_add(1_000),
        auction_subaccount_balance_after
      );
      assert_eq!(bob_balance_before.saturating_sub(1_000), bob_balance_after);

      // Second bidder
      set_block_number::<Test>(12);

      assert_ok!(AuctionsModule::bid(
        Origin::signed(CHARLIE),
        0,
        BalanceOf::<Test>::from(1_100_u32)
      ));
      expect_event(crate::Event::<Test>::BidPlaced(0, CHARLIE, bid_object(1100, 12)));

      let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
      let charlie_balance_after = Balances::free_balance(&CHARLIE);

      // The difference between bid amount and last bid is transferred to the auction subaccount
      assert_eq!(
        auction_subaccount_balance_before.saturating_add(2_100),
        auction_subaccount_balance_after
      );
      assert_eq!(charlie_balance_before.saturating_sub(1_100), charlie_balance_after);

      // Third bidder = First bidder
      set_block_number::<Test>(14);

      assert_ok!(AuctionsModule::bid(
        Origin::signed(BOB),
        0,
        BalanceOf::<Test>::from(1_500_u32)
      ));
      expect_event(crate::Event::<Test>::BidPlaced(0, BOB, bid_object(1500, 14)));

      let auction_subaccount_balance_after = Balances::free_balance(&get_auction_subaccount_id(0));
      let bob_balance_after = Balances::free_balance(&BOB);

      // The difference between bid amount and last bid is transferred to the auction subaccount
      assert_eq!(
        auction_subaccount_balance_before.saturating_add(3_600),
        auction_subaccount_balance_after
      );
      assert_eq!(bob_balance_before.saturating_sub(2_500), bob_balance_after);

      let auction = AuctionsModule::auctions(0).unwrap();
      let auction_check = match auction {
        Auction::TopUp(data) => {
          // Next bid step is updated
          assert_eq!(data.general_data.next_bid_min, 1650);

          // Auction time is extended with 1 block when end time is less than 10 blocks away
          assert_eq!(data.general_data.end, 24u64);

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

  /// Closing a TopUp auction
  ///
  /// Happy path
  #[test]
  fn close_topup_auction_with_winner_should_work() {
    predefined_test_ext().execute_with(|| {
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

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
      assert_eq!(Nft::owner(NFT_CLASS_ID_1, NFT_INSTANCE_ID_1), Some(CHARLIE));

      // Auction data is destroyed
      assert!(matches!(AuctionsModule::auctions(0), None));
      assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
    });
  }

  #[test]
  fn close_topup_auction_without_winner_should_work() {
    predefined_test_ext().execute_with(|| {
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.reserve_price = Some(1_500);

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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
      assert_eq!(Nft::owner(NFT_CLASS_ID_1, NFT_INSTANCE_ID_1), Some(ALICE));

      // Auction data is not destroyed
      let auction = AuctionsModule::auctions(0).unwrap();
      let auction_check = match auction {
        Auction::TopUp(data) => {
          assert!(data.general_data.closed);

          Ok(())
        },
        _ => Err(())
      };

      assert_ok!(auction_check);
    });
  }

  #[test]
  fn close_topup_auction_without_bidders_should_work() {
    predefined_test_ext().execute_with(|| {
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.reserve_price = Some(1_500);

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

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
      let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

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
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.reserve_price = Some(1_500);

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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

      // Auction data is destroyed
      assert!(matches!(AuctionsModule::auctions(0), None));
      assert!(matches!(AuctionsModule::auction_owner_by_id(0), None));
    });
  }

  #[test]
  fn claim_topup_auction_with_winner_should_not_work() {
    predefined_test_ext().execute_with(|| {
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.reserve_price = Some(1_500);

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.reserve_price = Some(1_500);

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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
      let mut general_auction_data = valid_general_auction_data();
      general_auction_data.reserve_price = Some(1_500);

      let auction = topup_auction_object(general_auction_data, valid_topup_specific_data());

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

}