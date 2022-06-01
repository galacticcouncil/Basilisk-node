use primitive_types::U256;
use primitives::Balance;
use proptest::prelude::*;
use sp_runtime::FixedU128;

pub const ONE: Balance = 1_000_000_000_000;
const TOLERANCE: Balance = 1_000;

#[macro_export]
macro_rules! assert_eq_approx {
	( $x:expr, $y:expr, $z:expr, $r:expr) => {{
		let diff = if $x >= $y { $x - $y } else { $y - $x };
		if diff > $z {
			panic!("\n{} not equal\n left: {:?}\nright: {:?}\n", $r, $x, $y);
		}
	}};
}

fn asset_reserve() -> impl Strategy<Value = Balance> {
	1000 * ONE..10_000_000 * ONE
}

fn trade_amount() -> impl Strategy<Value = Balance> {
	1u128..1000u128
}

fn assert_asset_invariant(
	old_state: (Balance, Balance),
	new_state: (Balance, Balance),
	tolerance: FixedU128,
	desc: &str,
) {
	let new_s = U256::from(new_state.0) * U256::from(new_state.1);
	let s1 = new_s.integer_sqrt();

	let old_s = U256::from(old_state.0) * U256::from(old_state.1);
	let s2 = old_s.integer_sqrt();

	assert!(new_s >= old_s, "Invariant decreased for {}", desc);

	let s1_u128 = Balance::try_from(s1).unwrap();
	let s2_u128 = Balance::try_from(s2).unwrap();

	let invariant = FixedU128::from((s1_u128, ONE)) / FixedU128::from((s2_u128, ONE));
	assert_eq_approx!(invariant, FixedU128::from(1u128), tolerance, desc);
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn sell_invariants( asset_in_reserve in asset_reserve(),
		asset_out_reserve in asset_reserve(),
		amount in  trade_amount()
	) {
		let amount_out = hydra_dx_math::xyk::calculate_out_given_in(asset_in_reserve, asset_out_reserve, amount).unwrap();

		assert_asset_invariant((asset_in_reserve, asset_out_reserve),
			(asset_in_reserve + amount, asset_out_reserve - amount_out),
			FixedU128::from((TOLERANCE,ONE)),
			"out given in"
		);
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn buy_invariants( asset_in_reserve in asset_reserve(),
		asset_out_reserve in asset_reserve(),
		amount in  trade_amount()
	) {
		let amount_in = hydra_dx_math::xyk::calculate_in_given_out(asset_out_reserve, asset_in_reserve, amount).unwrap();

		assert_asset_invariant((asset_in_reserve, asset_out_reserve),
			(asset_in_reserve + amount_in, asset_out_reserve - amount),
			FixedU128::from((TOLERANCE,ONE)),
			"in given out"
		);
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn add_liquidity_prices( asset_a_reserve in asset_reserve(),
		asset_b_reserve in asset_reserve(),
		amount in  trade_amount()
	) {
		let amount_b = hydra_dx_math::xyk::calculate_liquidity_in(asset_a_reserve, asset_b_reserve, amount).unwrap();

		let p0 = FixedU128::from((asset_a_reserve, asset_b_reserve));
		let p1 = FixedU128::from((asset_a_reserve + amount, asset_b_reserve + amount_b));

		// Price should not change
		assert_eq_approx!(p0,
			p1,
			FixedU128::from_float(0.0000000001),
			"Price has changed after add liquidity");
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn remove_liquidity_prices( asset_a_reserve in asset_reserve(),
		asset_b_reserve in asset_reserve(),
		shares in  trade_amount(),
		total_shares in asset_reserve(),
	) {
		let (amount_a, amount_b) = hydra_dx_math::xyk::calculate_liquidity_out(asset_a_reserve, asset_b_reserve, shares, total_shares).unwrap();

		let p0 = FixedU128::from((asset_a_reserve, asset_b_reserve));
		let p1 = FixedU128::from((asset_a_reserve - amount_a, asset_b_reserve - amount_b));

		// Price should not change
		assert_eq_approx!(p0,
			p1,
			FixedU128::from_float(0.0000000001),
			"Price has changed after add liquidity");
	}
}
