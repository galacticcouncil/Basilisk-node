use crate::tests::mock::*;
use crate::types::PoolId;
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

			let initial_liquiduity = (100 * ONE, 100 * ONE);
			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				initial_liquiduity,
				amplification,
				Permill::from_percent(0)
			));
		});
}

#[test]
fn add_liquidity_works() {
	ExtBuilder::default()
		.with_endowed_accounts(vec![(BOB, 1, 200 * ONE), (ALICE, 1, 200 * ONE), (ALICE, 2, 200 * ONE)])
		.with_registered_asset("one".as_bytes().to_vec(), 1)
		.with_registered_asset("two".as_bytes().to_vec(), 2)
		.build()
		.execute_with(|| {
			let asset_a: AssetId = 1;
			let asset_b: AssetId = 2;
			let amplification: Balance = 100;
			let initial_liquiduity = (100 * ONE, 100 * ONE);
			assert_ok!(Stableswap::create_pool(
				Origin::signed(ALICE),
				(asset_a, asset_b),
				initial_liquiduity,
				amplification,
				Permill::from_percent(0)
			));

			let pool_id = PoolId(3);

			let amount_added = 100 * ONE;

			assert_ok!(Stableswap::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				asset_a,
				amount_added,
			));
		});
}
