use frame_support::{assert_noop, assert_ok, BoundedVec};

use super::*;
use mock::{Event, *};
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
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));

		// not the owner
		assert_noop!(
			Market::set_price(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, Some(10)),
			Error::<Test>::NotTheTokenOwner
		);

		// NFT doesn't exist
		assert_noop!(
			Market::set_price(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_1, Some(10)),
			Error::<Test>::NotTheTokenOwner
		);

		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(10)
		));
		assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), Some(10));
		let event = Event::Marketplace(crate::Event::TokenPriceUpdated {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			price: Some(10),
		});
		assert_eq!(last_event(), event);

		// update of existing price should work
		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(20)
		));
		assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), Some(20));
		let event = Event::Marketplace(crate::Event::TokenPriceUpdated {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			price: Some(20),
		});
		assert_eq!(last_event(), event);

		// disable sell
		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			None
		));
		assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), None);
		let event = Event::Marketplace(crate::Event::TokenPriceUpdated {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			price: None,
		});
		assert_eq!(last_event(), event);
	});
}

#[test]
fn add_royalty_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));

		// 100% royalty is not allowed
		assert_noop!(
			Market::add_royalty(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, CHARLIE, 100),
			Error::<Test>::NotInRange
		);

		// NFT doesn't exist
		assert_noop!(
			Market::add_royalty(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_1, CHARLIE, 20),
			pallet_nft::Error::<Test>::ClassUnknown
		);

		// not the owner
		assert_noop!(
			Market::add_royalty(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0, CHARLIE, 20),
			pallet_nft::Error::<Test>::NotPermitted
		);

		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));
		assert_eq!(Market::marketplace_instances(CLASS_ID_0, INSTANCE_ID_0), Some(Royalty{author: CHARLIE, royalty: 20}));
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &ALICE), <Test as Config>::RoyaltyBondAmount::get());
		let event = Event::Marketplace(crate::Event::RoyaltyAdded {
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			author: CHARLIE,
			royalty: 20,
		});
		assert_eq!(last_event(), event);

		assert_noop!(
			Market::add_royalty(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, CHARLIE, 20),
			Error::<Test>::RoyaltyAlreadySet
		);
	});
}

#[test]
fn make_offer_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));

		assert_noop!(
			Market::make_offer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, 0, 1),
			Error::<Test>::OfferTooLow
		);

		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			1
		));
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB),
			Some(Offer {
				maker: BOB,
				amount: 50 * UNITS,
				expires: 1,
			})
		);
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &BOB), 50 * UNITS);
		let event = Event::Marketplace(crate::Event::OfferPlaced {
			who: BOB,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			amount: 50 * UNITS,
			expires: 1,
		});
		assert_eq!(last_event(), event);

		// update existing offer doesn't work
		assert_noop!(
			Market::make_offer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, 70 * UNITS, 1),
			Error::<Test>::AlreadyOffered
		);

		// offer for a non-existing NFT works
		assert_ok!(Market::make_offer(Origin::signed(CHARLIE), 2, 2, 50 * UNITS, 1));
	});
}

#[test]
fn withdraw_offer_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));

		// non-existing offer
		assert_noop!(
			Market::withdraw_offer(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0, CHARLIE),
			Error::<Test>::UnknownOffer
		);

		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			1
		));

		// not the NFT owner
		assert_noop!(
			Market::withdraw_offer(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0, BOB),
			Error::<Test>::WithdrawNotAuthorized
		);

		// offer maker can withdraw the offer
		assert_ok!(Market::withdraw_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB
		));
		assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &BOB), 0);
		let event = Event::Marketplace(crate::Event::OfferWithdrawn {
			who: BOB,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
		});
		assert_eq!(last_event(), event);

		// NFT owner can withdraw the offer
		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			1
		));
		assert_ok!(Market::withdraw_offer(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB
		));
		let event = Event::Marketplace(crate::Event::OfferWithdrawn {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
		});
		assert_eq!(last_event(), event);
		assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), ALICE), None);

		// non-existing NFT
		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			1
		));
		assert_ok!(NFT::burn(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0));
		// Alice don't have any rights over an offer for a burned NFT
		assert_noop!(
			Market::withdraw_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, BOB),
			Error::<Test>::WithdrawNotAuthorized
		);
		assert_ok!(
			Market::withdraw_offer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, BOB),
		);
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &BOB), 0);
		let event = Event::Marketplace(crate::Event::OfferWithdrawn {
			who: BOB,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
		});
		assert_eq!(last_event(), event);
		assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
	});
}

