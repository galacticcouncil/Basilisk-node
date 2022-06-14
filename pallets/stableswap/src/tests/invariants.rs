use crate::tests::mock::*;
use crate::traits::ShareAccountIdFor;
use crate::types::{PoolAssets, PoolId};
use frame_support::assert_ok;
use sp_runtime::{FixedU128, Permill};

use hydra_dx_math::stableswap::math::two_asset_pool_math::calculate_d;
use proptest::prelude::*;
use proptest::proptest;

pub const ONE: Balance = 1_000_000_000_000;

const RESERVE_RANGE: (Balance, Balance) = (500_000 * ONE, 100_000_000 * ONE);

fn trade_amount() -> impl Strategy<Value = Balance> {
	1000..100_000 * ONE
}

fn asset_reserve() -> impl Strategy<Value = Balance> {
	RESERVE_RANGE.0..RESERVE_RANGE.1
}

fn amplification() -> impl Strategy<Value = u16> {
	2..10000u16
}

fn trade_fee() -> impl Strategy<Value = Permill> {
	(0f64..50f64).prop_map(Permill::from_float)
}

fn percent() -> impl Strategy<Value = Permill> {
	(1u32..100u32).prop_map(Permill::from_percent)
}

#[macro_export]
macro_rules! assert_eq_approx {
	( $x:expr, $y:expr, $z:expr, $r:expr) => {{
		let diff = if $x >= $y { $x - $y } else { $y - $x };
		if diff > $z {
			panic!("\n{} not equal\n left: {:?}\nright: {:?}\n", $r, $x, $y);
		}
	}};
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn add_liquidity_price_no_changes(
		asset_a_liquidity in asset_reserve(),
		asset_b_liquidity in asset_reserve(),
		added_liquidity in asset_reserve(),
		amplification in amplification(),
		fee in trade_fee()

	) {
		ExtBuilder::default()
			.with_endowed_accounts(vec![
				(BOB, 1, added_liquidity),
				(BOB, 2, added_liquidity * 1000),
				(ALICE, 1, asset_a_liquidity),
				(ALICE, 2, asset_b_liquidity),
			])
			.with_registered_asset("one".as_bytes().to_vec(), 1)
			.with_registered_asset("two".as_bytes().to_vec(), 2)
			.build()
			.execute_with(|| {
				let asset_a: AssetId = 1;
				let asset_b: AssetId = 2;
				let initial_liquidity = (asset_a_liquidity, asset_b_liquidity);

				let pool_id = PoolId(retrieve_current_asset_id());

				assert_ok!(Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification,
					fee,
				));

				let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

				let asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
				let asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);

				assert_ok!(Stableswap::add_liquidity(
					Origin::signed(BOB),
					pool_id,
					asset_a,
					added_liquidity
				));

				let new_asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
				let new_asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);

				assert_eq_approx!(FixedU128::from((asset_a_reserve,asset_b_reserve)),
					FixedU128::from((new_asset_a_reserve,new_asset_b_reserve)),
					FixedU128::from_float(0.0000000001),
					"Price has changed after add liquidity");

			});
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn remove_liquidity_price_no_changes(
		asset_a_liquidity in asset_reserve(),
		asset_b_liquidity in asset_reserve(),
		added_liquidity in asset_reserve(),
		amplification in amplification(),
		fee in trade_fee(),
		withdraw_percentage in percent(),
	) {
		ExtBuilder::default()
			.with_endowed_accounts(vec![
				(BOB, 1, added_liquidity),
				(BOB, 2, added_liquidity * 1000),
				(ALICE, 1, asset_a_liquidity),
				(ALICE, 2, asset_b_liquidity),
			])
			.with_registered_asset("one".as_bytes().to_vec(), 1)
			.with_registered_asset("two".as_bytes().to_vec(), 2)
			.build()
			.execute_with(|| {
				let asset_a: AssetId = 1;
				let asset_b: AssetId = 2;
				let initial_liquidity = (asset_a_liquidity, asset_b_liquidity);

				let pool_id = PoolId(retrieve_current_asset_id());

				assert_ok!(Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification,
					fee,
				));

				let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

				assert_ok!(Stableswap::add_liquidity(
					Origin::signed(BOB),
					pool_id,
					asset_a,
					added_liquidity
				));

				let shares = Tokens::free_balance(pool_id.0, &BOB);

				let amount_withdrawn = withdraw_percentage.mul_floor(shares);

				let asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
				let asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);
				assert_ok!(Stableswap::remove_liquidity(Origin::signed(BOB), pool_id, amount_withdrawn));

				let new_asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
				let new_asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);

				assert_eq_approx!(FixedU128::from((asset_a_reserve,asset_b_reserve)),
					FixedU128::from((new_asset_a_reserve,new_asset_b_reserve)),
					FixedU128::from_float(0.0000000001),
					"Price has changed after remove liquidity");
			});
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn sell_invariants(
		asset_a_liquidity in asset_reserve(),
		asset_b_liquidity in asset_reserve(),
		amount in trade_amount(),
		amplification in amplification(),
	) {
		ExtBuilder::default()
			.with_endowed_accounts(vec![
				(BOB, 1, amount),
				(ALICE, 1, asset_a_liquidity),
				(ALICE, 2, asset_b_liquidity),
			])
			.with_registered_asset("one".as_bytes().to_vec(), 1)
			.with_registered_asset("two".as_bytes().to_vec(), 2)
			.build()
			.execute_with(|| {
				let asset_a: AssetId = 1;
				let asset_b: AssetId = 2;
				let initial_liquidity = (asset_a_liquidity, asset_b_liquidity);

				let pool_id = PoolId(retrieve_current_asset_id());

				assert_ok!(Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification,
					Permill::from_percent(0),
				));

				let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

				let asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
				let asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);
				let ann: Balance = amplification as u128 * 2 * 2;
				let d_prev = calculate_d::<128u8>(&[asset_a_reserve,asset_b_reserve], ann, 1u128).unwrap();

				assert_ok!(Stableswap::sell(
					Origin::signed(BOB),
					pool_id,
					asset_a,
					asset_b,
					amount,
					0u128, // not interested in this
				));

				let asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
				let asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);
				let ann: Balance = amplification as u128 * 2 * 2;
				let d = calculate_d::<128u8>(&[asset_a_reserve,asset_b_reserve], ann, 1u128).unwrap();

				assert!(d >= d_prev);
				assert!(d - d_prev <= 10u128);
			});
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn buy_invariants(
		asset_a_liquidity in asset_reserve(),
		asset_b_liquidity in asset_reserve(),
		amount in trade_amount(),
		amplification in amplification(),
	) {
		ExtBuilder::default()
			.with_endowed_accounts(vec![
				(BOB, 1, amount * 1000),
				(ALICE, 1, asset_a_liquidity),
				(ALICE, 2, asset_b_liquidity),
			])
			.with_registered_asset("one".as_bytes().to_vec(), 1)
			.with_registered_asset("two".as_bytes().to_vec(), 2)
			.build()
			.execute_with(|| {
				let asset_a: AssetId = 1;
				let asset_b: AssetId = 2;
				let initial_liquidity = (asset_a_liquidity, asset_b_liquidity);

				let pool_id = PoolId(retrieve_current_asset_id());

				assert_ok!(Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification,
					Permill::from_percent(0),
				));

				let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

				let asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
				let asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);
				let ann: Balance = amplification as u128 * 2 * 2;
				let d_prev = calculate_d::<128u8>(&[asset_a_reserve,asset_b_reserve], ann, 1u128).unwrap();

				assert_ok!(Stableswap::buy(
					Origin::signed(BOB),
					pool_id,
					asset_b,
					asset_a,
					amount,
					u128::MAX, // not interested in this
				));
				let asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
				let asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);
				let ann: Balance = amplification as u128 * 2 * 2;
				let d = calculate_d::<128u8>(&[asset_a_reserve,asset_b_reserve], ann, 1u128).unwrap();

				assert!(d >= d_prev);
				assert!(d - d_prev <= 10u128);
			});
	}
}
