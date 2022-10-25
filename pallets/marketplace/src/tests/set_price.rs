use super::*;
use pretty_assertions::assert_eq;

#[test]
fn set_price_should_work_when_nft_exists() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			//Act
			assert_ok!(Market::set_price(
				Origin::signed(ALICE),
				COLLECTION_ID_0,
				ITEM_ID_0,
				Some(10)
			));

			//Assert
			assert_eq!(Market::prices(COLLECTION_ID_0, ITEM_ID_0), Some(10));
			assert_eq!(
				last_event(),
				Event::TokenPriceUpdated {
					who: ALICE,
					collection: COLLECTION_ID_0,
					item: ITEM_ID_0,
					price: Some(10),
				}
				.into()
			);
		});
}

#[test]
fn set_price_should_fail_when_nft_does_not_exist() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(BOB, 15_000 * UNITS)])
		.build()
		.execute_with(|| {
			//Act and assert
			assert_noop!(
				Market::set_price(Origin::signed(BOB), COLLECTION_ID_0, ITEM_ID_1, Some(10)),
				Error::<Test>::NotTheTokenOwner
			);
		});
}

#[test]
fn set_price_should_fail_when_called_by_not_owner() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			//Act and assert
			assert_noop!(
				Market::set_price(Origin::signed(BOB), COLLECTION_ID_0, ITEM_ID_0, Some(10)),
				Error::<Test>::NotTheTokenOwner
			);
		});
}

#[test]
fn set_price_should_update_price_when_called_second_time() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::set_price(
				Origin::signed(ALICE),
				COLLECTION_ID_0,
				ITEM_ID_0,
				Some(10)
			));

			// Act
			assert_ok!(Market::set_price(
				Origin::signed(ALICE),
				COLLECTION_ID_0,
				ITEM_ID_0,
				Some(20)
			));

			assert_eq!(Market::prices(COLLECTION_ID_0, ITEM_ID_0), Some(20));
			let event = Event::TokenPriceUpdated {
				who: ALICE,
				collection: COLLECTION_ID_0,
				item: ITEM_ID_0,
				price: Some(20),
			}
			.into();
			assert_eq!(last_event(), event);
		});
}

#[test]
fn set_price_should_disable_price_when_called_with_no_price() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			//Act
			assert_ok!(Market::set_price(
				Origin::signed(ALICE),
				COLLECTION_ID_0,
				ITEM_ID_0,
				None
			));

			//Assert
			assert_eq!(Market::prices(COLLECTION_ID_0, ITEM_ID_0), None);
		});
}
