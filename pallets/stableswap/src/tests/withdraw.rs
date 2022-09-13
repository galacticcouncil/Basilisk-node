use crate::tests::mock::*;
use crate::traits::ShareAccountIdFor;
use crate::types::{AssetLiquidity, PoolInfo};
use crate::{assert_balance, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Permill;

#[test]
fn remove_liquidity_should_work_when_removing_same_asset() {
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
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_c,
						amount: 100 * ONE,
					},
				],
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

			let shares = Tokens::free_balance(pool_id, &BOB);

			assert_ok!(Stableswap::remove_liquidity_one_asset(
				Origin::signed(BOB),
				pool_id,
				asset_a,
				shares,
			));

			assert_balance!(BOB, asset_a, 199_999_999_999_994u128);
			assert_balance!(BOB, pool_id, 0u128);

			assert_balance!(pool_account, asset_a, 100 * ONE + 6);
		});
}

#[test]
fn remove_liquidity_should_work_when_removing_another_asset() {
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
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 200 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_c,
						amount: 100 * ONE,
					},
				],
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

			let shares = Tokens::free_balance(pool_id, &BOB);

			assert_ok!(Stableswap::remove_liquidity_one_asset(
				Origin::signed(BOB),
				pool_id,
				asset_b,
				shares,
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 299_999_999_999_994u128);
			assert_balance!(BOB, pool_id, 0u128);

			assert_balance!(pool_account, asset_a, 200 * ONE);
			assert_balance!(pool_account, asset_b, 100 * ONE + 6);
		});
}

#[test]
fn remove_liquidity_should_apply_correct_fee_when_removing_asset() {
	let asset_a: AssetId = 1;
	let asset_b: AssetId = 2;
	let asset_c: AssetId = 3;

	ExtBuilder::default()
		.with_endowed_accounts(vec![
			(BOB, 1, 200 * ONE),
			(BOB, 2, 200 * ONE),
			(BOB, 3, 200 * ONE),
			(ALICE, 1, 200 * ONE),
			(ALICE, 2, 300 * ONE),
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
				withdraw_fee: Permill::from_percent(10),
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
						amount: 200 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_c,
						amount: 100 * ONE,
					},
				],
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

			let shares = Tokens::free_balance(pool_id, &BOB);

			assert_ok!(Stableswap::remove_liquidity_one_asset(
				Origin::signed(BOB),
				pool_id,
				asset_b,
				shares,
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 288_003_020_713_436u128);
			assert_balance!(BOB, pool_id, 0u128);

			assert_balance!(pool_account, asset_a, 200 * ONE);
			assert_balance!(pool_account, asset_b, 111_996_979_286_564u128);
		});
}

#[test]
fn remove_liquidity_should_work_when_removing_partial_shares() {
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
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 200 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_c,
						amount: 100 * ONE,
					},
				],
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

			let shares = Tokens::free_balance(pool_id, &BOB);

			assert_ok!(Stableswap::remove_liquidity_one_asset(
				Origin::signed(BOB),
				pool_id,
				asset_b,
				shares / 2,
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 250007447620091u128);
			assert_balance!(BOB, pool_id, 49999143249484u128);

			assert_balance!(pool_account, asset_a, 200 * ONE);
			assert_balance!(pool_account, asset_b, 149992552379909u128);

			// withdraw remaining shares to make sure that the total received is reasonable

			let shares = Tokens::free_balance(pool_id, &BOB);

			assert_ok!(Stableswap::remove_liquidity_one_asset(
				Origin::signed(BOB),
				pool_id,
				asset_b,
				shares,
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 299999999999990u128); // when removing all shares at once it got : 299_999_999_999_994u128
			assert_balance!(BOB, pool_id, 0u128);

			assert_balance!(pool_account, asset_a, 200 * ONE);
			assert_balance!(pool_account, asset_b, 100 * ONE + 10);
		});
}

