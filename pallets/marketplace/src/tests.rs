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
		assert_ok!(NFT::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFT::mint(Origin::signed(ALICE), 0, bvec![0]));
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
		assert_ok!(NFT::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFT::mint(Origin::signed(ALICE), 0, bvec![0]));

		assert_noop!(Market::buy(Origin::signed(BOB), 0, 0), Error::<Test>::NotListed);

		assert_ok!(Market::list(Origin::signed(ALICE), 0, 0, DAVE, 25));

		assert_noop!(Market::buy(Origin::signed(BOB), 0, 0), Error::<Test>::NotForSale);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(22_222 * BSX)));

		assert_noop!(
			Market::buy(Origin::signed(BOB), 0, 0),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(1024 * BSX)));

		assert_ok!(Market::buy(Origin::signed(BOB), 0, 0));

		assert_eq!(
			Market::tokens(0, 0),
			Some(TokenInfo {
				author: DAVE,
				royalty: 25,
				price: None,
				offer: None,
			})
		);

		assert_eq!(Balances::free_balance(ALICE), 10_633 * BSX);
		assert_eq!(Balances::free_balance(BOB), 13_976 * BSX);
		assert_eq!(Balances::free_balance(DAVE), 200_256 * BSX);

		let event = Event::Marketplace(crate::Event::TokenSold(
			ALICE,
			BOB,
			0,
			0,
			768 * BSX,
			Some((DAVE, 25)),
			256 * BSX,
		));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn buy_works_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(NFT::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFT::mint(Origin::signed(ALICE), 0, bvec![0]));
		assert_ok!(Market::list(Origin::signed(ALICE), 0, 0, CHARLIE, 20));
		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(100 * BSX)));
		assert_ok!(Market::buy(Origin::signed(BOB), 0, 0));
		assert_eq!(pallet_uniques::Pallet::<Test>::owner(0, 0), Some(BOB));
		assert_eq!(Balances::total_balance(&ALICE), 20_080 * BSX);
		assert_eq!(Balances::total_balance(&BOB), 14_900 * BSX);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_020 * BSX);
	});
}

#[test]
fn free_trading_works() {
	new_test_ext().execute_with(|| {
		// Anyone can create a class
		assert_ok!(NFT::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFT::create_class(Origin::signed(BOB), ClassType::Art));
		assert_ok!(NFT::create_class(Origin::signed(CHARLIE), ClassType::PoolShare));

		// Anyone can mint a token in any class
		assert_ok!(NFT::mint(Origin::signed(ALICE), 0, bvec![0]));
		assert_ok!(NFT::mint(Origin::signed(ALICE), 1, bvec![0]));
		assert_ok!(NFT::mint(Origin::signed(ALICE), 2, bvec![0]));
		assert_ok!(NFT::mint(Origin::signed(BOB), 0, bvec![0]));
		assert_ok!(NFT::mint(Origin::signed(BOB), 1, bvec![0]));
		assert_ok!(NFT::mint(Origin::signed(BOB), 2, bvec![0]));
		assert_ok!(NFT::mint(Origin::signed(CHARLIE), 0, bvec![0]));
		assert_ok!(NFT::mint(Origin::signed(CHARLIE), 1, bvec![0]));
		assert_ok!(NFT::mint(Origin::signed(CHARLIE), 2, bvec![0]));

		// Only instance owner can burn their token
		assert_noop!(
			NFT::burn(Origin::signed(ALICE), 1, 1),
			pallet_uniques::Error::<Test, _>::NoPermission
		);
		assert_noop!(
			NFT::burn(Origin::signed(CHARLIE), 1, 1),
			pallet_uniques::Error::<Test, _>::NoPermission
		);
		assert_ok!(NFT::burn(Origin::signed(ALICE), 2, 0));
		assert_ok!(NFT::burn(Origin::signed(BOB), 2, 1));
		assert_ok!(NFT::burn(Origin::signed(CHARLIE), 2, 2));

		// Only instance owner can transfer their token
		assert_ok!(NFT::transfer(Origin::signed(BOB), 1, 1, CHARLIE));
		assert_ok!(NFT::transfer(Origin::signed(CHARLIE), 1, 1, BOB));
		assert_noop!(
			NFT::transfer(Origin::signed(ALICE), 1, 1, CHARLIE),
			pallet_uniques::Error::<Test, _>::NoPermission
		);

		// Only class owner or ForceOrigin can destroy their class
		assert_ok!(NFT::destroy_class(Origin::signed(CHARLIE), 2));
		assert_noop!(
			NFT::destroy_class(Origin::signed(CHARLIE), 1),
			pallet_uniques::Error::<Test, _>::NoPermission
		);

		// Only token owner can list their token on marketplace
		assert_noop!(
			Market::list(Origin::signed(CHARLIE), 1, 1, DAVE, 33),
			Error::<Test>::NotTheTokenOwner
		);
		assert_noop!(
			Market::list(Origin::signed(CHARLIE), 2, 1, DAVE, 33),
			Error::<Test>::NotTheTokenOwner
		);
		assert_ok!(Market::list(Origin::signed(BOB), 1, 1, DAVE, 33));

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

		// Classes and tokens cannot be transferred or burned by anyone when listed
		assert_noop!(
			NFT::transfer(Origin::signed(CHARLIE), 1, 1, BOB),
			pallet_uniques::Error::<Test, _>::Frozen
		);

		assert_noop!(
			NFT::burn(Origin::signed(CHARLIE), 1, 1),
			pallet_nft::Error::<Test>::TokenFrozen
		);

		assert_ok!(Market::unlist(Origin::signed(CHARLIE), 1, 1));

		assert_noop!(
			NFT::burn(Origin::signed(BOB), 1, 1),
			pallet_uniques::Error::<Test, _>::NoPermission
		);

		assert_ok!(NFT::burn(Origin::signed(CHARLIE), 1, 1));
	});
}

#[test]
fn offering_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(NFT::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFT::mint(Origin::signed(ALICE), 0, bvec![0]));
		assert_ok!(Market::list(Origin::signed(ALICE), 0, 0, CHARLIE, 20));
		assert_ok!(Market::set_price(Origin::signed(ALICE), 0, 0, Some(100 * BSX)));
		assert_ok!(Market::make_offer(Origin::signed(BOB), 0, 0, 50 * BSX));
		assert_ok!(Market::accept_offer(Origin::signed(ALICE), 0, 0));
		assert_eq!(pallet_uniques::Pallet::<Test>::owner(0, 0), Some(BOB));
		assert_eq!(Balances::total_balance(&ALICE), 20_040 * BSX);
		assert_eq!(Balances::total_balance(&BOB), 14_950 * BSX);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_010 * BSX);
	});
}
