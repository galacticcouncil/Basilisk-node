#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{ account, benchmarks };
use sp_runtime::traits::UniqueSaturatedInto;

use crate::Pallet as Auctions;
use pallet_nft as Nft;
use frame_system::RawOrigin;

const SEED: u32 = 1;
const NFT_INSTANCE_ID_1: u16 = 1;
const NFT_CLASS_ID_1: u16 = 1001;
const AUCTIONS_STRING_LIMIT: u32 = 128;

// fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
// 	let caller: T::AccountId = account(name, index, SEED);
// 	<T as crate::Config>::Currency::deposit_creating(&caller, Balance::from(1_000_000_000_000_000)).unwrap();
// 	caller
// }

const ENDOWMENT: u32 = 1_000_000;
const CLASS_ID_0: u32 = 1_000_000;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);

	let amount = dollar(ENDOWMENT);
	<T as crate::Config>::Currency::deposit_creating(&caller, amount.unique_saturated_into());

	caller
}

fn dollar(d: u32) -> u128 {
	let d: u128 = d.into();
	d.saturating_mul(100_000_000_000_000)
}

fn candle_auction_object<T: Config>(owner: T::AccountId) -> Auction<T> where <T as frame_system::Config>::BlockNumber: From<u32>, T::Balance: From<u32>,
																			 <T as pallet_uniques::Config>::ClassId: From<u16>,BalanceOf<T>: From<u32>
{
	let common_data = candle_common_data(owner);
	let specific_data = candle_specific_data();

	let auction_data = CandleAuction {
		common_data,
		specific_data
	};

	Auction::Candle(auction_data)
}

fn nft_class_id<T: Config>(id: u16) -> <T as pallet_uniques::Config>::ClassId where <T as pallet_uniques::Config>::ClassId: From<u16> {
	<T as pallet_uniques::Config>::ClassId::from(id)
}

fn nft_instance_id<T: Config>(id: u16) -> <T as pallet_uniques::Config>::InstanceId {
	<T as pallet_uniques::Config>::InstanceId::from(id)
}

fn candle_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T> where <T as frame_system::Config>::BlockNumber: From<u32>, T::Balance: From<u32>, BalanceOf<T>: From<u32>,
 <T as pallet_uniques::Config>::ClassId: From<u16>
{
	CommonAuctionData {
		name: vec![0; <T as pallet::Config>::AuctionsStringLimit::get() as usize].try_into().unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10.into(),
		end: 99_366.into(),
		closed: false,
		owner,
		token: (nft_class_id::<T>(NFT_CLASS_ID_1), nft_instance_id::<T>(NFT_INSTANCE_ID_1)),
		next_bid_min: BalanceOf::<T>::from(1u32),
	}
}

fn candle_specific_data<T: Config>() -> CandleAuctionData<T>
where <T as frame_system::Config>::BlockNumber: From<u32>,
{
	CandleAuctionData {
		closing_start: 27_366.into(),
		winner: None,
		winning_closing_range: None
	}
}

// fn to_bounded_name<T: Config>(name: Vec<u8>) -> Result<BoundedVec<u8, AUCTIONS_STRING_LIMIT>, Error<T>> {
//   name.try_into().map_err(|_| Error::<T>::TooLong)
// }

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

benchmarks! {
	where_clause {
		where <T as frame_system::Config>::BlockNumber: From<u32>,
		T::Balance: From<u32>,
		<T as pallet_uniques::Config>::ClassId: From<u16>,
		<<T as pallet::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance: From<u32>
	}

	create {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);

		/*
		Nft::Pallet::<T>::create_class(
      RawOrigin::Signed(owner.clone()).into(),
      nft_class_id(NFT_CLASS_ID_1),
      T::ClassType::Marketplace,
      bvec![0]
    );

    Nft::Pallet::<T>::mint(RawOrigin::Signed(owner.clone()).into(), nft_class_id::<T>(NFT_CLASS_ID_1), nft_instance_id::<T>(NFT_INSTANCE_ID_1), bvec![0]);
		 */

		let auction = candle_auction_object(owner.clone());
	}: _(RawOrigin::Signed(owner.clone()), auction)
	verify {
		//assert_eq!(Auctions::<T>::auction_owner_by_id(0), Some(owner));
	}

	update {
	}: {
	} verify {
	}

	destroy {
	}: {
	} verify {
	}

	bid {
	}: {
	} verify {
	}
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}
