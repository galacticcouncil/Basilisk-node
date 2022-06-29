pub use super::mock::*;
use crate::Error;
use frame_support::{assert_noop, assert_ok};
use hydradx_traits::AMM as AmmPool;

use primitives::Price;

#[test]
fn fee_calculation() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(XYK::calculate_discounted_fee(10000), Ok(7));
		assert_eq!(XYK::calculate_discounted_fee(100000), Ok(70));
		assert_eq!(XYK::calculate_discounted_fee(100000), Ok(70));

		assert_eq!(XYK::calculate_fee(100000), Ok(200));
		assert_eq!(XYK::calculate_fee(10000), Ok(20));
	});
	ExtBuilder::default()
		.with_exchange_fee((10, 1000))
		.build()
		.execute_with(|| {
			assert_eq!(XYK::calculate_fee(100000), Ok(1000));
			assert_eq!(XYK::calculate_fee(10000), Ok(100));
		});

	ExtBuilder::default()
		.with_exchange_fee((10, 0))
		.build()
		.execute_with(|| {
			assert_eq!(XYK::calculate_fee(100000), Ok(0));
		});

	ExtBuilder::default()
		.with_exchange_fee((10, 1))
		.build()
		.execute_with(|| {
			assert_noop!(XYK::calculate_fee(u128::MAX), Error::<Test>::FeeAmountInvalid);
		});
}

#[test]
fn get_fee_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(XYK::create_pool(
			Origin::signed(ALICE),
			HDX,
			DOT,
			1_000_000_000,
			Price::from(2)
		),);

		// existing pool
		let fee = XYK::get_fee(&HDX_DOT_POOL_ID);
		assert_eq!(fee, (2, 1_000));
		// non existing pool
		let fee = XYK::get_fee(&1_234);
		assert_eq!(fee, (2, 1_000));
	});
}
