use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::*;
use pallet_uniques as UNQ;

type NFTPallet = Pallet<Test>;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn create_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));

		assert_noop!(
			NFTPallet::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]),
			UNQ::Error::<Test>::InUse		
		);
	})
}

#[test]
fn mint_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));
		assert_ok!(NFTPallet::mint(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, ALICE, 10, bvec![0]));

		assert_noop!(
			NFTPallet::mint(Origin::signed(CHARLIE), CLASS_ID, 999, BOB, 10, bvec![0]),
			UNQ::Error::<Test>::NoPermission
		);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));
		assert_ok!(NFTPallet::mint(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, ALICE, 10, bvec![0]));

		assert_ok!(NFTPallet::transfer(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, BOB));

		assert_noop!(
			NFTPallet::transfer(Origin::signed(CHARLIE), CLASS_ID, TOKEN_ID, ALICE),
			UNQ::Error::<Test>::NoPermission
		);
	});
}

#[test]
fn burn_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));
		assert_ok!(NFTPallet::mint(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, ALICE, 10, bvec![0]));

		assert_noop!(
			NFTPallet::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID, Some(ALICE)),
			UNQ::Error::<Test>::NoPermission
		);

		assert_ok!(NFTPallet::burn(Origin::signed(ALICE), CLASS_ID, TOKEN_ID, Some(ALICE)));
	});
}

#[test]
fn destroy_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), CLASS_ID, ALICE, bvec![0]));

		/* Currently there is no public constructor for DestroyWitness struct
		assert_noop!(
			NFTPallet::destroy_class(
				Origin::signed(ALICE),
				CLASS_ID,
				DestroyWitness {
					instances: 1,
					instance_metadatas: 2,
					attributes: 3,
				}
				),
				UNQ::Error::<Test>::BadWitness
		);

		assert_ok!(NFTPallet::destroy_class(
			Origin::signed(ALICE),
			CLASS_ID,
			DestroyWitness {
				instances: 0,
				instance_metadatas: 0,
				attributes: 0
			}
		));
		let event = Event::Uniques(UNQ::Event::Destroyed(CLASS_ID));
		assert_eq!(last_event(), event);
		*/
	});
}