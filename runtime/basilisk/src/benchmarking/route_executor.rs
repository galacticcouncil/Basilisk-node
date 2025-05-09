// This file is part of Basilisk-node

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(clippy::result_large_err)]

use crate::{AccountId, AssetId, Balance, Currencies, EmaOracle, Router, Runtime, RuntimeOrigin, System, LBP, XYK};

use super::*;
use frame_benchmarking::{account, BenchmarkError, BenchmarkResult};
use frame_support::{
	assert_ok,
	dispatch::DispatchResult,
	ensure, parameter_types,
	traits::{OnFinalize, OnInitialize},
	weights::Weight,
};
use frame_system::RawOrigin;
use hydradx_traits::{
	router::{inverse_route, AssetPair, PoolType, RouteProvider, RouteSpotPriceProvider, RouterT, Trade},
	PriceOracle,
};
use orml_benchmarking::runtime_benchmarks;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use pallet_ema_oracle::OraclePeriod;
use primitives::{constants::currency::UNITS, BlockNumber};
use sp_runtime::{FixedPointNumber, FixedU128};
use sp_std::vec;
pub const INITIAL_BALANCE: Balance = 10_000_000 * UNITS;

const HDX: AssetId = 0;
const DAI: AssetId = 2;

parameter_types! {
	//NOTE: This should always be > 1 otherwise we will payout more than we collected as ED for
	//insufficient assets.
	pub InsufficientEDinBSX: Balance = FixedU128::from_rational(11, 10)
		.saturating_mul_int(<Runtime as pallet_balances::Config>::ExistentialDeposit::get());
}

fn funded_account(name: &'static str, index: u32, assets: &[AssetId]) -> AccountId {
	let account: AccountId = account(name, index, 0);
	//Necessary to pay ED for insufficient assets.
	<Currencies as MultiCurrencyExtended<_>>::update_balance(0, &account, INITIAL_BALANCE as i128).unwrap();

	for asset in assets {
		assert_ok!(<Currencies as MultiCurrencyExtended<_>>::update_balance(
			*asset,
			&account,
			INITIAL_BALANCE.try_into().unwrap(),
		));
	}
	account
}

fn setup_lbp(caller: AccountId, asset_in: AssetId, asset_out: AssetId) -> DispatchResult {
	let asset_in_amount = 1_000_000_000;
	let asset_out_amount = 2_000_000_000;
	let initial_weight = 20_000_000;
	let final_weight = 90_000_000;
	let fee = (2, 1_000);
	let fee_collector = caller.clone();
	let repay_target = 0;

	let pool_id = LBP::pair_account_from_assets(asset_in, asset_out);

	LBP::create_pool(
		RawOrigin::Root.into(),
		caller.clone(),
		asset_in,
		asset_in_amount,
		asset_out,
		asset_out_amount,
		initial_weight,
		final_weight,
		pallet_lbp::WeightCurveType::Linear,
		fee,
		fee_collector,
		repay_target,
	)?;
	ensure!(
		pallet_lbp::PoolData::<Runtime>::contains_key(&pool_id),
		"Pool does not exist."
	);

	let start = 1u32;
	let end = 11u32;

	LBP::update_pool_data(
		RawOrigin::Signed(caller).into(),
		pool_id,
		None,
		Some(start),
		Some(end),
		None,
		None,
		None,
		None,
		None,
	)?;

	System::set_block_number(2u32);
	Ok(())
}

fn create_xyk_pool(asset_a: u32, asset_b: u32) {
	let caller: AccountId = funded_account("caller", 3, &[asset_a, asset_b]);

	assert_ok!(Currencies::update_balance(
		RawOrigin::Root.into(),
		caller.clone(),
		0_u32,
		InsufficientEDinBSX::get() as i128,
	));

	let amount = 100000 * UNITS;
	assert_ok!(Currencies::update_balance(
		RuntimeOrigin::root(),
		caller.clone(),
		asset_a,
		amount as i128,
	));

	assert_ok!(Currencies::update_balance(
		RuntimeOrigin::root(),
		caller.clone(),
		asset_b,
		amount as i128,
	));

	assert_ok!(XYK::create_pool(
		RuntimeOrigin::signed(caller),
		asset_a,
		amount,
		asset_b,
		amount,
	));
}

fn set_period(to: u32) {
	while System::block_number() < Into::<BlockNumber>::into(to) {
		let b = System::block_number();

		System::on_finalize(b);
		EmaOracle::on_finalize(b);

		System::on_initialize(b + 1_u32);
		EmaOracle::on_initialize(b + 1_u32);

		System::set_block_number(b + 1_u32);
	}
}

