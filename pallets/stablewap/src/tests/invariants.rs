use crate::math::{calculate_d, calculate_in_given_out, calculate_out_given_in};
use crate::Balance;
use proptest::prelude::*;
use proptest::proptest;

pub const ONE: Balance = 1_000_000_000_000;

const RESERVE_RANGE: (Balance, Balance) = (100_000 * ONE, 100_000_000 * ONE);
const LOW_RESERVE_RANGE: (Balance, Balance) = (10_u128, 11_u128);
const HIGH_RESERVE_RANGE: (Balance, Balance) = (500_000_000_000 * ONE, 500_000_000_001 * ONE);

fn trade_amount() -> impl Strategy<Value = Balance> {
	//Just(1000 * ONE)
	1000..10000 * ONE
}

fn high_trade_amount() -> impl Strategy<Value = Balance> {
	500_000_000_000*ONE..500_000_000_001 * ONE
}

fn asset_reserve() -> impl Strategy<Value = Balance> {
	RESERVE_RANGE.0..RESERVE_RANGE.1
}
fn low_asset_reserve() -> impl Strategy<Value = Balance> {
	LOW_RESERVE_RANGE.0..LOW_RESERVE_RANGE.1
}
fn high_asset_reserve() -> impl Strategy<Value = Balance> {
	HIGH_RESERVE_RANGE.0..HIGH_RESERVE_RANGE.1
}

fn amplification() -> impl Strategy<Value = Balance> {
	5..10000u128
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn test_d_extreme(reserve_in in  low_asset_reserve(),
		reserve_out in  high_asset_reserve(),
		amp in amplification(),
	) {
		let ann = amp * 4u128;

		let precision = 1u128;

		let d = calculate_d(&[reserve_in, reserve_out], ann, precision);

		assert!(!d.is_none());
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn test_out_given_in_extreme(amount_in in high_trade_amount(),
		reserve_in in  low_asset_reserve(),
		reserve_out in  high_asset_reserve(),
		amp in amplification(),
	) {
		let ann = amp * 4u128;

		let precision = 1u128;

		let d1 = calculate_d(&[reserve_in, reserve_out], ann, precision).unwrap();

		let result = calculate_out_given_in(reserve_in, reserve_out, amount_in, precision, amp);

		assert!(result.is_some());

		let d2 = calculate_d(&[reserve_in + amount_in, reserve_out - result.unwrap() ], ann, precision);

		assert!(!d2.is_none())
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn test_out_given_in(amount_in in trade_amount(),
		reserve_in in  asset_reserve(),
		reserve_out in  asset_reserve(),
		amp in amplification(),
	) {
		let ann = amp * 4u128;

		let precision = 1u128;

		let d1 = calculate_d(&[reserve_in, reserve_out], ann, precision).unwrap();

		let result = calculate_out_given_in(reserve_in, reserve_out, amount_in, precision, amp);

		assert!(result.is_some());

		let d2 = calculate_d(&[reserve_in + amount_in, reserve_out - result.unwrap() ], ann, precision).unwrap();

		assert!(d2 >= d1);
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn test_in_given_out(amount_out in trade_amount(),
		reserve_in in  asset_reserve(),
		reserve_out in  asset_reserve(),
		amp in amplification(),
	) {
		let ann = amp*4u128;

		let precision = 1u128;

		let _d1 = calculate_d(&[reserve_in, reserve_out], ann, precision).unwrap();

		let result = calculate_in_given_out(reserve_in, reserve_out, amount_out, precision, amp);

		assert!(result.is_some());

		let _d2 = calculate_d(&[reserve_in + result.unwrap(), reserve_out - amount_out ], ann, precision).unwrap();

		assert!(_d2 >= _d1);
	}
}