#[test]
fn accept_offer_works() {
	new_test_ext().execute_with(|| {
		let price = 50 * UNITS;
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));

		// offer doesn't exist
		assert_noop!(
			Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, BOB),
			Error::<Test>::UnknownOffer
		);

		// expired offer
		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			price,
			1
		));
		assert_noop!(
			Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, BOB),
			Error::<Test>::OfferExpired
		);
		assert_ok!(
			Market::withdraw_offer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, BOB),
		);

		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			price,
			2
		));

		// NFT doesn't exist
		assert_noop!(
			Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_1, BOB),
			Error::<Test>::ClassOrInstanceUnknown
		);

		// not the NFT owner
		assert_noop!(
			Market::accept_offer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, BOB),
			Error::<Test>::AcceptNotAuthorized
		);

		// accept an offer without royalty
		let alice_initial_balance = Balances::free_balance(&ALICE);
		let bob_initial_balance = Balances::free_balance(&BOB);
		assert_ok!(Market::accept_offer(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB
		));
		let event = Event::Marketplace(crate::Event::OfferAccepted {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			amount: price,
			maker: BOB
		});
		assert_eq!(last_event(), event);
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
		assert_eq!(
			pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
			Some(BOB)
		);
		assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + price);
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &BOB), 0);
		assert_eq!(Balances::free_balance(&BOB), bob_initial_balance); // paid from the reserved amount
	});
}

#[test]
fn accept_offer_with_royalty_works() {
	new_test_ext().execute_with(|| {
		let price = 50 * UNITS;
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));
		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			price,
			2
		));

		let alice_initial_balance = Balances::free_balance(&ALICE);
		let bob_initial_balance = Balances::free_balance(&BOB);
		let charlie_initial_balance = Balances::free_balance(&CHARLIE);
		assert_ok!(Market::accept_offer(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB
		));
		let event = Event::Marketplace(crate::Event::OfferAccepted {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			amount: price,
			maker: BOB
		});
		assert_eq!(last_event(), event);
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
		assert_eq!(
			pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
			Some(BOB)
		);
		assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + 40 * UNITS); // price - royalty
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &BOB), 0);
		assert_eq!(Balances::free_balance(&BOB), bob_initial_balance); // paid from the reserved amount
		assert_eq!(Balances::free_balance(&CHARLIE), charlie_initial_balance + 10 * UNITS); // royalty
	});
}

#[test]
fn accept_offer_with_royalty_and_set_price_works() {
	new_test_ext().execute_with(|| {
		let price = 50 * UNITS;
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));
		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100 * UNITS)
		));
		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			price,
			2
		));

		// too expensive
		assert_noop!(Market::make_offer(
			Origin::signed(DAVE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			100_000_000 * UNITS,
			2),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);



		let alice_initial_balance = Balances::free_balance(&ALICE);
		let bob_initial_balance = Balances::free_balance(&BOB);
		let charlie_initial_balance = Balances::free_balance(&CHARLIE);
		// price set by the owner is ignored
		assert_ok!(Market::accept_offer(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB
		));
		let event = Event::Marketplace(crate::Event::OfferAccepted {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			amount: price,
			maker: BOB
		});
		assert_eq!(last_event(), event);
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
		assert_eq!(
			pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
			Some(BOB)
		);
		assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + 40 * UNITS); // price - royalty
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &BOB), 0);
		assert_eq!(Balances::free_balance(&BOB), bob_initial_balance); // paid from the reserved amount
		assert_eq!(Balances::free_balance(&CHARLIE), charlie_initial_balance + 10 * UNITS); // royalty
	});
}

