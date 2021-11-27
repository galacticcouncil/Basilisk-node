use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::{Event, *};

type Market = Pallet<Test>;

fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn set_price_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(ALICE),
			CLASS_ID_0,
			CHARLIE,
			20,
			b"metadata".to_vec(),
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
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(ALICE),
			CLASS_ID_0,
			CHARLIE,
			25,
			b"metadata".to_vec(),
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

		// e.g. Alice free balance = Total balance - Class deposit + Price - Royalty
		// = 20_000 - 10_000 + 1024 - 256 = 10_768
		assert_eq!(Balances::free_balance(ALICE), 10_768 * BSX);
		// BOB now owns NFT so reserved balance should be transferred to him
		assert_eq!(Balances::free_balance(BOB), 13_876 * BSX);
		assert_eq!(Balances::free_balance(CHARLIE), 150_256 * BSX);
		assert_eq!(Balances::free_balance(DAVE), 200_000 * BSX);

		let event = Event::Marketplace(crate::Event::TokenSold(
			ALICE,
			BOB,
			0,
			0,
			768 * BSX,
			(CHARLIE, 25),
			256 * BSX,
		));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn buy_works_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(ALICE),
			CLASS_ID_0,
			CHARLIE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(100 * BSX)));
		assert_ok!(Market::buy(Origin::signed(BOB), 0, 0));
		assert_eq!(pallet_uniques::Pallet::<Test>::owner(0, 0), Some(BOB));
		assert_eq!(Balances::total_balance(&ALICE), 20_080 * BSX);
		assert_eq!(Balances::total_balance(&BOB), 14_900 * BSX);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_020 * BSX);
		// Reserved 10_000 for the class
		assert_eq!(Balances::reserved_balance(&ALICE), 10_000 * BSX);
		// Reserved 100 for the transferred nft
		assert_eq!(Balances::reserved_balance(&BOB), 100 * BSX);
		assert_ok!(NFT::burn(Origin::signed(BOB), 0, 0));
		assert_eq!(Balances::reserved_balance(&ALICE), 10_000 * BSX);
		assert_eq!(Balances::reserved_balance(&BOB), 0);
	});
}

#[test]
fn free_trading_works() {
	new_test_ext().execute_with(|| {
		// Anyone can create a marketplace class
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFT::create_class(
			Origin::signed(BOB),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFT::create_class(
			Origin::signed(CHARLIE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFT::create_class(
			Origin::root(),
			ClassType::PoolShare,
			b"metadata".to_vec()
		));

		// Anyone can mint a token in any class
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(ALICE),
			0,
			ALICE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(ALICE),
			1,
			DAVE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(ALICE),
			2,
			DAVE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(BOB),
			0,
			DAVE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(BOB),
			1,
			DAVE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(BOB),
			2,
			DAVE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(CHARLIE),
			0,
			DAVE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(CHARLIE),
			1,
			DAVE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(CHARLIE),
			2,
			DAVE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(NFT::mint_for_liquidity_mining(
			Origin::root(),
			DAVE,
			3,
			123,
			654,
			b"metadata".to_vec(),
		));

		// Only instance owner can burn their token
		assert_noop!(
			NFT::burn(Origin::signed(ALICE), 1, 1),
			pallet_nft::Error::<Test>::NotPermitted
		);
		assert_noop!(
			NFT::burn(Origin::signed(CHARLIE), 1, 1),
			pallet_nft::Error::<Test>::NotPermitted
		);
		assert_ok!(NFT::burn(Origin::signed(ALICE), 2, 0));
		assert_ok!(NFT::burn(Origin::signed(BOB), 2, 1));
		assert_ok!(NFT::burn(Origin::signed(CHARLIE), 2, 2));

		// Only class owner or ForceOrigin can destroy their class
		assert_ok!(NFT::destroy_class(Origin::signed(CHARLIE), 2));

		assert_noop!(
			NFT::destroy_class(Origin::signed(CHARLIE), 1),
			pallet_nft::Error::<Test>::TokenClassNotEmpty
		);

		// Only token owner can set price of a token on marketplace
		assert_noop!(
			Market::set_price(Origin::signed(CHARLIE), 1, 1, Some(20)),
			Error::<Test>::NotTheTokenOwner
		);
		assert_ok!(Market::set_price(Origin::signed(BOB), 1, 1, Some(100)));

		// Anyone can trade NFTs freely from each other
		assert_ok!(Market::buy(Origin::signed(ALICE), 1, 1));
		assert_ok!(Market::set_price(Origin::signed(ALICE), 1, 1, Some(200)));

		assert_ok!(Market::buy(Origin::signed(BOB), 1, 1));
		assert_ok!(Market::set_price(Origin::signed(BOB), 1, 1, Some(300)));

		assert_ok!(Market::buy(Origin::signed(CHARLIE), 1, 1));
		assert_ok!(Market::set_price(Origin::signed(CHARLIE), 1, 1, Some(400)));

		assert_noop!(Market::buy(Origin::signed(CHARLIE), 1, 1), Error::<Test>::BuyFromSelf);

		assert_noop!(
			NFT::burn(Origin::signed(BOB), 1, 1),
			pallet_nft::Error::<Test>::NotPermitted
		);

		assert_ok!(NFT::burn(Origin::signed(CHARLIE), 1, 1));
	});
}

#[test]
fn offering_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFT::mint_for_marketplace(
			Origin::signed(ALICE),
			CLASS_ID_0,
			CHARLIE,
			20,
			b"metadata".to_vec(),
		));
		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(100 * BSX)));
		assert_noop!(
			Market::make_offer(Origin::signed(BOB), 0, 0, 0, 1),
			Error::<Test>::OfferTooLow
		);
		assert_ok!(Market::make_offer(Origin::signed(DAVE), 0, 0, 50 * BSX, 1));
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
		assert_eq!(Balances::total_balance(&ALICE), 20_040 * BSX);
		assert_eq!(Balances::total_balance(&BOB), 14_950 * BSX);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_010 * BSX);
	});
}
