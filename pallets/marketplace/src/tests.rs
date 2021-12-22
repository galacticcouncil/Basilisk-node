use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::{Event, *};
use primitives::nft::ClassType;
use std::convert::TryInto;

type Market = Pallet<Test>;

fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn set_price_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(Origin::signed(ALICE), Default::default(), metadata));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));

		assert_noop!(
			Market::set_price(Origin::signed(BOB), 0, 0, Some(10)),
			Error::<Test>::NotTheTokenOwner
		);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(10)));

		let event = Event::Marketplace(crate::Event::TokenPriceUpdated(ALICE, 0, 0, Some(10)));
		assert_eq!(last_event(), event);

		assert_eq!(Market::prices(0, 0), Some(10));

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, None));
		assert_eq!(Market::prices(0, 0), None);

		let event = Event::Marketplace(crate::Event::TokenPriceUpdated(ALICE, 0, 0, None));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn buy_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(Origin::signed(ALICE), Default::default(), metadata));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			25,
		));

		assert_noop!(
			Market::buy(Origin::signed(BOB), 666, 666),
			Error::<Test>::ClassOrInstanceUnknown
		);

		assert_noop!(Market::buy(Origin::signed(BOB), 0, 0), Error::<Test>::NotForSale);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(22_222 * BSX)));

		assert_noop!(
			Market::buy(Origin::signed(BOB), 0, 0),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(1024 * BSX)));

		assert_ok!(Market::buy(Origin::signed(BOB), 0, 0));

		assert_eq!(Market::prices(0, 0), None);

		assert_eq!(Balances::free_balance(ALICE), 190_668 * BSX);
		assert_eq!(Balances::free_balance(BOB), 13_976 * BSX);
		assert_eq!(Balances::free_balance(CHARLIE), 150_256 * BSX);
		assert_eq!(Balances::free_balance(DAVE), 200_000 * BSX);

		let event = Event::Marketplace(crate::Event::TokenSold(ALICE, BOB, 0, 0, 768 * BSX));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn buy_works_2() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(Origin::signed(ALICE), Default::default(), metadata));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));
		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(100 * BSX)));
		assert_ok!(Market::buy(Origin::signed(BOB), 0, 0));
		assert_eq!(pallet_uniques::Pallet::<Test>::owner(0, 0), Some(BOB));
		assert_eq!(Balances::total_balance(&ALICE), 200_080 * BSX);
		assert_eq!(Balances::total_balance(&BOB), 14_900 * BSX);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_020 * BSX);
		// Reserved 10_000 for the class
		assert_eq!(Balances::reserved_balance(&ALICE), 10_100 * BSX);
		// Reserved 100 for the transferred nft
		assert_eq!(Balances::reserved_balance(&BOB), 0);
		assert_ok!(NFT::burn(Origin::signed(BOB), 0, 0));
		assert_eq!(Balances::reserved_balance(&ALICE), 10_000 * BSX);
		assert_eq!(Balances::reserved_balance(&BOB), 0);
	});
}

#[test]
fn trading_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();

		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			metadata.clone()
		));
		assert_ok!(NFT::create_class(
			Origin::signed(BOB),
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::create_class(Origin::signed(CHARLIE), Default::default(), metadata));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0));
		assert_ok!(NFT::mint(Origin::signed(BOB), CLASS_ID_1));
		assert_ok!(NFT::mint(Origin::signed(BOB), CLASS_ID_1));

		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			ALICE,
			20,
		));

		// Only token owner can set price of a token on marketplace
		assert_noop!(
			Market::set_price(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0, Some(20)),
			Error::<Test>::NotTheTokenOwner
		);
		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100)
		));

		// Anyone can trade NFTs freely from each other
		assert_ok!(Market::buy(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0));
		assert_ok!(Market::set_price(
			Origin::signed(BOB),
			CLASS_ID_1,
			INSTANCE_ID_1,
			Some(200)
		));

		assert_ok!(Market::buy(Origin::signed(CHARLIE), CLASS_ID_1, INSTANCE_ID_1));
		assert_ok!(Market::set_price(
			Origin::signed(CHARLIE),
			CLASS_ID_1,
			INSTANCE_ID_1,
			Some(300)
		));

		assert_noop!(
			Market::buy(Origin::signed(CHARLIE), CLASS_ID_1, INSTANCE_ID_1),
			Error::<Test>::BuyFromSelf
		);

		assert_noop!(
			NFT::burn(Origin::signed(BOB), CLASS_ID_1, INSTANCE_ID_1),
			pallet_nft::Error::<Test>::NotPermitted
		);

		assert_ok!(NFT::burn(Origin::signed(CHARLIE), CLASS_ID_1, INSTANCE_ID_1));
	});
}

#[test]
fn offering_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0));
		assert_ok!(NFT::do_create_class(ALICE, ClassType::LiquidityMining, metadata));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(100 * BSX)));
		assert_noop!(
			Market::make_offer(Origin::signed(BOB), 0, 0, 0, 1),
			Error::<Test>::OfferTooLow
		);
		assert_ok!(Market::make_offer(Origin::signed(DAVE), 0, 0, 50 * BSX, 1));
		assert_ok!(Market::make_offer(Origin::signed(ALICE), 3, 0, 50 * BSX, 1));
		assert_noop!(
			Market::make_offer(Origin::signed(DAVE), 0, 0, 50 * BSX, 1),
			Error::<Test>::AlreadyOffered
		);
		assert_noop!(
			Market::withdraw_offer(Origin::signed(CHARLIE), 0, 0, CHARLIE),
			Error::<Test>::UnknownOffer
		);
		assert_noop!(
			Market::withdraw_offer(Origin::signed(BOB), 0, 0, DAVE),
			Error::<Test>::WithdrawNotAuthorized
		);
		assert_noop!(
			Market::accept_offer(Origin::signed(ALICE), 0, 0, DAVE),
			Error::<Test>::OfferExpired
		);
		assert_ok!(Market::withdraw_offer(Origin::signed(ALICE), 0, 0, DAVE));
		assert_ok!(Market::make_offer(Origin::signed(BOB), 0, 0, 50 * BSX, 666));
		assert_noop!(
			Market::accept_offer(Origin::signed(DAVE), 0, 0, BOB),
			Error::<Test>::AcceptNotAuthorized
		);
		assert_eq!(Market::prices(0, 0), Some(100 * BSX));
		assert_eq!(
			Market::offers((0, 0), BOB),
			Some(Offer {
				maker: BOB,
				amount: 50 * BSX,
				expires: 666,
			})
		);
		assert_eq!(frame_system::Pallet::<Test>::block_number(), 1);
		assert_ok!(Market::accept_offer(Origin::signed(ALICE), 0, 0, BOB));
		assert_eq!(pallet_uniques::Pallet::<Test>::owner(0, 0), Some(BOB));
		// Total = 20_000 + 50 - 10 = 20_040
		assert_eq!(Balances::total_balance(&ALICE), 200_040 * BSX);
		assert_eq!(Balances::total_balance(&BOB), 14_950 * BSX);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_010 * BSX);
	});
}

#[test]
fn add_royalty_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			metadata
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0));

		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));
	});
}
