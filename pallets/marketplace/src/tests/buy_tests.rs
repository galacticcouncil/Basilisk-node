use super::*;
use pretty_assertions::assert_eq;
use sp_core::{crypto::AccountId32};

type AccountId = AccountId32;

//TODO: Dani - rename to buy and merge the tests from buy as jgreen created 2 testcases for an issue we have: https://github.com/galacticcouncil/Basilisk-node/issues/521

#[test]
fn buy_should_work_when_there_is_no_royalty() {
    //Arrange
    ExtBuilder::default()
        .with_endowed_accounts(vec![
            (ALICE, 200_000 * UNITS),
            (CHARLIE, 150_000 * UNITS),
        ])
        .with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
        .build()
        .execute_with(|| {
            let price = 100 * UNITS;

            // make an offer to verify that it is ignored
            assert_ok!(Market::make_offer(
                Origin::signed(CHARLIE),
                CLASS_ID_0,
                INSTANCE_ID_0,
                50 * UNITS,
                2
		    ));

            assert_ok!(Market::set_price(
                Origin::signed(ALICE),
                CLASS_ID_0,
                INSTANCE_ID_0,
                Some(100 * UNITS)
            ));

            let alice_initial_balance = Balances::free_balance(&ALICE);
            let charlie_initial_balance = Balances::free_balance(&CHARLIE);

            //Act
            assert_ok!(Market::buy(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0,));

            //Assert
            assert_that_nft_ownership_is_transferred_to(CHARLIE);

            // existing orders are not removed from the storage
            assert!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), CHARLIE).is_some());

            assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), None);

            assert_eq!(last_event(), Event::Marketplace(crate::Event::TokenSold {
                owner: ALICE,
                buyer: CHARLIE,
                class: CLASS_ID_0,
                instance: INSTANCE_ID_0,
                price,
            }));

            assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + price);
            assert_eq!(<Test as Config>::Currency::reserved_balance(&CHARLIE), 50 * UNITS);
            assert_eq!(Balances::free_balance(&CHARLIE), charlie_initial_balance - price);
        });
}

#[test]
fn buy_should_fail_when_nft_does_not_exist() {
    //Arrange
    ExtBuilder::default()
        .with_endowed_accounts(vec![
            (BOB, 150_000 * UNITS),
        ])
        .build()
        .execute_with(|| {
            //Act and assert
            assert_noop!(
                Market::buy(Origin::signed(BOB), CLASS_ID_0, INSTANCE_ID_0),
                Error::<Test>::ClassOrInstanceUnknown
		    );
        });
}

#[test]
fn buy_should_fail_when_price_is_not_set_for_nft() {
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
            //Act and assert
            assert_noop!(
                Market::buy(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0),
                Error::<Test>::NotForSale
	    	);
        });
}

#[test]
fn buy_should_fail_when_buyer_has_insufficient_balance() {
    //Arrange
    let buyer_balance = 99;
    ExtBuilder::default()
        .with_endowed_accounts(vec![
            (ALICE, 200_000 * UNITS),
            (CHARLIE, buyer_balance * UNITS),
        ])
        .with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
        .build()
        .execute_with(|| {
            assert_ok!(Market::set_price(
                Origin::signed(ALICE),
                CLASS_ID_0,
                INSTANCE_ID_0,
                Some((buyer_balance + 1) * UNITS)
            ));

            //Act and assert
            assert_noop!(
                Market::buy(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0),
                pallet_balances::Error::<Test, _>::InsufficientBalance
		    );
        });
}

#[test]
fn buy_should_fail_when_buyer_is_the_owner() {
    //Arrange
    ExtBuilder::default()
        .with_endowed_accounts(vec![
            (ALICE, 200_000 * UNITS),
        ])
        .with_minted_nft((ALICE, CLASS_ID_0, INSTANCE_ID_0))
        .build()
        .execute_with(|| {
            assert_ok!(Market::set_price(
                Origin::signed(ALICE),
                CLASS_ID_0,
                INSTANCE_ID_0,
                Some(100 * UNITS)
            ));

            //Act and assert
            assert_noop!(
                Market::buy(Origin::signed(ALICE), CLASS_ID_0, INSTANCE_ID_0),
                Error::<Test>::BuyFromSelf
            );
        });
}

#[test]
fn buy_should_work_when_there_is_royalty() {
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
            let price = 100 * UNITS;
            assert_ok!(Market::add_royalty(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			BOB,
			20,
		));
            // make an offer to verify that it is ignored
            assert_ok!(Market::make_offer(
			Origin::signed(CHARLIE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			50 * UNITS,
			2
		));

            assert_ok!(Market::set_price(
			Origin::signed(ALICE),
			CLASS_ID_0,
			INSTANCE_ID_0,
			Some(100 * UNITS)
		));

            let alice_initial_balance = Balances::free_balance(&ALICE);
            let bob_initial_balance = Balances::free_balance(&BOB);
            let charlie_initial_balance = Balances::free_balance(&CHARLIE);

            //Act
            assert_ok!(Market::buy(Origin::signed(CHARLIE), CLASS_ID_0, INSTANCE_ID_0,));

            //Assert
            assert_that_nft_ownership_is_transferred_to(CHARLIE);

            assert!(Market::offers((CLASS_ID_0, INSTANCE_ID_0), CHARLIE).is_some());

            assert_eq!(Market::prices(CLASS_ID_0, INSTANCE_ID_0), None);
            expect_events(vec![
                crate::Event::RoyaltyPaid {
                    class: CLASS_ID_0,
                    instance: INSTANCE_ID_0,
                    author: BOB,
                    royalty: 20,
                    royalty_amount: 20 * UNITS,
                }
                    .into(),
                crate::Event::TokenSold {
                    owner: ALICE,
                    buyer: CHARLIE,
                    class: CLASS_ID_0,
                    instance: INSTANCE_ID_0,
                    price: 80 * UNITS,
                }
                    .into(),
            ]);
            assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance + 80 * UNITS); // price - royalty
            assert_eq!(<Test as Config>::Currency::reserved_balance(&CHARLIE), 50 * UNITS);
            assert_eq!(Balances::free_balance(&BOB), bob_initial_balance + 20 * UNITS); // royalty
            assert_eq!(Balances::free_balance(&CHARLIE), charlie_initial_balance - price);
        });
}

fn assert_that_nft_ownership_is_transferred_to(new_owner: AccountId) {
    assert_eq!(
        pallet_uniques::Pallet::<Test>::owner(CLASS_ID_0, INSTANCE_ID_0),
        Some(new_owner)
    );
}