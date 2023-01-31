pub use super::mock::*;
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use hydradx_traits::AMM as AmmPool;
use orml_traits::MultiCurrency;
use pallet_asset_registry::AssetType;
use sp_std::convert::TryInto;

use primitives::asset::AssetPair;

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		let asset_a = HDX;
		let asset_b = ACA;
		assert_ok!(XYK::create_pool(
			Origin::signed(ALICE),
			asset_a,
			100_000_000_000_000,
			asset_b,
			10 * 100_000_000_000_000,
		));

		let pair_account = XYK::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});
		let share_token = XYK::share_token(pair_account);

		assert_eq!(XYK::get_pool_assets(&pair_account), Some(vec![asset_a, asset_b]));

		assert_eq!(Currency::free_balance(asset_a, &pair_account), 100000000000000);
		assert_eq!(Currency::free_balance(asset_b, &pair_account), 1000000000000000);
		assert_eq!(Currency::free_balance(asset_a, &ALICE), 900000000000000);
		assert_eq!(Currency::free_balance(asset_b, &ALICE), 0);
		assert_eq!(Currency::free_balance(share_token, &ALICE), 100000000000000);
		assert_eq!(XYK::total_liquidity(&pair_account), 100000000000000);

		let name: Vec<u8> = vec![232, 3, 0, 0, 72, 68, 84, 184, 11, 0, 0];
		let bounded_name: BoundedVec<u8, <Test as pallet_asset_registry::Config>::StringLimit> =
			name.try_into().unwrap();

		expect_events(vec![
			pallet_asset_registry::Event::Registered {
				asset_id: share_token,
				asset_name: bounded_name,
				asset_type: AssetType::PoolShare(HDX, ACA),
			}
			.into(),
			Event::PoolCreated {
				who: ALICE,
				asset_a,
				asset_b,
				initial_shares_amount: 100000000000000,
				share_token,
				pool: pair_account,
			}
			.into(),
			frame_system::Event::NewAccount { account: pair_account }.into(),
			orml_tokens::Event::Endowed {
				currency_id: asset_a,
				who: pair_account,
				amount: 100000000000000,
			}
			.into(),
			orml_tokens::Event::Endowed {
				currency_id: asset_b,
				who: pair_account,
				amount: 1000000000000000,
			}
			.into(),
			orml_tokens::Event::Endowed {
				currency_id: share_token,
				who: ALICE,
				amount: 100000000000000,
			}
			.into(),
		]);
	});
}

#[test]
fn create_same_pool_should_not_work() {
	new_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = HDX;
		let asset_b = ACA;

		assert_ok!(XYK::create_pool(Origin::signed(user), asset_b, 1000, asset_a, 2000,));
		assert_noop!(
			XYK::create_pool(Origin::signed(user), asset_b, 999, asset_a, 2 * 999),
			Error::<Test>::InsufficientLiquidity
		);
		assert_noop!(
			XYK::create_pool(Origin::signed(user), asset_b, 1000, asset_a, 0),
			Error::<Test>::InsufficientLiquidity
		);
		assert_noop!(
			XYK::create_pool(Origin::signed(user), asset_a, 1000, asset_a, 2000),
			Error::<Test>::CannotCreatePoolWithSameAssets
		);
		assert_noop!(
			XYK::create_pool(Origin::signed(user), asset_b, 1000, asset_a, 2000),
			Error::<Test>::TokenPoolAlreadyExists
		);

		let pair_account = XYK::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});
		let share_token = XYK::share_token(pair_account);

		expect_events(vec![Event::PoolCreated {
			who: ALICE,
			asset_a: asset_b,
			asset_b: asset_a,
			initial_shares_amount: 2000,
			share_token,
			pool: pair_account,
		}
		.into()]);
	});
}

#[test]
fn create_pool_with_insufficient_balance_should_not_work() {
	new_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = HDX;

		assert_noop!(
			XYK::create_pool(
				Origin::signed(user),
				4000,
				100_000_000_000_000,
				asset_a,
				10 * 100_000_000_000_000,
			),
			Error::<Test>::InsufficientAssetBalance
		);

		assert_noop!(
			XYK::create_pool(
				Origin::signed(user),
				asset_a,
				100_000_000_000_000,
				4000,
				10 * 100_000_000_000_000,
			),
			Error::<Test>::InsufficientAssetBalance
		);
	});
}

