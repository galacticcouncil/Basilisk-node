use super::*;
use pretty_assertions::assert_eq;

const PRICE: Balance = 50 * UNITS;

#[test]
fn accept_offer_should_work_when_there_is_no_royalty() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				CLASS_ID_0,
				INSTANCE_ID_0,
				PRICE,
				2
			));

			let alice_initial_balance = Balances::free_balance(&ALICE);
			let bob_initial_balance = Balances::free_balance(&BOB);

			//Act
			assert_ok!(Market::accept_offer(
				Origin::signed(ALICE),
				CLASS_ID_0,
				INSTANCE_ID_0,
				BOB
			));

			//Assert
			assert_eq!(
				last_event(),
				Event::OfferAccepted {
					who: ALICE,
					class: CLASS_ID_0,
					instance: INSTANCE_ID_0,
					amount: PRICE,
					maker: BOB,
				}
				.into()
			);

			assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
			assert_eq!(
				pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
				Some(BOB)
			);

			assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + PRICE);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&BOB), 0);
			assert_eq!(Balances::free_balance(&BOB), bob_initial_balance); // paid from the reserved amount
		})
}

#[test]
fn accept_offer_should_work_when_there_is_royalty_present() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, 200_000 * UNITS),
			(BOB, 15_000 * UNITS),
			(CHARLIE, 150_000 * UNITS),
			(DAVE, 200_000 * UNITS),
		])
		.with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::add_royalty(
				Origin::signed(ALICE),
				CLASS_ID_0,
				INSTANCE_ID_0,
				CHARLIE,
				20,
			));
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				CLASS_ID_0,
				INSTANCE_ID_0,
				PRICE,
				2
			));

			let alice_initial_balance = Balances::free_balance(&ALICE);
			let bob_initial_balance = Balances::free_balance(&BOB);
			let charlie_initial_balance = Balances::free_balance(&CHARLIE);

			//Act
			assert_ok!(Market::accept_offer(
				Origin::signed(ALICE),
				CLASS_ID_0,
				INSTANCE_ID_0,
				BOB
			));

			//Assert
			assert_eq!(
				last_event(),
				Event::OfferAccepted {
					who: ALICE,
					class: CLASS_ID_0,
					instance: INSTANCE_ID_0,
					amount: PRICE,
					maker: BOB,
				}
				.into()
			);
			assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
			assert_eq!(
				pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
				Some(BOB)
			);
			assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + 40 * UNITS); // price - royalty
			assert_eq!(<Test as Config>::Currency::reserved_balance(&BOB), 0);
			assert_eq!(Balances::free_balance(&BOB), bob_initial_balance); // paid from the reserved amount
			assert_eq!(Balances::free_balance(&CHARLIE), charlie_initial_balance + 10 * UNITS);
		});
}

#[test]
fn accept_offer_should_fail_when_there_is_no_offer_present() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
		.build()
		.execute_with(|| {
			//Act and Assert
			assert_noop!(
				Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, BOB),
				Error::<Test>::UnknownOffer
			);
		})
}

#[test]
fn accept_offer_should_fail_when_offer_is_expired() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
		.build()
		.execute_with(|| {
			let first_block = 1;

			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				CLASS_ID_0,
				INSTANCE_ID_0,
				PRICE,
				first_block
			));

			//Act and Assert
			assert_noop!(
				Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, BOB),
				Error::<Test>::OfferExpired
			);
		})
}

#[test]
fn accept_offer_should_fail_when_nft_does_not_exist() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.build()
		.execute_with(|| {
			//Act and Assert
			assert_noop!(
				Market::accept_offer(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0, BOB),
				Error::<Test>::ClassOrInstanceUnknown
			);
		})
}

#[test]
fn accept_offer_should_fail_when_called_by_not_nft_owner() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 200_000 * UNITS), (BOB, 15_000 * UNITS)])
		.with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				CLASS_ID_0,
				INSTANCE_ID_0,
				PRICE,
				2
			));

			//Act and Assert
			assert_noop!(
				Market::accept_offer(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0, BOB),
				Error::<Test>::AcceptNotAuthorized
			);
		})
}

#[test]
fn accept_offer_should_work_when_nft_has_royalty_and_price_is_set() {
	//Arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(ALICE, 200_000 * UNITS),
			(BOB, 15_000 * UNITS),
			(CHARLIE, 150_000 * UNITS),
		])
		.with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::add_royalty(
				Origin::signed(ALICE),
				CLASS_ID_0,
				INSTANCE_ID_0,
				CHARLIE,
				20,
			));
			assert_ok!(Market::set_price(
				Origin::signed(ALICE),
				CLASS_ID_0,
				INSTANCE_ID_0,
				Some(100 * UNITS)
			));
			assert_ok!(Market::make_offer(
				Origin::signed(BOB),
				CLASS_ID_0,
				INSTANCE_ID_0,
				PRICE,
				2
			));

			let alice_initial_balance = Balances::free_balance(&ALICE);
			let bob_initial_balance = Balances::free_balance(&BOB);
			let charlie_initial_balance = Balances::free_balance(&CHARLIE);

			//Act
			// price set by the owner is ignored
			assert_ok!(Market::accept_offer(
				Origin::signed(ALICE),
				CLASS_ID_0,
				INSTANCE_ID_0,
				BOB
			));

			//Assert
			assert_eq!(
				last_event(),
				Event::OfferAccepted {
					who: ALICE,
					class: CLASS_ID_0,
					instance: INSTANCE_ID_0,
					amount: PRICE,
					maker: BOB,
				}
				.into()
			);
			assert_eq!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), BOB), None);
			assert_eq!(
				pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
				Some(BOB)
			);
			assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + 40 * UNITS); // price - royalty
			assert_eq!(<Test as Config>::Currency::reserved_balance(&BOB), 0);
			assert_eq!(Balances::free_balance(&BOB), bob_initial_balance); // paid from the reserved amount
			assert_eq!(Balances::free_balance(&CHARLIE), charlie_initial_balance + 10 * UNITS);
		});
}

#[test]
fn buy_should_set_price_to_none_when_offer_accepted() {
	// arrange
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 2_000_000 * UNITS), (DAVE, 2_000_000 * UNITS)])
		.with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
		.build()
		.execute_with(|| {
			assert_ok!(Market::make_offer(
				Origin::signed(DAVE),
				CLASS_ID_0,
				INSTANCE_ID_0,
				50 * UNITS,
				1000000
			));

			// Act
			assert_ok!(Market::accept_offer(
				Origin::signed(ALICE),
				CLASS_ID_0,
				INSTANCE_ID_0,
				DAVE
			));

			// Assert
			assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), None);
		});
}
