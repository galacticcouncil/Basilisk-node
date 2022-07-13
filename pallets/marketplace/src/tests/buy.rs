use super::*;
use pretty_assertions::assert_eq;

#[test]
fn buy_should_set_price_to_none_when_successfully_executed() {
	new_test_ext().execute_with(|| {
		// arrange
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));

		// act & assert
		assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100 * UNITS)
		));
        assert_ok!(Market::buy(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0));
		assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), None);
	});
}

#[test]
fn buy_should_set_price_to_none_when_offer_accepted() {
	new_test_ext().execute_with(|| {
		// arrange
		let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE),
			CLASS_ID_0,
			Default::default(),
			metadata.clone()
		));
		assert_ok!(NFT::mint(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, metadata));

        assert_ok!(Market::make_offer(
			Origin::signed(DAVE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			1000000
		));

        assert_ok!(Market::accept_offer(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			DAVE
		));

		assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), None);
	});
}