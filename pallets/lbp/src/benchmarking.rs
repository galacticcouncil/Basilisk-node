#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

use primitives::AssetId;

use crate::Pallet as LBP;

const SEED: u32 = 1;

const ASSET_HDX: AssetId = 0;
const ASSET_A_ID: AssetId = 1;
const ASSET_B_ID: AssetId = 2;
const ASSET_A_AMOUNT: Balance = 1_000_000_000;
const ASSET_B_AMOUNT: Balance = 2_000_000_000;
const INITIAL_WEIGHT: LBPWeight = 20_000_000;
const FINAL_WEIGHT: LBPWeight = 90_000_000;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::MultiCurrency::update_balance(ASSET_HDX, &caller, 1_000_000_000_000_000).unwrap();
	T::MultiCurrency::update_balance(ASSET_A_ID, &caller, 1_000_000_000_000_000).unwrap();
	T::MultiCurrency::update_balance(ASSET_B_ID, &caller, 1_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	create_pool {
		let caller = funded_account::<T>("caller", 0);
		let pool_id = T::AssetPairPoolId::from_assets(ASSET_A_ID, ASSET_B_ID);

	}: _(RawOrigin::Root, caller.clone(), ASSET_A_ID, ASSET_A_AMOUNT, ASSET_B_ID, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, Fee::default(), caller)
	verify {
		assert!(PoolData::<T>::contains_key(&pool_id));
	}

	update_pool_data {
		let caller = funded_account::<T>("caller", 0);
		let fee_collector = funded_account::<T>("fee_collector", 0);
		let pool_id = T::AssetPairPoolId::from_assets(ASSET_A_ID, ASSET_B_ID);
		let new_start = T::BlockNumber::from(50_u32);
		let new_end = T::BlockNumber::from(100_u32);
		let new_initial_weight = 45_250_600;
		let new_final_weight = 55_250_600;

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), ASSET_A_ID, ASSET_A_AMOUNT, ASSET_B_ID, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, Fee { numerator: 5, denominator: 1000 }, caller.clone())?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller.clone()), pool_id.clone(), Some(caller.clone()), Some(new_start), Some(new_end), Some(new_initial_weight), Some(new_final_weight), Some(Fee::default()), Some(fee_collector))
	verify {
		let pool_data = LBP::<T>::pool_data(pool_id);
		assert_eq!(pool_data.start, new_start);
		assert_eq!(pool_data.end, new_end);
		assert_eq!(pool_data.initial_weight, new_initial_weight);
		assert_eq!(pool_data.final_weight, new_final_weight);
	}

	add_liquidity {
		let caller = funded_account::<T>("caller", 0);
		let pool_id = T::AssetPairPoolId::from_assets(ASSET_A_ID, ASSET_B_ID);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), ASSET_A_ID, ASSET_A_AMOUNT, ASSET_B_ID, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, Fee::default(), caller.clone())?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller), (ASSET_A_ID, 1_000_000_000_u128), (ASSET_B_ID, 2_000_000_000_u128))
	verify {
		assert_eq!(T::MultiCurrency::free_balance(ASSET_A_ID, &pool_id), 2_000_000_000_u128);
		assert_eq!(T::MultiCurrency::free_balance(ASSET_B_ID, &pool_id), 4_000_000_000_u128);
	}

	remove_liquidity {
		let caller = funded_account::<T>("caller", 0);
		let pool_id = T::AssetPairPoolId::from_assets(ASSET_A_ID, ASSET_B_ID);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), ASSET_A_ID, ASSET_A_AMOUNT, ASSET_B_ID, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, Fee::default(), caller.clone())?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller.clone()), pool_id.clone())
	verify {
		assert!(!PoolData::<T>::contains_key(&pool_id));
		assert_eq!(T::MultiCurrency::free_balance(ASSET_A_ID, &caller), 1000000000000000);
		assert_eq!(T::MultiCurrency::free_balance(ASSET_B_ID, &caller), 1000000000000000);
	}

	sell {
		let caller = funded_account::<T>("caller", 0);
		let fee_collector = funded_account::<T>("fee_collector", 0);
		let asset_in: AssetId = ASSET_A_ID;
		let asset_out: AssetId = ASSET_B_ID;
		let amount : Balance = 100_000_000;
		let max_limit: Balance = 10_000_000;

		let pool_id = T::AssetPairPoolId::from_assets(ASSET_A_ID, ASSET_B_ID);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), ASSET_A_ID, ASSET_A_AMOUNT, ASSET_B_ID, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, Fee::default(), fee_collector.clone())?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

		let start = T::BlockNumber::from(1u32);
		let end = T::BlockNumber::from(11u32);

		LBP::<T>::update_pool_data(RawOrigin::Signed(caller.clone()).into(), pool_id.clone(), None, Some(start), Some(end), None, None, None, None)?;

		let pool_data = LBP::<T>::pool_data(&pool_id);

	}: _(RawOrigin::Signed(caller.clone()), asset_in, asset_out, amount, max_limit)
	verify{
		assert_eq!(T::MultiCurrency::free_balance(asset_in, &caller), 999998900000000);
		assert_eq!(T::MultiCurrency::free_balance(asset_out, &caller), 999998046997637);
		assert_eq!(T::MultiCurrency::free_balance(asset_out, &fee_collector), 1000000000094183);
	}

	buy {
		let caller = funded_account::<T>("caller", 0);
		let fee_collector = funded_account::<T>("fee_collector", 0);
		let asset_out: AssetId = ASSET_A_ID;
		let asset_in: AssetId = ASSET_B_ID;
		let amount : Balance = 100_000_000;
		let max_limit: Balance = 1_000_000_000;
		let pool_id = T::AssetPairPoolId::from_assets(ASSET_A_ID, ASSET_B_ID);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), ASSET_A_ID, ASSET_A_AMOUNT, ASSET_B_ID, ASSET_B_AMOUNT, INITIAL_WEIGHT, FINAL_WEIGHT, WeightCurveType::Linear, Fee::default(), fee_collector.clone())?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

		let start = T::BlockNumber::from(1u32);
		let end = T::BlockNumber::from(11u32);

		LBP::<T>::update_pool_data(RawOrigin::Signed(caller.clone()).into(), pool_id.clone(), None, Some(start), Some(end), None, None, None, None)?;

		let pool_data = LBP::<T>::pool_data(&pool_id);

	}: _(RawOrigin::Signed(caller.clone()), asset_out, asset_in, amount, max_limit)
	verify{
		assert_eq!(T::MultiCurrency::free_balance(asset_out, &caller), 999999100000000);
		assert_eq!(T::MultiCurrency::free_balance(asset_in, &caller), 999997946513050);
		assert_eq!(T::MultiCurrency::free_balance(asset_in, &fee_collector), 1000000000106760);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_create_pool());
			assert_ok!(Pallet::<Test>::test_benchmark_update_pool_data());
			assert_ok!(Pallet::<Test>::test_benchmark_add_liquidity());
			assert_ok!(Pallet::<Test>::test_benchmark_remove_liquidity());
			assert_ok!(Pallet::<Test>::test_benchmark_sell());
			assert_ok!(Pallet::<Test>::test_benchmark_buy());
		});
	}
}
