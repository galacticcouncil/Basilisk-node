use frame_support::{assert_noop, assert_ok};

use super::*;
use mock::{Event, *};
use pallet_nft::types::*;

type Redeemables = Pallet<Test>;

#[test]
fn buy_works() { 
	ExtBuilder::default().build().execute_with(|| {
		let bc = BondingCurve { exponent: 2, slope: 1_000_000_000 };
		let class_id = NFT::create_class_for_redeemables(150, bc).unwrap();
		for _ in 0..4 {
			assert_ok!(Redeemables::buy(Origin::signed(ALICE), class_id));
		}
		assert_ok!(Redeemables::buy(Origin::signed(BOB), class_id));
		assert_ok!(Redeemables::buy(Origin::signed(CHARLIE), class_id));
		
		assert_ok!(Redeemables::sell(Origin::signed(ALICE), class_id, 0));

		// assert_ok!(Redeemables::sell(Origin::signed(CHARLIE), class_id, 6));

		// assert_ok!(Redeemables::buy(Origin::signed(CHARLIE), class_id));
		// assert_ok!(Redeemables::sell(Origin::signed(CHARLIE), class_id, 7));
		// assert_ok!(Redeemables::buy(Origin::signed(BOB), class_id));

		// assert_noop!(Redeemables::buy(Origin::signed(ALICE), class_id), pallet_nft::Error::<Test>::ReachedMaxSupply);
	})
}
