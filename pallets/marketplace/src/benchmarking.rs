#![cfg(feature = "runtime-benchmarks")]

use super::*;

//use crate as MARKET;
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_runtime::traits::UniqueSaturatedInto;

const SEED: u32 = 0;
const ENDOWMENT: u32 = 1_000_000;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);

	let amount = dollar(ENDOWMENT);
	T::Currency::deposit_creating(&caller, amount.unique_saturated_into());

	caller
}

fn dollar(d: u32) -> u128 {
	let d: u128 = d.into();
	d.saturating_mul(100_000_000_000_000)
}

benchmarks! {
	buy {
		let caller = create_account::<T>("caller", 0);
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		
	}

	allow_sell {
		let caller = create_account::<T>("caller", 0);
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		
	}

	withdraw_from_market {
		let caller = create_account::<T>("caller", 0);
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		
	}
}

#[cfg(test)]
mod tests {
	use super::mock::Test;
	use super::*;
	use crate::mock::*;
	use frame_support::assert_ok;

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>test_benchmark_buy());
			assert_ok!(Pallet::<Test>test_benchmark_allow_sell());
			assert_ok!(Pallet::<Test>test_benchmark_withdraw_from_market());
		});
	}
}
