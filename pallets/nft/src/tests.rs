use frame_support::{assert_noop, assert_ok, error::BadOrigin};

use super::*;
use mock::{Event, *};

type NftModule = Module<Test>;

#[test]
fn create_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));
		let event = Event::pallet_nft(crate::Event::NFTTokenClassCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn create_class_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NftModule::create_class(Origin::none(), "a class".as_bytes().to_vec(), Default::default()),
			BadOrigin
		);
	})
}

#[test]
fn mint_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));
		let event = Event::pallet_nft(crate::Event::NFTTokenClassCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);

		assert_ok!(NftModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
			TEST_QUANTITY,
		));
		let event = Event::pallet_nft(crate::Event::NFTTokenMinted(ALICE, CLASS_ID, TEST_QUANTITY));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn mint_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));
		let event = Event::pallet_nft(crate::Event::NFTTokenClassCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);

		assert_noop!(
			NftModule::mint(
				Origin::signed(BOB),
				0,
				"a token".as_bytes().to_vec(),
				TokenData { locked: false },
				TEST_QUANTITY,
			),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));

		assert_ok!(NftModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
			TEST_QUANTITY,
		));

		assert_ok!(NftModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)));
		let event = Event::pallet_nft(crate::Event::NFTTokenTransferred(ALICE, BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn transfer_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));

		assert_ok!(NftModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
			TEST_QUANTITY,
		));

		assert_noop!(
			NftModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn burn_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));

		assert_ok!(NftModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
			TEST_QUANTITY,
		));

		assert_ok!(NftModule::burn(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));
		let event = Event::pallet_nft(crate::Event::NFTTokenBurned(ALICE, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn burn_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));

		assert_ok!(NftModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
			TEST_QUANTITY,
		));

		assert_noop!(
			NftModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn destroy_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));

		assert_ok!(NftModule::destroy_class(Origin::signed(ALICE), CLASS_ID));
	});
}

#[test]
fn destroy_class_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"a class".as_bytes().to_vec(),
			Default::default()
		));

		assert_ok!(NftModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
			TEST_QUANTITY,
		));

		assert_noop!(
			NftModule::destroy_class(Origin::signed(ALICE), CLASS_ID),
			Error::<Test>::CannotDestroyClass
		);
	});
}
