use super::*;

use crate::mock::*;
use crate::mocked_objects::*;

use frame_support::{assert_ok, BoundedVec};
use primitives::nft::ClassType;
use sp_core::crypto::AccountId32;
use sp_std::convert::TryInto;

pub type AuctionsModule = Pallet<Test>;

#[cfg(test)]
mod english;

#[cfg(test)]
mod topup;

#[cfg(test)]
mod candle;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn last_event() -> crate::mock::Event {
	frame_system::Pallet::<Test>::events()
		.pop()
		.expect("An event expected")
		.event
}

pub fn expect_event<E: Into<TestEvent>>(e: E) {
	assert_eq!(last_event(), e.into());
}

pub fn set_block_number<T: frame_system::Config<BlockNumber = u64>>(n: u64) {
	frame_system::Pallet::<T>::set_block_number(n);
}

pub fn to_bounded_name(name: Vec<u8>) -> Result<BoundedVec<u8, AuctionsStringLimit>, Error<Test>> {
	name.try_into().map_err(|_| Error::<Test>::TooLong)
}

pub fn bid_object(amount: BalanceOf<Test>, block_number: <Test as frame_system::Config>::BlockNumber) -> Bid<Test> {
	Bid { amount, block_number }
}

pub fn get_auction_subaccount_id(auction_id: <Test as pallet::Config>::AuctionId) -> AccountId32 {
	<Test as pallet::Config>::PalletId::get().into_sub_account(("ac", auction_id))
}

fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| set_block_number::<Test>(1));
	ext
}

fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_ok!(Nft::create_class(
			Origin::signed(ALICE),
			mocked_nft_class_id_1::<Test>(),
			ClassType::Marketplace,
			bvec![0]
		));
		assert_ok!(Nft::mint(
			Origin::signed(ALICE),
			mocked_nft_class_id_1::<Test>(),
			mocked_nft_instance_id_1::<Test>(),
			bvec![0]
		));
	});

	ext
}
