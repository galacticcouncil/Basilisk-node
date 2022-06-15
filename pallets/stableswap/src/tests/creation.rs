use crate::tests::mock::*;
use crate::types::PoolId;
use crate::Error;
use crate::Pools;
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

			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				amplification,
				Permill::from_percent(0)
			));

			assert!(<Pools<Test>>::get(pool_id).is_some());
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

			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_b, asset_a),
				amplification,
				Permill::from_percent(0)
			));

			assert!(<Pools<Test>>::get(pool_id).is_some());
		});
}

#[test]
fn create_pool_with_same_assets_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_a: AssetId = 1;
		let amplification: u16 = 100;

		assert_noop!(
			Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_a),
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

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(not_registered, registered),
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::AssetNotRegistered
			);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(registered, not_registered),
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::AssetNotRegistered
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

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				amplification,
				Permill::from_percent(0)
			));

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::PoolExists
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

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					amplification_min,
					Permill::from_percent(0)
				),
				Error::<Test>::InvalidAmplification
			);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					(asset_a, asset_b),
					amplification_max,
					Permill::from_percent(0)
				),
				Error::<Test>::InvalidAmplification
			);
		});
}
