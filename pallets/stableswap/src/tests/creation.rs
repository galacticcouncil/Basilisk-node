use crate::tests::mock::*;
use crate::types::PoolId;
use crate::Error;
use crate::Pools;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Permill;

#[test]
fn create_two_asset_pool_should_work_when_assets_are_registered() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;

	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1, 200 * ONE), (ALICE, 2, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), asset_a)
		.with_registered_asset("two".as_bytes().to_vec(), asset_b)
		.build()
		.execute_with(|| {
			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				vec![asset_a, asset_b],
				100u16,
				Permill::from_percent(0)
			));

			assert!(<Pools<Test>>::get(pool_id).is_some());
		});
}

#[test]
fn create_multi_asset_pool_should_work_when_assets_are_registered() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;
	let asset_c: AssetId = 3;
	let asset_d: AssetId = 4;

	ExtBuilder::default()
		.with_endowed_accounts(vec![(ALICE, 1, 200 * ONE), (ALICE, 2, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), asset_a)
		.with_registered_asset("two".as_bytes().to_vec(), asset_b)
		.with_registered_asset("three".as_bytes().to_vec(), asset_c)
		.with_registered_asset("four".as_bytes().to_vec(), asset_d)
		.build()
		.execute_with(|| {
			let pool_id = PoolId(retrieve_current_asset_id());

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				vec![asset_a, asset_b, asset_c, asset_d],
				100u16,
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
				vec![asset_b, asset_a],
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
				vec![asset_a, asset_a],
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
					vec![not_registered, registered],
					amplification,
					Permill::from_percent(0)
				),
				Error::<Test>::AssetNotRegistered
			);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					vec![registered, not_registered],
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
				vec![asset_a, asset_b],
				amplification,
				Permill::from_percent(0)
			));

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					vec![asset_a, asset_b],
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
					vec![asset_a, asset_b],
					amplification_min,
					Permill::from_percent(0)
				),
				Error::<Test>::InvalidAmplification
			);

			assert_noop!(
				Stableswap::create_pool(
					Origin::signed(ALICE),
					vec![asset_a, asset_b],
					amplification_max,
					Permill::from_percent(0)
				),
				Error::<Test>::InvalidAmplification
			);
		});
}
