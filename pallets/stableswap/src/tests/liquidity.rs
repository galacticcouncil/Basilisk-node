use crate::tests::mock::*;
use crate::traits::ShareAccountIdFor;
use crate::types::{AssetLiquidity, PoolInfo};
use crate::{assert_balance, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Permill;

#[test]
fn add_liquidity_should_work_when_providing_initial_liquidity_of_one_asset() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;
	let asset_c: AssetId = 3;

	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(BOB, 3, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
			(ALICE, 3, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), asset_a)
		.with_registered_asset("two".as_bytes().to_vec(), asset_b)
		.with_registered_asset("three".as_bytes().to_vec(), asset_c)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: vec![asset_a, asset_b, asset_c].try_into().unwrap(),
				amplification: 100u16,
				trade_fee: Permill::from_percent(0),
				withdraw_fee: Permill::from_percent(0),
			},
			InitialLiquidity {
				account: ALICE,
				assets: vec![],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);

			let pool_account = AccountIdConstructor::from_assets(&vec![asset_a, asset_b, asset_c], None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				vec![AssetLiquidity {
					asset_id: asset_a,
					amount: 100 * ONE,
				},]
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 200 * ONE);
			assert_balance!(BOB, pool_id, 100 * ONE);

			assert_balance!(pool_account, asset_a, 100 * ONE);
			assert_balance!(pool_account, asset_b, 0u128);
		});
}

#[test]
fn add_liquidity_should_work_when_providing_initial_liquidity_of_all_assets() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;
	let asset_c: AssetId = 3;

	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(BOB, 3, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
			(ALICE, 3, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), asset_a)
		.with_registered_asset("two".as_bytes().to_vec(), asset_b)
		.with_registered_asset("three".as_bytes().to_vec(), asset_c)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: vec![asset_a, asset_b, asset_c].try_into().unwrap(),
				amplification: 100u16,
				trade_fee: Permill::from_percent(0),
				withdraw_fee: Permill::from_percent(0),
			},
			InitialLiquidity {
				account: ALICE,
				assets: vec![],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);

			let pool_account = AccountIdConstructor::from_assets(&vec![asset_a, asset_b, asset_c], None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 10 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_c,
						amount: 5 * ONE,
					},
				]
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 190 * ONE);
			assert_balance!(BOB, asset_c, 195 * ONE);
			assert_balance!(BOB, pool_id, 114_569_737_012_942u128);

			assert_balance!(pool_account, asset_a, 100 * ONE);
			assert_balance!(pool_account, asset_b, 10 * ONE);
			assert_balance!(pool_account, asset_c, 5 * ONE);
		});
}

#[test]
fn add_liquidity_should_work_when_providing_multiple_liquidity_of_same_assets() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;
	let asset_c: AssetId = 3;

	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(BOB, 3, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
			(ALICE, 3, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), asset_a)
		.with_registered_asset("two".as_bytes().to_vec(), asset_b)
		.with_registered_asset("three".as_bytes().to_vec(), asset_c)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: vec![asset_a, asset_b, asset_c].try_into().unwrap(),
				amplification: 100u16,
				trade_fee: Permill::from_percent(0),
				withdraw_fee: Permill::from_percent(0),
			},
			InitialLiquidity {
				account: ALICE,
				assets: vec![],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);

			let pool_account = AccountIdConstructor::from_assets(&vec![asset_a, asset_b, asset_c], None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_a,
						amount: 10 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_a,
						amount: 5 * ONE,
					},
				]
			));

			assert_balance!(BOB, asset_a, 85 * ONE);
			assert_balance!(BOB, asset_b, 200 * ONE);
			assert_balance!(BOB, asset_c, 200 * ONE);
			assert_balance!(BOB, pool_id, 115 * ONE);

			assert_balance!(pool_account, asset_a, 115 * ONE);
			assert_balance!(pool_account, asset_b, 0u128);
			assert_balance!(pool_account, asset_c, 0u128);
		});
}

