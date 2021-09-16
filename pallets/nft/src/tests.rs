use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::*;

type NFTModule = Pallet<Test>;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn create_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));
	})
}

#[test]
fn mint_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));
		assert_ok!(NFTModule::mint(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, ALICE, 10, bvec![0]));

		assert_noop!(
			NFTModule::mint(Origin::signed(CHARLIE), CLASS_ID, 999, BOB, 10, bvec![0]),
			pallet_uniques::Error::<Test>::NoPermission
		);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));
		assert_ok!(NFTModule::mint(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, ALICE, 10, bvec![0]));

		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, BOB));

		assert_noop!(
			NFTModule::transfer(Origin::signed(CHARLIE), CLASS_ID, TOKEN_ID, ALICE),
			pallet_uniques::Error::<Test>::NoPermission
		);
	});
}

#[test]
fn burn_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));
		assert_ok!(NFTModule::mint(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, ALICE, 10, bvec![0]));

		assert_noop!(
			NFTModule::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, Some(ALICE)),
			pallet_uniques::Error::<Test>::NoPermission
		);

		assert_ok!(NFTModule::burn(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, Some(ALICE)));
	});
}

#[test]
fn destroy_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));

		/* Currently there is no public constructor for DestroyWitness struct
		assert_noop!(
			NFTModule::destroy_class(
				Origin::signed(ALICE),
				CLASS_ID,
				DestroyWitness {
					instances: 1,
					instance_metadatas: 2,
					attributes: 3,
				}
				),
				pallet_uniques::Error::<Test>::BadWitness
		);

		assert_ok!(NFTModule::destroy_class(
			Origin::signed(ALICE),
			CLASS_ID,
			DestroyWitness {
				instances: 0,
				instance_metadatas: 0,
				attributes: 0
			}
		));
		let event = Event::Uniques(pallet_uniques::Event::Destroyed(CLASS_ID));
		assert_eq!(last_event(), event);
		*/
	});
}