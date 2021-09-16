#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, vec};
use frame_system::RawOrigin;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::convert::TryInto;

const SEED: u32 = 0;
const ENDOWMENT: u32 = 1_000_000;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);

	let amount = dollar(ENDOWMENT);
	<T as pallet_nft::Config>::Currency::deposit_creating(&caller, amount.unique_saturated_into());

	caller
}

fn dollar(d: u32) -> u128 {
	let d: u128 = d.into();
	d.saturating_mul(100_000_000_000_000)
}

benchmarks! {

	set_price {
		let caller = create_account::<T>("caller", 0);

		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), T::Lookup::unlookup(caller.clone()), bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), Default::default(), 0u16.into(), T::Lookup::unlookup(caller.clone()), 10u8, bvec![0])?;

	}: _(RawOrigin::Signed(caller.clone()), Default::default(), 0u16.into(), Some(1u32.into()))
	verify {

	}

	buy {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), T::Lookup::unlookup(caller.clone()), bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), Default::default(), 0u16.into(), T::Lookup::unlookup(caller.clone()), 10u8, bvec![0])?;

	}: _(RawOrigin::Signed(caller2.clone()), caller, Default::default(), 0u16.into())
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
			assert_ok!(Pallet::<Test>::test_benchmark_buy());
			assert_ok!(Pallet::<Test>::test_benchmark_set_price());
		});
	}
}
