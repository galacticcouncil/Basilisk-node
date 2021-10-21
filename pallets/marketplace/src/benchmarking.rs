#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::useless_conversion)]

use super::*;

use crate::Pallet as Marketplace;
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
	buy {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), bvec![0])?;
		Marketplace::<T>::list(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), 0u32.into(), caller.clone(), 20)?;
		Marketplace::<T>::set_price(RawOrigin::Signed(caller.clone()).into(), 0u16.into(), 0u16.into(), Some(u32::max_value().into()))?;
	}: _(RawOrigin::Signed(caller2.clone()), 0u16.into(), 0u16.into())
	verify {
		assert_eq!(pallet_uniques::Pallet::<T>::owner(T::ClassId::from(0u32), T::InstanceId::from(0u32)), Some(caller2))
	}

	set_price {
		let caller = create_account::<T>("caller", 0);
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), bvec![0])?;
		Marketplace::<T>::list(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), 0u32.into(), caller.clone(), 20)?;
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into(), Some(u32::max_value().into()))
	verify {
		assert_eq!(Marketplace::<T>::tokens(T::ClassId::from(0u32), T::InstanceId::from(0u32)).unwrap().price, Some(u32::max_value().into()))
	}

	list {
		let caller = create_account::<T>("caller", 0);
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), bvec![0])?;
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into(), caller.clone(), 20)
	verify {
		assert_eq!(
			Marketplace::<T>::tokens(T::ClassId::from(0u32), T::InstanceId::from(0u32)),
			Some(TokenInfo {author: caller, royalty: 20, price: None, offer: None})
		)
	}

	unlist {
		let caller = create_account::<T>("caller", 0);
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), bvec![0])?;
		Marketplace::<T>::list(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), 0u32.into(), caller.clone(), 20)?;
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into())
	verify {
		assert_eq!(
			Marketplace::<T>::tokens(T::ClassId::from(0u32), T::InstanceId::from(0u32)),
			None
		)
	}

	make_offer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), bvec![0])?;
		Marketplace::<T>::list(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), 0u32.into(), caller.clone(), 20)?;
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into(), 1000u32.into(), 666u32.into())
	verify {
		assert_eq!(
			Marketplace::<T>::tokens(T::ClassId::from(0u32), T::InstanceId::from(0u32)),
			Some(TokenInfo {author: caller.clone(), royalty: 20, price: None, offer: Some((caller.clone(), 1000u32.into(), T::BlockNumber::from(666u32)))})
		)
	}

	withdraw_offer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), bvec![0])?;
		Marketplace::<T>::list(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), 0u32.into(), caller.clone(), 20)?;
		Marketplace::<T>::make_offer(RawOrigin::Signed(caller2.clone()).into(), 0u32.into(), 0u32.into(), 1000u32.into(), 666u32.into())?;
	}: _(RawOrigin::Signed(caller2.clone()), 0u32.into(), 0u32.into())
	verify {
		assert_eq!(
			Marketplace::<T>::tokens(T::ClassId::from(0u32), T::InstanceId::from(0u32)),
			Some(TokenInfo {author: caller.clone(), royalty: 20, price: None, offer: None})
		)
	}

	accept_offer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Art, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), bvec![0])?;
		Marketplace::<T>::list(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), 0u32.into(), caller.clone(), 20)?;
		Marketplace::<T>::make_offer(RawOrigin::Signed(caller2.clone()).into(), 0u32.into(), 0u32.into(), 1000u32.into(), 666u32.into())?;
	}: _(RawOrigin::Signed(caller2.clone()), 0u32.into(), 0u32.into())
	verify {
		assert_eq!(
			Marketplace::<T>::tokens(T::ClassId::from(0u32), T::InstanceId::from(0u32)),
			Some(TokenInfo {author: caller.clone(), royalty: 20, price: None, offer: None})
		)
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
			assert_ok!(Pallet::<Test>::test_benchmark_list());
			assert_ok!(Pallet::<Test>::test_benchmark_unlist());
			assert_ok!(Pallet::<Test>::test_benchmark_make_offer());
			assert_ok!(Pallet::<Test>::test_benchmark_withdraw_offer());
			assert_ok!(Pallet::<Test>::test_benchmark_accept_offer());
			assert_ok!(Pallet::<Test>::test_benchmark_set_price());
		});
	}
}
