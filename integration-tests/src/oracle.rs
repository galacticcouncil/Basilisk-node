#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{EmaOracle, RuntimeOrigin, XYKOracleSourceIdentifier, XYK};
use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use hydradx_traits::{AggregatedPriceOracle, OraclePeriod::*};
use pallet_ema_oracle::OracleError;
use xcm_emulator::TestExt;

#[test]
fn xyk_trades_are_ingested_into_oracle() {
	TestNet::reset();

	let asset_a = 1;
	let asset_b = 2;

	Basilisk::execute_with(|| {
		// arrange
		basilisk_run_to_next_block();

		assert_ok!(XYK::create_pool(
			RuntimeOrigin::signed(ALICE.into()),
			asset_a,
			100 * UNITS,
			asset_b,
			200 * UNITS,
		));
		assert_ok!(XYK::sell(
			RuntimeOrigin::signed(ALICE.into()),
			asset_a,
			asset_b,
			5 * UNITS,
			UNITS,
			false,
		));

		// act
		// will store the data received in the sell as oracle values
		basilisk_run_to_next_block();

		// assert
		let expected = ((105000000000000, 190504761904760).into(), 0);
		assert_eq!(
			EmaOracle::get_price(asset_a, asset_b, LastBlock, XYKOracleSourceIdentifier::get()),
			Ok(expected)
		);
		// ten minutes oracle not configured/supported
		assert_eq!(
			EmaOracle::get_price(asset_a, asset_b, TenMinutes, XYKOracleSourceIdentifier::get()),
			Err(OracleError::NotPresent)
		);
		assert_eq!(
			EmaOracle::get_price(asset_a, asset_b, Hour, XYKOracleSourceIdentifier::get()),
			Ok(expected)
		);
		assert_eq!(
			EmaOracle::get_price(asset_a, asset_b, Day, XYKOracleSourceIdentifier::get()),
			Ok(expected)
		);
		assert_eq!(
			EmaOracle::get_price(asset_a, asset_b, Week, XYKOracleSourceIdentifier::get()),
			Ok(expected)
		);
	});
}
