use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::{Event, *};
use sp_std::convert::TryInto;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

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
		assert_ok!(Nft::mint(Origin::signed(ALICE), 0, 0, bvec![0]));
		assert_ok!(Market::list(Origin::signed(ALICE), 0, 0, CHARLIE, 20));

		assert_noop!(
			Market::set_price(Origin::signed(BOB), 0, 0, Some(10)),
			Error::<Test>::NotTheTokenOwner
		);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(10)));

		let event = Event::Marketplace(crate::Event::TokenPriceUpdated(ALICE, 0, 0, Some(10)));
		assert_eq!(last_event(), event);

		assert_eq!(
			Market::tokens(0, 0),
			Some(TokenInfo {
				author: CHARLIE,
				royalty: 20u8,
				price: Some(10),
				offer: None,
			})
		);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, None));
		assert_eq!(
			Market::tokens(0, 0),
			Some(TokenInfo {
				author: CHARLIE,
				royalty: 20u8,
				price: None,
				offer: None,
			})
		);

		let event = Event::Marketplace(crate::Event::TokenPriceUpdated(ALICE, 0, 0, None));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn buy_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Nft::create_class(Origin::signed(ALICE), 0, ClassType::Art));
		assert_ok!(Nft::mint(Origin::signed(ALICE), 0, 0, bvec![0]));

		assert_noop!(Market::buy(Origin::signed(BOB), 0, 0), Error::<Test>::NotListed);

		assert_ok!(Market::list(Origin::signed(ALICE), 0, 0, DAVE, 33));

		assert_noop!(Market::buy(Origin::signed(BOB), 0, 0), Error::<Test>::NotForSale);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(22222 * BSX)));

		assert_noop!(
			Market::buy(Origin::signed(BOB), 0, 0),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(1111 * BSX)));

		assert_ok!(Market::buy(Origin::signed(BOB), 0, 0));

		assert_eq!(
			Market::tokens(0, 0),
			Some(TokenInfo {
				author: DAVE,
				royalty: 33,
				price: None,
				offer: None,
			})
		);

		assert_eq!(Balances::free_balance(ALICE), 1070937000000000);
		assert_eq!(Balances::free_balance(BOB), 1378900000000000);

		let event = Event::Marketplace(crate::Event::TokenSold(ALICE, BOB, 0, 0, 74437000000000));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn list_works() {
	new_test_ext().execute_with(|| {});
}

#[test]
fn unlist_works() {
	new_test_ext().execute_with(|| {});
}

