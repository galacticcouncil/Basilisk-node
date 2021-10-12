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
		assert_ok!(Nft::create_class(Origin::signed(ALICE), 0, ClassType::Art));
		assert_ok!(Nft::mint(Origin::signed(ALICE), 0, 0));

		assert_noop!(
			Market::set_price(Origin::signed(BOB), 0, 0, Some(10)),
			Error::<Test>::NotTheTokenOwner
		);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(10)));

		let event = Event::Marketplace(crate::Event::TokenPriceUpdated(ALICE, 0, 0, Some(10)));
		assert_eq!(last_event(), event);

		assert_eq!(Market::tokens(0, 0), Some(TokenInfo{author: ALICE, royalty: 20u8, price: Some(10)}));

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, None));
		assert_eq!(Market::tokens(0, 0), Some(TokenInfo{author: ALICE, royalty: 20u8, price: None}));

		let event = Event::Marketplace(crate::Event::TokenPriceUpdated(ALICE, 0, 0, None));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn buy_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Uniques::create(Origin::signed(ALICE), 0, ALICE));
		assert_ok!(Uniques::mint(Origin::signed(ALICE), 0, 0, ALICE));

		assert_noop!(
			Market::buy(Origin::signed(ALICE), 0, 0),
			Error::<Test>::BuyFromSelf
		);
		assert_noop!(Market::buy(Origin::signed(BOB), 1, 0), Error::<Test>::NotForSale);
		assert_noop!(Market::buy(Origin::signed(BOB), 0, 0), Error::<Test>::NotForSale);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(2222 * BSX)));

		assert_noop!(
			Market::buy(Origin::signed(BOB), 0, 0),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(1111 * BSX)));

		assert_ok!(Market::buy(Origin::signed(BOB), 0, 0));

		assert_eq!(Market::tokens(0, 0), Some(TokenInfo{author: ALICE, royalty: 20u8, price: None}));

		assert_eq!(Balances::free_balance(ALICE), 11111 * BSX);
		assert_eq!(Balances::free_balance(BOB), 889 * BSX);

		let event = Event::Marketplace(crate::Event::TokenSold(ALICE, BOB, 0, 0, 1111 * BSX));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn list_works() {
	new_test_ext().execute_with(|| {
		
	});
}

#[test]
fn unlist_works() {
	new_test_ext().execute_with(|| {
		
	});
}

#[test]
fn free_trading_works() {
	new_test_ext().execute_with(|| {
		
	});
}