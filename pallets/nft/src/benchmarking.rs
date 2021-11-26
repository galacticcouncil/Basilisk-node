#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate as NFT;
use crate::types::ClassType;
use frame_benchmarking::{account, benchmarks, vec};
use frame_support::traits::{tokens::nonfungibles::InspectEnumerable, Currency, Get};
use frame_system::RawOrigin;
use pallet_uniques as UNQ;
use sp_runtime::traits::UniqueSaturatedInto;

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

fn do_create_class<T: Config>() -> (T::AccountId, <T::Lookup as StaticLookup>::Source, Vec<u8>) {
	let caller = create_account::<T>("caller", 0);
	let caller_lookup = T::Lookup::unlookup(caller.clone());
	let metadata = vec![0; <T as UNQ::Config>::StringLimit::get() as usize];
	assert!(NFT::Pallet::<T>::create_class(
		RawOrigin::Signed(caller.clone()).into(),
		ClassType::Marketplace,
		metadata.clone()
	)
	.is_ok());
	(caller, caller_lookup, metadata)
}

fn do_mint<T: Config>(class_id: u32) {
	let caller = create_account::<T>("caller", 0);
	let metadata = vec![0; <T as UNQ::Config>::StringLimit::get() as usize];

	assert!(NFT::Pallet::<T>::mint(
		RawOrigin::Signed(caller.clone()).into(),
		caller.clone(),
		class_id.into(),
		Some(caller),
		Some(20),
		Some(metadata),
		Some(123u32.into()),
		Some(321u32.into())
	)
	.is_ok());
}

benchmarks! {
	create_class {
		let caller = create_account::<T>("caller", 0);
		let metadata = vec![0; <T as UNQ::Config>::StringLimit::get() as usize];
	}: _(RawOrigin::Signed(caller.clone()), ClassType::Marketplace, metadata)
	verify {
		assert_eq!(UNQ::Pallet::<T>::class_owner(&T::NftClassId::from(0u32).into()), Some(caller));
	}

	mint {
		let (caller, caller_lookup, metadata) = do_create_class::<T>();
	}: _(RawOrigin::Signed(caller.clone()), caller.clone(), 0u32.into(), Some(caller.clone()), Some(20), Some(metadata), Some(123u32.into()), Some(321u32.into()))
	verify {
		assert_eq!(UNQ::Pallet::<T>::owner(T::NftClassId::from(0u32).into(), T::NftInstanceId::from(0u32).into()), Some(caller));
	}

	transfer {
		let caller2 = create_account::<T>("caller2", 1);
		let caller2_lookup = T::Lookup::unlookup(caller2.clone());
		let (caller, caller_lookup, metadata) = do_create_class::<T>();
		do_mint::<T>(0);
	}: _(RawOrigin::Signed(caller), 0u32.into(), 0u32.into(), caller2_lookup)
	verify {
		assert_eq!(UNQ::Pallet::<T>::owner(T::NftClassId::from(0u32).into(), T::NftInstanceId::from(0u32).into()), Some(caller2));
	}

	destroy_class {
		let (caller, caller_lookup, metadata) = do_create_class::<T>();
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into())
	verify {
		assert_eq!(UNQ::Pallet::<T>::classes().count(), 0);
	}

	burn {
		let (caller, caller_lookup, metadata) = do_create_class::<T>();
		do_mint::<T>(0);
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into())
	verify {
		assert_eq!(UNQ::Pallet::<T>::owned(&caller).count(), 0);
	}
}

#[cfg(test)]
#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}
