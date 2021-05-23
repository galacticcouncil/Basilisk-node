#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::{RawOrigin, Pallet as System};

use primitives::AssetId;

use crate::Pallet as LBP;

const SEED: u32 = 1;

const ASSET_HDX: AssetId = 0;
const ASSET_ID_A: AssetId = 1;
const ASSET_ID_B: AssetId = 2;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::MultiCurrency::update_balance(ASSET_HDX, &caller, 1_000_000_000_000_000).unwrap();
	T::MultiCurrency::update_balance(ASSET_ID_A, &caller, 1_000_000_000_000_000).unwrap();
	T::MultiCurrency::update_balance(ASSET_ID_B, &caller, 1_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	create_pool {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

	}: _(RawOrigin::Root, caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)
	verify {
		assert_eq!(PoolData::<T>::contains_key(&pool_id), true);
	}

	update_pool_data {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

		let new_duration = (T::BlockNumber::from(50_u32), T::BlockNumber::from(100_u32));
		let new_initial_weights = ((asset_a.id, 1_000),(asset_b.id, 5_000_000));
		let new_final_weights = ((asset_a.id, 300_000_000),(asset_b.id, 60));

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller), pool_id.clone(), Some(new_duration.0), Some(new_duration.1), Some(new_initial_weights), Some(new_final_weights))
	verify {
		let pool_data = LBP::<T>::pool_data(pool_id);
		assert_eq!(pool_data.start, new_duration.0);
		assert_eq!(pool_data.end, new_duration.1);
		assert_eq!(pool_data.initial_weights, new_initial_weights);
		assert_eq!(pool_data.final_weights, new_final_weights);
	}

	pause_pool {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

		LBP::<T>::unpause_pool(RawOrigin::Signed(caller.clone()).into(), pool_id.clone())?;
		let pool_data = LBP::<T>::pool_data(&pool_id);
		assert_eq!(pool_data.paused, false);

	}: _(RawOrigin::Signed(caller), pool_id.clone())
	verify {
		let pool_data = LBP::<T>::pool_data(pool_id);
		assert_eq!(pool_data.paused, true);
	}

	unpause_pool {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");
		let pool_data = LBP::<T>::pool_data(&pool_id);
		assert_eq!(pool_data.paused, true);

	}: _(RawOrigin::Signed(caller), pool_id.clone())
	verify {
		let pool_data = LBP::<T>::pool_data(pool_id);
		assert_eq!(pool_data.paused, false);
	}

	add_liquidity {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller), pool_id.clone(), 1_000_000_000_u128, 2_000_000_000_u128)
	verify {
		assert_eq!(T::MultiCurrency::free_balance(asset_a.id, &pool_id), 2_000_000_000_u128);
		assert_eq!(T::MultiCurrency::free_balance(asset_b.id, &pool_id), 4_000_000_000_u128);
	}

	remove_liquidity {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

	}: _(RawOrigin::Signed(caller), pool_id.clone(), 500_000_000_u128, 1_000_000_000_u128)
	verify {
		assert_eq!(T::MultiCurrency::free_balance(asset_a.id, &pool_id), 500_000_000_u128);
		assert_eq!(T::MultiCurrency::free_balance(asset_b.id, &pool_id), 1_000_000_000_u128);
	}

	destroy_pool {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

		System::<T>::set_block_number(21u32.into());

	}: _(RawOrigin::Signed(caller), pool_id.clone())
	verify {
		assert_eq!(PoolData::<T>::contains_key(&pool_id), false);
	}

	sell {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let asset_in: AssetId = ASSET_ID_A;
		let asset_out: AssetId = ASSET_ID_B;
		let amount : Balance = 100_000_000;
		let max_limit: Balance = 10_000_000;

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

		LBP::<T>::unpause_pool(RawOrigin::Signed(caller.clone()).into(), pool_id.clone())?;
		let pool_data = LBP::<T>::pool_data(&pool_id);
		assert_eq!(pool_data.paused, false);

		System::<T>::set_block_number(12u32.into());

	}: _(RawOrigin::Signed(caller.clone()), asset_in, asset_out, amount, max_limit)
	verify{
		assert_eq!(T::MultiCurrency::free_balance(asset_in, &caller), 999998900000000);
		assert_eq!(T::MultiCurrency::free_balance(asset_out, &caller), 999998095642676);
	}

	buy {
		let caller = funded_account::<T>("caller", 0);
		let duration = (T::BlockNumber::from(10_u32), T::BlockNumber::from(20_u32));
		let asset_a = LBPAssetInfo{
			id: ASSET_ID_A,
			amount: BalanceOf::<T>::from(1_000_000_000_u32),
			initial_weight: 20,
			final_weight: 90
		};
		let asset_b = LBPAssetInfo{
			id: ASSET_ID_B,
			amount: BalanceOf::<T>::from(2_000_000_000_u32),
			initial_weight: 80,
			final_weight: 10
		};

		let asset_in: AssetId = ASSET_ID_A;
		let asset_out: AssetId = ASSET_ID_B;
		let amount : Balance = 100_000_000;
		let max_limit: Balance = 1_000_000_000;

		let pool_id = T::AssetPairPoolId::from_assets(asset_a.id, asset_b.id);

		LBP::<T>::create_pool(RawOrigin::Root.into(), caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, true)?;
		ensure!(PoolData::<T>::contains_key(&pool_id), "Pool does not exist.");

		LBP::<T>::unpause_pool(RawOrigin::Signed(caller.clone()).into(), pool_id.clone())?;
		let pool_data = LBP::<T>::pool_data(&pool_id);
		assert_eq!(pool_data.paused, false);

		System::<T>::set_block_number(12u32.into());

	}: _(RawOrigin::Signed(caller.clone()), asset_in, asset_out, amount, max_limit)
	verify{
		assert_eq!(T::MultiCurrency::free_balance(asset_in, &caller), 999999100000000);
		assert_eq!(T::MultiCurrency::free_balance(asset_out, &caller), 999997888194028);
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
			assert_ok!(test_benchmark_create_pool::<Test>());
			assert_ok!(test_benchmark_update_pool_data::<Test>());
			assert_ok!(test_benchmark_pause_pool::<Test>());
			assert_ok!(test_benchmark_unpause_pool::<Test>());
			assert_ok!(test_benchmark_add_liquidity::<Test>());
			assert_ok!(test_benchmark_remove_liquidity::<Test>());
			assert_ok!(test_benchmark_destroy_pool::<Test>());
			assert_ok!(test_benchmark_sell::<Test>());
			assert_ok!(test_benchmark_buy::<Test>());
		});
	}
}