runtime_benchmarks! {
	{Runtime, pallet_route_executor}

	// Calculates the weight all the logic of ED handling of insufficient assets
	skip_ed_handling_for_trade_with_insufficient_assets{
		let asset_1 = register_external_asset(b"FCA".to_vec()).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_2 = register_external_asset(b"FCB".to_vec()).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let trade = Trade {
			pool: PoolType::LBP,
			asset_in: asset_1,
			asset_out: asset_2
		};

	}: {
		//We assume the worst case, so we change the state 4 times(1 add, 2 modify, 1 kill)
		Router::disable_ed_handling_for_insufficient_assets(3, 0, trade);
		Router::disable_ed_handling_for_insufficient_assets(3, 1, trade);
		Router::disable_ed_handling_for_insufficient_assets(3, 2, trade);
	}
	verify {
	}

	// Calculates the weight of LBP trade. Used in the calculation to determine the weight of the overhead.
	calculate_and_execute_sell_in_lbp {
		let c in 0..1;	// if c == 1, calculate_sell_trade_amounts is executed

		let asset_in = register_asset(b"FCA".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_out = register_asset(b"FCB".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let caller: AccountId = funded_account("caller", 7, &[asset_in, asset_out]);
		let seller: AccountId = funded_account("seller", 8, &[asset_in, asset_out]);

		setup_lbp(caller, asset_in, asset_out)?;

		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in,
			asset_out
		}];

		let amount_to_sell: Balance = 100_000_000;

	}: {
		if c != 0 {
			Router::calculate_sell_trade_amounts(trades.as_slice(), amount_to_sell)?;
		}
		Router::sell(RawOrigin::Signed(seller.clone()).into(), asset_in, asset_out, amount_to_sell, 0u128, trades.clone())?;
	}
	verify {
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(
		asset_in,
		&seller,
		), INITIAL_BALANCE - amount_to_sell);
	}

	// Calculates the weight of LBP trade. Used in the calculation to determine the weight of the overhead.
	calculate_and_execute_buy_in_lbp {
		let c in 1..2;	// number of times `calculate_buy_trade_amounts` is executed
		let b in 0..1;	// if e == 1, buy is executed

		let asset_in = register_asset(b"FCA".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_out = register_asset(b"FCB".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let caller: AccountId = funded_account("caller", 0, &[asset_in, asset_out]);
		let buyer: AccountId = funded_account("buyer", 1, &[asset_in, asset_out]);

		setup_lbp(caller, asset_in, asset_out)?;

		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in,
			asset_out
		}];

		let amount_to_buy = 100_000_000;

	}: {
		for _ in 1..c {
			Router::calculate_buy_trade_amounts(trades.as_slice(), amount_to_buy)?;
		}
		if b != 0 {
			Router::buy(RawOrigin::Signed(buyer.clone()).into(), asset_in, asset_out, amount_to_buy, u128::MAX, trades)?
		}
	}
	verify {
		if b != 0 {
			assert!(<Currencies as MultiCurrency<_>>::free_balance(
			asset_in,
			&buyer,
			) < INITIAL_BALANCE);
		}
	}

	// Calculates the weight of xyk set route. Used in the calculation to determine the weight of the overhead.
	set_route_for_xyk {
		let asset_1 = register_asset(b"AS1".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_2 = register_asset(b"AS2".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_3 = register_asset(b"AS3".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_4 = register_asset(b"AS4".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_5 = register_asset(b"AS5".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_6 = register_asset(b"AS6".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller: AccountId = funded_account("caller", 0, &[asset_1, asset_2, asset_3, asset_4, asset_5, asset_6]);
		create_xyk_pool(HDX, asset_2);
		create_xyk_pool(asset_2, asset_3);
		create_xyk_pool(asset_3, asset_4);
		create_xyk_pool(asset_4, asset_5);
		create_xyk_pool(asset_5, asset_6);


		let route = vec![Trade {
			pool: PoolType::XYK,
			asset_in: HDX,
			asset_out: asset_2
		},Trade {
			pool: PoolType::XYK,
			asset_in: asset_2,
			asset_out: asset_3
		},Trade {
			pool: PoolType::XYK,
			asset_in: asset_3,
			asset_out: asset_4
		},Trade {
			pool: PoolType::XYK,
			asset_in: asset_4,
			asset_out: asset_5
		},Trade {
			pool: PoolType::XYK,
			asset_in: asset_5,
			asset_out: asset_6
		}];

		assert_ok!(Router::sell(
			RawOrigin::Signed(caller.clone()).into(),
			HDX,
			asset_6,
			10_000,
			0,
			route.clone()
		));

		set_period(10);

		Router::set_route(
			RawOrigin::Signed(caller.clone()).into(),
			AssetPair::new(HDX, asset_6),
			route,
		)?;

		create_xyk_pool(HDX, asset_6);

		let better_route = vec![Trade {
			pool: PoolType::XYK,
			asset_in: HDX,
			asset_out: asset_6
		},];

		assert_ok!(Router::sell(
			RawOrigin::Signed(caller.clone()).into(),
			HDX,
			asset_6,
			10_000,
			0,
			better_route.clone()
		));

		set_period(11);

	}: {
		Router::set_route(
			RawOrigin::Signed(caller.clone()).into(),
			AssetPair::new(HDX, asset_6),
			better_route.clone(),
		)?;
	}
	verify {
		let stored_route = Router::route(AssetPair::new(HDX, asset_6)).unwrap();
		assert_eq!(stored_route, better_route);
	}

	force_insert_route {
		let asset_1 = register_asset(b"AS1".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_2 = register_asset(b"AS2".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_3 = register_asset(b"AS3".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_4 = register_asset(b"AS4".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_5 = register_asset(b"AS5".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_6 = register_asset(b"AS6".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller: AccountId = funded_account("caller", 0, &[asset_1, asset_2,asset_3]);
		create_xyk_pool(HDX, asset_2);
		create_xyk_pool(asset_2, asset_3);
		create_xyk_pool(asset_3, asset_4);
		create_xyk_pool(asset_4, asset_5);
		create_xyk_pool(asset_5, asset_6);
		create_xyk_pool(HDX, asset_6);

		let route = vec![Trade {
			pool: PoolType::XYK,
			asset_in: asset_6,
			asset_out: asset_5
		},Trade {
			pool: PoolType::XYK,
			asset_in: asset_5,
			asset_out: asset_4
		},Trade {
			pool: PoolType::XYK,
			asset_in: asset_4,
			asset_out: asset_3
		},Trade {
			pool: PoolType::XYK,
			asset_in: asset_3,
			asset_out: asset_2
		},Trade {
			pool: PoolType::XYK,
			asset_in: asset_2,
			asset_out: HDX
		}];
	}: {
		Router::force_insert_route(
			RawOrigin::Root.into(),
			AssetPair::new(asset_6, HDX),
			route.clone(),
		)?;
	}
	verify {
		let stored_route = Router::route(AssetPair::new(HDX, asset_6)).unwrap();
		assert_eq!(inverse_route(stored_route.to_vec()), route);
	}

	get_route {
		let route = vec![Trade {
			pool: PoolType::XYK,
			asset_in: HDX,
			asset_out: DAI
		}];

		Router::force_insert_route(
			RawOrigin::Root.into(),
			AssetPair::new(HDX, DAI),
			route,
		)?;
	}: {
		Router::get_route(AssetPair::new(HDX, DAI))
	}

	//To calculate the overweight properly we use this to substract
	get_oracle_price_for_xyk {
		let asset_2 = register_asset(b"AS2".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;

		let caller: AccountId = funded_account("caller", 0, &[asset_2]);
		create_xyk_pool(HDX, asset_2);

		let route = vec![Trade {
			pool: PoolType::XYK,
			asset_in: HDX,
			asset_out: asset_2
		}];

		assert_ok!(Router::sell(
			RawOrigin::Signed(caller.clone()).into(),
			HDX,
			asset_2,
			10_000,
			0,
			route.clone()
		));

		set_period(10);
		let price;
	}: {
		 price = <Runtime as pallet_route_executor::Config>::OraclePriceProvider::price(&route, OraclePeriod::Short);
	}
	verify {
		assert!(price.is_some());
	}

	get_oracle_price_for_omnipool {
		// not used in Basilisk runtime
		Err(
			BenchmarkError::Override(BenchmarkResult::from_weight(Weight::MAX))
		)?;
	}: {
	}

	// Calculates the weight of LBP spot price with fee calculation. Used in the calculation to determine the weight of the overhead.
	calculate_spot_price_with_fee_in_lbp {
		let asset_in = register_asset(b"FCA".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let asset_out = register_asset(b"FCB".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
		let caller: AccountId = funded_account("caller", 7, &[asset_in, asset_out]);
		let seller: AccountId = funded_account("seller", 8, &[asset_in, asset_out]);

		setup_lbp(caller, asset_in, asset_out)?;

		let trades = vec![Trade {
			pool: PoolType::LBP,
			asset_in,
			asset_out
		}];

	}: {
		Router::spot_price_with_fee(trades.as_slice());
	}
	verify {
		assert!(Router::spot_price_with_fee(trades.as_slice()).is_some());
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::NativeExistentialDeposit;
	use orml_benchmarking::impl_benchmark_test_suite;
	use sp_runtime::BuildStorage;

	fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		pallet_asset_registry::GenesisConfig::<crate::Runtime> {
			registered_assets: vec![(b"DAI".to_vec(), 1_000, Some(2))],
			native_asset_name: b"BSX".to_vec(),
			native_existential_deposit: NativeExistentialDeposit::get(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		sp_io::TestExternalities::new(t)
	}

	impl_benchmark_test_suite!(new_test_ext(),);
}
