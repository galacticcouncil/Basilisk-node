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
		let share_asset = create_stableswap_pool(vec![NEW_BOOTSTRAPPED_TOKEN, KSM, BSX, DAI, MOVR], 10_000);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::Stableswap(share_asset),
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
		let amount_out = 9_999_999_999_837_u128;

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
		let ausd_bsx_id = create_stableswap_pool(vec![AUSD, BSX], 10_000);
		let ausd_movr_id = create_stableswap_pool(vec![AUSD, MOVR], 10_000);
		let movr_ksm_id = create_stableswap_pool(vec![KSM, MOVR], 10_000);

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, MOVR);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::Stableswap(ausd_bsx_id),
				asset_in: BSX,
				asset_out: AUSD,
			},
			Trade {
				pool: PoolType::Stableswap(ausd_movr_id),
				asset_in: AUSD,
				asset_out: MOVR,
			},
			Trade {
				pool: PoolType::Stableswap(movr_ksm_id),
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
		let amount_out = 9_999_999_849_997_u128;

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
fn sell_should_fail_when_there_is_no_asset_in_the_pool() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		let ksm_bsx_id = create_stableswap_pool(vec![KSM, BSX], 10_000);

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::Stableswap(ksm_bsx_id),
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
			pallet_stableswap::Error::<basilisk_runtime::Runtime>::AssetNotInPool
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
		let bsx_ausd_id = create_stableswap_pool(vec![AUSD, BSX], 10_000);

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::Stableswap(bsx_ausd_id),
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
		let bsx_ausd_id = create_stableswap_pool(vec![AUSD, BSX], 10_000);

		let amount_to_sell = 9999 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::Stableswap(bsx_ausd_id),
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
		let bsx_ausd_id = create_stableswap_pool(vec![AUSD, BSX], 10_000);

		let amount_to_sell = basilisk_runtime::StableswapMinTradingLimit::get() - 1;
		let limit = 0;

		let trades = vec![Trade {
			pool: PoolType::Stableswap(bsx_ausd_id),
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::sell(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_sell, limit, trades),
			pallet_stableswap::Error::<basilisk_runtime::Runtime>::InsufficientTradingAmount
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
		let pool_id = create_stableswap_pool(vec![BSX, KSM, DAI], 10_000);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_buy = 10 * UNITS;
		let limit = 30 * UNITS;
		let trades = vec![Trade {
			pool: PoolType::Stableswap(pool_id),
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
		let amount_in = 10000000011115;

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
		let pool_id = create_stableswap_pool(vec![BSX, KSM, DAI], 10_000);
		let pool_id_2 = create_stableswap_pool(vec![KSM, AUSD], 10_000);

		assert_trader_bsx_balance!(BOB_INITIAL_AUSD_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_buy = UNITS;
		let limit = 10 * UNITS;
		let trades = vec![
			Trade {
				pool: PoolType::Stableswap(pool_id),
				asset_in: BSX,
				asset_out: KSM,
			},
			Trade {
				pool: PoolType::Stableswap(pool_id_2),
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
		let amount_in = 1_000_000_000_618;

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
		let pool_id = create_stableswap_pool(vec![BSX, KSM, DAI], 10_000);
		let pool_id_2 = create_stableswap_pool(vec![KSM, MOVR], 10_000);
		let pool_id_3 = create_stableswap_pool(vec![MOVR, AUSD, BSX], 10_000);

		assert_trader_bsx_balance!(BOB_INITIAL_AUSD_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, MOVR);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_buy = UNITS;
		let limit = 10 * UNITS;
		let trades = vec![
			Trade {
				pool: PoolType::Stableswap(pool_id),
				asset_in: BSX,
				asset_out: KSM,
			},
			Trade {
				pool: PoolType::Stableswap(pool_id_2),
				asset_in: KSM,
				asset_out: MOVR,
			},
			Trade {
				pool: PoolType::Stableswap(pool_id_3),
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
		let amount_in = 1_000_000_000_733;

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
fn buy_should_fail_when_there_is_no_asset_in_the_pool() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		let pool_id = create_stableswap_pool(vec![BSX, KSM, DAI], 10_000);

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::Stableswap(pool_id),
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
			pallet_stableswap::Error::<basilisk_runtime::Runtime>::AssetNotInPool
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
		let pool_id = create_stableswap_pool(vec![BSX, KSM, DAI], 10_000);

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance!(0, KSM);

		let amount_to_sell = 10;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::Stableswap(pool_id),
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
		let pool_id = create_stableswap_pool(vec![AUSD, KSM, DAI], 10_000);

		assert_trader_non_native_balance!(0, KSM);
		let amount_to_buy = 100_001 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::Stableswap(pool_id),
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
			pallet_stableswap::Error::<basilisk_runtime::Runtime>::InsufficientLiquidity
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn buy_should_fail_when_trading_limit_is_below_minimum() {
	Basilisk::execute_with(|| {
		//Arrange
		let pool_id = create_stableswap_pool(vec![AUSD, BSX, DAI], 10_000);

		let amount_to_buy = basilisk_runtime::StableswapMinTradingLimit::get() - 1;
		let limit = 100 * UNITS;

		let trades = vec![Trade {
			pool: PoolType::Stableswap(pool_id),
			asset_in: BSX,
			asset_out: AUSD,
		}];

		//Act and Assert
		assert_noop!(
			Router::buy(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_buy, limit, trades),
			pallet_stableswap::Error::<basilisk_runtime::Runtime>::InsufficientTradingAmount
		);

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}