#[test]
fn add_initial_liquidity_with_insufficient_balance_fails() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 20 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: u16 = 100;

			let pool_id = retrieve_current_asset_id();

			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				vec![asset_a, asset_b],
				amplification,
				Permill::from_percent(0),
				Permill::from_percent(0),
			));

			let initial_liquidity_amount = 100 * ONE;

			let pool_account = AccountIdConstructor::from_assets(&vec![asset_a, asset_b], None);

			assert_noop!(
				Stableswap::add_liquidity(
					Origin::signed(BOB),
					pool_id,
					vec![
						AssetLiquidity {
							asset_id: asset_a,
							amount: initial_liquidity_amount
						},
						AssetLiquidity {
							asset_id: asset_b,
							amount: initial_liquidity_amount
						}
					]
				),
				Error::<Test>::InsufficientBalance
			);

			assert_balance!(BOB, asset_a, 200 * ONE);
			assert_balance!(BOB, asset_b, 20 * ONE);
			assert_balance!(BOB, pool_id, 0u128);
			assert_balance!(pool_account, asset_a, 0u128);
			assert_balance!(pool_account, asset_b, 0u128);
		});
}
#[test]
fn add_liquidity_works() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;

	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), asset_a)
		.with_registered_asset("two".as_bytes().to_vec(), asset_b)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: vec![asset_a, asset_b].try_into().unwrap(),
				amplification: 100u16,
				trade_fee: Permill::from_percent(0),
				withdraw_fee: Permill::from_percent(0),
			},
			InitialLiquidity {
				account: ALICE,
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 100 * ONE,
					},
				],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);

			let amount_added = 100 * ONE;

			let pool_account = AccountIdConstructor::from_assets(&vec![asset_a, asset_b], None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: amount_added
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: amount_added
					}
				]
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 100 * ONE);
			assert_balance!(BOB, pool_id, 199999999999996u128);
			assert_balance!(pool_account, asset_a, 200 * ONE);
			assert_balance!(pool_account, asset_b, 200 * ONE);
		});
}

#[test]
fn add_liquidity_other_asset_works() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;

	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: vec![asset_a, asset_b].try_into().unwrap(),
				amplification: 100u16,
				trade_fee: Permill::from_percent(0),
				withdraw_fee: Permill::from_percent(0),
			},
			InitialLiquidity {
				account: ALICE,
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 100 * ONE,
					},
				],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);

			let amount_added = 100 * ONE;

			let pool_account = AccountIdConstructor::from_assets(&vec![asset_a, asset_b], None);

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				vec![
					AssetLiquidity {
						asset_id: asset_b,
						amount: amount_added
					},
					AssetLiquidity {
						asset_id: asset_a,
						amount: amount_added
					}
				]
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 100 * ONE);
			assert_balance!(BOB, pool_id, 199999999999996u128);
			assert_balance!(pool_account, asset_a, 200 * ONE);
			assert_balance!(pool_account, asset_b, 200 * ONE);
		});
}

#[test]
fn add_insufficient_liquidity_fails() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;

	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 200 * ONE),
		])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.with_pool(
			ALICE,
			PoolInfo::<AssetId> {
				assets: vec![asset_a, asset_b].try_into().unwrap(),
				amplification: 100u16,
				trade_fee: Permill::from_percent(0),
				withdraw_fee: Permill::from_percent(0),
			},
			InitialLiquidity {
				account: ALICE,
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 100 * ONE,
					},
				],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);
			let amount_added = 100;

			assert_noop!(
				Stableswap::add_liquidity(
					Origin::signed(BOB),
					pool_id,
					vec![
						AssetLiquidity {
							asset_id: asset_a,
							amount: amount_added
						},
						AssetLiquidity {
							asset_id: asset_b,
							amount: amount_added
						}
					]
				),
				Error::<Test>::InsufficientLiquidity
			);
		});
}
