#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::boxed::Box;
use sp_std::vec;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	caller
}

benchmarks! {
	create_class {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = 123;
	}: _(RawOrigin::Signed(caller.clone()), class_metadata, class_data)
	verify {
	}

	mint {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let token_data = TokenData { locked:false };
		let class_data = 123;
		let class_id = orml_nft::Module::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_quantity = 1;
	}: _(RawOrigin::Signed(caller.clone()), class_id, class_metadata, token_data, token_quantity)
	verify {
	}

	transfer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = 123;
		let token_data = TokenData { locked:false };
		let class_id = orml_nft::Module::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Module::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller.clone()), T::Lookup::unlookup(caller2.clone()), token)
	verify {
	}

	destroy_class {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = 123;
		let class_id = orml_nft::Module::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), class_id)
	verify {
	}

	burn {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = 123;
		let token_data = TokenData { locked:false };
		let class_id = orml_nft::Module::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Module::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller.clone()), token)
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
			assert_ok!(test_benchmark_create_class::<Test>());
			assert_ok!(test_benchmark_mint::<Test>());
			assert_ok!(test_benchmark_transfer::<Test>());
			assert_ok!(test_benchmark_burn::<Test>());
			assert_ok!(test_benchmark_destroy_class::<Test>());
		});
	}
}
