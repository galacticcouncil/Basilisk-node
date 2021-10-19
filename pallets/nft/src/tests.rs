use frame_support::{assert_noop, assert_ok};

use crate::types::ClassType;

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
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), ClassType::Art));
		NextClassId::<Test>::mutate(|id| *id = <Test as UNQ::Config>::ClassId::max_value());
		assert_noop!(
			NFTPallet::create_class(Origin::signed(ALICE), ClassType::Art),
			Error::<Test>::NoAvailableClassId
		);
	})
}

#[test]
fn mint_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFTPallet::mint(Origin::signed(ALICE), CLASS_ID, bvec![0]));
		assert_ok!(NFTPallet::mint(Origin::signed(BOB), CLASS_ID, bvec![0]));
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFTPallet::mint(Origin::signed(ALICE), CLASS_ID, bvec![0]));

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
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFTPallet::mint(Origin::signed(ALICE), CLASS_ID, bvec![0]));

		assert_noop!(
			NFTPallet::burn(Origin::signed(BOB), CLASS_ID, TOKEN_ID),
			UNQ::Error::<Test>::NoPermission
		);

		assert_ok!(NFTPallet::burn(Origin::signed(ALICE), CLASS_ID, TOKEN_ID));
	});
}

#[test]
fn destroy_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(Origin::signed(ALICE), ClassType::Art));
		assert_ok!(NFTPallet::mint(Origin::signed(BOB), CLASS_ID, bvec![0]));

		assert_noop!(
			NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID),
			UNQ::Error::<Test>::NoPermission
		);

		assert_noop!(
			NFTPallet::burn(Origin::signed(ALICE), CLASS_ID, 0,),
			UNQ::Error::<Test>::NoPermission
		);

		assert_ok!(NFTPallet::burn(Origin::signed(BOB), CLASS_ID, 0));
		assert_ok!(NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID));
	});
}
