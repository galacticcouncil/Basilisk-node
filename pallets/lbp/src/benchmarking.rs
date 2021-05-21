#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_system::RawOrigin;

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

		let available_amount = T::MultiCurrency::free_balance(asset_a.id, &caller)
			.saturating_sub(asset_a.amount);

	}: _(RawOrigin::Root, caller.clone(), asset_a, asset_b, duration, WeightCurveType::Linear, false)
	verify {
		assert_eq!(T::MultiCurrency::free_balance(asset_a.id, &caller), available_amount);
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
		});
	}
}
