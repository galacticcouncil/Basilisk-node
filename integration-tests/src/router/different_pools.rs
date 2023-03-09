#![cfg(test)]
#![allow(clippy::identity_op)]
use crate::assert_trader_bsx_balance;
use crate::assert_trader_non_native_balance;

use super::*;

#[test]
fn sell_should_work_when_route_contains_trades_with_different_pools() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(AUSD, BSX);
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);
		create_xyk_pool(MOVR, KSM);
		let share_asset = create_stableswap_pool(vec![NEW_BOOTSTRAPPED_TOKEN, KSM, DOT, DAI, MOVR], 10_000);

		let amount_to_sell = 10 * UNITS;
		let limit = 0;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: AUSD,
				asset_out: BSX,
			},
			Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: NEW_BOOTSTRAPPED_TOKEN,
			},
			Trade {
				pool: PoolType::Stableswap(share_asset),
				asset_in: NEW_BOOTSTRAPPED_TOKEN,
				asset_out: MOVR,
			},
			Trade {
				pool: PoolType::XYK,
				asset_in: MOVR,
				asset_out: KSM,
			},
		];

		start_lbp_campaign();

		//Act
		assert_ok!(Router::sell(
			Origin::signed(TRADER.into()),
			AUSD,
			KSM,
			amount_to_sell,
			limit,
			trades
		));

		//Assert
		let amount_out = 1208552472388;

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE - amount_to_sell, AUSD);
		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE, NEW_BOOTSTRAPPED_TOKEN);
		assert_trader_non_native_balance!(amount_out, KSM);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: AUSD,
			asset_out: KSM,
			amount_in: amount_to_sell,
			amount_out,
		}
		.into()]);
	});
}

#[test]
fn buy_should_work_when_route_contains_trades_with_different_pools() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_xyk_pool(AUSD, BSX);
		create_lbp_pool(BSX, NEW_BOOTSTRAPPED_TOKEN);
		create_xyk_pool(MOVR, KSM);
		let share_asset = create_stableswap_pool(vec![NEW_BOOTSTRAPPED_TOKEN, KSM, DOT, DAI, MOVR], 10_000);

		let amount_to_buy = 1 * UNITS;
		let limit = 100 * UNITS;
		let trades = vec![
			Trade {
				pool: PoolType::XYK,
				asset_in: AUSD,
				asset_out: BSX,
			},
			Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: NEW_BOOTSTRAPPED_TOKEN,
			},
			Trade {
				pool: PoolType::Stableswap(share_asset),
				asset_in: NEW_BOOTSTRAPPED_TOKEN,
				asset_out: MOVR,
			},
			Trade {
				pool: PoolType::XYK,
				asset_in: MOVR,
				asset_out: KSM,
			},
		];

		start_lbp_campaign();

		//Act
		assert_ok!(Router::buy(
			Origin::signed(TRADER.into()),
			AUSD,
			KSM,
			amount_to_buy,
			limit,
			trades
		));

		//Assert
		let amount_in = 8049720201692;

		assert_trader_non_native_balance!(BOB_INITIAL_AUSD_BALANCE - amount_in, AUSD);
		assert_trader_bsx_balance!(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance!(BOB_INITIAL_NEW_BOOTSTRAPPED_TOKEN_BALANCE, NEW_BOOTSTRAPPED_TOKEN);
		assert_trader_non_native_balance!(amount_to_buy, KSM);

		expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
			asset_in: AUSD,
			asset_out: KSM,
			amount_in,
			amount_out: amount_to_buy,
		}
		.into()]);
	});
}
