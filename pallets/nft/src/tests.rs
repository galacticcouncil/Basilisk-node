use frame_support::{assert_noop, assert_ok};

use crate::types::ClassType;

use super::*;
use mock::*;
use pallet_uniques as UNQ;

type NFTPallet = Pallet<Test>;

#[test]
fn create_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		// assert_noop!(
		// 	NFTPallet::create_class(Origin::signed(ALICE), ClassType::PoolShare, b"metadata".to_vec()),
		// 	UNQ::Error::<Test>::NoPermission
		// );
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_noop!(
			NFTPallet::create_class(Origin::signed(ALICE), ClassType::Marketplace, vec![0; <Test as UNQ::Config>::StringLimit::get() as usize + 1]),
			Error::<Test>::TooLong
		);
		NextClassId::<Test>::mutate(|id| *id = <Test as UNQ::Config>::ClassId::max_value());
		assert_noop!(
			NFTPallet::create_class(Origin::signed(ALICE), ClassType::Marketplace, b"metadata".to_vec()),
			Error::<Test>::NoAvailableClassId
		);
	})
}

#[test]
fn mint_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Some(ALICE),
			Some(20),
			Some(b"metadata".to_vec())
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			CLASS_ID_0,
			Some(CHARLIE),
			Some(20),
			Some(b"metadata".to_vec())
		));

		assert_ok!(NFTPallet::create_class(
			Origin::signed(BOB),
			ClassType::PoolShare,
			b"metadata".to_vec()
		));
		assert_noop!(
			NFTPallet::mint(
				Origin::signed(ALICE),
				999, // non existent class id
				Some(CHARLIE),
				Some(20),
				Some(b"metadata".to_vec())
			),
			Error::<Test>::ClassUnknown
		);
		assert_noop!(
			NFTPallet::mint(
				Origin::signed(ALICE),
				CLASS_ID_0,
				None,
				Some(20),
				Some(b"metadata".to_vec())
			),
			Error::<Test>::AuthorNotSet
		);
		assert_noop!(
			NFTPallet::mint(
				Origin::signed(ALICE),
				CLASS_ID_0,
				Some(CHARLIE),
				None,
				Some(b"metadata".to_vec())
			),
			Error::<Test>::RoyaltyNotSet
		);
		assert_noop!(
			NFTPallet::mint(
				Origin::signed(ALICE),
				CLASS_ID_0,
				Some(CHARLIE),
				Some(20),
				None
			),
			Error::<Test>::MetadataNotSet
		);
		assert_noop!(
			NFTPallet::mint(
				Origin::signed(ALICE),
				CLASS_ID_0,
				Some(CHARLIE),
				Some(123),
				Some(b"metadata".to_vec())
			),
			Error::<Test>::NotInRange
		);
		//assert_noop!(NFTPallet::mint(Origin::signed(BOB), CLASS_ID_1, b"metadata".to_vec()), BadOrigin);
		//assert_noop!(NFTPallet::mint(Origin::signed(ALICE), CLASS_ID_1, b"metadata".to_vec()), BadOrigin);
		//assert_ok!(NFTPallet::mint(Origin::root(), CLASS_ID_1, b"metadata".to_vec()));
		//assert_ok!(NFTPallet::create_class(Origin::root(), ClassType::PoolShare, b"metadata".to_vec()));
		NextInstanceId::<Test>::mutate(CLASS_ID_0, |id| *id = <Test as UNQ::Config>::InstanceId::max_value());
		assert_noop!(
			NFTPallet::mint(
				Origin::signed(ALICE),
				CLASS_ID_0,
				Some(CHARLIE),
				Some(20),
				Some(b"metadata".to_vec())
			),
			Error::<Test>::NoAvailableInstanceId
		);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Some(ALICE),
			Some(20),
			Some(b"metadata".to_vec())
		));

		assert_ok!(NFTPallet::transfer(Origin::signed(ALICE), CLASS_ID_0, TOKEN_ID, BOB));

		assert_noop!(
			NFTPallet::transfer(Origin::signed(CHARLIE), CLASS_ID_0, TOKEN_ID, ALICE),
			UNQ::Error::<Test>::NoPermission
		);
	});
}

#[test]
fn burn_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Some(ALICE),
			Some(20),
			Some(b"metadata".to_vec())
		));

		assert_noop!(
			NFTPallet::burn(Origin::signed(BOB), CLASS_ID_0, TOKEN_ID),
			UNQ::Error::<Test>::NoPermission
		);

		assert_ok!(NFTPallet::burn(Origin::signed(ALICE), CLASS_ID_0, TOKEN_ID));
	});
}

#[test]
fn destroy_class_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			ClassType::Marketplace,
			b"metadata".to_vec()
		));
		assert_ok!(NFTPallet::mint(
			Origin::signed(BOB),
			CLASS_ID_0,
			Some(CHARLIE),
			Some(20),
			Some(b"metadata".to_vec())
		));

		// assert_noop!(
		// 	NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID_0),
		// 	Error::<Test>::TokenClassNotEmpty
		// );

		assert_ok!(NFTPallet::burn(Origin::signed(BOB), CLASS_ID_0, 0));
		assert_ok!(NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID_0));
		assert_noop!(
			NFTPallet::destroy_class(Origin::signed(ALICE), CLASS_ID_0),
			Error::<Test>::NoWitness
		);
	});
}
