use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_std::convert::TryInto;

pub type AuctionsModule = Pallet<Test>;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn to_bounded_name(name: Vec<u8>) -> Result<BoundedVec<u8, AuctionsStringLimit>, Error<Test>> {
	name.try_into().map_err(|_| Error::<Test>::TooLong)
}

#[test]
fn can_create_english_auction() {
	let english_auction_data = EnglishAuctionData { reserve_price: 0 };

	let valid_common_auction_data = GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		last_bid: None,
		start: 10u64,
		end: 21u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, 0u16.into()),
		next_bid_min: 55,
	};

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			ALICE,
			bvec![0]
		));
		assert_ok!(Nft::mint(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			0u16.into(),
			ALICE,
			10u8,
			bvec![0]
		));

		// Error AuctionStartTimeAlreadyPassed
		let mut common_auction_data = valid_common_auction_data.clone();
		common_auction_data.start = 0u64;

		let auction_data = EnglishAuction {
			general_data: common_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::AuctionStartTimeAlreadyPassed
		);

		// Error InvalidTimeConfiguration (end is zero)
		let mut common_auction_data = valid_common_auction_data.clone();
		common_auction_data.end = 0u64;

		let auction_data = EnglishAuction {
			general_data: common_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);

		// // Error InvalidTimeConfiguration (duration too short)
		let mut common_auction_data = valid_common_auction_data.clone();
		common_auction_data.end = 20u64;

		let auction_data = EnglishAuction {
			general_data: common_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);

		// Error EmptyAuctionName
		let mut common_auction_data = valid_common_auction_data.clone();
		common_auction_data.name = to_bounded_name(b"".to_vec()).unwrap();

		let auction_data = EnglishAuction {
			general_data: common_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::EmptyAuctionName
		);

		// Error NotATokenOwner
		let mut common_auction_data = valid_common_auction_data.clone();
		common_auction_data.owner = BOB;

		let auction_data = EnglishAuction {
			general_data: common_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::NotATokenOwner
		);

		// Error CannotSetAuctionClosed
		let mut common_auction_data = valid_common_auction_data.clone();
		common_auction_data.closed = true;

		let auction_data = EnglishAuction {
			general_data: common_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::CannotSetAuctionClosed
		);

		// happy path
		let auction_data = EnglishAuction {
			general_data: valid_common_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction,));

		expect_event(crate::Event::<Test>::AuctionCreated(ALICE, 0));

		let auction = AuctionsModule::auctions(0).unwrap();

		let Auction::English(data) = auction;
		assert_eq!(String::from_utf8(data.general_data.name.to_vec()).unwrap(), "Auction 0");
		assert_eq!(data.general_data.last_bid, None);
		assert_eq!(data.general_data.start, 10u64);
		assert_eq!(data.general_data.end, 21u64);
		assert_eq!(data.general_data.owner, ALICE);
		assert_eq!(data.general_data.token, (NFT_CLASS_ID_1, 0u16.into()));
		assert_eq!(data.general_data.next_bid_min, 55);

		assert_eq!(AuctionsModule::auction_owner_by_id(0), ALICE);

		// Error TokenFrozen
		let auction_data = EnglishAuction {
			general_data: valid_common_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction,),
			Error::<Test>::TokenFrozen
		);

		// TODO test frozen NFT transfer
	});
}

