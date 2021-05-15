#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use frame_support::dispatch::DispatchResultWithPostInfo;

use primitives::{AssetId};

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

		let asset_a: AssetId = ASSET_ID_A;
		let asset_b: AssetId = ASSET_ID_B;
		let amount_a = BalanceOf::<T>::from(1_000_000_000_u32);
		let amount_b = BalanceOf::<T>::from(2_000_000_000_u32);
		let start = T::BlockNumber::from(10_u32);
		let end = T::BlockNumber::from(20_u32);
		let last_weight_update = T::BlockNumber::from(0_u32);
		let pool_data = Pool {
			start: start,
			end: end,
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: last_weight_update,
			last_weights: ((1, 20), (2, 80)),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		let available_amount = T::MultiCurrency::free_balance(asset_a, &caller)
			.saturating_sub(amount_a);

	}: _(RawOrigin::Signed(caller.clone()), asset_a, amount_a, asset_b, amount_b, pool_data)
	verify {
		assert_eq!(T::MultiCurrency::free_balance(asset_a, &caller), available_amount);
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