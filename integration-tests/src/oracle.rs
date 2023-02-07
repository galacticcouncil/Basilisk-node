#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{EmaOracle, Origin, XYK};
use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use hydradx_traits::{AggregatedPriceOracle, OraclePeriod::*};
use pallet_ema_oracle::OracleError;
use polkadot_primitives::v2::BlockNumber;
use xcm_emulator::TestExt;

pub fn basilisk_run_to_block(to: BlockNumber) {
	while basilisk_runtime::System::block_number() < to {
		let b = basilisk_runtime::System::block_number();

		basilisk_runtime::System::on_finalize(b);
		basilisk_runtime::EmaOracle::on_finalize(b);

		basilisk_runtime::System::on_initialize(b + 1);
		basilisk_runtime::EmaOracle::on_initialize(b + 1);

		basilisk_runtime::System::set_block_number(b + 1);
	}
}

use pallet_xyk::SOURCE;

#[test]
fn xyk_trades_are_ingested_into_oracle() {
	TestNet::reset();

	let asset_a = 1;
	let asset_b = 2;

	Basilisk::execute_with(|| {
		// arrange
		basilisk_run_to_block(2);

		assert_ok!(XYK::create_pool(
			Origin::signed(ALICE.into()),
			asset_a,
			100 * UNITS,
			asset_b,
			200 * UNITS,
		));
		assert_ok!(XYK::sell(
			Origin::signed(ALICE.into()),
			asset_a,
			asset_b,
			5 * UNITS,
			UNITS,
			false,
		));

		// act
		// will store the data received in the sell as oracle values
		basilisk_run_to_block(3);

		// assert
		let expected = ((5000000000000, 9495238095240).into(), 0);
		assert_eq!(EmaOracle::get_price(asset_a, asset_b, LastBlock, SOURCE), Ok(expected));
		assert_eq!(EmaOracle::get_price(asset_a, asset_b, TenMinutes, SOURCE), Ok(expected));
		// hourly oracle not configured/supported
		assert_eq!(
			EmaOracle::get_price(asset_a, asset_b, Hour, SOURCE),
			Err(OracleError::NotPresent)
		);
		assert_eq!(EmaOracle::get_price(asset_a, asset_b, Day, SOURCE), Ok(expected));
		assert_eq!(EmaOracle::get_price(asset_a, asset_b, Week, SOURCE), Ok(expected));
	});
}