#[test]
fn can_update_english_auction() {
	let general_auction_data = GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		last_bid: None,
		start: 10u64,
		end: 21u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, 0u16.into()),
		next_bid_min: 55,
	};

	let english_auction_data = EnglishAuctionData { reserve_price: 0 };

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			ALICE,
			bvec![0]
		));
		assert_ok!(Nft::mint(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			0u16.into(),
			ALICE,
			10u8,
			bvec![0]
		));

		let auction_data = EnglishAuction {
			general_data: general_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		// Error AuctionNotExist
		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction.clone()),
			Error::<Test>::AuctionNotExist,
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		System::set_block_number(3);

		// Error CannotSetAuctionClosed
		let mut updated_general_data = general_auction_data.clone();
		updated_general_data.closed = true;

		let auction_data = EnglishAuction {
			general_data: updated_general_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction.clone()),
			Error::<Test>::CannotSetAuctionClosed
		);

		let mut updated_general_data = general_auction_data.clone();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction_data = EnglishAuction {
			general_data: updated_general_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		// Error NotAuctionOwner when caller is not owner
		assert_noop!(
			AuctionsModule::update(Origin::signed(BOB), 0, auction.clone()),
			Error::<Test>::NotAuctionOwner,
		);

		// Happy path
		assert_ok!(AuctionsModule::update(Origin::signed(ALICE), 0, auction.clone()));

		let auction_result = AuctionsModule::auctions(0).unwrap();
		let Auction::English(data) = auction_result;
		assert_eq!(
			String::from_utf8(data.general_data.name.to_vec()).unwrap(),
			"Auction renamed"
		);

		// Error AuctionAlreadyStarted
		System::set_block_number(10);
		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction.clone()),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

#[test]
fn can_destroy_english_auction() {
	let general_auction_data = GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		last_bid: None,
		start: 10u64,
		end: 21u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, 0u16.into()),
		next_bid_min: 55,
	};

	let english_auction_data = EnglishAuctionData { reserve_price: 0 };

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			ALICE,
			bvec![0]
		));
		assert_ok!(Nft::mint(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			0u16.into(),
			ALICE,
			10u8,
			bvec![0]
		));

		let auction_data = EnglishAuction {
			general_data: general_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		// Error AuctionNotExist when auction is not found
		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionNotExist,
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction.clone()));

		System::set_block_number(3);

		// Error NotAuctionOwner when caller is not owner
		assert_noop!(
			AuctionsModule::destroy(Origin::signed(BOB), 0),
			Error::<Test>::NotAuctionOwner,
		);

		// Happy path
		assert_ok!(AuctionsModule::destroy(Origin::signed(ALICE), 0));

		assert_eq!(AuctionsModule::auctions(0), None);
		assert_eq!(AuctionsModule::auction_owner_by_id(0), Default::default());

		expect_event(crate::Event::<Test>::AuctionRemoved(0));

		// NFT can be transferred
		assert_ok!(Nft::transfer(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			0u16.into(),
			CHARLIE
		));
		assert_ok!(Nft::transfer(
			Origin::signed(CHARLIE),
			NFT_CLASS_ID_1,
			0u16.into(),
			ALICE
		));

		// Error AuctionAlreadyStarted
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction.clone()));
		System::set_block_number(10);
		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 1),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

#[test]
fn can_bid_english_auction() {
	let general_auction_data = GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		last_bid: None,
		start: 10u64,
		end: 21u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, 0u16.into()),
		next_bid_min: 55,
	};

	let english_auction_data = EnglishAuctionData { reserve_price: 0 };

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			ALICE,
			bvec![0]
		));
		assert_ok!(Nft::mint(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			0u16.into(),
			ALICE,
			10u8,
			bvec![0]
		));

		let auction_data = EnglishAuction {
			general_data: general_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		// Create auction ID 0 with no next_bid_min and no last_bid
		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		// Error BidOnOwnAuction
		assert_noop!(
			AuctionsModule::bid(Origin::signed(ALICE), 0, BalanceOf::<Test>::from(2_000_u32)),
			Error::<Test>::BidOnOwnAuction,
		);

		// Error AuctionNotStarted
		assert_noop!(
			AuctionsModule::bid(Origin::signed(BOB), 0, BalanceOf::<Test>::from(2_000_u32)),
			Error::<Test>::AuctionNotStarted,
		);

		System::set_block_number(11);

		// Error InvalidBidPrice when bid is zero and auction has no minimal_price
		assert_noop!(
			AuctionsModule::bid(Origin::signed(BOB), 0, BalanceOf::<Test>::zero()),
			Error::<Test>::InvalidBidPrice,
		);

		// Happy path: First highest bidder
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		// Tokens of highest bidder are locked
		assert_noop!(
			Balances::transfer(Origin::signed(BOB), ALICE, 2_000 * BSX),
			pallet_balances::Error::<Test>::LiquidityRestrictions
		);

		// Error InvalidBidPrice when second bid <= last_bid
		assert_noop!(
			AuctionsModule::bid(Origin::signed(BOB), 0, BalanceOf::<Test>::from(1_000_u32)),
			Error::<Test>::InvalidBidPrice,
		);

		// Error InvalidBidPrice when second bid < minimal_bid (10% above previous bid)
		assert_noop!(
			AuctionsModule::bid(Origin::signed(CHARLIE), 0, BalanceOf::<Test>::from(1_099_u32)),
			Error::<Test>::InvalidBidPrice,
		);

		// Happy path: Second highest bidder
		System::set_block_number(12);
		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));
		expect_event(crate::Event::<Test>::Bid(0, CHARLIE, 1100));

		// Tokens of previous highest bidder are unlocked
		assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, 2_000 * BSX));

		let auction = AuctionsModule::auctions(0).unwrap();
		let Auction::English(data) = auction;

		// Next bid step is updated
		assert_eq!(data.general_data.next_bid_min, 1210);

		// Auction time is extended with 1 block when end time is less than 10 blocks away
		assert_eq!(data.general_data.end, 22u64);

		// Error AuctionEndTimeReached
		System::set_block_number(22);
		assert_noop!(
			AuctionsModule::bid(Origin::signed(BOB), 0, BalanceOf::<Test>::from(2_000_u32)),
			Error::<Test>::AuctionEndTimeReached,
		);
	});
}

