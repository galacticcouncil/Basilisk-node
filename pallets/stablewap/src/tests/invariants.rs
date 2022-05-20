use crate::math::{calculate_d, calculate_in_given_out, calculate_out_given_in};
use crate::Balance;
use proptest::prelude::*;
use proptest::proptest;

pub const ONE: Balance = 1_000_000_000_000;

const RESERVE_RANGE: (Balance, Balance) = (100_000 * ONE, 10_000_000 * ONE);

fn trade_amount() -> impl Strategy<Value = Balance> {
	//Just(1000 * ONE)
	1000..10000 * ONE
}

fn asset_reserve() -> impl Strategy<Value = Balance> {
	RESERVE_RANGE.0..RESERVE_RANGE.1
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn test_out_given_in(amount_in in trade_amount(),
		reserve_in in  asset_reserve(),
		reserve_out in  asset_reserve(),
	) {
		let amp = 1u128;
		let ann = 4u128;

		let precision = 1u128;

		let _d1 = calculate_d(&[reserve_in, reserve_out], ann, precision).unwrap();

		let result = calculate_out_given_in(reserve_in, reserve_out, amount_in, precision, amp);

		assert!(result.is_some());

		let _d2 = calculate_d(&[reserve_in + amount_in, reserve_out - result.unwrap() ], ann, precision).unwrap();

		//assert!(d2 >= d1);
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn test_in_given_out(amount_out in trade_amount(),
		reserve_in in  asset_reserve(),
		reserve_out in  asset_reserve(),
	) {
		let amp = 1u128;
		let ann = 4u128;

		let precision = 1u128;

		let _d1 = calculate_d(&[reserve_in, reserve_out], ann, precision).unwrap();

		let result = calculate_in_given_out(reserve_in, reserve_out, amount_out, precision, amp);

		assert!(result.is_some());

		let _d2 = calculate_d(&[reserve_in + result.unwrap(), reserve_out - amount_out ], ann, precision).unwrap();

		//assert!(_d2 >= _d1);
	}
}
