#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate as NFT;
use frame_benchmarking::{account, benchmarks, vec};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::convert::TryInto;

const SEED: u32 = 0;
const ENDOWMENT: u32 = 1_000_000;
const CURRENT_STRING_LIMIT: u32 = 128;

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
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let metadata: BoundedVec<u8, T::StringLimit> = vec![0; CURRENT_STRING_LIMIT as usize].try_into().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), caller_lookup, metadata)
	verify {
		// assert_eq!(pallet_uniques::Pallet::<T>::Class::get(class_id).iter().count(), 1);
	}

	mint {
		let caller = create_account::<T>("caller", 0);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let metadata: BoundedVec<u8, T::StringLimit> = vec![0; CURRENT_STRING_LIMIT as usize].try_into().unwrap();
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), caller_lookup.clone(), metadata.clone()).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), Default::default(), caller_lookup, 10u8, metadata)
	verify {
		// assert_eq!(orml_nft::Pallet::<T>::tokens_by_owner((caller, token.0, token.1)), ());
	}

	transfer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let caller2_lookup = T::Lookup::unlookup(caller.clone());
		let metadata: BoundedVec<u8, T::StringLimit> = vec![0; CURRENT_STRING_LIMIT as usize].try_into().unwrap();
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), caller_lookup.clone(), metadata.clone()).unwrap_or_default();
		NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), Default::default(), Default::default(), caller_lookup, 10u8, metadata).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), Default::default(), caller2_lookup)
	verify {
		// let transferred_token = orml_nft::Pallet::<T>::tokens(class_id, token_id);
		// assert_eq!(transferred_token.unwrap().owner, caller2);
	}

	/*** Currently there is no public constructor for DestroyWitness struct (Class storage is private as well)
	destroy_class {
		let n in 0 .. 1_000;
		let caller = create_account::<T>("caller", 0);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), caller_lookup.clone()).unwrap_or_default();
		for i in 0..n {
			NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), Default::default(), (i as u16).into(), caller_lookup.clone()).unwrap_or_default();
		}
		let witness = <pallet_uniques::Class::<T>>::get(Default::default()).unwrap().destroy_witness();
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), witness)
	verify {
		//assert_eq!(orml_nft::Pallet::<T>::classes(class_id).iter().count(), 0);
	}
	***/

	burn {
		let caller = create_account::<T>("caller", 0);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let metadata: BoundedVec<u8, T::StringLimit> = vec![0; CURRENT_STRING_LIMIT as usize].try_into().unwrap();
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), caller_lookup.clone(), metadata.clone()).unwrap_or_default();
		NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), Default::default(), Default::default(), caller_lookup.clone(), 0u8, metadata).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), Default::default(), Some(caller_lookup))
	verify {
		//assert_eq!(orml_nft::Pallet::<T>::tokens(class_id, token_id).iter().count(), 0);
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
			//assert_ok!(Pallet::<Test>::test_benchmark_destroy_class());
		});
	}
}
