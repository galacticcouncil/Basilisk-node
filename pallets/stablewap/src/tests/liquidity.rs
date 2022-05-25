use crate::assert_balance;
use crate::tests::mock::*;
use crate::traits::ShareAccountIdFor;
use crate::types::{PoolAssets, PoolId};
use frame_support::assert_ok;
use sp_runtime::Permill;

#[test]
fn create_pool_works() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1, 200 * ONE), (ALICE, 2, 200 * ONE)])
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

			let pool_account = AccountIdConstructor::from_assets(&PoolAssets(asset_a, asset_b), None);

			assert_balance!(ALICE, asset_a, 100 * ONE);
			assert_balance!(ALICE, asset_b, 150 * ONE);

			assert_balance!(ALICE, pool_id.0, 149_953_401_556_131u128);

			assert_balance!(pool_account, asset_a, 100 * ONE);
			assert_balance!(pool_account, asset_b, 50 * ONE);
		});
}

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

			assert_ok!(Stableswap::add_liquidity(Origin::signed(BOB), pool_id, amount_added));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 100 * ONE);
			assert_balance!(BOB, pool_id.0, 200028462782014u128);
			assert_balance!(pool_account, asset_a, 200 * ONE);
			assert_balance!(pool_account, asset_b, 150 * ONE);
		});
}

#[test]
fn remove_all_liquidity_works() {
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

			assert_ok!(Stableswap::add_liquidity(Origin::signed(BOB), pool_id, amount_added));

			let shares = Tokens::free_balance(pool_id.0, &BOB);

			assert_eq!(shares, 200028462782014u128);

			assert_ok!(Stableswap::remove_liquidity(Origin::signed(BOB), pool_id, shares));

			assert_balance!(BOB, asset_a, 214_307_901_731_016u128);
			assert_balance!(BOB, asset_b, 185_730_926_298_262u128);

			assert_balance!(BOB, pool_id.0, 0u128);

			assert_balance!(pool_account, asset_a, 85692098268984u128);
			assert_balance!(pool_account, asset_b, 64269073701738u128);
		});
}

// TODO: add tests for create pool:
//  - create pool with same asset ids fails
//  - create pool with same asset ids swapped fails
//  - create pool with reverted assets ( asset a id > asset b id ) should correctly transfer amounts
