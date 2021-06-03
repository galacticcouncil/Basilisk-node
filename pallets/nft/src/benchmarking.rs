#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const SEED: u32 = 0;
const EMOTE: &str = "RMRK::EMOTE::RMRK1.0.0::0aff6865bed3a66b-VALHELLO-POTION_HEAL-0000000000000001::1F389";

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::deposit_into_existing(&caller, T::CurrencyBalance::from(1_000_000_u128).into()).unwrap();
	caller
}

benchmarks! {
	create_class {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = "just some class data".as_bytes().to_vec();
	}: _(RawOrigin::Signed(caller.clone()), class_metadata, class_data, T::CurrencyBalance::from(666_u128).into())
	verify {
	}

	mint {
		let i in 1 .. 1000;
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let token_data = TokenData { locked:false, emote:EMOTE.as_bytes().to_vec() };
		let class_data = "just some class data".as_bytes().to_vec();
		let class_id = orml_nft::Pallet::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), class_id, class_metadata, token_data, i)
	verify {
	}

	transfer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = "just some class data".as_bytes().to_vec();
		let token_data = TokenData { locked:false, emote:EMOTE.as_bytes().to_vec() };
		let class_id = orml_nft::Pallet::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Pallet::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller.clone()), T::Lookup::unlookup(caller2.clone()), token)
	verify {
	}

	destroy_class {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = "just some class data".as_bytes().to_vec();
		let class_id = orml_nft::Pallet::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), class_id)
	verify {
	}

	burn {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = "just some class data".as_bytes().to_vec();
		let token_data = TokenData { locked:false, emote:EMOTE.as_bytes().to_vec() };
		let class_id = orml_nft::Pallet::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Pallet::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller.clone()), token)
	verify {
	}

	buy_from_pool {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = "just some class data".as_bytes().to_vec();
		let token_data = TokenData { locked:false, emote:EMOTE.as_bytes().to_vec() };
		let class_id = orml_nft::Pallet::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Pallet::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller2.clone()), token)
	verify {
	}

	sell_to_pool {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = "just some class data".as_bytes().to_vec();
		let token_data = TokenData { locked:false, emote:EMOTE.as_bytes().to_vec() };
		let class_id = orml_nft::Pallet::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Pallet::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
		orml_nft::Pallet::<T>::transfer(&caller, &caller2, token).unwrap_or_default();
	}: _(RawOrigin::Signed(caller2.clone()), token)
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
			assert_ok!(test_benchmark_buy_from_pool::<Test>());
			assert_ok!(test_benchmark_sell_to_pool::<Test>());
		});
	}
}