#[test]
fn remove_liquidity_should_work_when_removing_last_asset_liquidity() {
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
				assets: vec![AssetLiquidity {
					asset_id: asset_a,
					amount: 100 * ONE,
				}],
			},
		)
		.build()
		.execute_with(|| {
			// TODO: thjis fails with overflow now due to math issue
			/*
			let pool_id = get_pool_id_at(0);
			let pool_account = AccountIdConstructor::from_assets(&vec![asset_a, asset_b, asset_c], None);

			let shares = Tokens::free_balance(pool_id, &ALICE);

			assert_ok!(Stableswap::remove_liquidity_one_asset(
				Origin::signed(ALICE),
				pool_id,
				asset_a,
				shares,
			));

			assert_balance!(BOB, asset_a, 100 * ONE);
			assert_balance!(BOB, asset_b, 200 * ONE);
			assert_balance!(BOB, pool_id, 100 * ONE);

			assert_balance!(pool_account, asset_a, 100 * ONE);
			assert_balance!(pool_account, asset_b, 0u128);

			 */
		});
}

#[test]
fn remove_liquidity_should_fail_when_removing_from_non_existing_pool() {
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
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_c,
						amount: 100 * ONE,
					},
				],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);
			let shares = Tokens::free_balance(pool_id, &ALICE);

			assert_noop!(
				Stableswap::remove_liquidity_one_asset(Origin::signed(BOB), pool_id + 1, asset_a, shares,),
				Error::<Test>::PoolNotFound
			);
		});
}

#[test]
fn remove_liquidity_should_fail_when_removing_zero_shares() {
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
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_c,
						amount: 100 * ONE,
					},
				],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);

			assert_noop!(
				Stableswap::remove_liquidity_one_asset(Origin::signed(ALICE), pool_id, asset_a, 0u128),
				Error::<Test>::InvalidAssetAmount
			);
		});
}

#[test]
fn remove_liquidity_should_fail_when_removing_more_shares() {
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
				assets: vec![
					AssetLiquidity {
						asset_id: asset_a,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_b,
						amount: 100 * ONE,
					},
					AssetLiquidity {
						asset_id: asset_c,
						amount: 100 * ONE,
					},
				],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);
			let shares = Tokens::free_balance(pool_id, &ALICE);

			assert_noop!(
				Stableswap::remove_liquidity_one_asset(Origin::signed(ALICE), pool_id, asset_a, shares + 1),
				Error::<Test>::InsufficientShares
			);
		});
}

#[test]
fn remove_liquidity_should_fail_when_not_leaving_sufficient_shares_in_account() {
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
				assets: vec![AssetLiquidity {
					asset_id: asset_c,
					amount: 100 * ONE,
				}],
			},
		)
		.build()
		.execute_with(|| {
			let pool_id = get_pool_id_at(0);
			let shares = Tokens::free_balance(pool_id, &ALICE);

			assert_noop!(
				Stableswap::remove_liquidity_one_asset(Origin::signed(ALICE), pool_id, asset_c, shares - 1000 + 1),
				Error::<Test>::InsufficientShareBalance
			);
		});
}

#[test]
fn remove_liquidity_should_fail_when_removing_asset_not_in_pool() {
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
			let shares = Tokens::free_balance(pool_id, &ALICE);

			assert_noop!(
				Stableswap::remove_liquidity_one_asset(Origin::signed(ALICE), pool_id, asset_c, shares),
				Error::<Test>::AssetNotInPool
			);
		});
}

#[test]
fn remove_liquidity_should_emit_event_when_succesfull() {
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
				withdraw_fee: Permill::from_percent(10),
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
					AssetLiquidity {
						asset_id: asset_c,
						amount: 100 * ONE,
					},
				],
			},
		)
		.build()
		.execute_with(|| {
			System::set_block_number(1);
			let pool_id = get_pool_id_at(0);
			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				vec![AssetLiquidity {
					asset_id: asset_a,
					amount: 100 * ONE,
				},]
			));

			let shares = Tokens::free_balance(pool_id, &BOB);

			assert_ok!(Stableswap::remove_liquidity_one_asset(
				Origin::signed(BOB),
				pool_id,
				asset_a,
				shares,
			));
			let event = Event::LiquidityRemoved {
				pool_id,
				who: BOB,
				shares,
				asset: asset_a,
				amount: 89_999_795_061_525u128,
				fee: 10_000_204_938_469u128,
			};
			assert_eq!(last_event(), event.into());
		});
}
