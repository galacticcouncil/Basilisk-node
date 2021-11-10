#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet as AUCTIONS;
use sp_std::vec;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const SEED: u32 = 0;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	caller
}

// TODO: don't use unwrap_or_default

benchmarks! {
	create_auction {
		let caller = create_account::<T>("caller", 0);

		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), T::Lookup::unlookup(caller.clone()), bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), Default::default(), 0u16.into(), T::Lookup::unlookup(caller.clone()), 10u8, bvec![0])?;

		let auction_info = AuctionInfo {
			// TODO: replace with worst case
			name: "Aukce1".as_bytes().to_vec(),
			last_bid: None,
			start: T::BlockNumber::from(1u32),
			end: T::BlockNumber::from(20u32),
			owner: caller.clone(),
			auction_type: AuctionType::English,
			token: (Default::default(), 0u16.into()),
			minimal_bid: T::CurrencyBalance::from(T::Balance::from(50u32)).into(),
		};
	}: _(RawOrigin::Signed(caller.clone()), auction_info)
	verify {
	}

	bid_value {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);

		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), T::Lookup::unlookup(caller.clone()), bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), Default::default(), 0u16.into(), T::Lookup::unlookup(caller.clone()), 10u8, bvec![0])?;

		let auction_info = AuctionInfo {
			// TODO: replace with worst case
			name: "Aukce1".as_bytes().to_vec(),
			last_bid: None,
			start: T::BlockNumber::from(0u32),
			end: T::BlockNumber::from(20u32),
			owner: caller,
			auction_type: AuctionType::English,
			token: (Default::default(), 0u16.into()),
			minimal_bid: T::CurrencyBalance::from(T::Balance::from(50u32)).into(),
		};
		let auction_id = AUCTIONS::<T>::new_auction(auction_info).unwrap_or_default();

	}: _(RawOrigin::Signed(caller2.clone()), auction_id, 1_000_000_u32.into())
	verify {
	}

	destroy_auction {
		let caller = create_account::<T>("caller", 0);

		pallet_nft::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), Default::default(), T::Lookup::unlookup(caller.clone()), bvec![0])?;
		pallet_nft::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), Default::default(), 0u16.into(), T::Lookup::unlookup(caller.clone()), 10u8, bvec![0])?;

		let auction_info = AuctionInfo {
			// TODO: replace with worst case
			name: "Aukce1".as_bytes().to_vec(),
			last_bid: None,
			start: T::BlockNumber::from(20u32),
			end: T::BlockNumber::from(50u32),
			owner: caller.clone(),
			auction_type: AuctionType::English,
			token: (Default::default(), 0u16.into()),
			minimal_bid: T::CurrencyBalance::from(T::Balance::from(50u32)).into(),
		};
		let auction_id = AUCTIONS::<T>::new_auction(auction_info).unwrap_or_default();

	}: _(RawOrigin::Signed(caller.clone()), auction_id)
	verify {
	}
}

#[cfg(test)]
mod tests {
	use super::mock::Test;
	use crate::tests::new_test_ext;
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_create_auction::<Test>());
			assert_ok!(test_benchmark_bid_value::<Test>());
			assert_ok!(test_benchmark_destroy_auction::<Test>());
		});
	}
}
