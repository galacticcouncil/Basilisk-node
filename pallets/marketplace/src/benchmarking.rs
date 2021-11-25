#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::useless_conversion)]

use super::*;

use crate::Pallet as Marketplace;
use frame_benchmarking::{account, benchmarks, vec};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_runtime::{traits::UniqueSaturatedInto, SaturatedConversion};
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

	let amount = unit(ENDOWMENT);
	<T as pallet_nft::Config>::Currency::deposit_creating(&caller, amount.unique_saturated_into());

	caller
}

fn unit(d: u32) -> u128 {
	let d: u128 = d.into();
	d.saturating_mul(1_000_000_000_000)
}

benchmarks! {
	buy {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		let metadata = vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize];
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Marketplace, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), 0u32.into(), Some(caller.clone()), Some(20), Some(metadata), Some(123u32.into()), Some(321u32.into())).unwrap_or_default();
		Marketplace::<T>::set_price(RawOrigin::Signed(caller).into(), 0u16.into(), 0u16.into(), Some(u32::max_value().into()))?;
	}: _(RawOrigin::Signed(caller2.clone()), 0u16.into(), 0u16.into())
	verify {
		assert_eq!(pallet_uniques::Pallet::<T>::owner(T::NftClassId::from(0u32).into(), T::NftInstanceId::from(0u32).into()), Some(caller2))
	}

	set_price {
		let caller = create_account::<T>("caller", 0);
		let metadata = vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize];
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Marketplace, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), 0u32.into(), Some(caller.clone()), Some(20), Some(metadata), Some(123u32.into()), Some(321u32.into())).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into(), Some(u32::max_value().into()))
	verify {
		assert_eq!(Marketplace::<T>::prices(T::NftClassId::from(0u32), T::NftInstanceId::from(0u32)), Some(u32::max_value().into()))
	}

	make_offer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		let metadata = vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize];
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Marketplace, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), 0u32.into(), Some(caller.clone()), Some(20), Some(metadata), Some(123u32.into()), Some(321u32.into())).unwrap_or_default();
	}: _(RawOrigin::Signed(caller.clone()), 0u32.into(), 0u32.into(), unit(1_000).saturated_into(), 666u32.into())
	verify {
		assert_eq!(
			Marketplace::<T>::offers((T::NftClassId::from(0u32), T::NftInstanceId::from(0u32)), caller.clone()),
			Some( Offer {maker: caller, amount: unit(1_000).saturated_into(), expires: T::BlockNumber::from(666u32)})
		)
	}

	withdraw_offer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		let metadata = vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize];
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Marketplace, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), 0u32.into(), Some(caller), Some(20), Some(metadata), Some(123u32.into()), Some(321u32.into())).unwrap_or_default();
		Marketplace::<T>::make_offer(RawOrigin::Signed(caller2.clone()).into(), 0u32.into(), 0u32.into(), unit(1_000).saturated_into(), 666u32.into())?;
	}: _(RawOrigin::Signed(caller2.clone()), 0u32.into(), 0u32.into(), caller2.clone())
	verify {
		assert_eq!(
			Marketplace::<T>::offers((T::NftClassId::from(0u32), T::NftInstanceId::from(0u32)), caller2),
			None
		)
	}

	accept_offer {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 0);
		let metadata = vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize];
		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), ClassType::Marketplace, bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), 0u32.into(), Some(caller.clone()), Some(20), Some(metadata), Some(123u32.into()), Some(321u32.into())).unwrap_or_default();
		Marketplace::<T>::make_offer(RawOrigin::Signed(caller2.clone()).into(), 0u32.into(), 0u32.into(), unit(1_000).saturated_into(), 666u32.into())?;
	}: _(RawOrigin::Signed(caller), 0u32.into(), 0u32.into(), caller2.clone())
	verify {
		assert_eq!(
			Marketplace::<T>::offers((T::NftClassId::from(0u32), T::NftInstanceId::from(0u32)), caller2),
			None
		)
	}

}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}
