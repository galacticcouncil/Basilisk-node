#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use sp_runtime::traits::UniqueSaturatedInto;


use crate::Pallet as Auctions;

use crate::mocked_auction_objects::*;

use frame_system::RawOrigin;
use pallet_nft as Nft;
use primitives::nft::ClassType;

const SEED: u32 = 1;
const INITIAL_BALANCE: u128 = 10_000;
const UNITS: u128 = 1_000_000_000_000;

//
// Helper functions
//
fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	let amount: u128 = INITIAL_BALANCE.saturating_mul(UNITS);

	<T as crate::Config>::Currency::deposit_creating(&caller, amount.unique_saturated_into());

	caller
}

fn prepare_environment<T: Config>(owner: T::AccountId) -> DispatchResult
where
	<T as pallet_nft::Config>::ClassType: sp_std::convert::From<primitives::nft::ClassType>,
{
	Nft::Pallet::<T>::create_class(
		RawOrigin::Signed(owner.clone()).into(),
		mocked_nft_class_id_1::<T>(),
		ClassType::Marketplace.into(),
		sp_std::vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize]
			.try_into()
			.unwrap(),
	)?;

	Nft::Pallet::<T>::mint(
		RawOrigin::Signed(owner).into(),
		mocked_nft_class_id_1::<T>(),
		mocked_nft_instance_id_1::<T>(),
		sp_std::vec![0; <T as pallet_uniques::Config>::StringLimit::get() as usize]
			.try_into()
			.unwrap(),
	)?;

	Ok(())
}

// English Auction object
fn english_auction_object<T: Config>(
	common_data: CommonAuctionData<T>,
	specific_data: EnglishAuctionData,
) -> Auction<T> {
	let auction_data = EnglishAuction {
		common_data,
		specific_data,
	};

	Auction::English(auction_data)
}

fn english_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T> {
	CommonAuctionData {
		name: sp_std::vec![0; <T as pallet::Config>::AuctionsStringLimit::get() as usize]
			.try_into()
			.unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u32.into(),
		end: 99_366u32.into(),
		closed: false,
		owner,
		token: (
			mocked_nft_class_id_1::<T>(),
			mocked_nft_instance_id_1::<T>(),
		),
		next_bid_min: BalanceOf::<T>::from(1u32),
	}
}

fn english_specific_data<T: Config>() -> EnglishAuctionData {
	EnglishAuctionData {}
}

// Candle Auction object
// fn mocked_candle_auction_object<T: Config>(
// 	common_data: CommonAuctionData<T>,
// 	specific_data: CandleAuctionData<T>,
// ) -> Auction<T> {
// 	let auction_data = CandleAuction {
// 		common_data,
// 		specific_data,
// 	};

// 	Auction::Candle(auction_data)
// }

// fn mocked_candle_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T> {
// 	CommonAuctionData {
// 		name: sp_std::vec![0; <T as pallet::Config>::AuctionsStringLimit::get() as usize]
// 			.try_into()
// 			.unwrap(),
// 		reserve_price: None,
// 		last_bid: None,
// 		start: 10u32.into(),
// 		end: 99_366u32.into(),
// 		closed: false,
// 		owner,
// 		token: (
// 			nft_class_id::<T>(NFT_CLASS_ID_1),
// 			nft_instance_id::<T>(NFT_INSTANCE_ID_1),
// 		),
// 		next_bid_min: BalanceOf::<T>::from(1u32),
// 	}
// }

// fn candle_specific_data<T: Config>() -> CandleAuctionData<T> {
// 	CandleAuctionData {
// 		closing_start: 27_366u32.into(),
// 		winner: None,
// 		winning_closing_range: None,
// 	}
// }

// TopUp Auction Object
fn topup_auction_object<T: Config>(common_data: CommonAuctionData<T>, specific_data: TopUpAuctionData) -> Auction<T> {
	let auction_data = TopUpAuction {
		common_data,
		specific_data,
	};

	Auction::TopUp(auction_data)
}

fn topup_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T> {
	CommonAuctionData {
		name: sp_std::vec![0; <T as pallet::Config>::AuctionsStringLimit::get() as usize]
			.try_into()
			.unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u32.into(),
		end: 99_366u32.into(),
		closed: false,
		owner,
		token: (
			mocked_nft_class_id_1::<T>(),
			mocked_nft_instance_id_1::<T>(),
		),
		next_bid_min: BalanceOf::<T>::from(1u32),
	}
}

