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

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| run_to_block::<Test>(1));
	ext
}

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
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
	});

	ext
}

fn valid_general_auction_data() -> GeneralAuctionData<Test> {
	GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		last_bid: None,
		start: 10u64,
		end: 21u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, 0u16.into()),
		next_bid_min: 55,
	}
}

/// English auction tests
fn english_auction_object(general_data: GeneralAuctionData<Test>, specific_data: EnglishAuctionData) -> Auction<Test> {
	let auction_data = EnglishAuction {
		general_data: general_data, specific_data: specific_data
	};

	Auction::English(auction_data)
}

fn valid_english_specific_data() -> EnglishAuctionData {
	EnglishAuctionData {}
}

/// Creating an English auction
/// 
/// Happy path
#[test]
fn create_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

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
	});
}

/// Error AuctionStartTimeAlreadyPassed
#[test]
fn create_english_auction_starting_in_the_past_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut general_auction_data = valid_general_auction_data();
		general_auction_data.start = 0u64;

		let auction = english_auction_object(
			general_auction_data, valid_english_specific_data()
		);

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
		let mut general_auction_data = valid_general_auction_data();
		general_auction_data.end = 0u64;

		let auction = english_auction_object(
			general_auction_data, valid_english_specific_data()
		);

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
		let mut general_auction_data = valid_general_auction_data();
		general_auction_data.end = 20u64;

		let auction = english_auction_object(
			general_auction_data, valid_english_specific_data()
		);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidTimeConfiguration
		);
	});
}

/// Error EmptyAuctionName
#[test]
fn create_english_auction_with_empty_name_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut general_auction_data = valid_general_auction_data();
		general_auction_data.name = to_bounded_name(b"".to_vec()).unwrap();

		let auction = english_auction_object(
			general_auction_data, valid_english_specific_data()
		);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::EmptyAuctionName
		);
	});
}

/// Error NotATokenOwner
#[test]
fn create_english_auction_when_not_token_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut general_auction_data = valid_general_auction_data();
		general_auction_data.owner = BOB;

		let auction = english_auction_object(
			general_auction_data, valid_english_specific_data()
		);

		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::NotATokenOwner
		);
	});
}

/// Error CannotSetAuctionClosed
#[test]
fn create_english_auction_with_closed_true_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut general_auction_data = valid_general_auction_data();
		general_auction_data.closed = true;

		let auction = english_auction_object(
			general_auction_data, valid_english_specific_data()
		);

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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(3);

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction_data = EnglishAuction {
			general_data: updated_general_data,
			specific_data: valid_english_specific_data(),
		};
		let auction = Auction::English(auction_data);

		assert_ok!(AuctionsModule::update(Origin::signed(ALICE), 0, auction.clone()));

		let auction_result = AuctionsModule::auctions(0).unwrap();
		let Auction::English(data) = auction_result;
		assert_eq!(
			String::from_utf8(data.general_data.name.to_vec()).unwrap(),
			"Auction renamed"
		);
	});
}

/// Error AuctionNotExist
#[test]
fn update_english_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::AuctionNotExist,
		);
	});
}

/// Error CannotSetAuctionClosed
#[test]
fn update_english_auction_with_closed_true_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.closed = true;

		let auction = english_auction_object(
			updated_general_data, valid_english_specific_data()
		);

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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = english_auction_object(
			updated_general_data, valid_english_specific_data()
		);

		assert_noop!(
			AuctionsModule::update(Origin::signed(BOB), 0, auction),
			Error::<Test>::NotAuctionOwner,
		);
	});
}

/// Error AuctionAlreadyStarted
#[test]
fn update_english_auction_after_auction_start_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = english_auction_object(
			updated_general_data, valid_english_specific_data()
		);

		run_to_block::<Test>(10);

		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::AuctionAlreadyStarted,
		);
	});
}

/// Destroying an English auction
/// 
/// Happy path
#[test]
fn destroy_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(3);

		assert_ok!(AuctionsModule::destroy(Origin::signed(ALICE), 0));

		assert_eq!(AuctionsModule::auctions(0), None);
		assert_eq!(AuctionsModule::auction_owner_by_id(0), Default::default());

		expect_event(crate::Event::<Test>::AuctionDestroyed(0));

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
	});
}

/// Error AuctionNotExist
#[test]
fn destroy_english_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			AuctionsModule::destroy(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionNotExist,
		);
	});
}

/// Error NotAuctionOwner
#[test]
fn destroy_english_auction_by_non_auction_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(10);

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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(11);

		// First highest bidder
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

		// Second highest bidder
		run_to_block::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));
		expect_event(crate::Event::<Test>::BidPlaced(0, CHARLIE, 1100));

		// Tokens of previous highest bidder are unlocked
		assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, 2_000 * BSX));

		let auction = AuctionsModule::auctions(0).unwrap();
		let Auction::English(data) = auction;

		// Next bid step is updated
		assert_eq!(data.general_data.next_bid_min, 1210);

		// Auction time is extended with 1 block when end time is less than 10 blocks away
		assert_eq!(data.general_data.end, 22u64);
	});
}

/// Error AuctionNotStarted
#[test]
fn bid_english_auction_before_auction_start_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(10);

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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(11);

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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(11);

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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(11);

		// First bid
		assert_ok!(AuctionsModule::bid(
			Origin::signed(BOB),
			0,
			BalanceOf::<Test>::from(1_000_u32)
		));

		run_to_block::<Test>(12);

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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(22);

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
fn close_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(11);

		let bid = BalanceOf::<Test>::from(1_000_u32);
		assert_ok!(AuctionsModule::bid(Origin::signed(BOB), 0, bid));

		let alice_balance_before = Balances::free_balance(&ALICE);
		let bob_balance_before = Balances::free_balance(&BOB);

		run_to_block::<Test>(21);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		let alice_balance_after = Balances::free_balance(&ALICE);
		let bob_balance_after = Balances::free_balance(&BOB);

		// NFT can be transferred; Current version of nft pallet has no ownership check
		assert_ok!(Nft::transfer(Origin::signed(BOB), NFT_CLASS_ID_1, 0u16.into(), CHARLIE));

		assert_eq!(alice_balance_before.saturating_add(bid), alice_balance_after);
		assert_eq!(bob_balance_before.saturating_sub(bid), bob_balance_after);

		let auction = AuctionsModule::auctions(0).unwrap();
		let Auction::English(data) = auction;

		// Attributed closed is updated
		assert!(data.general_data.closed);
	});
}

/// Error AuctionEndTimeNotReached
#[test]
fn close_english_auction_before_auction_end_time_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(11);

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
		let auction = english_auction_object(
			valid_general_auction_data(), valid_english_specific_data()
		);

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		run_to_block::<Test>(21);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		run_to_block::<Test>(23);

		assert_noop!(
			AuctionsModule::close(Origin::signed(ALICE), 0),
			Error::<Test>::AuctionClosed,
		);
	});
}
