#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{BlockNumber, Origin, Router, XYK};
use primitives::{AssetId, Price};
use xcm_emulator::TestExt;

use sp_arithmetic::fixed_point::FixedPointNumber;

use frame_support::{assert_noop};
use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};

use hydradx_traits::router::PoolType;
use orml_traits::MultiCurrency;
use pallet_route_executor::Trade;
use primitives::asset::AssetPair;

const BSX: u32 = 0;
const AUSD: u32 = 1;
const MOVR: u32 = 2;
const KSM: u32 = 3;

const TRADER: [u8; 32] = BOB;
pub const BOB_INITIAL_AUSD_BALANCE: u128 = BOB_INITIAL_ASSET_1_BALANCE;

#[test]
fn sell_should_work_when_route_contains_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, KSM);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(0, KSM);

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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance(amount_out, KSM);

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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
		assert_trader_non_native_balance(amount_out, KSM);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, MOVR);

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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn sell_should_fail_when_first_trade_is_successful_but_second_trade_has_no_supported_pool() {
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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, KSM);
	});
}

#[test]
fn sell_should_fail_when_balance_is_not_sufficient() {
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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn sell_should_fail_when_trading_limit_is_below_minimum() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn sell_should_fail_when_buying_more_than_max_in_ratio_out() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, AUSD);

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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}
#[test]
fn buy_should_work_when_route_contains_single_trade() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		//Arrange
		create_pool(BSX, KSM);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(0, KSM);

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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_in);
		assert_trader_non_native_balance(amount_to_buy, KSM);

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
		create_pool(BSX, KSM);
		create_pool(KSM, AUSD);

		assert_trader_bsx_balance(BOB_INITIAL_AUSD_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, KSM);

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
fn buy_should_work_when_route_contains_multiple_trades() {
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
fn buy_should_fail_when_there_is_no_pool_for_specific_asset_pair() {
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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn buy_should_fail_when_first_trade_is_successful_but_second_trade_has_no_supported_pool() {
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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
		assert_trader_non_native_balance(0, KSM);
	});
}

#[test]
fn buy_should_fail_when_balance_is_not_sufficient() {
	Basilisk::execute_with(|| {
		//Arrange
		create_pool(KSM, AUSD);

		assert_trader_non_native_balance(0, KSM);
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

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn buy_should_fail_when_trading_limit_is_below_minimum() {
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
			Router::buy(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_buy, limit, trades),
			pallet_xyk::Error::<basilisk_runtime::Runtime>::InsufficientTradingAmount
		);

		assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE);
		assert_trader_non_native_balance(BOB_INITIAL_AUSD_BALANCE, AUSD);
	});
}

