use frame_support::{assert_err, assert_noop, assert_ok, error::BadOrigin};

use super::*;
use mock::{Event, *};

type NFTModule = Pallet<Test>;

#[test]
fn create_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));
		let event = Event::NFT(crate::Event::TokenClassCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn create_class_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NFTModule::create_class(
				Origin::none(),
				"some metadata about token".as_bytes().to_vec(),
				ClassData { is_pool: true },
			),
			BadOrigin
		);

		assert_noop!(
			NFTModule::create_class(
				Origin::signed(ALICE),
				vec![1; <Test as orml_nft::Config>::MaxClassMetadata::get() as usize + 1],
				ClassData { is_pool: true },
			),
			orml_nft::Error::<Test>::MaxMetadataExceeded
		);

		orml_nft::NextClassId::<Test>::put(u64::MAX);
		assert_noop!(
			NFTModule::create_class(
				Origin::signed(ALICE),
				"some metadata about token".as_bytes().to_vec(),
				ClassData { is_pool: true },
			),
			orml_nft::Error::<Test>::NoAvailableClassId
		);
	})
}

#[test]
fn create_pool_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_pool(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
			TEST_PRICE,
		));
		let event = Event::NFT(crate::Event::TokenPoolCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn create_pool_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NFTModule::create_pool(
				Origin::none(),
				"some metadata about token".as_bytes().to_vec(),
				ClassData { is_pool: true },
				TEST_PRICE,
			),
			BadOrigin
		);

		assert_noop!(
			NFTModule::create_pool(
				Origin::signed(ALICE),
				vec![1; <Test as orml_nft::Config>::MaxClassMetadata::get() as usize + 1],
				ClassData { is_pool: true },
				TEST_PRICE,
			),
			orml_nft::Error::<Test>::MaxMetadataExceeded
		);
	})
}

#[test]
fn mint_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));
		let event = Event::NFT(crate::Event::TokenClassCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));
		let event = Event::NFT(crate::Event::TokenMinted(ALICE, CLASS_ID, 0u32.into()));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn mint_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));
		let event = Event::NFT(crate::Event::TokenClassCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);

		assert_noop!(
			NFTModule::mint(
				Origin::signed(ALICE),
				NOT_EXISTING_CLASS_ID,
				"a token".as_bytes().to_vec(),
				TokenData { locked: false }
			),
			Error::<Test>::ClassNotFound
		);

		assert_noop!(
			NFTModule::mint(
				Origin::signed(BOB),
				0,
				"a token".as_bytes().to_vec(),
				TokenData { locked: false },
			),
			Error::<Test>::NotClassOwner
		);

		assert_noop!(
			NFTModule::mint(
				Origin::signed(ALICE),
				0,
				vec![1; <Test as orml_nft::Config>::MaxTokenMetadata::get() as usize + 1],
				TokenData { locked: false },
			),
			orml_nft::Error::<Test>::MaxMetadataExceeded
		);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)));
		let event = Event::NFT(crate::Event::TokenTransferred(ALICE, BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn transfer_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: true },
		));

		assert_noop!(
			NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)),
			Error::<Test>::NotTokenOwner
		);

		assert_noop!(
			NFTModule::transfer(Origin::signed(ALICE), ALICE, (CLASS_ID, TOKEN_ID)),
			Error::<Test>::CannotSendToSelf
		);

		assert_noop!(
			NFTModule::transfer(
				Origin::signed(ALICE),
				BOB,
				(NOT_EXISTING_CLASS_ID, NOT_EXISTING_TOKEN_ID)
			),
			Error::<Test>::TokenNotFound
		);

		assert_noop!(
			NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, 1)),
			Error::<Test>::TokenLocked
		);
	});
}

#[test]
fn burn_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::burn(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)));
		let event = Event::NFT(crate::Event::TokenBurned(ALICE, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn burn_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: true },
		));

		assert_noop!(
			NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Test>::NotTokenOwner
		);

		assert_noop!(
			NFTModule::burn(Origin::signed(ALICE), (NOT_EXISTING_CLASS_ID, NOT_EXISTING_TOKEN_ID)),
			Error::<Test>::TokenNotFound
		);

		assert_noop!(
			NFTModule::burn(Origin::signed(ALICE), (CLASS_ID, 1)),
			Error::<Test>::TokenLocked
		);
	});
}

#[test]
fn destroy_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::destroy_class(Origin::signed(ALICE), CLASS_ID));
	});
}

#[test]
fn destroy_class_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(ALICE), CLASS_ID),
			Error::<Test>::NonZeroIssuance
		);

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(ALICE), NOT_EXISTING_CLASS_ID),
			Error::<Test>::ClassNotFound
		);
	});
}

#[test]
fn destroy_pool_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_pool(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
			TEST_PRICE,
		));

		assert_ok!(NFTModule::destroy_pool(Origin::signed(ALICE), CLASS_ID));
	});
}

