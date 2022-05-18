#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use sp_runtime::traits::UniqueSaturatedInto;

use crate::Pallet as Auctions;

// Contains mock of objects shared between tests and benchmarking
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

		let auction = mocked_english_auction_object::<T>(mocked_english_common_data::<T>(owner.clone()), mocked_english_specific_data::<T>());
	}: { Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), Some(owner));
	}

	update_english {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_english_auction_object::<T>(mocked_english_common_data::<T>(owner.clone()), mocked_english_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;

		let mut updated_common_data = mocked_english_common_data::<T>(owner.clone());
		updated_common_data.start = 11u32.into();
		updated_common_data.end = 99_367u32.into();


		let updated_auction = mocked_english_auction_object::<T>(updated_common_data, mocked_english_specific_data::<T>());
	}: { Auctions::<T>::update(RawOrigin::Signed(owner.clone()).into(), 0.into(), updated_auction.clone())?; }
	verify {
		assert_eq!(Auctions::<T>::auctions(T::AuctionId::from(0u8)).unwrap(), updated_auction);
	}

	destroy_english {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_english_auction_object::<T>(mocked_english_common_data::<T>(owner.clone()), mocked_english_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;
	}: { Auctions::<T>::destroy(RawOrigin::Signed(owner.clone()).into(), 0.into())?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), None);
	}

  bid_english {
		let owner = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_english_auction_object::<T>(mocked_english_common_data::<T>(owner.clone()), mocked_english_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner).into(), auction)?;

		frame_system::Pallet::<T>::set_block_number(18u32.into());

		let bidder_1 = create_account::<T>("bidder_1", 1);
		let bid_amount = 5u128.saturating_mul(UNITS);

    Auctions::<T>::bid(RawOrigin::Signed(bidder_1).into(), 0.into(), bid_amount.into())?;

		frame_system::Pallet::<T>::set_block_number(20u32.into());

    let bidder_2 = create_account::<T>("bidder_2", 2);
		let bid_amount = 10u128.saturating_mul(UNITS);
	}: { Auctions::<T>::bid(RawOrigin::Signed(bidder_2.clone()).into(), 0.into(), bid_amount.into())?; }
	verify {
    let auction = Auctions::<T>::auctions(T::AuctionId::from(0u8)).unwrap();

    let auction_check = match auction {
			Auction::English(data) => {
				assert_eq!(data.common_data.last_bid, Some((bidder_2, bid_amount.into())));

				Ok(())
			}
			_ => Err(()),
		};

		assert_eq!(auction_check, Ok(()));
	}

  // Candle Auction benchmarks
  create_candle {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_candle_auction_object::<T>(mocked_candle_common_data::<T>(owner.clone()), mocked_candle_specific_data::<T>());
	}: { Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), Some(owner));
	}

	update_candle {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_candle_auction_object::<T>(mocked_candle_common_data::<T>(owner.clone()), mocked_candle_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;

		let mut updated_common_data = mocked_candle_common_data::<T>(owner.clone());
		updated_common_data.start = 11u32.into();
		updated_common_data.end = 99_367u32.into();

		let mut updated_specific_data = mocked_candle_specific_data::<T>();
		updated_specific_data.closing_start = 27_367u32.into();

		let updated_auction = mocked_candle_auction_object::<T>(updated_common_data, updated_specific_data);
	}: { Auctions::<T>::update(RawOrigin::Signed(owner.clone()).into(), 0.into(), updated_auction.clone())?; }
	verify {
		assert_eq!(Auctions::<T>::auctions(T::AuctionId::from(0u8)).unwrap(), updated_auction);
	}

	destroy_candle {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_candle_auction_object::<T>(mocked_candle_common_data::<T>(owner.clone()), mocked_candle_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;
	}: { Auctions::<T>::destroy(RawOrigin::Signed(owner.clone()).into(), 0.into())?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), None);
	}

	bid_candle {
		let owner = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_candle_auction_object::<T>(mocked_candle_common_data::<T>(owner.clone()), mocked_candle_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner).into(), auction)?;

		frame_system::Pallet::<T>::set_block_number(99_365u32.into());

		let bidder = create_account::<T>("bidder", 1);
		let bid_amount = 5u128.saturating_mul(UNITS);
	}: { Auctions::<T>::bid(RawOrigin::Signed(bidder.clone()).into(), 0.into(), bid_amount.into())?; }
	verify {
		assert_eq!(
			Auctions::<T>::highest_bidders_by_auction_closing_range(T::AuctionId::from(0u8), 99u32).unwrap(),
			bidder
		);
	}

  // TopUp Auction benchmarks
  create_topup {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_topup_auction_object::<T>(mocked_topup_common_data::<T>(owner.clone()), mocked_topup_specific_data::<T>());
	}: { Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), Some(owner));
	}

	update_topup {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_topup_auction_object::<T>(mocked_topup_common_data::<T>(owner.clone()), mocked_topup_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;

		let mut updated_common_data = mocked_topup_common_data::<T>(owner.clone());
		updated_common_data.start = 11u32.into();
		updated_common_data.end = 99_367u32.into();


		let updated_auction = mocked_topup_auction_object::<T>(updated_common_data, mocked_topup_specific_data::<T>());
	}: { Auctions::<T>::update(RawOrigin::Signed(owner.clone()).into(), 0.into(), updated_auction.clone())?; }
	verify {
		assert_eq!(Auctions::<T>::auctions(T::AuctionId::from(0u8)).unwrap(), updated_auction);
	}

	destroy_topup {
		let owner: T::AccountId = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_topup_auction_object::<T>(mocked_topup_common_data::<T>(owner.clone()), mocked_topup_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner.clone()).into(), auction)?;
	}: { Auctions::<T>::destroy(RawOrigin::Signed(owner.clone()).into(), 0.into())?; }
	verify {
		assert_eq!(Auctions::<T>::auction_owner_by_id(T::AuctionId::from(0u8)), None);
	}

  bid_topup {
		let owner = create_account::<T>("auction_owner", 0);
		prepare_environment::<T>(owner.clone())?;

		let auction = mocked_english_auction_object::<T>(mocked_english_common_data::<T>(owner.clone()), mocked_english_specific_data::<T>());
		Auctions::<T>::create(RawOrigin::Signed(owner).into(), auction)?;

		frame_system::Pallet::<T>::set_block_number(20u32.into());

    let bidder = create_account::<T>("bidder", 1);
		let bid_amount = 5u128.saturating_mul(UNITS);
	}: { Auctions::<T>::bid(RawOrigin::Signed(bidder.clone()).into(), 0.into(), bid_amount.into())?; }
	verify {
		let auction = Auctions::<T>::auctions(T::AuctionId::from(0u8)).unwrap();

    let auction_check = match auction {
			Auction::English(data) => {
				assert_eq!(data.common_data.last_bid, Some((bidder, bid_amount.into())));

				Ok(())
			}
			_ => Err(()),
		};

		assert_eq!(auction_check, Ok(()));
	}
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}