#[test]
fn buy_without_royalty_works() {
	new_test_ext().execute_with(|| {
		let price = 100 * UNITS;
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));
		// make an offer to verify that it is ignored
		assert_ok!(Market::make_offer(
			Origin::signed(CHARLIE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			2
		));

		assert_noop!(
			Market::buy(Origin::signed(BOB), CLASS_ID_0, CLASS_ID_1),
			Error::<Test>::ClassOrInstanceUnknown
		);

		assert_noop!(Market::buy(
			Origin::signed(CHARLIE),
			CLASS_ID_0,
			INSTANCE_ID_0,
		),
		Error::<Test>::NotForSale);

		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100_000_000 * UNITS)
		));

		assert_noop!(
			Market::buy(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100 * UNITS)
		));

		assert_noop!(Market::buy(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
		),
		Error::<Test>::BuyFromSelf);

		let alice_initial_balance = Balances::free_balance(&ALICE);
		let charlie_initial_balance = Balances::free_balance(&CHARLIE);
		assert_ok!(Market::buy(
			Origin::signed(CHARLIE),
			CLASS_ID_0,
			INSTANCE_ID_0,
		));
		// NFT ownership is transferred
		assert_eq!(
			pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
			Some(CHARLIE)
		);
		// existing orders are not removed from the storage
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), CHARLIE),
			Some(Offer {
				maker: CHARLIE,
				amount: 50 * UNITS,
				expires: 2,
			})
		);
		assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), None);
		let event = Event::Marketplace(crate::Event::TokenSold {
			owner: ALICE,
			buyer: CHARLIE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			price,
		});
		assert_eq!(last_event(), event);
		assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + price);
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &CHARLIE), 50 * UNITS);
		assert_eq!(Balances::free_balance(&CHARLIE), charlie_initial_balance - price);
	});
}

#[test]
fn buy_with_royalty_works() {
	new_test_ext().execute_with(|| {
		let price = 100 * UNITS;
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB,
			20,
		));
		// make an offer to verify that it is ignored
		assert_ok!(Market::make_offer(
			Origin::signed(CHARLIE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			2
		));

		assert_noop!(Market::buy(
			Origin::signed(CHARLIE),
			CLASS_ID_0,
			INSTANCE_ID_0,
		),
		Error::<Test>::NotForSale);

		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100_000_000 * UNITS)
		));

		assert_noop!(
			Market::buy(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100 * UNITS)
		));

		assert_noop!(Market::buy(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
		),
		Error::<Test>::BuyFromSelf);

		let alice_initial_balance = Balances::free_balance(&ALICE);
		let bob_initial_balance = Balances::free_balance(&BOB);
		let charlie_initial_balance = Balances::free_balance(&CHARLIE);
		assert_ok!(Market::buy(
			Origin::signed(CHARLIE),
			CLASS_ID_0,
			INSTANCE_ID_0,
		));
		// NFT ownership is transferred
		assert_eq!(
			pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
			Some(CHARLIE)
		);
		// existing orders are not removed from the storage
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), CHARLIE),
			Some(Offer {
				maker: CHARLIE,
				amount: 50 * UNITS,
				expires: 2,
			})
		);
		assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), None);
		expect_events(vec![
			crate::Event::RoyaltyPaid {
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			author: BOB,
			royalty: 20,
			royalty_amount: 20 * UNITS,
		}
		.into(),
			crate::Event::TokenSold {
			owner: ALICE,
			buyer: CHARLIE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			price: 80 * UNITS,
		}
		.into()]);
		assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + 80 * UNITS); // price - royalty
		assert_eq!(<Test as pallet_nft::Config>::Currency::reserved_balance_named(&RESERVE_ID, &CHARLIE), 50 * UNITS);
		assert_eq!(Balances::free_balance(&BOB), bob_initial_balance + 20 * UNITS); // royalty
		assert_eq!(Balances::free_balance(&CHARLIE), charlie_initial_balance - price);
	});
}

