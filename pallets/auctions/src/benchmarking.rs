#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks};
use sp_runtime::traits::UniqueSaturatedInto;

use crate::Pallet as Auctions;
use pallet_nft as Nft;
use frame_system::RawOrigin;
use primitives::nft::ClassType;

const SEED: u32 = 1;
const INITIAL_BALANCE: u128 = 10_000;
const UNITS: u128 = 1_000_000_000_000;

const NFT_INSTANCE_ID_1: u16 = 1;
const NFT_CLASS_ID_1: u16 = 1001;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);

	let amount: u128 = INITIAL_BALANCE.saturating_mul(UNITS);

	<T as crate::Config>::Currency::deposit_creating(&caller, amount.unique_saturated_into());

	caller
}

fn candle_auction_object<T: Config>(owner: T::AccountId) -> Auction<T>
{
	let common_data = candle_common_data(owner);
	let specific_data = candle_specific_data();

	let auction_data = CandleAuction {
		common_data,
		specific_data,
	};

	Auction::Candle(auction_data)
}

fn nft_class_id<T: Config>(id: u16) -> <T as pallet_nft::Config>::NftClassId
{
	<T as pallet_nft::Config>::NftClassId::from(id)
}

fn nft_instance_id<T: Config>(id: u16) -> <T as pallet_nft::Config>::NftInstanceId {
	<T as pallet_nft::Config>::NftInstanceId::from(id)
}

fn candle_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T>
{
	CommonAuctionData {
		name: vec![0; <T as pallet::Config>::AuctionsStringLimit::get() as usize]
			.try_into()
			.unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u32.into(),
		end: 99_366u32.into(),
		closed: false,
		owner,
		token: (
			nft_class_id::<T>(NFT_CLASS_ID_1),
			nft_instance_id::<T>(NFT_INSTANCE_ID_1),
		),
		next_bid_min: BalanceOf::<T>::from(1u32),
	}
}

fn candle_specific_data<T: Config>() -> CandleAuctionData<T>
{
	CandleAuctionData {
		closing_start: 27_366u32.into(),
		winner: None,
		winning_closing_range: None,
	}
}

fn prepare_environment<T: Config>(owner: T::AccountId) -> DispatchResult
where <T as pallet_nft::Config>::ClassType: std::convert::From<primitives::nft::ClassType>
{
	Nft::Pallet::<T>::create_class(
		RawOrigin::Signed(owner.clone()).into(),
		nft_class_id::<T>(NFT_CLASS_ID_1),
		ClassType::Marketplace.into(),
		vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize]
			.try_into()
			.unwrap()
	)?;

	Nft::Pallet::<T>::mint(
		RawOrigin::Signed(owner).into(),
		nft_class_id::<T>(NFT_CLASS_ID_1),
		nft_instance_id::<T>(NFT_INSTANCE_ID_1),
		vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize]
			.try_into()
			.unwrap()
	)?;

	Ok(())
}

benchmarks! {
	where_clause {
		where <T as pallet_nft::Config>::ClassType: std::convert::From<primitives::nft::ClassType>,
	}

	create {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);

		prepare_environment::<T>(owner.clone())?;

		let auction = candle_auction_object::<T>(owner.clone());
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
