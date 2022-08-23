#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{Origin, Router, XYK};
use primitives::{Price};
use xcm_emulator::TestExt;

use sp_arithmetic::fixed_point::FixedPointNumber;

use frame_support::{assert_noop, assert_ok};
use hydradx_traits::router::PoolType;
use orml_traits::MultiCurrency;
use pallet_router::types::Trade;

const BSX: u32 = 1;
const AUSD: u32 = 2;
const MOVR: u32 = 3;
const KSM: u32 = 4;

const TRADER: [u8; 32] = BOB;

#[test]
fn execute_sell_should_work_when_route_contains_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		assert_trader_balance(0, AUSD);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: BSX,
				asset_out: AUSD,
			}
		];

		//Act
		assert_ok!(Router::execute_sell(
			Origin::signed(TRADER.into()),
			BSX,
			AUSD,
			amount_to_sell,
			limit,
			trades
		));

		//Assert
		let amount_out = 453_181_818_1819u128;

		assert_trader_balance(BOB_INITIAL_BSX_BALANCE - amount_to_sell , BSX);
		assert_trader_balance(amount_out, AUSD);

		expect_basilisk_events(vec![pallet_router::Event::TradeIsExecuted {
			asset_in: BSX,
			asset_out: AUSD,
			amount_in: amount_to_sell,
			amount_out,
		}.into()]);
	});
}

#[test]
fn execute_sell_should_work_when_route_contains_multiple_trades() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);
		create_pool(AUSD, MOVR);
		create_pool(MOVR, KSM);

		assert_trader_balance(0, AUSD);
		assert_trader_balance(0, MOVR);
		assert_trader_balance(0, KSM);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: BSX,
				asset_out: AUSD,
			},
			Trade {
				pool: PoolType::XYK,
				asset_in: AUSD,
				asset_out: MOVR,
			},
			Trade {
				pool: PoolType::XYK,
				asset_in: MOVR,
				asset_out: KSM,
			}
		];

		//Act
		assert_ok!(Router::execute_sell(
			Origin::signed(TRADER.into()),
			BSX,
			KSM,
			amount_to_sell,
			limit,
			trades
		));

		//Assert
		let amount_out = 105_455_305_9484u128;

		assert_trader_balance(BOB_INITIAL_BSX_BALANCE - amount_to_sell , BSX);
		assert_trader_balance(amount_out, KSM);
		assert_trader_balance(0, AUSD);
		assert_trader_balance(0, MOVR);

		expect_basilisk_events(vec![pallet_router::Event::TradeIsExecuted {
			asset_in: BSX,
			asset_out: KSM,
			amount_in: amount_to_sell,
			amount_out,
		}.into()]);
	});
}

#[test]
fn execute_sell_should_fail_when_there_is_no_pool_for_specific_asset_pair() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		assert_trader_balance(0, AUSD);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: BSX,
				asset_out: AUSD,
			}
		];

		//Act and Assert
		assert_noop!(Router::execute_sell(
			Origin::signed(TRADER.into()),
			BSX,
			AUSD,
			amount_to_sell * UNITS,
			limit,
			trades
		),
		pallet_router::Error::<basilisk_runtime::Runtime>::PriceCalculationFailed);

		assert_trader_balance(BOB_INITIAL_BSX_BALANCE, BSX);
		assert_trader_balance(0, AUSD);

	});
}

#[test]
fn execute_sell_should_fail_when_firs_trade_is_successful_but_second_trade_has_no_supported_pool() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		assert_trader_balance(0, AUSD);
		assert_trader_balance(0, KSM);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: BSX,
				asset_out: AUSD,
			},
			Trade {
				pool: PoolType::Omnipool,
				asset_in: AUSD,
				asset_out: KSM,
			}
		];

		//Act and Assert
		assert_noop!(Router::execute_sell(
			Origin::signed(TRADER.into()),
			BSX,
			KSM,
			amount_to_sell * UNITS,
			limit,
			trades
		),
		pallet_router::Error::<basilisk_runtime::Runtime>::PoolNotSupported);

		assert_trader_balance(BOB_INITIAL_BSX_BALANCE, BSX);
		assert_trader_balance(0, AUSD);
		assert_trader_balance(0, KSM);
	});
}

fn create_pool(asset_a: u32, asset_b: u32) {
	assert_ok!(XYK::create_pool(
		Origin::signed(ALICE.into()),
		asset_a,
		asset_b,
		100 * UNITS,
		Price::checked_from_rational(1, 2).unwrap()
	));
}

fn assert_trader_balance(balance: u128, asset_id: u32, ) {
	let bob_balance = basilisk_runtime::Tokens::free_balance(asset_id, &AccountId::from(TRADER));
	assert_eq!(bob_balance, balance);
}