#[test]
fn offering_works() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));
		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100 * UNITS)
		));

		assert_noop!(
			Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, DAVE),
			Error::<Test>::UnknownOffer
		);

		assert_noop!(
			Market::make_offer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, 0, 1),
			Error::<Test>::OfferTooLow
		);
		assert_ok!(Market::make_offer(
			Origin::signed(DAVE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			1
		));
		let event = Event::Marketplace(crate::Event::OfferPlaced {
			who: DAVE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			amount: 50 * UNITS,
			expires: 1,
		});
		assert_eq!(last_event(), event);

		assert_ok!(Market::make_offer(Origin::signed(ALICE), 3, 0, 50 * UNITS, 1));
		assert_noop!(
			Market::make_offer(Origin::signed(DAVE), CLASS_ID_0, INSTANCE_ID_0, 50 * UNITS, 1),
			Error::<Test>::AlreadyOffered
		);
		assert_noop!(
			Market::withdraw_offer(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0, CHARLIE),
			Error::<Test>::UnknownOffer
		);
		assert_noop!(
			Market::withdraw_offer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, DAVE),
			Error::<Test>::WithdrawNotAuthorized
		);
		assert_noop!(
			Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, DAVE),
			Error::<Test>::OfferExpired
		);
		assert_ok!(Market::withdraw_offer(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			DAVE
		));
		let event = Event::Marketplace(crate::Event::OfferWithdrawn {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
		});
		assert_eq!(last_event(), event);
		assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
		assert_ok!(Market::make_offer(
			Origin::signed(BOB),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			666
		));
		assert_noop!(
			Market::accept_offer(Origin::signed(DAVE), CLASS_ID_0, INSTANCE_ID_0, BOB),
			Error::<Test>::AcceptNotAuthorized
		);
		assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), Some(100 * UNITS));
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB),
			Some(Offer {
				maker: BOB,
				amount: 50 * UNITS,
				expires: 666,
			})
		);
		assert_eq!(frame_system::Pallet::<Test>::block_number(), 1);
		assert_ok!(Market::accept_offer(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB
		));
		let event = Event::Marketplace(crate::Event::OfferAccepted {
			who: ALICE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			amount: 50 * UNITS,
			maker: BOB
		});
		assert_eq!(last_event(), event);
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
		assert_eq!(
			pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
			Some(BOB)
		);
		// Total = 20_000 + 50 - 10 = 20_040
		assert_eq!(Balances::total_balance(&ALICE), 200_040 * UNITS);
		assert_eq!(Balances::total_balance(&BOB), 14_950 * UNITS);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_010 * UNITS);

		// trying to withdraw an offer to non-existing nft throws error
		assert_ok!(Market::make_offer(
			Origin::signed(DAVE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			1
		));
		assert_ok!(NFT::burn(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0));
		assert_noop!(
			Market::withdraw_offer(Origin::signed(DAVE), CLASS_ID_0, INSTANCE_ID_0, DAVE),
			Error::<Test>::ClassOrInstanceUnknown
		);
		assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), DAVE), None);
		assert_noop!(
			Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, DAVE),
			Error::<Test>::ClassOrInstanceUnknown
		);
	});
}

#[test]
fn buy_works_2() {
	new_test_ext().execute_with(|| {
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));
		assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			CHARLIE,
			20,
		));
		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100 * UNITS)
		));
		assert_ok!(Market::buy(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0));
		assert_eq!(
			pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
			Some(BOB)
		);
		assert_eq!(Balances::total_balance(&ALICE), 200_080 * UNITS);
		assert_eq!(Balances::total_balance(&BOB), 14_900 * UNITS);
		assert_eq!(Balances::total_balance(&CHARLIE), 150_020 * UNITS);
		// Reserved:
		// 10_000 class
		// 100 instance
		// 200 royalty
		assert_eq!(Balances::reserved_balance(&ALICE), 10_300 * UNITS);
		assert_eq!(Balances::reserved_balance(&BOB), 0);
		assert_ok!(NFT::burn(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0));
		assert_eq!(Balances::reserved_balance(&ALICE), 10_200 * UNITS);
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
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::create_class(
			Origin::signed(BOB),
			CLASS_ID_1,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::create_class(
			Origin::signed(CHARLIE),
			CLASS_ID_2,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			metadata.clone()
		));
		assert_ok!(NFT::mint(
			Origin::signed(BOB),
			CLASS_ID_1,
			INSTANCE_ID_0,
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(BOB), CLASS_ID_1, INSTANCE_ID_1, metadata));

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
