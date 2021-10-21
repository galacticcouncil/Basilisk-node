#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate as NFT;
use frame_benchmarking::{account, benchmarks, vec};
use frame_support::traits::{tokens::nonfungibles::InspectEnumerable, Currency, Get};
use frame_system::RawOrigin;
use pallet_uniques as UNQ;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::convert::TryInto;
use crate::types::ClassType;

const SEED: u32 = 0;
const ENDOWMENT: u32 = 1_000_000;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);

	let amount = dollar(ENDOWMENT);
	<T as NFT::Config>::Currency::deposit_creating(&caller, amount.unique_saturated_into());

	caller
}

fn dollar(d: u32) -> u128 {
	let d: u128 = d.into();
	d.saturating_mul(100_000_000_000_000)
}

benchmarks! {
	create_class {
		let caller = create_account::<T>("caller", 0);
		let metadata: BoundedVec<u8, T::ValueLimit> = vec![0; <T as UNQ::Config>::ValueLimit::get() as usize].try_into().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), ClassType::Art, metadata)
	verify {
		assert_eq!(UNQ::Pallet::<T>::class_owner(&T::ClassId::from(0u32)), Some(caller));
	}

	mint {
		let caller = create_account::<T>("caller", 0);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let metadata: BoundedVec<u8, T::ValueLimit> = vec![0; <T as UNQ::Config>::ValueLimit::get() as usize].try_into().unwrap();
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, metadata.clone()).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), metadata)
	verify {
		assert_eq!(UNQ::Pallet::<T>::owner(T::ClassId::from(0u32), T::InstanceId::from(0u32)), Some(caller));
	}

	transfer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let caller2_lookup = T::Lookup::unlookup(caller2.clone());
		let metadata: BoundedVec<u8, T::ValueLimit> = vec![0; <T as UNQ::Config>::ValueLimit::get() as usize].try_into().unwrap();
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, metadata.clone()).unwrap_or_default();
		NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), metadata).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into(), caller2_lookup)
	verify {
		assert_eq!(UNQ::Pallet::<T>::owner(T::ClassId::from(0u32), T::InstanceId::from(0u32)), Some(caller2));
	}

	destroy_class {
		let caller = create_account::<T>("caller", 0);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let metadata: BoundedVec<u8, T::ValueLimit> = vec![0; <T as UNQ::Config>::ValueLimit::get() as usize].try_into().unwrap();
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, metadata.clone()).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into())
	verify {
		assert_eq!(UNQ::Pallet::<T>::classes().count(), 0);
	}

	burn {
		let caller = create_account::<T>("caller", 0);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let metadata: BoundedVec<u8, T::ValueLimit> = vec![0; <T as UNQ::Config>::ValueLimit::get() as usize].try_into().unwrap();
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, metadata.clone()).unwrap_or_default();
		NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), metadata).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into())
	verify {
		assert_eq!(UNQ::Pallet::<T>::owned(&caller).count(), 0);
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
			assert_ok!(Pallet::<Test>::test_benchmark_create_class());
			assert_ok!(Pallet::<Test>::test_benchmark_mint());
			assert_ok!(Pallet::<Test>::test_benchmark_transfer());
			assert_ok!(Pallet::<Test>::test_benchmark_burn());
			assert_ok!(Pallet::<Test>::test_benchmark_destroy_class());
		});
	}
}
