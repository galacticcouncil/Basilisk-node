#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{Origin, Router, XYK};
use primitives::Price;
use xcm_emulator::TestExt;

use sp_arithmetic::fixed_point::FixedPointNumber;

use frame_support::{assert_noop, assert_ok};
use hydradx_traits::router::PoolType;
use orml_traits::MultiCurrency;
use pallet_route_executor::types::Trade;

const BSX: u32 = 0;
const AUSD: u32 = 1;
const MOVR: u32 = 2;
const KSM: u32 = 3;

const TRADER: [u8; 32] = BOB;
pub const BOB_INITIAL_AUSD_BALANCE: u128 = BOB_INITIAL_ASSET_1_BALANCE;

#[test]
fn execute_sell_should_work_when_route_contains_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE + amount_out, AUSD);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: AUSD,
			amount_in: amount_to_sell,
			amount_out: amount_out,
		}
		.into()]);
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

		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, MOVR);
		assert_trader_non_native_balance(0, KSM);

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
			},
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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance(amount_out, KSM);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, MOVR);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: KSM,
			amount_in: amount_to_sell,
			amount_out: amount_out,
		}
		.into()]);
	});
}

#[test]
fn execute_sell_should_fail_when_there_is_no_pool_for_specific_asset_pair() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::execute_sell(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell * UNITS,
				limit,
				trades
			),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::TokenPoolNotFound
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn execute_sell_should_fail_when_first_trade_is_successful_but_second_trade_has_no_supported_pool() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, KSM);

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
			},
		];

		//Act and Assert
		assert_noop!(
			Router::execute_sell(
				Origin::signed(TRADER.into()),
				BSX,
				KSM,
				amount_to_sell * UNITS,
				limit,
				trades
			),
			pallet_route_executor::Error::<basilisk_runtime::Runtime>::PoolNotSupported
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, KSM);
	});
}

#[test]
fn execute_sell_should_fail_when_balance_is_not_sufficient() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		let amount_to_sell = 9999 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::execute_sell(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell * UNITS,
				0,
				trades
			),
			pallet_route_executor::Error::<basilisk_runtime::Runtime>::InsufficientBalance
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn execute_sell_should_fail_when_trading_limit_is_below_minimum() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		let amount_to_sell = primitives::constants::chain::MIN_TRADING_LIMIT - 1;
		let limit = 0 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::execute_sell(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_sell, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::InsufficientTradingAmount
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn execute_sell_should_fail_when_buying_more_than_max_in_ratio_out() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		let amount_to_sell = 1000 * UNITS;
		let limit = 0 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::execute_sell(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_sell, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::MaxInRatioExceeded
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}
///to this

#[test]
fn execute_buy_should_work_when_route_contains_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);

		let amount_to_buy = 10 * UNITS;
		let limit = 30 * UNITS;
		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act
		assert_ok!(Router::execute_buy(
			Origin::signed(TRADER.into()),
			BSX,
			AUSD,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 25075000000001;

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE + amount_to_buy, AUSD);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: AUSD,
			amount_in,
			amount_out: amount_to_buy,
		}
		.into()]);
	});
}

#[test]
fn execute_buy_should_work_when_route_contains_two_trades() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, KSM);
		create_pool(KSM, AUSD);

		assert_trader_bsx_balance(BOB_INITIAL_AUSD_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, KSM);

		let amount_to_buy = 1 * UNITS;
		let limit = 10 * UNITS;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: BSX,
				asset_out: KSM,
			},
			Trade {
				pool: PoolType::XYK,
				asset_in: KSM,
				asset_out: AUSD,
			},
		];

		//Act
		assert_ok!(Router::execute_buy(
			Origin::signed(TRADER.into()),
			BSX,
			AUSD,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 428_143_592_7986;

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE + amount_to_buy, AUSD);
		assert_trader_non_native_balance(0, KSM);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: AUSD,
			amount_in,
			amount_out: amount_to_buy,
		}
		.into()]);
	});
}

#[test]
fn execute_buy_should_work_when_route_contains_multiple_trades() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, KSM);
		create_pool(KSM, MOVR);
		create_pool(MOVR, AUSD);

		assert_trader_bsx_balance(BOB_INITIAL_AUSD_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, MOVR);
		assert_trader_non_native_balance(0, KSM);

		let amount_to_buy = 1 * UNITS;
		let limit = 10 * UNITS;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: BSX,
				asset_out: KSM,
			},
			Trade {
				pool: PoolType::XYK,
				asset_in: KSM,
				asset_out: MOVR,
			},
			Trade {
				pool: PoolType::XYK,
				asset_in: MOVR,
				asset_out: AUSD,
			},
		];

		//Act
		assert_ok!(Router::execute_buy(
			Origin::signed(TRADER.into()),
			BSX,
			AUSD,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 939_285_894_6762;

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE + amount_to_buy, AUSD);
		assert_trader_non_native_balance(0, MOVR);
		assert_trader_non_native_balance(0, KSM);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: AUSD,
			amount_in,
			amount_out: amount_to_buy,
		}
		.into()]);
	});
}

#[test]
fn execute_buy_should_fail_when_there_is_no_pool_for_specific_asset_pair() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::execute_buy(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell * UNITS,
				limit,
				trades
			),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::TokenPoolNotFound
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn execute_buy_should_fail_when_first_trade_is_successful_but_second_trade_has_no_supported_pool() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, KSM);

		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, KSM);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: BSX,
				asset_out: KSM,
			},
			Trade {
				pool: PoolType::Omnipool,
				asset_in: KSM,
				asset_out: AUSD,
			},
		];

		//Act and Assert
		assert_noop!(
			Router::execute_buy(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell * UNITS,
				limit,
				trades
			),
			pallet_route_executor::Error::<basilisk_runtime::Runtime>::PoolNotSupported
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, KSM);
	});
}

#[test]
fn execute_buy_should_fail_when_balance_is_not_sufficient() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		let amount_to_buy = BOB_INITIAL_BSX_BALANCE + 1;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::execute_buy(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_buy * UNITS,
				0,
				trades
			),
			pallet_route_executor::Error::<basilisk_runtime::Runtime>::InsufficientBalance
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn execute_buy_should_fail_when_trading_limit_is_below_minimum() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		let amount_to_buy = primitives::constants::chain::MIN_TRADING_LIMIT - 1;
		let limit = 100 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::execute_buy(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_buy, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::InsufficientTradingAmount
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn execute_buy_should_fail_when_buying_more_than_max_ratio_out() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

		let amount_to_buy = 20 * UNITS;
		let limit = 100 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::execute_buy(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_buy, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::MaxOutRatioExceeded
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
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

fn assert_trader_non_native_balance(balance: u128, asset_id: u32) {
	let trader_balance = basilisk_runtime::Tokens::free_balance(asset_id, &AccountId::from(TRADER));
	assert_eq!(
		trader_balance, balance,
		"\r\nNon native asset({}) balance '{}' is not as expected '{}'",
		asset_id, trader_balance, balance
	);
}

fn assert_trader_bsx_balance(balance: u128) {
	let trader_balance = basilisk_runtime::Balances::free_balance(&AccountId::from(TRADER));
	assert_eq!(
		trader_balance, balance,
		"\r\nBSX asset balance '{}' is not as expected '{}'",
		trader_balance, balance
	);
}
