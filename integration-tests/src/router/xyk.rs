#![cfg(test)]
#![allow(clippy::identity_op)]
use crate::assert_trader_bsx_balance;
use crate::assert_trader_non_native_balance;

use super::*;

#[test]
fn sell_should_work_when_route_contains_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, KSM);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: KSM,
		}];

		//Act
		assert_ok!(Router::sell(
			Origin::signed(TRADER.into()),
			BSX,
			KSM,
			amount_to_sell,
			limit,
			trades
		));

		//Assert
		let amount_out = 4_531_818_181_819_u128;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance!(amount_out, KSM);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: KSM,
			amount_in: amount_to_sell,
			amount_out,
		}
		.into()]);
	});
}

#[test]
fn sell_should_work_when_route_contains_multiple_trades() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, AUSD);
		create_xyk_pool(AUSD, MOVR);
		create_xyk_pool(MOVR, KSM);

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, MOVR);
		assert_trader_non_native_balance!(0, KSM);

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
		assert_ok!(Router::sell(
			Origin::signed(TRADER.into()),
			BSX,
			KSM,
			amount_to_sell,
			limit,
			trades
		));

		//Assert
		let amount_out = 1_054_553_059_484_u128;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance!(amount_out, KSM);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, MOVR);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: KSM,
			amount_in: amount_to_sell,
			amount_out,
		}
		.into()]);
	});
}

#[test]
fn sell_should_fail_when_there_is_no_pool_for_specific_asset_pair() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::sell(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell * UNITS,
				limit,
				trades
			),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::TokenPoolNotFound
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn sell_should_fail_when_first_trade_is_successful_but_second_trade_has_no_supported_pool() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, AUSD);

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, KSM);

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
			Router::sell(
				Origin::signed(TRADER.into()),
				BSX,
				KSM,
				amount_to_sell * UNITS,
				limit,
				trades
			),
			pallet_route_executor::Error::<basilisk_runtime::Runtime>::PoolNotSupported
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, KSM);
	});
}

#[test]
fn sell_should_fail_when_balance_is_not_sufficient() {
	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, AUSD);

		let amount_to_sell = 9999 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::sell(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell * UNITS,
				0,
				trades
			),
			pallet_route_executor::Error::<basilisk_runtime::Runtime>::InsufficientBalance
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn sell_should_fail_when_trading_limit_is_below_minimum() {
	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, AUSD);

		let amount_to_sell = primitives::constants::chain::MIN_TRADING_LIMIT - 1;
		let limit = 0;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::sell(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_sell, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::InsufficientTradingAmount
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn sell_should_fail_when_buying_more_than_max_in_ratio_out() {
	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, AUSD);

		let amount_to_sell = 1000 * UNITS;
		let limit = 0;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::sell(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_sell, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::MaxInRatioExceeded
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn buy_should_work_when_route_contains_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, KSM);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_buy = 10 * UNITS;
		let limit = 30 * UNITS;
		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: KSM,
		}];

		//Act
		assert_ok!(Router::buy(
			Origin::signed(TRADER.into()),
			BSX,
			KSM,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 25075000000001;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance!(amount_to_buy, KSM);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: KSM,
			amount_in,
			amount_out: amount_to_buy,
		}
		.into()]);
	});
}

#[test]
fn buy_should_work_when_route_contains_two_trades() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, KSM);
		create_xyk_pool(KSM, AUSD);

		assert_trader_bsx_balance!(BOB_INITIAL_AUSD_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_buy = UNITS;
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
		assert_ok!(Router::buy(
			Origin::signed(TRADER.into()),
			BSX,
			AUSD,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 4_281_435_927_986;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE + amount_to_buy, AUSD);
		assert_trader_non_native_balance!(0, KSM);

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
fn buy_should_work_when_route_contains_multiple_trades() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, KSM);
		create_xyk_pool(KSM, MOVR);
		create_xyk_pool(MOVR, AUSD);

		assert_trader_bsx_balance!(BOB_INITIAL_AUSD_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, MOVR);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_buy = UNITS;
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
		assert_ok!(Router::buy(
			Origin::signed(TRADER.into()),
			BSX,
			AUSD,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 9_392_858_946_762;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE + amount_to_buy, AUSD);
		assert_trader_non_native_balance!(0, MOVR);
		assert_trader_non_native_balance!(0, KSM);

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
fn buy_should_fail_when_there_is_no_pool_for_specific_asset_pair() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::buy(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell * UNITS,
				limit,
				trades
			),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::TokenPoolNotFound
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn buy_should_fail_when_first_trade_is_successful_but_second_trade_has_no_supported_pool() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, KSM);

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, KSM);

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
			Router::buy(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell * UNITS,
				limit,
				trades
			),
			pallet_route_executor::Error::<basilisk_runtime::Runtime>::PoolNotSupported
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, KSM);
	});
}

#[test]
fn buy_should_fail_when_balance_is_not_sufficient() {
	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(KSM, AUSD);

		assert_trader_non_native_balance!(0, KSM);
		let amount_to_buy = 10 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: KSM,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::buy(
				Origin::signed(TRADER.into()),
				KSM,
				AUSD,
				amount_to_buy,
				150 * UNITS,
				trades
			),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::InsufficientAssetBalance
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn buy_should_fail_when_trading_limit_is_below_minimum() {
	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, AUSD);

		let amount_to_buy = primitives::constants::chain::MIN_TRADING_LIMIT - 1;
		let limit = 100 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::buy(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_buy, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::InsufficientTradingAmount
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn buy_should_fail_when_buying_more_than_max_ratio_out() {
	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(BSX, AUSD);

		let amount_to_buy = 20 * UNITS;
		let limit = 100 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::buy(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_buy, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::MaxOutRatioExceeded
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}
