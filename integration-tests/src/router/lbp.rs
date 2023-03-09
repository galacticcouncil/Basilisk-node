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
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in: BSX,
			asset_out: NEW_BOOTSTRAPPED_TOKEN,
		}];

		start_lbp_campaign();

		//Act
		assert_ok!(Router::sell(
			Origin::signed(TRADER.into()),
			BSX,
			NEW_BOOTSTRAPPED_TOKEN,
			amount_to_sell,
			limit,
			trades
		));

		//Assert
		let amount_out = 5304848460209;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance!(
			BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE + amount_out,
			NEW_BOOTSTRAPPED_TOKEN
		);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: NEW_BOOTSTRAPPED_TOKEN,
			amount_in: amount_to_sell,
			amount_out,
		}
		.into()]);
	});
}

#[test]
fn sell_should_work_when_selling_distributed_asset_in_a_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in: NEW_BOOTSTRAPPED_TOKEN,
			asset_out: BSX,
		}];

		start_lbp_campaign();

		//Act
		assert_ok!(Router::sell(
			Origin::signed(TRADER.into()),
			NEW_BOOTSTRAPPED_TOKEN,
			BSX,
			amount_to_sell,
			limit,
			trades
		));

		//Assert
		let amount_out = 15853065839194;

		assert_trader_non_native_balance!(
			BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE - amount_to_sell,
			NEW_BOOTSTRAPPED_TOKEN
		);
		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE + amount_out);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: NEW_BOOTSTRAPPED_TOKEN,
			asset_out: BSX,
			amount_in: amount_to_sell,
			amount_out,
		}
		.into()]);
	});
}

#[test]
fn sell_should_work_when_route_contains_double_trades_with_selling_accumulated_assets() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);
		create_lbp_pool(NEW_BOOTSTRAPPED_TOKEN, KSM);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: NEW_BOOTSTRAPPED_TOKEN,
			},
			Trade {
				pool: PoolType::LBP,
				asset_in: NEW_BOOTSTRAPPED_TOKEN,
				asset_out: KSM,
			},
		];

		start_lbp_campaign();

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
		let amount_out = 2894653262401;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance!(BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE, NEW_BOOTSTRAPPED_TOKEN);
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
fn sell_should_work_when_route_contains_double_trades_with_selling_distributed_assets() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(NEW_BOOTSTRAPPED_TOKEN, BSX);
		create_lbp_pool(KSM, NEW_BOOTSTRAPPED_TOKEN);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: NEW_BOOTSTRAPPED_TOKEN,
			},
			Trade {
				pool: PoolType::LBP,
				asset_in: NEW_BOOTSTRAPPED_TOKEN,
				asset_out: KSM,
			},
		];

		start_lbp_campaign();

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
		let amount_out = 23648946648916;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance!(BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE, NEW_BOOTSTRAPPED_TOKEN);
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
fn lbp_direct_sell_should_yield_the_same_result_as_router_sell() {
	TestNet::reset();

	let amount_to_sell = 10 * UNITS;
	let limit = 0;
	let received_amount_out = 5304848460209;

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);

		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in: BSX,
			asset_out: NEW_BOOTSTRAPPED_TOKEN,
		}];

		start_lbp_campaign();

		//Act
		assert_ok!(Router::sell(
			Origin::signed(TRADER.into()),
			BSX,
			NEW_BOOTSTRAPPED_TOKEN,
			amount_to_sell,
			limit,
			trades
		));

		//Assert
		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance!(
			BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE + received_amount_out,
			NEW_BOOTSTRAPPED_TOKEN
		);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: NEW_BOOTSTRAPPED_TOKEN,
			amount_in: amount_to_sell,
			amount_out: received_amount_out,
		}
		.into()]);
	});

	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);

		start_lbp_campaign();

		//Act
		assert_ok!(LBP::sell(
			Origin::signed(TRADER.into()),
			BSX,
			NEW_BOOTSTRAPPED_TOKEN,
			amount_to_sell,
			limit
		));

		//Assert
		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance!(
			BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE + received_amount_out,
			NEW_BOOTSTRAPPED_TOKEN
		);
	});
}

