use super::mock::*;
use super::*;

use proptest::prelude::*;

use frame_support::assert_ok;
use primitive_types::U256;
use sp_runtime::FixedU128;

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
    ONE..100 * ONE
}


proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
	#[test]
	fn add_liquidity(initial_liquidity in asset_reserve(),
        added_liquidity in asset_reserve(),
	) {
		let asset_a = HDX;
		let asset_b = DOT;

		ExtBuilder::default()
			.with_exchange_fee((0, 0))
			.with_accounts(vec![
				(ALICE, asset_a,initial_liquidity),
				(ALICE, asset_b,initial_liquidity * 1000),
				(BOB, asset_a, added_liquidity),
				(BOB, asset_b, added_liquidity * 1_000_000),
			])
			.build()
			.execute_with(|| {
				assert_ok!(XYK::create_pool(
					Origin::signed(ALICE),
					asset_a,
					asset_b,
					initial_liquidity,
					Price::from_float(0.6544)
				));

				let pool_account = XYK::get_pair_id(AssetPair {
					asset_in: asset_a,
					asset_out: asset_b,
				});
				let share_token = XYK::share_token(pool_account);

				let pool_balance_a = Currency::free_balance(asset_a, &pool_account);
				let pool_balance_b = Currency::free_balance(asset_b, &pool_account);

				let bob_balance_a = Currency::free_balance(asset_a, &BOB);
				let bob_balance_b = Currency::free_balance(asset_b, &BOB);

				assert_ok!(XYK::add_liquidity(
					Origin::signed(BOB),
					asset_a,
					asset_b,
					added_liquidity,
					added_liquidity * 1_000_000, // do not care about the limit here
				));

				let new_pool_balance_a = Currency::free_balance(asset_a, &pool_account);
				let new_pool_balance_b = Currency::free_balance(asset_b, &pool_account);

				let new_bob_balance_a = Currency::free_balance(asset_a, &BOB);
				let new_bob_balance_b = Currency::free_balance(asset_b, &BOB);

				let bob_shares = Currency::free_balance(share_token, &BOB);

				let p0 = FixedU128::from((pool_balance_a, pool_balance_b));
				let p1 = FixedU128::from((new_pool_balance_a, new_pool_balance_b));

				// Price should not change
				assert_eq_approx!(
					p0,
					p1,
					FixedU128::from_float(0.0000000001),
					"Price has changed after add liquidity"
				);

				let issuance = Currency::total_issuance(share_token);

				let s = U256::from(issuance);
				let delta_s = U256::from(bob_shares);
				let delta_x = U256::from(bob_balance_a - new_bob_balance_a);
				let delta_y = U256::from(bob_balance_b - new_bob_balance_b);
				let x = U256::from(pool_balance_a);
				let y = U256::from(pool_balance_b);

				let l = delta_s * x;
				let r = s * delta_x;

				assert!(l <= r);

				let l = delta_s * y;
				let r = s * delta_y;

				assert!(l <= r);
			});
	}
}