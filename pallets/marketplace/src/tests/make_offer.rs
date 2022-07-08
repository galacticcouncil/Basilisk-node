use super::*;
use pretty_assertions::assert_eq;

#[test]
fn make_offer_should_work_when_nft_exists() {
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

		// act
		assert_ok!(Market::make_offer(
			Origin::signed(CHARLIE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			2
		));

		// assert
		assert_eq!(
			Market::offers((CLASS_ID_0, INSTANCE_ID_0), CHARLIE),
			Some(Offer {
				maker: CHARLIE,
				amount: 50 * UNITS,
				expires: 2,
			})
		);
		let event = Event::Marketplace(crate::Event::OfferPlaced {
			who: CHARLIE,
			class: CLASS_ID_0,
			instance: INSTANCE_ID_0,
			amount: 50 * UNITS,
			expires: 2,
		});
		assert_eq!(last_event(), event);
	});
}

#[test]
fn make_offer_should_fail_when_offer_is_lower_than_minimal_amount() {
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
		assert_noop!(
			Market::make_offer(
				Origin::signed(BOB),
				CLASS_ID_0,
				INSTANCE_ID_0,
				<Test as Config>::MinimumOfferAmount::get() - 1,
				1
			),
			Error::<Test>::OfferTooLow
		);
		assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), CHARLIE), None);
	});
}