fn topup_specific_data<T: Config>() -> TopUpAuctionData {
	TopUpAuctionData {}
}

// Pallet benchmarks
benchmarks! {
	where_clause {
		where
			<T as pallet_nft::Config>::ClassType: sp_std::convert::From<primitives::nft::ClassType>,
			T::AuctionId: sp_std::convert::From<u8>,
			<<T as pallet::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance: From<u128>
	}

  // English Auction benchmarks
  create_english {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = english_auction_object::<T>(english_common_data::<T>(owner.clone()), english_specific_data::<T>());
	}: { Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), Some(owner));
	}

	update_english {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = english_auction_object::<T>(english_common_data::<T>(owner.clone()), english_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;

		let mut updated_common_data = english_common_data::<T>(owner.clone());
		updated_common_data.start = 11u32.into();
		updated_common_data.end = 99_367u32.into();


		let updated_auction = english_auction_object::<T>(updated_common_data, english_specific_data::<T>());
	}: { Auctions::<T>::update(RawOrigin::Signed(owner.clone()).into(), 0.into(), updated_auction.clone())?; }
	verify {
		assert_eq!(Auctions::<T>::auctions(T::AuctionId::from(0u8)).unwrap(), updated_auction);
	}

	destroy_english {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = english_auction_object::<T>(english_common_data::<T>(owner.clone()), english_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;
	}: { Auctions::<T>::destroy(RawOrigin::Signed(owner.clone()).into(), 0.into())?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), None);
	}

  // Candle Auction benchmarks
  create_candle {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_candle_auction_object::<T>(mocked_candle_common_data::<T>(owner.clone()), candle_specific_data::<T>());
	}: { Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), Some(owner));
	}

	update_candle {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_candle_auction_object::<T>(mocked_candle_common_data::<T>(owner.clone()), candle_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;

		let mut updated_common_data = mocked_candle_common_data::<T>(owner.clone());
		updated_common_data.start = 11u32.into();
		updated_common_data.end = 99_367u32.into();

		let mut updated_specific_data = candle_specific_data::<T>();
		updated_specific_data.closing_start = 27_367u32.into();

		let updated_auction = mocked_candle_auction_object::<T>(updated_common_data, updated_specific_data);
	}: { Auctions::<T>::update(RawOrigin::Signed(owner.clone()).into(), 0.into(), updated_auction.clone())?; }
	verify {
		assert_eq!(Auctions::<T>::auctions(T::AuctionId::from(0u8)).unwrap(), updated_auction);
	}

	destroy_candle {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_candle_auction_object::<T>(mocked_candle_common_data::<T>(owner.clone()), candle_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;
	}: { Auctions::<T>::destroy(RawOrigin::Signed(owner.clone()).into(), 0.into())?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), None);
	}

	bid_candle {
		let owner = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_candle_auction_object::<T>(mocked_candle_common_data::<T>(owner.clone()), candle_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;

		frame_system::Pallet::<T>::set_block_number(10u32.into());

		let bidder = create_account::<T>("bidder", 1);
		let bid_amount = 5u128.saturating_mul(UNITS.into());
	}: { Auctions::<T>::bid(RawOrigin::Signed(bidder.clone()).into(), 0.into(), bid_amount.into())?; }
	verify {
		assert_eq!(
			Auctions::<T>::highest_bidders_by_auction_closing_range(T::AuctionId::from(0u8), 1u32).unwrap(),
			bidder
		);
	}

  // TopUp Auction benchmarks
  create_topup {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = topup_auction_object::<T>(topup_common_data::<T>(owner.clone()), topup_specific_data::<T>());
	}: { Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), Some(owner));
	}

	update_topup {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = topup_auction_object::<T>(topup_common_data::<T>(owner.clone()), topup_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;

		let mut updated_common_data = topup_common_data::<T>(owner.clone());
		updated_common_data.start = 11u32.into();
		updated_common_data.end = 99_367u32.into();


		let updated_auction = topup_auction_object::<T>(updated_common_data, topup_specific_data::<T>());
	}: { Auctions::<T>::update(RawOrigin::Signed(owner.clone()).into(), 0.into(), updated_auction.clone())?; }
	verify {
		assert_eq!(Auctions::<T>::auctions(T::AuctionId::from(0u8)).unwrap(), updated_auction);
	}

	destroy_topup {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = topup_auction_object::<T>(topup_common_data::<T>(owner.clone()), topup_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;
	}: { Auctions::<T>::destroy(RawOrigin::Signed(owner.clone()).into(), 0.into())?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), None);
	}
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}
