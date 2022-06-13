use crate::assert_balance;
use crate::tests::mock::*;
use crate::traits::ShareAccountIdFor;
use crate::types::{PoolAssets, PoolId};
use crate::Error;
use frame_support::{assert_noop, assert_ok};
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
			let amplification: u16 = 100;

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
fn create_pool_with_asset_order_swapped_works() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1, 200 * ONE), (ALICE, 2, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: u16 = 100;

			let initial_liquidity = (50 * ONE, 100 * ONE);

			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_b, asset_a),
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
fn create_pool_with_same_assets_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_a: AssetId = 1;
		let asset_b: AssetId = 1;
		let amplification: u16 = 100;

		let initial_liquidity = (50 * ONE, 100 * ONE);

		assert_noop!(
			Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_b, asset_a),
				initial_liquidity,
				amplification,
				Permill::from_percent(0)
			),
			Error::<Test>::SameAssets
		);
	});
}

#[test]
fn create_pool_with_no_registered_assets_fails() {
	ExtBuilder::default()
		.with_registered_asset("one".as_bytes().to_vec(), 1000)
		.build()
		.execute_with(|| {
			let registered: AssetId = 1000;
			let not_registered: AssetId = 2000;
			let amplification: u16 = 100;

			let initial_liquidity = (50 * ONE, 100 * ONE);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(not_registered, registered),
					initial_liquidity,
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::AssetNotRegistered
			);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(registered, not_registered),
					initial_liquidity,
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::AssetNotRegistered
			);
		});
}

#[test]
fn create_pool_with_zero_initial_liquiduity_fails() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1000, 200 * ONE), (ALICE, 2000, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), 1000)
		.with_registered_asset("two".as_bytes().to_vec(), 2000)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1000;
			let asset_b: AssetId = 2000;
			let amplification: u16 = 100;

			let initial_liquidity = (0u128, 100 * ONE);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_b, asset_a),
					initial_liquidity,
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::InvalidInitialLiquidity
			);

			let initial_liquidity = (100 * ONE, 0u128);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_b, asset_a),
					initial_liquidity,
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::InvalidInitialLiquidity
			);
		});
}

#[test]
fn create_existing_pool_fails() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1, 200 * ONE), (ALICE, 2, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: u16 = 100;

			let initial_liquidity = (100 * ONE, 50 * ONE);

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				initial_liquidity,
				amplification,
				Permill::from_percent(0)
			));

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::PoolExists
			);
		});
}

#[test]
fn create_pool_with_insufficient_amount_fails() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1, 200 * ONE), (ALICE, 2, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: u16 = 100;

			let initial_liquidity = (1000 * ONE, 1000 * ONE);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::BalanceTooLow
			);
			let initial_liquidity = (100 * ONE, 1000 * ONE);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::BalanceTooLow
			);
		});
}

#[test]
fn create_pool_with_insufficient_liquidity_fails() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1000, 200 * ONE), (ALICE, 2000, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), 1000)
		.with_registered_asset("two".as_bytes().to_vec(), 2000)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1000;
			let asset_b: AssetId = 2000;
			let amplification: u16 = 100;

			let initial_liquidity = (100, 100);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::InsufficientLiquidity
			);
		});
}

#[test]
fn create_pool_with_invalid_amp_fails() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1000, 200 * ONE), (ALICE, 2000, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), 1000)
		.with_registered_asset("two".as_bytes().to_vec(), 2000)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1000;
			let asset_b: AssetId = 2000;
			let amplification_min: u16 = 1;
			let amplification_max: u16 = 10_001;

			let initial_liquidity = (100, 100);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification_min,
					Permill::from_percent(0)
				),
				Error::<Test>::InvalidAmplification
			);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					initial_liquidity,
					amplification_max,
					Permill::from_percent(0)
				),
				Error::<Test>::InvalidAmplification
			);
		});
}