#[test]
fn free_trading_works() {
	new_test_ext().execute_with(|| {
		// Anyone can create a class
		assert_ok!(Nft::create_class(Origin::signed(ALICE), 0, ClassType::Art));
		assert_ok!(Nft::create_class(Origin::signed(BOB), 1, ClassType::Art));
		assert_ok!(Nft::create_class(Origin::signed(CHARLIE), 2, ClassType::PoolShare));

		// Anyone can mint a token in any class
		assert_ok!(Nft::mint(Origin::signed(ALICE), 0, 0, bvec![0]));
		assert_ok!(Nft::mint(Origin::signed(ALICE), 1, 0, bvec![0]));
		assert_ok!(Nft::mint(Origin::signed(ALICE), 2, 0, bvec![0]));
		assert_ok!(Nft::mint(Origin::signed(BOB), 0, 1, bvec![0]));
		assert_ok!(Nft::mint(Origin::signed(BOB), 1, 1, bvec![0]));
		assert_ok!(Nft::mint(Origin::signed(BOB), 2, 1, bvec![0]));
		assert_ok!(Nft::mint(Origin::signed(CHARLIE), 0, 2, bvec![0]));
		assert_ok!(Nft::mint(Origin::signed(CHARLIE), 1, 2, bvec![0]));
		assert_ok!(Nft::mint(Origin::signed(CHARLIE), 2, 2, bvec![0]));

		// Only instance owner can burn their token
		assert_noop!(
			Nft::burn(Origin::signed(ALICE), 1, 1),
			pallet_uniques::Error::<Test, _>::NoPermission
		);
		//assert_noop!(
		//	Nft::burn(Origin::signed(ALICE), 1, 1),
		//	pallet_uniques::Error::<Test, _>::NoPermission
		//);
		assert_ok!(Nft::burn(Origin::signed(ALICE), 0, 0));

		// Only instance owner can transfer their token
		assert_ok!(Nft::transfer(Origin::signed(BOB), 1, 1, CHARLIE));
		assert_ok!(Nft::transfer(Origin::signed(CHARLIE), 1, 1, BOB));
		assert_noop!(
			Nft::transfer(Origin::signed(ALICE), 1, 1, CHARLIE),
			pallet_uniques::Error::<Test, _>::NoPermission
		);
		//assert_noop!(
		//	Nft::transfer(Origin::signed(ALICE), 0, 1, CHARLIE),
		//	pallet_uniques::Error::<Test, _>::NoPermission
		//);

		// Only class owner or ForceOrigin can destroy their class
		assert_ok!(Nft::destroy_class(Origin::signed(CHARLIE), 2));
		assert_noop!(
			Nft::destroy_class(Origin::signed(CHARLIE), 1),
			pallet_uniques::Error::<Test, _>::NoPermission
		);

		// Only token owner can list their token on marketplace
		assert_noop!(
			Market::list(Origin::signed(CHARLIE), 1, 1, DAVE, 33),
			Error::<Test>::NotTheTokenOwner
		);
		//assert_noop!(
		//	Market::list(Origin::signed(CHARLIE), 2, 1, DAVE, 33),
		//	Error::<Test>::NotTheTokenOwner
		//);
		assert_ok!(Market::list(Origin::signed(BOB), 1, 1, DAVE, 33));

		// Only token owner can set price of a token on marketplace
		assert_noop!(
			Market::set_price(Origin::signed(CHARLIE), 1, 1, Some(20)),
			Error::<Test>::NotTheTokenOwner
		);
		// Only token owner can set price of a token on marketplace
		//assert_noop!(
		//	Market::set_price(Origin::signed(CHARLIE), 2, 1, Some(20)),
		//	Error::<Test>::NotTheTokenOwner
		//);
		assert_ok!(Market::set_price(Origin::signed(BOB), 1, 1, Some(100)));

		// Anyone can trade NFTs freely from each other
		assert_ok!(Market::buy(Origin::signed(ALICE), 1, 1));
		assert_ok!(Market::set_price(Origin::signed(ALICE), 1, 1, Some(200)));

		assert_ok!(Market::buy(Origin::signed(BOB), 1, 1));
		assert_ok!(Market::set_price(Origin::signed(BOB), 1, 1, Some(300)));

		assert_ok!(Market::buy(Origin::signed(CHARLIE), 1, 1));
		assert_ok!(Market::set_price(Origin::signed(CHARLIE), 1, 1, Some(400)));

		assert_noop!(Market::buy(Origin::signed(CHARLIE), 1, 1), Error::<Test>::BuyFromSelf);

		// Classes and tokens cannot be transferred or burned by anyone when listed
		assert_noop!(
			Nft::transfer(Origin::signed(CHARLIE), 1, 1, BOB),
			pallet_uniques::Error::<Test, _>::Frozen
		);

		assert_noop!(
			Nft::burn(Origin::signed(CHARLIE), 1, 1),
			pallet_nft::Error::<Test>::TokenFrozen
		);

		//assert_noop!(
		//	Nft::destroy_class(Origin::signed(ALICE), 0),
		//	pallet_uniques::Error::<Test, _>::Frozen
		//);

		assert_ok!(Market::unlist(Origin::signed(CHARLIE), 1, 1));

		//assert_noop!(
		//	Nft::burn(Origin::signed(BOB), 1, 1),
		//	pallet_uniques::Error::<Test, _>::NoPermission
		//);

		assert_ok!(Nft::burn(Origin::signed(CHARLIE), 1, 1));
	});
}

#[test]
fn offering_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Nft::create_class(Origin::signed(ALICE), 0, ClassType::Art));
		assert_ok!(Nft::mint(Origin::signed(ALICE), 0, 0, bvec![0]));
		assert_ok!(Market::list(Origin::signed(ALICE), 0, 0, CHARLIE, 20));
		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(100 * BSX)));
		assert_ok!(Market::make_offer(Origin::signed(BOB), 0, 0, 50 * BSX));
		assert_ok!(Market::accept_offer(Origin::signed(ALICE), 0, 0));
		assert_eq!(pallet_uniques::Pallet::<Test>::owner(0, 0), Some(BOB));
		assert_eq!(Balances::total_balance(&ALICE), 20_080 * BSX);
		assert_eq!(Balances::total_balance(&BOB), 14_900 * BSX);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_020 * BSX);
	});
}
