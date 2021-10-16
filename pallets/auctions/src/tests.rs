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
fn can_create_auction() {
  ExtBuilder::default().build().execute_with(|| {
    assert_ok!(Nft::create_class(Origin::signed(ALICE), NFT_CLASS_ID, ALICE, bvec![0]));
    assert_ok!(Nft::mint(Origin::signed(ALICE), NFT_CLASS_ID, 0u16.into(), ALICE, 10u8, bvec![0]));
    
    // start before current block
    let auction_info = AuctionInfo {
      name: "Auction 1s".as_bytes().to_vec(),
      last_bid: None,
      start: 0,
      end: 21u64,
      owner: ALICE,
      auction_type: AuctionType::English,
      token: (NFT_CLASS_ID, 0u16.into()),
      minimal_bid: 55,
    };
    assert_noop!(
      AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
      Error::<Test>::AuctionStartTimeAlreadyPassed
    );

    // start before current block
    let auction_info = AuctionInfo {
      name: "Auction 1s".as_bytes().to_vec(),
      last_bid: None,
      start: 0u64,
      end: 21u64,
      owner: ALICE,
      auction_type: AuctionType::English,
      token: (NFT_CLASS_ID, 0u16.into()),
      minimal_bid: 55,
    };
    assert_noop!(
      AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
      Error::<Test>::AuctionStartTimeAlreadyPassed
    );

    // end is zero
    let auction_info = AuctionInfo {
      name: "Auction 1s".as_bytes().to_vec(),
      last_bid: None,
      start: 1u64,
      end: 0u64,
      owner: ALICE,
      auction_type: AuctionType::English,
      token: (NFT_CLASS_ID, 0u16.into()),
      minimal_bid: 55,
    };
    assert_noop!(
      AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
      Error::<Test>::InvalidTimeConfiguration
    );

    // duration too short
    let auction_info = AuctionInfo {
      name: "Auction 1s".as_bytes().to_vec(),
      last_bid: None,
      start: 10u64,
      end: 20u64,
      owner: ALICE,
      auction_type: AuctionType::English,
      token: (NFT_CLASS_ID, 0u16.into()),
      minimal_bid: 55,
    };
    assert_noop!(
      AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
      Error::<Test>::InvalidTimeConfiguration
    );

    // auction name empty
    let auction_info = AuctionInfo {
      name: "".as_bytes().to_vec(),
      last_bid: None,
      start: 10u64,
      end: 21u64,
      owner: ALICE,
      auction_type: AuctionType::English,
      token: (NFT_CLASS_ID, 0u16.into()),
      minimal_bid: 55,
    };
    assert_noop!(
      AuctionsModule::create_auction(Origin::signed(ALICE), auction_info),
      Error::<Test>::EmptyAuctionName
    );

    // Caller isn't owner
    let auction_info = AuctionInfo {
      name: "".as_bytes().to_vec(),
      last_bid: None,
      start: 10u64,
      end: 21u64,
      owner: BOB,
      auction_type: AuctionType::English,
      token: (NFT_CLASS_ID, 0u16.into()),
      minimal_bid: 55,
    };
    assert_noop!(
      AuctionsModule::create_auction(Origin::signed(BOB), auction_info),
      Error::<Test>::EmptyAuctionName
    );

    // TO DO: test can_transfer

    // happy path
    let auction_info = AuctionInfo {
      name: "Auction 1s".as_bytes().to_vec(),
      last_bid: None,
      start: 10u64,
      end: 21u64,
      owner: ALICE,
      auction_type: AuctionType::English,
      token: (NFT_CLASS_ID, 0u16.into()),
      minimal_bid: 55,
    };

    assert_ok!(AuctionsModule::create_auction(
      Origin::signed(ALICE),
      auction_info,
    ));
  });
}