#[test]
fn create_pool_with_insufficient_liquidity_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			XYK::create_pool(Origin::signed(ALICE), ACA, 500, HDX, 5000),
			Error::<Test>::InsufficientLiquidity
		);

		assert_noop!(
			XYK::create_pool(Origin::signed(ALICE), ACA, 5000, HDX, 500),
			Error::<Test>::InsufficientLiquidity
		);
	});
}

#[test]
fn create_pool_small_fixed_point_amount_should_work() {
	new_test_ext().execute_with(|| {
		let asset_a = HDX;
		let asset_b = ACA;

		assert_ok!(XYK::create_pool(
			Origin::signed(ALICE),
			asset_a,
			100_000_000_000_000,
			asset_b,
			1_000_000_000,
		));

		let pair_account = XYK::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});
		let share_token = XYK::share_token(pair_account);

		assert_eq!(Currency::free_balance(asset_a, &pair_account), 100000000000000);
		assert_eq!(Currency::free_balance(asset_b, &pair_account), 1000000000);
		assert_eq!(Currency::free_balance(asset_a, &ALICE), 900000000000000);
		assert_eq!(Currency::free_balance(asset_b, &ALICE), 999999000000000);
		assert_eq!(Currency::free_balance(share_token, &ALICE), 100000000000000);
		assert_eq!(XYK::total_liquidity(&pair_account), 100000000000000);

		expect_events(vec![Event::PoolCreated {
			who: ALICE,
			asset_a,
			asset_b,
			initial_shares_amount: 100000000000000,
			share_token,
			pool: pair_account,
		}
		.into()]);
	});
}

#[test]
fn destroy_pool_on_remove_liquidity_and_recreate_should_work() {
	new_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = HDX;
		let asset_b = DOT;

		assert_ok!(XYK::create_pool(
			Origin::signed(user),
			asset_a,
			100_000_000,
			asset_b,
			1_000_000_000_000,
		));

		let asset_pair = AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		};

		let pair_account = XYK::get_pair_id(asset_pair);
		let share_token = XYK::share_token(pair_account);

		assert!(XYK::exists(asset_pair));

		assert_ok!(XYK::remove_liquidity(
			Origin::signed(user),
			asset_a,
			asset_b,
			100_000_000
		));

		assert_eq!(XYK::total_liquidity(&pair_account), 0);

		assert!(!XYK::exists(asset_pair));

		// It should be possible to recreate the pool again

		assert_ok!(XYK::create_pool(
			Origin::signed(user),
			asset_a,
			100_000_000,
			asset_b,
			1_000_000_000_000
		));

		expect_events(vec![
			Event::PoolCreated {
				who: user,
				asset_a,
				asset_b,
				initial_shares_amount: 100_000_000,
				share_token,
				pool: pair_account,
			}
			.into(),
			frame_system::Event::KilledAccount { account: pair_account }.into(),
			Event::LiquidityRemoved {
				who: user,
				asset_a,
				asset_b,
				shares: 100_000_000,
			}
			.into(),
			Event::PoolDestroyed {
				who: user,
				asset_a,
				asset_b,
				share_token,
				pool: pair_account,
			}
			.into(),
			frame_system::Event::NewAccount { account: pair_account }.into(),
			orml_tokens::Event::Endowed {
				currency_id: asset_a,
				who: pair_account,
				amount: 100000000,
			}
			.into(),
			orml_tokens::Event::Endowed {
				currency_id: asset_b,
				who: pair_account,
				amount: 1000000000000,
			}
			.into(),
			orml_tokens::Event::Endowed {
				currency_id: share_token,
				who: ALICE,
				amount: 100000000,
			}
			.into(),
			Event::PoolCreated {
				who: user,
				asset_a,
				asset_b,
				initial_shares_amount: 100_000_000,
				share_token,
				pool: pair_account,
			}
			.into(),
		]);
	});
}

#[test]
fn create_pool_with_same_assets_should_not_be_allowed() {
	new_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = HDX;

		assert_noop!(
			XYK::create_pool(
				Origin::signed(user),
				asset_a,
				100_000_000,
				asset_a,
				100_000_000_000_000_000_000
			),
			Error::<Test>::CannotCreatePoolWithSameAssets
		);
	})
}

#[test]
fn can_create_pool_should_work() {
	new_test_ext().execute_with(|| {
		let asset_a = 10u32;
		let asset_b = 10u32;
		assert_noop!(
			XYK::create_pool(
				Origin::signed(ALICE),
				asset_a,
				100_000_000_000_000,
				asset_b,
				1_000_000_000_000_000,
			),
			Error::<Test>::CannotCreatePool
		);
	});
}
