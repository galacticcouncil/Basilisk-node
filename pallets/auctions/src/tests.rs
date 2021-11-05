use super::*;
use crate::{mock::*};
use frame_support::{assert_noop, assert_ok};

pub type AuctionsModule = Pallet<Test>;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn can_create_english_auction() {

  let english_auction_details = EnglishAuctionData {
    reserve_price: 0,
  };

  let valid_auction_info = CommonAuctionData {
    name: "Auction 0".as_bytes().to_vec(),
    last_bid: None,
    start: 10u64,
    end: 21u64,
    owner: ALICE,
    // auction_type: AuctionType::English,
    token: (NFT_CLASS_ID_1, 0u16.into()),
    minimal_bid: 55,
  };

  ExtBuilder::default().build().execute_with(|| {
    assert_ok!(Nft::create_class(Origin::signed(ALICE), NFT_CLASS_ID_1, ALICE, bvec![0]));
    assert_ok!(Nft::mint(Origin::signed(ALICE), NFT_CLASS_ID_1, 0u16.into(), ALICE, 10u8, bvec![0]));
    
    // Error AuctionStartTimeAlreadyPassed
    let mut auction_info = valid_auction_info.clone();
    auction_info.start = 0u64;

    let auction_data = EnglishAuction {
      common_data: auction_info,
      specific_data: english_auction_details.clone(),
    };
    let auction = Auction::English(auction_data);
    assert_noop!(
      AuctionsModule::create_auction(Origin::signed(ALICE), auction),
      Error::<Test>::AuctionStartTimeAlreadyPassed
    );

    // // Error InvalidTimeConfiguration (end is zero)
    // auction_info = valid_auction_info.clone();
    // auction_info.end = 0u64;
    // assert_noop!(
    //   AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
    //   Error::<Test>::InvalidTimeConfiguration
    // );

    // // Error InvalidTimeConfiguration (duration too short)
    // auction_info = valid_auction_info.clone();
    // auction_info.end = 20u64;
    // assert_noop!(
    //   AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
    //   Error::<Test>::InvalidTimeConfiguration
    // );

    // // Error EmptyAuctionName
    // auction_info = valid_auction_info.clone();
    // auction_info.name = "".as_bytes().to_vec();
    // assert_noop!(
    //   AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
    //   Error::<Test>::EmptyAuctionName
    // );

    // // Error NotATokenOwner
    // auction_info = valid_auction_info.clone();
    // auction_info.owner = BOB;
    // assert_noop!(
    //   AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
    //   Error::<Test>::NotATokenOwner
    // );

    // happy path
    let auction_info = valid_auction_info.clone();
    let auction_data = EnglishAuction {
      common_data: auction_info,
      specific_data: english_auction_details.clone()
    };
    let auction = Auction::English(auction_data);

    assert_ok!(AuctionsModule::create_auction(
      Origin::signed(ALICE),
      auction,
    ));

    expect_event(crate::Event::<Test>::AuctionCreated(ALICE, 0));

    let auction = AuctionsModule::auctions(0);

    if let Some(Auction::English(data)) = auction {
      assert_eq!(String::from_utf8(data.common_data.name).unwrap(), "Auction 0");
      assert_eq!(data.common_data.last_bid, None);
      assert_eq!(data.common_data.start, 10u64);
      assert_eq!(data.common_data.end, 21u64);
      assert_eq!(data.common_data.owner, ALICE);
      // assert_eq!(data.common.auction_type, AuctionType::English);
      assert_eq!(data.common_data.token, (NFT_CLASS_ID_1, 0u16.into()));
      assert_eq!(data.common_data.minimal_bid, 55);
    }
    // assert_eq!(AuctionsModule::auction_owner_by_id(0), ALICE);
    // assert_eq!(AuctionsModule::auction_end_time(21u64, 0).unwrap(), ());

    // // Error TokenFrozen
    // assert_noop!(
    //   AuctionsModule::create_auction(
    //     Origin::signed(ALICE),
    //     valid_auction_info.clone(),
    //   ),
    //   Error::<Test>::TokenFrozen
    // );

    // TODO test frozen NFT transfer
  });
}

// #[test]
// fn can_delete_english_auction() {
//   ExtBuilder::default().build().execute_with(|| {
//     assert_ok!(Nft::create_class(Origin::signed(ALICE), NFT_CLASS_ID_1, ALICE, bvec![0]));
//     assert_ok!(Nft::mint(Origin::signed(ALICE), NFT_CLASS_ID_1, 0u16.into(), ALICE, 10u8, bvec![0]));
//     let auction_info = AuctionInfo {
//       name: "Auction 0".as_bytes().to_vec(),
//       last_bid: None,
//       start: 10u64,
//       end: 21u64,
//       owner: ALICE,
//       auction_type: AuctionType::English,
//       token: (NFT_CLASS_ID_1, 0u16.into()),
//       minimal_bid: 55,
//     };