#[test]
fn buy_should_work_when_when_buying_distributed_asset() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);

		let amount_to_buy = 10 * UNITS;
		let limit = 100 * UNITS;
		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in: BSX,
			asset_out: NEW_BOOTSTRAPPED_TOKEN,
		}];

		start_lbp_campaign();

		//Act
		assert_ok!(Router::buy(
			Origin::signed(TRADER.into()),
			BSX,
			NEW_BOOTSTRAPPED_TOKEN,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 19944392706756;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance!(
			BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE + amount_to_buy,
			NEW_BOOTSTRAPPED_TOKEN
		);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: NEW_BOOTSTRAPPED_TOKEN,
			amount_in,
			amount_out: amount_to_buy,
		}
		.into()]);
	});
}

#[test]
fn buy_should_work_when_buying_accumulated_asset_in_a_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);

		let amount_to_buy = 10 * UNITS;
		let limit = 100 * UNITS;
		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in: NEW_BOOTSTRAPPED_TOKEN,
			asset_out: BSX,
		}];

		start_lbp_campaign();

		//Act
		assert_ok!(Router::buy(
			Origin::signed(TRADER.into()),
			NEW_BOOTSTRAPPED_TOKEN,
			BSX,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 6045520606503;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE + amount_to_buy);
		assert_trader_non_native_balance!(
			BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE - amount_in,
			NEW_BOOTSTRAPPED_TOKEN
		);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: NEW_BOOTSTRAPPED_TOKEN,
			asset_out: BSX,
			amount_in,
			amount_out: amount_to_buy,
		}
		.into()]);
	});
}

#[test]
fn buy_should_work_when_having_double_trades_with_buying_distributed_asset() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);
		create_lbp_pool(NEW_BOOTSTRAPPED_TOKEN, KSM);

		let amount_to_buy = 1 * UNITS;
		let limit = 100 * UNITS;
		let trades = vec![
			Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: NEW_BOOTSTRAPPED_TOKEN,
			},
			Trade {
				pool: PoolType::LBP,
				asset_in: NEW_BOOTSTRAPPED_TOKEN,
				asset_out: KSM,
			},
		];

		start_lbp_campaign();

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
		let amount_in = 3244461635777;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance!(BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE, NEW_BOOTSTRAPPED_TOKEN);
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
fn buy_should_work_when_having_double_trades_with_buying_accumulated_asset() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(NEW_BOOTSTRAPPED_TOKEN, BSX);
		create_lbp_pool(KSM, NEW_BOOTSTRAPPED_TOKEN);

		let amount_to_buy = 1 * UNITS;
		let limit = 100 * UNITS;
		let trades = vec![
			Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: NEW_BOOTSTRAPPED_TOKEN,
			},
			Trade {
				pool: PoolType::LBP,
				asset_in: NEW_BOOTSTRAPPED_TOKEN,
				asset_out: KSM,
			},
		];

		start_lbp_campaign();

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
		let amount_in = 322733714720;

		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance!(BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE, NEW_BOOTSTRAPPED_TOKEN);
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
fn lbp_direct_buy_should_yield_the_same_result_as_router_buy() {
	TestNet::reset();

	let amount_to_buy = 10 * UNITS;
	let limit = 100 * UNITS;
	let spent_amount_in = 19944392706756;

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);

		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in: BSX,
			asset_out: NEW_BOOTSTRAPPED_TOKEN,
		}];

		start_lbp_campaign();

		//Act
		assert_ok!(Router::buy(
			Origin::signed(TRADER.into()),
			BSX,
			NEW_BOOTSTRAPPED_TOKEN,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - spent_amount_in);
		assert_trader_non_native_balance!(
			BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE + amount_to_buy,
			NEW_BOOTSTRAPPED_TOKEN
		);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: BSX,
			asset_out: NEW_BOOTSTRAPPED_TOKEN,
			amount_in: spent_amount_in,
			amount_out: amount_to_buy,
		}
		.into()]);
	});

	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);

		start_lbp_campaign();

		//Act
		assert_ok!(LBP::buy(
			Origin::signed(TRADER.into()),
			NEW_BOOTSTRAPPED_TOKEN,
			BSX,
			amount_to_buy,
			limit
		));

		//Assert
		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE - spent_amount_in);
		assert_trader_non_native_balance!(
			BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE + amount_to_buy,
			NEW_BOOTSTRAPPED_TOKEN
		);
	});
}
