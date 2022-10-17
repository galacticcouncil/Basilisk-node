use std::ops::Mul;
use super::mock::*;
use crate::*;

use proptest::prelude::*;

use crate::tests::mock::*;
use frame_support::assert_ok;
use hydra_dx_math::types::FixedBalance;
use primitive_types::U256;
use primitives::Price;
use sp_runtime::FixedU128;
const TOLERANCE: Balance = 1_000;
use rug::{Assign, Integer, Float, Rational};
use rug::ops::*;

#[macro_export]
macro_rules! assert_eq_approx {
	( $x:expr, $y:expr, $z:expr, $r:expr) => {{
		let diff = if $x >= $y { $x - $y } else { $y - $x };
		if diff > $z {
			println!("diff {:?}", diff);
			println!("tolerance {:?}", $z);
			panic!("\n{} not equal\n left: {:?}\nright: {:?}\n", $r, $x, $y);
		} else{
			println!("diff {:?}", diff);
			println!("tolerance {:?}", $z);
		}
	}};
}

fn asset_reserve() -> impl Strategy<Value = Balance> {
	1000 * ONE..10_000_000 * ONE
}

fn trade_amount() -> impl Strategy<Value = Balance> {
	ONE..100 * ONE
}

fn price() -> impl Strategy<Value = f64> {
	0.1f64..2f64
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

fn convert_to_fixed(value: Balance) -> FixedBalance {
	//TODO: convert to fixed has rounding error, ask Martin what he thinks
	if value == Balance::from(1u32) {
		return FixedBalance::from_num(1);
	}

	// Unwrap is safer here
	let f = value.checked_div(ONE).unwrap();
	let r = value - (f.checked_mul(ONE).unwrap());
	FixedBalance::from_num(f) + (FixedBalance::from_num(r) / ONE)
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1))]
	#[test]
	fn sell_invariant(
		initial_liquidity in asset_reserve(),
		added_liquidity in asset_reserve(),
		amount in trade_amount(),
		price in price(),
	) {
		let asset_a = HDX;
		let asset_b = DOT;

		ExtBuilder::default()
			//.with_exchange_fee((0, 0))
			.with_accounts(vec![
				(ALICE, asset_a,initial_liquidity + added_liquidity),
				(ALICE, asset_b,initial_liquidity * 1000 + added_liquidity * 1_000_000),
				(CHARLIE, asset_a, amount),
			])
			.build()
			.execute_with(|| {
				assert_ok!(LBPPallet::create_pool(
					Origin::root(),
					ALICE,
					asset_a,
					1_000_000_000,
					asset_b,
					2_000_000_000,
					20_000_000,
					80_000_000,
					WeightCurveType::Linear,
					DEFAULT_FEE,
					CHARLIE,
					0
				));

				let pool_account = LBPPallet::get_pair_id(AssetPair {
					asset_in: asset_a,
					asset_out: asset_b,
				});

				assert_ok!(LBPPallet::update_pool_data(
					Origin::signed(ALICE),
					pool_account,
					None,
					SALE_START,
					SALE_END,
					None,
					None,
					None,
					None,
					None,
				));

				assert_ok!(LBPPallet::add_liquidity(
					Origin::signed(ALICE),
					(asset_a, added_liquidity),
					(asset_b, added_liquidity)
				));
				let pool_balance_a = Currency::free_balance(asset_a, &pool_account);
				let pool_balance_b = Currency::free_balance(asset_b, &pool_account);

				//start sale
				set_block_number::<Test>(11);

				assert_ok!(LBPPallet::sell(
						Origin::signed(CHARLIE),
						asset_a,
						asset_b,
						amount,
						0u128, // limit not interesting here,
				));

				let new_pool_balance_a = Currency::free_balance(asset_a, &pool_account);
				let new_pool_balance_b = Currency::free_balance(asset_b, &pool_account);
			    dbg!(new_pool_balance_a.clone());
				println!("BEFORE ALL");

				let pool_data = LBPPallet::pool_data(pool_account).unwrap();
				let (weight_a_pre,weight_b_pre) = LBPPallet::calculate_weights(&pool_data, 11).unwrap();

				let max_weight = MAX_WEIGHT;

				let precision = 150;
			    let weight_a = Float::with_val(precision,Rational::from((weight_a_pre,max_weight.clone())));
			    let weight_b = Float::with_val(precision,Rational::from((weight_b_pre,max_weight)));

				let new_pool_balance_a = Float::with_val(precision,new_pool_balance_a);
				let new_pool_balance_b = Float::with_val(precision,new_pool_balance_b);
			    let new_weighted_reserve_for_asset_a = new_pool_balance_a.clone().pow(weight_a.clone());
				let new_weighted_reserve_for_asset_b = new_pool_balance_a.clone().pow(weight_b.clone());

				let old_pool_balance_a = Float::with_val(precision,pool_balance_a);
				let old_pool_balance_b = Float::with_val(precision,pool_balance_b);
				let old_weighted_reserve_for_asset_a  = old_pool_balance_a.clone().pow(weight_a.clone());
				let old_weighted_reserve_for_asset_b  = old_pool_balance_b.clone().pow(weight_b.clone());

				let s1_u128 = new_weighted_reserve_for_asset_a.clone().mul(new_weighted_reserve_for_asset_b.clone());
				let s2_u128 = old_weighted_reserve_for_asset_a.clone().mul(old_weighted_reserve_for_asset_b.clone());
				println!("RUG CRATE");
				dbg!(weight_a.clone());
				dbg!(weight_b.clone());
				dbg!(new_pool_balance_a.clone());
				dbg!(new_pool_balance_b.clone());
				dbg!(new_weighted_reserve_for_asset_a.clone());
				dbg!(new_weighted_reserve_for_asset_b.clone());
				dbg!(s1_u128.clone());
				dbg!(s2_u128.clone());

			    if true {
					println!("NORMAL CRATE");

					let new_pool_balance_a = Currency::free_balance(asset_a, &pool_account);
					let new_pool_balance_b = Currency::free_balance(asset_b, &pool_account);

					let pool_data = LBPPallet::pool_data(pool_account).unwrap();
					let (weight_a_pre,wight_b_pre) = LBPPallet::calculate_weights(&pool_data, 11).unwrap();

					let max_weight = convert_to_fixed(MAX_WEIGHT.into());
					let weight_a = convert_to_fixed(weight_a_pre.into()).checked_div(max_weight).unwrap();
					let weight_b = convert_to_fixed(weight_b_pre.into()).checked_div(max_weight).unwrap();

									println!("before converted fixed {:?}", new_pool_balance_a);

					let new_pool_balance_a = convert_to_fixed(new_pool_balance_a);
									println!("after converted fixed {:?}", new_pool_balance_a);

					let new_pool_balance_b = convert_to_fixed(new_pool_balance_b);
					let new_weighted_reserve_for_asset_a: FixedBalance  = hydra_dx_math::transcendental::pow(new_pool_balance_a, weight_a).unwrap();
					let new_weighted_reserve_for_asset_b: FixedBalance  = hydra_dx_math::transcendental::pow(new_pool_balance_b, weight_b).unwrap();

					let old_pool_balance_a = convert_to_fixed(pool_balance_a);
					let old_pool_balance_b = convert_to_fixed(pool_balance_b);
					let old_weighted_reserve_for_asset_a : FixedBalance  = hydra_dx_math::transcendental::pow(old_pool_balance_a, weight_a).unwrap();
					let old_weighted_reserve_for_asset_b : FixedBalance  = hydra_dx_math::transcendental::pow(old_pool_balance_b, weight_b).unwrap();

					let s1_u128 = new_weighted_reserve_for_asset_a * new_weighted_reserve_for_asset_b;
					let s2_u128 = old_weighted_reserve_for_asset_a * old_weighted_reserve_for_asset_b;
					dbg!(weight_a.clone());
					dbg!(weight_b.clone());
					dbg!(new_pool_balance_a.clone());
					dbg!(new_pool_balance_b.clone());
					dbg!(new_weighted_reserve_for_asset_a.clone());
					dbg!(new_weighted_reserve_for_asset_b.clone());
					dbg!(s1_u128.clone());
					dbg!(s2_u128.clone());
				}


				//give rug crate a try


				let tolerance = Float::with_val(150,0.1);
				assert_eq_approx!(s1_u128.clone(), s2_u128.clone(), tolerance, "The invariant does not hold");
			assert_eq!(3,4);
			});
	}
}