//     // Error AuctionNotExist when auction is not found
//     assert_noop!(
//       AuctionsModule::delete_auction(Origin::signed(ALICE), 0),
//       Error::<Test>::AuctionNotExist,
//     );
    
//     assert_ok!(AuctionsModule::create_auction(Origin::signed(ALICE), auction_info.clone()));

//     System::set_block_number(3);

//     // Error NotAuctionOwner when caller is not owner
//     assert_noop!(
//       AuctionsModule::delete_auction(Origin::signed(BOB), 0),
//       Error::<Test>::NotAuctionOwner,
//     );

//     // Happy path
//     assert_ok!(
//       AuctionsModule::delete_auction(Origin::signed(ALICE), 0)
//     );

//     assert_eq!(AuctionsModule::auctions(0), None);
//     assert_eq!(AuctionsModule::auction_owner_by_id(0), Default::default());

//     expect_event(crate::Event::<Test>::AuctionRemoved(0));

//     // NFT can be transferred
//     assert_ok!(Nft::transfer(Origin::signed(ALICE), NFT_CLASS_ID_1, 0u16.into(), CHARLIE));
//     assert_ok!(Nft::transfer(Origin::signed(CHARLIE), NFT_CLASS_ID_1, 0u16.into(), ALICE));

//     // Error AuctionAlreadyStarted
//     assert_ok!(AuctionsModule::create_auction(Origin::signed(ALICE), auction_info.clone()));
//     System::set_block_number(10);
//     assert_noop!(
//       AuctionsModule::delete_auction(Origin::signed(ALICE), 1),
//       Error::<Test>::AuctionAlreadyStarted,
//     );
//   });
// }
 
// #[test]
// fn can_update_english_auction() {
//   ExtBuilder::default().build().execute_with(|| {
//     assert_ok!(Nft::create_class(Origin::signed(ALICE), NFT_CLASS_ID_1, ALICE, bvec![0]));
//     assert_ok!(Nft::mint(Origin::signed(ALICE), NFT_CLASS_ID_1, 0u16.into(), ALICE, 10u8, bvec![0]));
//     let auction_info = AuctionInfo {
//       name: "Auction 0".as_bytes().to_vec(),
//       last_bid: None,
//       start: 10u64,
//       end: 21u64,
//       owner: ALICE,
//       auction_type: AuctionType::English,
//       token: (NFT_CLASS_ID_1, 0u16.into()),
//       minimal_bid: 55,
//     };

//     let mut update_auction_info = auction_info.clone();
//     update_auction_info.name = "Auction renamed".as_bytes().to_vec();

//     // Error AuctionNotExist
//     assert_noop!(
//       AuctionsModule::update_auction(Origin::signed(ALICE), 0, update_auction_info.clone()),
//       Error::<Test>::AuctionNotExist,
//     );

//     assert_ok!(AuctionsModule::create_auction(Origin::signed(ALICE), auction_info));

//     System::set_block_number(3);

//     // Error NotAuctionOwner when caller is not owner
//     assert_noop!(
//       AuctionsModule::update_auction(Origin::signed(BOB), 0, update_auction_info.clone()),
//       Error::<Test>::NotAuctionOwner,
//     );

//     // Happy path
//     assert_ok!(AuctionsModule::update_auction(Origin::signed(ALICE), 0, update_auction_info.clone()));

//     let auction = AuctionsModule::auctions(0).unwrap();
//     assert_eq!(String::from_utf8(auction.name).unwrap(), "Auction renamed");

//     // Error AuctionAlreadyStarted
//     System::set_block_number(10);
//     assert_noop!(
//       AuctionsModule::update_auction(Origin::signed(ALICE), 0, update_auction_info.clone()),
//       Error::<Test>::AuctionAlreadyStarted,
//     );
//   });
// }

// #[test]
// fn can_bid_value_english_auction() {
//   let auction_0_info = AuctionInfo {
//     name: "Auction 0".as_bytes().to_vec(),
//     last_bid: None,
//     start: 10u64,
//     end: 21u64,
//     owner: ALICE,
//     auction_type: AuctionType::English,
//     token: (NFT_CLASS_ID_1, 0u16.into()),
//     minimal_bid: 0,
//   };

//   ExtBuilder::default().build().execute_with(|| {
//     assert_ok!(Nft::create_class(Origin::signed(ALICE), NFT_CLASS_ID_1, ALICE, bvec![0]));
//     assert_ok!(Nft::mint(Origin::signed(ALICE), NFT_CLASS_ID_1, 0u16.into(), ALICE, 10u8, bvec![0]));

//     // Create auction ID 0 with no minimal_bid and no last_bid
//     assert_ok!(AuctionsModule::create_auction(Origin::signed(ALICE), auction_0_info));

