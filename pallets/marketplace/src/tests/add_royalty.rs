use super::*;
use pretty_assertions::assert_eq;

#[test]
fn add_royalty_should_work_when_nft_exists() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			let reserved_before_royalty = <Test as Config>::Currency::reserved_balance(&ALICE);

			//Act
			assert_ok!(Market::add_royalty(
				Origin::signed(ALICE),
				COLLECTION_ID_0,
				ITEM_ID_0,
				CHARLIE,
				2_000,
			));

			//Assert
			assert_eq!(
				Market::marketplace_items(COLLECTION_ID_0, ITEM_ID_0),
				Some(Royalty {
					author: CHARLIE,
					royalty: 2_000
				})
			);

			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&ALICE) - reserved_before_royalty,
				<Test as Config>::RoyaltyBondAmount::get()
			);

			assert_eq!(
				last_event(),
				Event::RoyaltyAdded {
					collection: COLLECTION_ID_0,
					item: ITEM_ID_0,
					author: CHARLIE,
					royalty: 2_000,
				}
				.into()
			);
		});
}

#[test]
fn add_royalty_should_fail_when_royalty_is_set_to_100_percent() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			//Act and assert
			assert_noop!(
				Market::add_royalty(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0, CHARLIE, 10_000),
				Error::<Test>::NotInRange
			);
		});
}

#[test]
fn add_royalty_should_fail_when_nft_does_not_exist() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS)])
		.build()
		.execute_with(|| {
			//Act and assert
			assert_noop!(
				Market::add_royalty(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_1, CHARLIE, 2_000),
				pallet_nft::Error::<Test>::CollectionUnknown
			);
		});
}

#[test]
fn add_royalty_should_fail_when_called_by_non_owner() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (CHARLIE, 150_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			//Act and assert
			assert_noop!(
				Market::add_royalty(Origin::signed(CHARLIE), COLLECTION_ID_0, ITEM_ID_0, CHARLIE, 2_000),
				pallet_nft::Error::<Test>::NotPermitted
			);
		});
}

#[test]
fn add_royalty_should_should_fail_when_royalty_is_already_set() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS)])
		.with_minted_nft((ALICE, COLLECTION_ID_0, ITEM_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::add_royalty(
				Origin::signed(ALICE),
				COLLECTION_ID_0,
				ITEM_ID_0,
				CHARLIE,
				2_000,
			));

			//Assert and assert
			assert_noop!(
				Market::add_royalty(Origin::signed(ALICE), COLLECTION_ID_0, ITEM_ID_0, CHARLIE, 2_000),
				Error::<Test>::RoyaltyAlreadySet
			);
		});
}
