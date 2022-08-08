use frame_support::{assert_noop, assert_ok};

use crate::types::BondingCurve;

use super::*;
use mock::*;
use crate::mocked_objects::*;

type Redeemables = Pallet<Test>;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

#[test]
fn trading_works() {
	ExtBuilder::default().build().execute_with(|| {
		let bc = BondingCurve {
			exponent: 2,
			slope: 1_000_000_000,
		};

		let class_id = mocked_nft_class_id_1::<Test>();
		let instance_id = mocked_nft_instance_id_1::<Test>();

		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			class_id,
			ClassType::Redeemable,
			bvec![0]
		));

		Pallet::<Test>::add_redeemables_class_info(Origin::signed(ALICE), class_id, 150, bc).unwrap();
		for _ in 1..149 {
			assert_ok!(Redeemables::buy(Origin::signed(ALICE), class_id));
		}

		for x in 0..9 {
			assert_ok!(Redeemables::sell(Origin::signed(ALICE), class_id, x));
		}

		for x in 10..19 {
			assert_ok!(Redeemables::redeem(Origin::signed(ALICE), class_id, x));
		}

		for _ in 0..10 {
			assert_ok!(Redeemables::buy(Origin::signed(ALICE), class_id));
		}

		assert_noop!(
			Redeemables::buy(Origin::signed(ALICE), class_id),
			Error::<Test>::ReachedMaxSupply
		);
	})
}