//     // Error BidOnOwnAuction
//     assert_noop!(
//       AuctionsModule::bid_value(Origin::signed(ALICE), 0, BalanceOf::<Test>::from(2_000_u32)),
//       Error::<Test>::BidOnOwnAuction,
//     );

//     // Error AuctionNotStarted
//     assert_noop!(
//       AuctionsModule::bid_value(Origin::signed(BOB), 0, BalanceOf::<Test>::from(2_000_u32)),
//       Error::<Test>::AuctionNotStarted,
//     );

//     System::set_block_number(11);

//     // Error InvalidBidPrice when bid is zero and auction has no minimal_price
//     assert_noop!(
//       AuctionsModule::bid_value(Origin::signed(BOB), 0, BalanceOf::<Test>::zero()),
//       Error::<Test>::InvalidBidPrice,
//     );

//     // Happy path: First highest bidder
//     assert_ok!(AuctionsModule::bid_value(Origin::signed(BOB), 0, BalanceOf::<Test>::from(1_000_u32)));

//     // Tokens of highest bidder are locked
//     assert_noop!(
//       Balances::transfer(Origin::signed(BOB), ALICE, 2_000 * BSX),
//       pallet_balances::Error::<Test>::LiquidityRestrictions
//     );

//     // Error InvalidBidPrice when second bid <= last_bid
//     assert_noop!(
//       AuctionsModule::bid_value(Origin::signed(BOB), 0, BalanceOf::<Test>::from(1_000_u32)),
//       Error::<Test>::InvalidBidPrice,
//     );
    
//     // Error InvalidBidPrice when second bid < minimal_bid (10% above previous bid)
//     assert_noop!(
//       AuctionsModule::bid_value(Origin::signed(CHARLIE), 0, BalanceOf::<Test>::from(1_099_u32)),
//       Error::<Test>::InvalidBidPrice,
//     );
    
//     // Happy path: Second highest bidder
//     System::set_block_number(12);
//     assert_ok!(AuctionsModule::bid_value(Origin::signed(CHARLIE), 0, BalanceOf::<Test>::from(1_100_u32)));
//     expect_event(crate::Event::<Test>::Bid(0, CHARLIE, 1100));

//     // Tokens of previous highest bidder are unlocked
//     assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, 2_000 * BSX));

//     let auction = AuctionsModule::auctions(0).unwrap();
//     // Next bid step is updated
//     assert_eq!(auction.minimal_bid, 1210);

//     // Auction time is extended with 1 block when end time is less than 10 blocks away
//     assert_eq!(auction.end, 22u64);

//     // Error AuctionAlreadyConcluded
//     System::set_block_number(22);
//     assert_noop!(
//       AuctionsModule::bid_value(Origin::signed(BOB), 0, BalanceOf::<Test>::from(2_000_u32)),
//       Error::<Test>::AuctionAlreadyConcluded,
//     );
//   });
// }

// #[test]
// fn can_close_english_auction() {
//   let auction_info = AuctionInfo {
//     name: "Auction 0".as_bytes().to_vec(),
//     last_bid: None,
//     start: 10u64,
//     end: 21u64,
//     owner: ALICE,
//     auction_type: AuctionType::English,
//     token: (NFT_CLASS_ID_1, 0u16.into()),
//     minimal_bid: 0,
//   };

//   ExtBuilder::default().build().execute_with(|| {
//     assert_ok!(Nft::create_class(Origin::signed(ALICE), NFT_CLASS_ID_1, ALICE, bvec![0]));
//     assert_ok!(Nft::mint(Origin::signed(ALICE), NFT_CLASS_ID_1, 0u16.into(), ALICE, 10u8, bvec![0]));
    
//     assert_ok!(AuctionsModule::create_auction(Origin::signed(ALICE), auction_info));
    
//     System::set_block_number(11);
    
//     let bid_value = BalanceOf::<Test>::from(1_000_u32);
//     assert_ok!(AuctionsModule::bid_value(Origin::signed(BOB), 0, bid_value));
    
//     let alice_balance_before = Balances::free_balance(&ALICE);
//     let bob_balance_before = Balances::free_balance(&BOB);
    
//     System::set_block_number(21);
    
//     assert_ok!(AuctionsModule::conclude_auction(21));

//     let alice_balance_after = Balances::free_balance(&ALICE);
//     let bob_balance_after = Balances::free_balance(&BOB);

//     // NFT can be transferred; Current version of nft pallet has no ownership check
//     assert_ok!(Nft::transfer(Origin::signed(BOB), NFT_CLASS_ID_1, 0u16.into(), CHARLIE));

//     assert_eq!(alice_balance_before.saturating_add(bid_value), alice_balance_after);
//     assert_eq!(bob_balance_before.saturating_sub(bid_value), bob_balance_after);
//   });
// }
