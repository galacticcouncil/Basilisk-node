use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use primitives::nft::ClassType;
use sp_core::crypto::AccountId32;
use sp_std::convert::TryInto;

pub type AuctionsModule = Pallet<Test>;
type AccountId = AccountId32;

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
	ext.execute_with(|| set_block_number::<Test>(1));
	ext
}

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			NFT_CLASS_ID_1,
			ClassType::Marketplace,
			bvec![0]
		));
		assert_ok!(Nft::mint(Origin::signed(ALICE), NFT_CLASS_ID_1, NFT_INSTANCE_ID_1, bvec![0]));
	});

	ext
}

fn valid_general_auction_data() -> GeneralAuctionData<Test> {
	GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u64,
		end: 21u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, NFT_INSTANCE_ID_1),
		next_bid_min: 1,
	}
}

/// English auction tests
fn english_auction_object(general_data: GeneralAuctionData<Test>, specific_data: EnglishAuctionData) -> Auction<Test> {
	let auction_data = EnglishAuction {
		general_data,
		specific_data,
	};

	Auction::English(auction_data)
}

fn valid_english_specific_data() -> EnglishAuctionData {
	EnglishAuctionData {}
}

/// TopUp auction tests
fn topup_auction_object(general_data: GeneralAuctionData<Test>, specific_data: TopUpAuctionData) -> Auction<Test> {
	let auction_data = TopUpAuction {
		general_data,
		specific_data,
	};

	Auction::TopUp(auction_data)
}

fn valid_topup_specific_data() -> TopUpAuctionData {
	TopUpAuctionData {}
}

fn bid_object(amount: BalanceOf<Test>, block_number: <Test as frame_system::Config>::BlockNumber) -> Bid<Test> {
	Bid { amount, block_number }
}

fn get_auction_subaccount_id(auction_id: <Test as pallet::Config>::AuctionId) -> AccountId {
	<Test as pallet::Config>::PalletId::get().into_sub_account(("ac", auction_id))
}

/// Candle auction tests
fn candle_auction_object(general_data: GeneralAuctionData<Test>, specific_data: CandleAuctionData<Test>) -> Auction<Test> {
	let auction_data = CandleAuction {
		general_data,
		specific_data,
	};

	Auction::Candle(auction_data)
}

fn valid_candle_general_auction_data() -> GeneralAuctionData<Test> {
	GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u64,
		end: 99_366u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, NFT_INSTANCE_ID_1),
		next_bid_min: 1,
	}
}

fn valid_candle_specific_data() -> CandleAuctionData<Test> {
	CandleAuctionData {
		closing_start: 27_366,
		winner: None,
		winning_closing_range: None
	}
}