#[test]
fn buy_should_fail_when_buying_more_than_max_ratio_out() {
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
			Router::buy(Origin::signed(TRADER.into()), BSX, AUSD, amount_to_buy, limit, trades),
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


mod lbp_router_tests {
	use crate::kusama_test_net::*;

	use basilisk_runtime::{Origin, Router, LBP, BlockNumber};
	use primitives::{AssetId, Price};
	use xcm_emulator::TestExt;

	use sp_arithmetic::fixed_point::FixedPointNumber;
	use hydradx_traits::AMM;

	use crate::router::{BSX};
	use crate::router::KSM;
	use crate::router::AUSD;
	use crate::router::TRADER;
	use frame_support::{assert_noop, assert_ok};
	use hydradx_traits::router::PoolType;
	use orml_traits::MultiCurrency;
	use pallet_route_executor::Trade;
	use polkadot_xcm::prelude::AccountId32;
	use pallet_lbp::WeightCurveType;
	use primitives::asset::AssetPair;

	pub const SALE_START: Option<BlockNumber> = Some(10);
	pub const SALE_END: Option<BlockNumber> = Some(40);

	//TODO: handle validation error:
	// -make the validation togleable
	// abstract the validaton away
	// taking the fee from somewhere else?
	#[test]
	fn sell_should_work_when_route_contains_single_trade() {
		TestNet::reset();

		Basilisk::execute_with(|| {
			//Arrange
			create_pool(BSX, KSM);

			let amount_to_sell = 10 * UNITS;
			let limit = 0;
			let trades = vec![Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: KSM,
			}];

			set_relaychain_block_number(SALE_START.unwrap() + 1);

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
			let amount_out = 5304848609011;

			super::assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
			super::assert_trader_non_native_balance(amount_out, KSM);

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
	fn sell_should_work_when_selling_distributed_asset_in_a_single_trade() {
		TestNet::reset();

		Basilisk::execute_with(|| {
			//Arrange
			create_pool(BSX, AUSD);

			let amount_to_sell = 10 * UNITS;
			let limit = 0;
			let trades = vec![Trade {
				pool: PoolType::LBP,
				asset_in: AUSD,
				asset_out: BSX,
			}];

			set_relaychain_block_number(SALE_START.unwrap() + 1);

			//Act
			assert_ok!(Router::sell(
				Origin::signed(TRADER.into()),
				AUSD,
				BSX,
				amount_to_sell,
				limit,
				trades
			));

			//Assert
			let amount_out = 15853066253648;

			super::assert_trader_non_native_balance(BOB_INITIAL_ASSET_1_BALANCE - amount_to_sell, AUSD);
			super::assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE + amount_out);

			expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
				asset_in: AUSD,
				asset_out: BSX,
				amount_in: amount_to_sell,
				amount_out,
			}
				.into()]);
		});
	}

	#[test]
	fn sell_should_work_when_route_contains_double_trades() {
		TestNet::reset();

		Basilisk::execute_with(|| {
			//Arrange
			create_pool(BSX, KSM);
			super::create_pool(KSM, AUSD);

			let amount_to_sell = 10 * UNITS;
			let limit = 0;
			let trades = vec![Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: KSM,
			},
			  Trade {
				  pool: PoolType::XYK,
				  asset_in: KSM,
				  asset_out: AUSD,
			  }];

			set_relaychain_block_number(SALE_START.unwrap() + 1);

			//Act
			assert_ok!(Router::sell(
				Origin::signed(TRADER.into()),
				BSX,
				AUSD,
				amount_to_sell,
				limit,
				trades
			));

			//Assert
			let amount_out = 2511249070223;

			super::assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_to_sell);
			super::assert_trader_non_native_balance(0, KSM);
			super::assert_trader_non_native_balance(BOB_INITIAL_ASSET_1_BALANCE + amount_out, AUSD);

			expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
				asset_in: BSX,
				asset_out: AUSD,
				amount_in: amount_to_sell,
				amount_out,
			}
				.into()]);
		});
	}

	#[test]
	fn buy_should_work_when_when_buying_distributed_asset() {
		TestNet::reset();

		Basilisk::execute_with(|| {
			//Arrange
			create_pool(BSX, KSM);

			let amount_to_buy = 10 * UNITS;
			let limit = 100 * UNITS;
			let trades = vec![Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: KSM,
			}];

			set_relaychain_block_number(SALE_START.unwrap() + 1);

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
			let amount_in = 19944393324840;

			super::assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - amount_in);
			super::assert_trader_non_native_balance(amount_to_buy, KSM);

			expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
				asset_in: BSX,
				asset_out: KSM,
				amount_in,
				amount_out: amount_to_buy,
			}.into()]);
		});
	}

	#[test]
	fn buy_should_work_when_buying_accumulated_asset_in_a_single_trade() {
		TestNet::reset();

		Basilisk::execute_with(|| {
			//Arrange
			create_pool(BSX, AUSD);

			let amount_to_buy = 10 * UNITS;
			let limit = 100 * UNITS;
			let trades = vec![Trade {
				pool: PoolType::LBP,
				asset_in: AUSD,
				asset_out: BSX,
			}];

			set_relaychain_block_number(SALE_START.unwrap() + 1);

			//Act
			assert_ok!(Router::buy(
				Origin::signed(TRADER.into()),
				AUSD,
				BSX,
				amount_to_buy,
				limit,
				trades
			));

			//Assert
			let amount_in = 6045520780025;

			super::assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE + amount_to_buy);
			super::assert_trader_non_native_balance(BOB_INITIAL_ASSET_1_BALANCE - amount_in, AUSD);

			expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
				asset_in: AUSD,
				asset_out: BSX,
				amount_in,
				amount_out: amount_to_buy,
			}.into()]);
		});
	}

	#[test]
	fn lbp_direct_buy_should_yield_the_same_result_as_router_buy() {
		TestNet::reset();

		let amount_to_buy = 10 * UNITS;
		let limit = 100 * UNITS;
		let spent_amount_in = 19944393324840;

		Basilisk::execute_with(|| {
			//Arrange
			create_pool(BSX, KSM);

			let trades = vec![Trade {
				pool: PoolType::LBP,
				asset_in: BSX,
				asset_out: KSM,
			}];

			set_relaychain_block_number(SALE_START.unwrap() + 1);

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
			super::assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - spent_amount_in);
			super::assert_trader_non_native_balance(amount_to_buy, KSM);

			expect_basilisk_events(vec![pallet_route_executor::Event::RouteExecuted {
				asset_in: BSX,
				asset_out: KSM,
				amount_in: spent_amount_in,
				amount_out: amount_to_buy,
			}.into()]);
		});

		TestNet::reset();

		Basilisk::execute_with(|| {
			//Arrange
			create_pool(BSX, KSM);

			set_relaychain_block_number(SALE_START.unwrap() + 1);

			//Act
			assert_ok!(LBP::buy(Origin::signed(TRADER.into()), KSM, BSX, amount_to_buy, limit));

			//Assert
			super::assert_trader_bsx_balance(BOB_INITIAL_BSX_BALANCE - spent_amount_in);
			super::assert_trader_non_native_balance(amount_to_buy, KSM);
		});
	}


	fn create_pool(asset_a: u32, asset_b: u32) {
		assert_ok!(LBP::create_pool(
			Origin::root(),
			ALICE.into(),
			asset_a,
			100 * UNITS,
			asset_b,
			200 * UNITS,
			20_000_000,
			80_000_000,
			WeightCurveType::Linear,
			(2, 1_000),
			CHARLIE.into(),
			0,
		));

		let accountId = pool_account(asset_a, asset_b);

		assert_ok!(LBP::update_pool_data(
			Origin::signed(AccountId::from(ALICE)),
			accountId.into(),
			None,
			SALE_START,
			SALE_END,
			None,
			None,
			None,
			None,
			None,
		));
	}

	fn pool_account(asset_a: AssetId, asset_b: AssetId) -> AccountId {
		let asset_pair = AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		};
		LBP::get_pair_id(asset_pair)
	}

	pub fn set_block_number<T: frame_system::Config<BlockNumber = u32>>(n: u32) {
		use frame_support::traits::OnFinalize;

		frame_system::Pallet::<T>::set_block_number(n);
		basilisk_runtime::System::on_finalize(n);
		basilisk_runtime::MultiTransactionPayment::on_finalize(n);
	}

}