#[test]
fn destroy_pool_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_pool(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
			TEST_PRICE,
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_noop!(
			NFTModule::destroy_pool(Origin::signed(ALICE), CLASS_ID),
			Error::<Test>::NonZeroIssuance
		);

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(ALICE), NOT_EXISTING_CLASS_ID),
			Error::<Test>::ClassNotFound
		);
	});
}

#[test]
fn toggle_lock_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::toggle_lock(&ALICE, (CLASS_ID, TOKEN_ID)));
		let locked = NFTModule::is_locked((CLASS_ID, TOKEN_ID));
		assert!(locked.unwrap());
	});
}

#[test]
fn toggle_lock_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_noop!(
			NFTModule::toggle_lock(&BOB, (CLASS_ID, TOKEN_ID)),
			Error::<Test>::NotTokenOwner
		);

		assert_noop!(
			NFTModule::toggle_lock(&ALICE, (NOT_EXISTING_CLASS_ID, TOKEN_ID)),
			Error::<Test>::ClassNotFound
		);
	});
}

#[test]
fn buy_from_pool_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_pool(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
			TEST_PRICE,
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::buy_from_pool(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
		let event = Event::NFT(crate::Event::BoughtFromPool(ALICE, BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn buy_from_pool_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_pool(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
			TEST_PRICE,
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"locked token".as_bytes().to_vec(),
			TokenData { locked: true },
		));

		assert_noop!(
			NFTModule::buy_from_pool(Origin::signed(ALICE), (NOT_EXISTING_CLASS_ID, TOKEN_ID)),
			Error::<Test>::ClassNotFound
		);

		assert_noop!(
			NFTModule::buy_from_pool(Origin::signed(ALICE), (CLASS_ID, NOT_EXISTING_TOKEN_ID)),
			Error::<Test>::TokenNotFound
		);

		assert_noop!(
			NFTModule::buy_from_pool(Origin::signed(BOB), (CLASS_ID, 1)),
			Error::<Test>::TokenLocked
		);

		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)));

		assert_noop!(
			NFTModule::buy_from_pool(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
			Error::<Test>::TokenAlreadyHasAnOwner
		);

		assert_ok!(NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)));

		assert_noop!(
			NFTModule::buy_from_pool(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
			Error::<Test>::CannotBuyOwnToken
		);
	});
}

#[test]
fn buy_from_pool_fails_notapool() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: false },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_noop!(
			NFTModule::buy_from_pool(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Test>::NotAPool
		);
	});
}

#[test]
fn sell_to_pool_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)));

		assert_ok!(NFTModule::sell_to_pool(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
		let event = Event::NFT(crate::Event::SoldToPool(BOB, ALICE, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn sell_to_pool_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"not a pool".as_bytes().to_vec(),
			ClassData { is_pool: false },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			1,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_noop!(
			NFTModule::sell_to_pool(Origin::signed(ALICE), (NOT_EXISTING_CLASS_ID, TOKEN_ID)),
			Error::<Test>::ClassNotFound
		);

		assert_noop!(
			NFTModule::sell_to_pool(Origin::signed(ALICE), (CLASS_ID, NOT_EXISTING_TOKEN_ID)),
			Error::<Test>::TokenNotFound
		);

		assert_noop!(
			NFTModule::sell_to_pool(Origin::signed(ALICE), (1, TOKEN_ID)),
			Error::<Test>::NotAPool
		);

		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, 1)));

		assert_ok!(NFTModule::toggle_lock(&BOB, (CLASS_ID, 1)));

		assert_noop!(
			NFTModule::sell_to_pool(Origin::signed(BOB), (CLASS_ID, 1)),
			Error::<Test>::TokenLocked
		);

		assert_noop!(
			NFTModule::sell_to_pool(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
			Error::<Test>::CannotSellPoolToken
		);

		assert_ok!(NFTModule::transfer(
			Origin::signed(ALICE),
			CHARLIE,
			(CLASS_ID, TOKEN_ID)
		));

		assert_noop!(
			NFTModule::sell_to_pool(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Test>::NotTokenOwner
		);
	});
}

#[test]
fn is_owner_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));
		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));
		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));

		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), CHARLIE, (CLASS_ID, 1)));

		assert!(NFTModule::is_owner(&ALICE, (CLASS_ID, TOKEN_ID)));
		assert!(NFTModule::is_owner(&CHARLIE, (CLASS_ID, 1)));
	});
}

#[test]
fn is_locked_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			"some metadata about token".as_bytes().to_vec(),
			ClassData { is_pool: true },
		));
		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"a token".as_bytes().to_vec(),
			TokenData { locked: false },
		));
		assert_ok!(NFTModule::mint(
			Origin::signed(ALICE),
			0,
			"locked token".as_bytes().to_vec(),
			TokenData { locked: true },
		));

		assert_err!(NFTModule::is_locked((CLASS_ID, 198)), Error::<Test>::TokenNotFound);

		assert!(!NFTModule::is_locked((CLASS_ID, TOKEN_ID)).unwrap());
		assert!(NFTModule::is_locked((CLASS_ID, 1)).unwrap());
	});
}

// TODO: price!