// -------------- English auction tests -------------- //
/// Creating an English auction
///
/// Happy path
#[test]
fn create_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		expect_event(crate::Event::<Test>::AuctionCreated(ALICE, 0));

		let auction = AuctionsModule::auctions(0).unwrap();

		let auction_check = match auction {
			Auction::English(data) => {
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

/// Error AuctionStartTimeAlreadyPassed
#[test]
fn create_english_auction_starting_in_the_past_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let mut general_auction_data = valid_general_auction_data();
		general_auction_data.start = 0u64;

		let auction = english_auction_object(general_auction_data, valid_english_specific_data());

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

		let auction = english_auction_object(general_auction_data, valid_english_specific_data());

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

		let auction = english_auction_object(general_auction_data, valid_english_specific_data());

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
		let mut general_auction_data = valid_general_auction_data();

		general_auction_data.next_bid_min = 0;
		let auction = english_auction_object(general_auction_data.clone(), valid_english_specific_data());

		// next_bid_min is below BidMinAmount
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);

		general_auction_data.next_bid_min = 10;
		let auction = english_auction_object(general_auction_data.clone(), valid_english_specific_data());

		// next_bid_min cannot be set when reserve_price is None
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);

		general_auction_data.reserve_price = Some(20);
		let auction = english_auction_object(general_auction_data, valid_english_specific_data());

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
		let mut general_auction_data = valid_general_auction_data();
		general_auction_data.name = to_bounded_name(b"".to_vec()).unwrap();

		let auction = english_auction_object(general_auction_data, valid_english_specific_data());

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

		let auction = english_auction_object(general_auction_data, valid_english_specific_data());

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

		let auction = english_auction_object(general_auction_data, valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(3);

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction_data = EnglishAuction {
			general_data: updated_general_data,
			specific_data: valid_english_specific_data(),
		};
		let auction = Auction::English(auction_data);

		assert_ok!(AuctionsModule::update(Origin::signed(ALICE), 0, auction));

		let auction_result = AuctionsModule::auctions(0).unwrap();

		let auction_check = match auction_result {
			Auction::English(data) => {
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
fn update_english_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.next_bid_min = 0;

		let auction = english_auction_object(updated_general_data.clone(), valid_english_specific_data());

		// next_bid_min is below BidMinAmount
		assert_noop!(
			AuctionsModule::create(Origin::signed(ALICE), auction),
			Error::<Test>::InvalidNextBidMin
		);

		updated_general_data.next_bid_min = 10;

		let auction = english_auction_object(updated_general_data.clone(), valid_english_specific_data());

		// next_bid_min is set while reserve_price is None
		assert_noop!(
			AuctionsModule::update(Origin::signed(ALICE), 0, auction),
			Error::<Test>::InvalidNextBidMin
		);

		updated_general_data.reserve_price = Some(20);
		let auction = english_auction_object(updated_general_data, valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.closed = true;

		let auction = english_auction_object(updated_general_data, valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = english_auction_object(updated_general_data, valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = english_auction_object(updated_general_data, valid_english_specific_data());

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
		let auction = topup_auction_object(valid_general_auction_data(), valid_topup_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = english_auction_object(updated_general_data, valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

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
		set_block_number::<Test>(12);

		assert_ok!(AuctionsModule::bid(
			Origin::signed(CHARLIE),
			0,
			BalanceOf::<Test>::from(1_100_u32)
		));
		expect_event(crate::Event::<Test>::BidPlaced(0, CHARLIE, bid_object(1100, 12)));

		// Tokens of previous highest bidder are unlocked
		assert_ok!(Balances::transfer(Origin::signed(BOB), ALICE, 2_000 * BSX));

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::English(data) => {
				// Next bid step is updated
				assert_eq!(data.general_data.next_bid_min, 1210);

				// Auction time is extended with 1 block when end time is less than 10 blocks away
				assert_eq!(data.general_data.end, 22u64);

				Ok(())
			},
			_ => Err(())
		};

		assert_ok!(auction_check);
	});
}

/// Error AuctionNotStarted
#[test]
fn bid_english_auction_before_auction_start_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(10);

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
fn close_english_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(11);

		let bid = BalanceOf::<Test>::from(1_000_u32);
		assert_ok!(AuctionsModule::bid(Origin::signed(BOB), 0, bid));

		let alice_balance_before = Balances::free_balance(&ALICE);
		let bob_balance_before = Balances::free_balance(&BOB);

		set_block_number::<Test>(21);

		assert_ok!(AuctionsModule::close(Origin::signed(ALICE), 0));

		let alice_balance_after = Balances::free_balance(&ALICE);
		let bob_balance_after = Balances::free_balance(&BOB);

		// The auction winner is the new owner of the NFT
		assert_eq!(Nft::owner(NFT_CLASS_ID_1, NFT_INSTANCE_ID_1), Some(BOB));

		assert_eq!(alice_balance_before.saturating_add(bid), alice_balance_after);
		assert_eq!(bob_balance_before.saturating_sub(bid), bob_balance_after);

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

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

// -------------- Candle auction tests -------------- //
///
/// Creating a Candle auction
///
/// Happy path
///
#[test]
fn create_candle_auction_should_work() {
	predefined_test_ext().execute_with(|| {
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		expect_event(crate::Event::<Test>::AuctionCreated(ALICE, 0));

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::Candle(data) => {
				assert_eq!(String::from_utf8(data.general_data.name.to_vec()).unwrap(), "Auction 0");
				assert_eq!(data.general_data.reserve_price, None);
				assert_eq!(data.general_data.last_bid, None);
				assert_eq!(data.general_data.start, 10u64);
				assert_eq!(data.general_data.end, 99_366u64);
				assert_eq!(data.general_data.owner, ALICE);
				assert_eq!(data.general_data.token, (NFT_CLASS_ID_1, NFT_INSTANCE_ID_1));
				assert_eq!(data.general_data.next_bid_min, 1);
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
		let mut general_auction_data = valid_candle_general_auction_data();
		general_auction_data.end = 0u64;

		let auction = candle_auction_object(general_auction_data, valid_candle_specific_data());

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
		let mut general_auction_data = valid_candle_general_auction_data();
		general_auction_data.end = 20u64;

		let auction = candle_auction_object(general_auction_data, valid_candle_specific_data());

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
		let mut general_auction_data = valid_candle_general_auction_data();

		general_auction_data.next_bid_min = 0;
		let auction = candle_auction_object(general_auction_data, valid_candle_specific_data());

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
		let mut general_auction_data = valid_candle_general_auction_data();
		general_auction_data.name = to_bounded_name(b"".to_vec()).unwrap();

		let auction = candle_auction_object(general_auction_data, valid_candle_specific_data());

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
		let mut general_auction_data = valid_candle_general_auction_data();
		general_auction_data.owner = BOB;

		let auction = candle_auction_object(general_auction_data, valid_candle_specific_data());

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
		let mut general_auction_data = valid_candle_general_auction_data();
		general_auction_data.closed = true;

		let auction = candle_auction_object(general_auction_data, valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

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
		let mut general_auction_data = valid_candle_general_auction_data();
		general_auction_data.end = 99_367;

		let auction = candle_auction_object(general_auction_data, valid_candle_specific_data());

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
		let mut specific_data = valid_candle_specific_data();
		specific_data.closing_start = 27_367;

		let auction = candle_auction_object(valid_candle_general_auction_data(), specific_data);

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
		let mut general_auction_data = valid_candle_general_auction_data();
		general_auction_data.reserve_price = Some(100);

		let auction = candle_auction_object(general_auction_data, valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		set_block_number::<Test>(3);

		let mut updated_general_data = valid_candle_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction_data = CandleAuction {
			general_data: updated_general_data,
			specific_data: valid_candle_specific_data(),
		};
		let auction = Auction::Candle(auction_data);

		assert_ok!(AuctionsModule::update(Origin::signed(ALICE), 0, auction));

		let auction = AuctionsModule::auctions(0).unwrap();
		let auction_check = match auction {
			Auction::Candle(data) => {
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
fn update_candle_auction_with_nonexisting_auction_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_candle_general_auction_data();
		updated_general_data.next_bid_min = 0;

		let auction = candle_auction_object(updated_general_data, valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_candle_general_auction_data();
		updated_general_data.closed = true;

		let auction = candle_auction_object(updated_general_data, valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_candle_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = candle_auction_object(updated_general_data, valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_candle_general_auction_data();
		updated_general_data.start = 18;
		updated_general_data.end = 99_374;

		let mut updated_specific_data = valid_candle_specific_data();
		updated_specific_data.closing_start = 27_374;

		let auction = candle_auction_object(updated_general_data, updated_specific_data);

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
		let auction = english_auction_object(valid_general_auction_data(), valid_english_specific_data());

		assert_ok!(AuctionsModule::create(Origin::signed(ALICE), auction));

		let mut updated_general_data = valid_candle_general_auction_data();
		updated_general_data.name = to_bounded_name(b"Auction renamed".to_vec()).unwrap();

		let auction = candle_auction_object(updated_general_data, valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

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
				assert_eq!(data.general_data.next_bid_min, 1650);

				// Auction time is extended with 1 block when end time is less than 10 blocks away
				assert_eq!(data.general_data.end, 99_366);

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());
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
		assert_eq!(Nft::owner(NFT_CLASS_ID_1, NFT_INSTANCE_ID_1), Some(CHARLIE));

		let auction = AuctionsModule::auctions(0).unwrap();

		let auction_check = match auction {
			Auction::Candle(data) => {
				assert!(data.general_data.closed);
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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());
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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());
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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());

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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());
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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());
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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());
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
		let auction = candle_auction_object(valid_candle_general_auction_data(), valid_candle_specific_data());
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
