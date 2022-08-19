#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{Origin, Router, XYK};
use primitives::Price;
use xcm_emulator::TestExt;

use sp_arithmetic::fixed_point::FixedPointNumber;

use frame_support::assert_ok;
use hydradx_traits::router::PoolType;
use orml_traits::MultiCurrency;
use pallet_router::types::Trade;

#[test]
fn router() {
	TestNet::reset();

	let asset_a = 1;
	let asset_b = 2;
	let asset_c = 3;

	Basilisk::execute_with(|| {
		//arrange
		assert_ok!(XYK::create_pool(
			Origin::signed(ALICE.into()),
			asset_a,
			asset_b,
			100 * UNITS,
			Price::checked_from_rational(1, 2).unwrap()
		));
		assert_ok!(XYK::create_pool(
			Origin::signed(ALICE.into()),
			asset_b,
			asset_c,
			100 * UNITS,
			Price::checked_from_rational(1, 3).unwrap()
		));
		assert_ok!(Router::execute_sell(
			Origin::signed(BOB.into()),
			asset_a,
			asset_c,
			10 * UNITS,
			0u128,
			vec![
				Trade {
					pool: PoolType::XYK,
					asset_in: asset_a,
					asset_out: asset_b,
				},
				Trade {
					pool: PoolType::XYK,
					asset_in: asset_b,
					asset_out: asset_c,
				}
			]
		));

		let bob_balance = basilisk_runtime::Tokens::free_balance(3, &AccountId::from(BOB));
		assert_eq!(bob_balance, 144_078_068_1540u128);
	});
}
