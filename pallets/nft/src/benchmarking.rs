#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate as NFT;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::vec;

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
	create_class {
		let caller = create_account::<T>("caller", 0);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let class_metadata = big_vec.clone();
		let class_data = ClassData { is_pool:false };
		let class_id = orml_nft::Pallet::<T>::next_class_id();
	}: _(RawOrigin::Signed(caller.clone()), class_metadata.clone(), class_data.clone())
	verify {
		assert_eq!(orml_nft::Pallet::<T>::classes(class_id).iter().count(), 1);
	}

	create_pool {
		let caller = create_account::<T>("caller", 0);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let class_metadata = big_vec.clone();
		let class_data = ClassData { is_pool:true };
		let class_id = orml_nft::Pallet::<T>::next_class_id();
		let price: BalanceOf<T> = u32::MAX.into();
	}: _(RawOrigin::Signed(caller.clone()), class_metadata.clone(), class_data.clone(), price)
	verify {
		assert_eq!(orml_nft::Pallet::<T>::classes(class_id).iter().count(), 1);
	}

	mint {
		let caller = create_account::<T>("caller", 0);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let big_emote = vec![1; T::MaxEmoteLength::get() as usize];
		let class_metadata = big_vec.clone();
		let token_data = TokenData { locked:false, emote:big_emote };
		let class_data = ClassData { is_pool:false };
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), class_metadata.clone(), class_data).unwrap_or_default();
		let class_id = 0u32.into();
		let token_id = orml_nft::Pallet::<T>::next_token_id(class_id);
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller.clone()), class_id, class_metadata, token_data, 1u32.into())
	verify {
		assert_eq!(orml_nft::Pallet::<T>::tokens_by_owner((caller, token.0, token.1)), ());
	}

	transfer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let big_emote = vec![1; T::MaxEmoteLength::get() as usize];
		let class_metadata = big_vec.clone();
		let class_data = ClassData { is_pool:false };
		let token_data = TokenData { locked:false, emote:big_emote };
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), class_metadata.clone(), class_data).unwrap_or_default();
		let class_id = 0u32.into();
		NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), class_id, class_metadata.clone(), token_data, T::MintMaxQuantity::get()).unwrap_or_default();
		let token_id = 0u32.into();
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller.clone()), T::Lookup::unlookup(caller2.clone()), token)
	verify {
		let transferred_token = orml_nft::Pallet::<T>::tokens(class_id, token_id);
		assert_eq!(transferred_token.unwrap().owner, caller2);
	}

	destroy_class {
		let caller = create_account::<T>("caller", 0);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let class_metadata = big_vec.clone();
		let class_data = ClassData { is_pool:true };
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), class_metadata.clone(), class_data).unwrap_or_default();
		let class_id = 0u32.into();
	}: _(RawOrigin::Signed(caller.clone()), class_id)
	verify {
		assert_eq!(orml_nft::Pallet::<T>::classes(class_id).iter().count(), 0);
	}

	destroy_pool {
		let caller = create_account::<T>("caller", 0);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let class_metadata = big_vec.clone();
		let class_data = ClassData { is_pool:true };
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), class_metadata.clone(), class_data).unwrap_or_default();
		let class_id = 0u32.into();
	}: _(RawOrigin::Signed(caller.clone()), class_id)
	verify {
		assert_eq!(orml_nft::Pallet::<T>::classes(class_id).iter().count(), 0);
	}

	burn {
		let caller = create_account::<T>("caller", 0);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let big_emote = vec![1; T::MaxEmoteLength::get() as usize];
		let class_metadata = big_vec.clone();
		let class_data = ClassData { is_pool:true };
		let token_data = TokenData { locked:false, emote:big_emote };
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), class_metadata.clone(), class_data).unwrap_or_default();
		let class_id = 0u32.into();
		NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), class_id, class_metadata.clone(), token_data, T::MintMaxQuantity::get()).unwrap_or_default();
		let token_id = 0u32.into();
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller.clone()), token)
	verify {
		assert_eq!(orml_nft::Pallet::<T>::tokens(class_id, token_id).iter().count(), 0);
	}

	buy_from_pool {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let big_emote = vec![1; T::MaxEmoteLength::get() as usize];
		let class_metadata = big_vec.clone();
		let class_data = ClassData { is_pool:true };
		let token_data = TokenData { locked:false, emote:big_emote };
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), class_metadata.clone(), class_data).unwrap_or_default();
		let class_id = 0u32.into();
		NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), class_id, class_metadata.clone(), token_data, T::MintMaxQuantity::get()).unwrap_or_default();
		let token_id = 0u32.into();
		let token = (class_id, token_id);
	}: _(RawOrigin::Signed(caller2.clone()), token)
	verify {
		let bought_token = orml_nft::Pallet::<T>::tokens(class_id, token_id);
		assert_eq!(bought_token.unwrap().owner, caller2);
	}

	sell_to_pool {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let big_vec = vec![1; <T as orml_nft::Config>::MaxClassMetadata::get() as usize];
		let big_emote = vec![1; T::MaxEmoteLength::get() as usize];
		let class_metadata = big_vec.clone();
		let class_data = ClassData { is_pool:true };
		let token_data = TokenData { locked:false, emote:big_emote };
		NFT::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), class_metadata.clone(), class_data).unwrap_or_default();
		let class_id = 0u32.into();
		NFT::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), class_id, class_metadata.clone(), token_data, T::MintMaxQuantity::get()).unwrap_or_default();
		let token_id = 0u32.into();
		let token = (class_id, token_id);
		NFT::Pallet::<T>::transfer(RawOrigin::Signed(caller.clone()).into(), T::Lookup::unlookup(caller2.clone()), token).unwrap_or_default();
	}: _(RawOrigin::Signed(caller2.clone()), token)
	verify {
		let sold_token = orml_nft::Pallet::<T>::tokens(class_id, token_id);
		assert_eq!(sold_token.unwrap().owner, caller);
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
			assert_ok!(Pallet::<Test>::test_benchmark_buy_from_pool());
			assert_ok!(Pallet::<Test>::test_benchmark_sell_to_pool());
		});
	}
}
