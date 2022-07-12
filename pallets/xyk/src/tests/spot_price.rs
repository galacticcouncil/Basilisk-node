use super::mock::*;
use crate::XYKSpotPrice;
use crate::*;
use frame_support::assert_ok;
use hydradx_traits::pools::SpotPriceProvider;
use primitives::asset::AssetPair;
use primitives::Price;

#[test]
fn spot_price_provider_should_return_correct_price_when_pool_exists() {
	let asset_a = ACA;
	let asset_b = DOT;

	let initial = 99_000_000_000_000u128;

	ExtBuilder::default()
		.with_exchange_fee((0, 0))
		.with_accounts(vec![
			(ALICE, asset_a, initial),
			(ALICE, asset_b, initial),
			(BOB, asset_a, 1_000),
			(BOB, asset_b, 1100),
			(CHARLIE, asset_b, 100 * ONE),
		])
		.build()
		.execute_with(|| {
			assert_ok!(XYK::create_pool(
				Origin::signed(ALICE),
				asset_a,
				asset_b,
				initial,
				Price::from_float(0.4)
			));

			let pool_account = XYK::get_pair_id(AssetPair {
				asset_in: asset_a,
				asset_out: asset_b,
			});
			let share_token = XYK::share_token(pool_account);

			let price = XYKSpotPrice::<Test>::spot_price(asset_a, asset_b);

			assert_eq!(price, Some(Price::from_float(0.4)));
		});
}

#[test]
fn spot_price_provider_should_return_none_when_pool_does_not_exist() {
	let asset_a = ACA;
	let asset_b = DOT;

	ExtBuilder::default().build().execute_with(|| {
		let price = XYKSpotPrice::<Test>::spot_price(asset_a, asset_b);

		assert_eq!(price, None);
	});
}