/*
proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn buy_invariant(initial_liquidity in asset_reserve(),
		added_liquidity in asset_reserve(),
		amount in trade_amount(),
		price in price(),
	) {
		let asset_a = ACA;
		let asset_b = DOT;

		ExtBuilder::default()
			.with_exchange_fee((0, 0))
			.with_accounts(vec![
				(ALICE, asset_a,initial_liquidity * 1000),
				(ALICE, HDX,initial_liquidity),
				(ALICE, asset_b,initial_liquidity * 1000),
				(BOB, asset_a, added_liquidity),
				(BOB, asset_b, added_liquidity * 1_000_000),
				(CHARLIE, asset_a, amount * 1_000),
				(CHARLIE, HDX, amount * 1_000),
			])
			.build()
			.execute_with(|| {
				assert_ok!(XYK::create_pool(
					Origin::signed(ALICE),
					asset_a,
					asset_b,
					initial_liquidity,
					Price::from_float(price)
				));

				let pool_account = XYK::get_pair_id(AssetPair {
					asset_in: asset_a,
					asset_out: asset_b,
				});

				assert_ok!(XYK::add_liquidity(
					Origin::signed(BOB),
					asset_a,
					asset_b,
					added_liquidity,
					added_liquidity * 1_000_000, // do not care about the limit here
				));
				let pool_balance_a = Currency::free_balance(asset_a, &pool_account);
				let pool_balance_b = Currency::free_balance(asset_b, &pool_account);

				assert_ok!(XYK::buy(
						Origin::signed(CHARLIE),
						asset_b,
						asset_a,
						amount,
						u128::MAX, // limit not interesting here,
						false,
				));

				let new_pool_balance_a = Currency::free_balance(asset_a, &pool_account);
				let new_pool_balance_b = Currency::free_balance(asset_b, &pool_account);

				 assert_asset_invariant((pool_balance_a, pool_balance_b),
					(new_pool_balance_a, new_pool_balance_b),
					FixedU128::from((TOLERANCE,ONE)),
					"buy"
				);

			});
	}
}*/
