use crate::assert_balance;
use crate::tests::mock::*;
use crate::traits::ShareAccountIdFor;
use crate::types::{PoolAssets, PoolId};
use frame_support::assert_ok;
use sp_runtime::Permill;

#[test]
fn add_liquidity_works() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: Balance = 100;
			let initial_liquidity = (100 * ONE, 50 * ONE);

			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				initial_liquidity,
				amplification,
				Permill::from_percent(0)
			));

			let amount_added = 100 * ONE;

			let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				asset_a,
				amount_added
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 150 * ONE);
			assert_balance!(BOB, pool_id.0, 149953401556127u128);
			assert_balance!(pool_account, asset_a, 200 * ONE);
			assert_balance!(pool_account, asset_b, 100 * ONE);
		});
}

#[test]
fn add_liquidity_other_asset_works() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: Balance = 100;
			let initial_liquidity = (100 * ONE, 50 * ONE);

			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				initial_liquidity,
				amplification,
				Permill::from_percent(0)
			));

			let amount_added = 100 * ONE;

			let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				asset_b,
				amount_added
			));

			assert_balance!(BOB, asset_a, 0u128);
			assert_balance!(BOB, asset_b, 100 * ONE);
			assert_balance!(BOB, pool_id.0, 299906803112256u128);
			assert_balance!(pool_account, asset_a, 300 * ONE);
			assert_balance!(pool_account, asset_b, 150 * ONE);
		});
}

#[test]
fn remove_all_liquidity_works() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 100 * ONE),
			(BOB, 2, 100 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: Balance = 100;
			let initial_liquidity = (100 * ONE, 50 * ONE);

			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				initial_liquidity,
				amplification,
				Permill::from_percent(0)
			));

			let amount_added = 100 * ONE;

			let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				asset_a,
				amount_added
			));

			let shares = Tokens::free_balance(pool_id.0, &BOB);

			assert_eq!(shares, 149953401556127u128);

			assert_ok!(Stableswap::remove_liquidity(Origin::signed(BOB), pool_id, shares));

			assert_balance!(BOB, asset_a, 100 * ONE - 2);
			assert_balance!(BOB, asset_b, 100 * ONE - 1);

			assert_balance!(BOB, pool_id.0, 0u128);

			assert_balance!(pool_account, asset_a, 100 * ONE + 2);
			assert_balance!(pool_account, asset_b, 50 * ONE + 1);
		});
}

#[test]
fn remove_partial_liquidity_works() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 100 * ONE),
			(BOB, 2, 100 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: Balance = 100;
			let initial_liquidity = (100 * ONE, 50 * ONE);

			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				initial_liquidity,
				amplification,
				Permill::from_percent(0)
			));

			let amount_added = 100 * ONE;

			let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				asset_a,
				amount_added
			));

			let asset_a_reserve = Tokens::free_balance(asset_a, &pool_account);
			let asset_b_reserve = Tokens::free_balance(asset_b, &pool_account);

			let shares = Tokens::free_balance(pool_id.0, &BOB);

			assert_eq!(shares, 149_953_401_556_127u128);

			let shares_withdrawn = 60_000_000_000_000u128;

			let lp_a = Tokens::free_balance(asset_a, &BOB);
			let lp_b = Tokens::free_balance(asset_b, &BOB);

			assert_ok!(Stableswap::remove_liquidity(
				Origin::signed(BOB),
				pool_id,
				shares_withdrawn
			));

			let lp_a = Tokens::free_balance(asset_a, &BOB) - lp_a;
			let lp_b = Tokens::free_balance(asset_b, &BOB) - lp_b;

			assert_balance!(BOB, asset_a, lp_a);
			assert_balance!(BOB, asset_b, 70_006_215_056_595u128);

			assert_balance!(BOB, pool_id.0, shares - shares_withdrawn);

			let a_diff = asset_a_reserve - Tokens::free_balance(asset_a, &pool_account);
			let b_diff = asset_b_reserve - Tokens::free_balance(asset_b, &pool_account);

			assert_balance!(pool_account, asset_a, 159_987_569_886_809u128);
			assert_balance!(pool_account, asset_b, 79_993_784_943_405u128);

			assert_eq!(a_diff, lp_a);
			assert_eq!(b_diff, lp_b);
		});
}
