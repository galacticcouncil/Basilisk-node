use super::*;
use pretty_assertions::assert_eq;

#[test]
fn withdraw_offer_should_work_when_offer_has_been_already_made() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				COLLECTION_ID_0,
				ITEM_ID_0,
				50 * UNITS,
				1
			));

			//Act
			assert_ok!(Market::withdraw_offer(
				Origin::signed(BOB),
				COLLECTION_ID_0,
				ITEM_ID_0,
				BOB
			));

			//Assert
			assert_eq!(Market::offers((COLLECTION_ID_0, ITEM_ID_0), BOB), None);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&BOB), 0);

			assert_eq!(
				last_event(),
				Event::OfferWithdrawn {
					who: BOB,
					collection: COLLECTION_ID_0,
					item: ITEM_ID_0,
				}
				.into()
			);
		});
}

#[test]
fn withdraw_offer_should_fail_when_offer_does_not_exist() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (CHARLIE, 150_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			//Act and assert
			assert_noop!(
				Market::withdraw_offer(Origin::signed(CHARLIE), COLLECTION_ID_0, ITEM_ID_0, CHARLIE),
				Error::<Test>::UnknownOffer
			);
		});
}

#[test]
fn withdraw_offer_should_work_when_called_by_non_nft_owner() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				COLLECTION_ID_0,
				ITEM_ID_0,
				50 * UNITS,
				1
			));

			//Act and assert
			assert_noop!(
				Market::withdraw_offer(Origin::signed(CHARLIE), COLLECTION_ID_0, ITEM_ID_0, BOB),
				Error::<Test>::WithdrawNotAuthorized
			);
		});
}

#[test]
fn withdraw_offer_should_work_when_called_by_nft_owner() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				COLLECTION_ID_0,
				ITEM_ID_0,
				50 * UNITS,
				1
			));

			//Act
			assert_ok!(Market::withdraw_offer(
				Origin::signed(ALICE),
				COLLECTION_ID_0,
				ITEM_ID_0,
				BOB
			));

			//Assert
			assert_eq!(Market::offers((COLLECTION_ID_0, ITEM_ID_0), ALICE), None);
		});
}

#[test]
fn nft_owner_should_not_have_rights_for_withdrawing_when_nft_is_burned() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				COLLECTION_ID_0,
				ITEM_ID_0,
				50 * UNITS,
				1
			));

			assert_ok!(NFT::burn(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0));

			//Act and assert
			assert_noop!(
				Market::withdraw_offer(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0, BOB),
				Error::<Test>::WithdrawNotAuthorized
			);
		});
}

#[test]
fn nft_offer_maker_should_have_rights_for_withdrawing_when_nft_is_burned() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				COLLECTION_ID_0,
				ITEM_ID_0,
				50 * UNITS,
				1
			));

			assert_ok!(NFT::burn(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0));

			//Act
			assert_ok!(Market::withdraw_offer(
				Origin::signed(BOB),
				COLLECTION_ID_0,
				ITEM_ID_0,
				BOB
			));

			//Assert
			assert_eq!(Market::offers((COLLECTION_ID_0, ITEM_ID_0), ALICE), None);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&BOB), 0);
		});
}