#[test]
fn can_close_english_auction() {
	let general_auction_data = GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		last_bid: None,
		start: 10u64,
		end: 21u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, 0u16.into()),
		next_bid_min: 55,
	};

	let english_auction_data = EnglishAuctionData { reserve_price: 0 };

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			ALICE,
			bvec![0]
		));
		assert_ok!(Nft::mint(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			0u16.into(),
			ALICE,
			10u8,
			bvec![0]
		));

		let auction_data = EnglishAuction {
			general_data: general_auction_data.clone(),
			specific_data: english_auction_data.clone(),
		};
		let auction = Auction::English(auction_data);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		System::set_block_number(11);

		let bid = BalanceOf::<Test>::from(1_000_u32);
		assert_ok!(AuctionsModule::bid(Origin::signed(BOB), 0, bid));

		let alice_balance_before = Balances::free_balance(&ALICE);
		let bob_balance_before = Balances::free_balance(&BOB);

		// Error AuctionEndTimeNotReached
		assert_noop!(
			AuctionsModule::close(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionEndTimeNotReached,
		);

		// Happy path 1: bid above reserve_price
		System::set_block_number(21);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		let alice_balance_after = Balances::free_balance(&ALICE);
		let bob_balance_after = Balances::free_balance(&BOB);

		// NFT can be transferred; Current version of nft pallet has no ownership check
		// assert_ok!(Nft::transfer(Origin::signed(BOB), NFT_CLASS_ID_1, 0u16.into(), CHARLIE));

		assert_eq!(alice_balance_before.saturating_add(bid), alice_balance_after);
		assert_eq!(bob_balance_before.saturating_sub(bid), bob_balance_after);

		let auction = AuctionsModule::auctions(0).unwrap();
		let Auction::English(data) = auction;

		// Attributed closed is updated
		assert_eq!(data.general_data.closed, true);

		// Error AuctionClosed
		assert_noop!(
			AuctionsModule::close(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionClosed,
		);

		System::set_block_number(22);

		// Happy path 2: bid under reserve_price
		let mut general_data = general_auction_data.clone();
		let mut specific_data = english_auction_data.clone();

		general_data.owner = BOB;
		general_data.start = 23u64;
		general_data.end = 34u64;

		specific_data.reserve_price = 300;

		let auction_data = EnglishAuction {
			general_data: general_data,
			specific_data: specific_data,
		};
		let auction = Auction::English(auction_data);

		// TODO This raises NoPermission
		assert_ok!(AuctionsModule::create(Origin::signed(BOB), auction));

		// TODO finalize after tests
	});
}
