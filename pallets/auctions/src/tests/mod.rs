use super::*;
use crate::mock::*;
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
      NFT_CLASS_ID_1,
      ClassType::Marketplace,
      bvec![0]
    ));
    assert_ok!(Nft::mint(Origin::signed(ALICE), NFT_CLASS_ID_1, NFT_INSTANCE_ID_1, bvec![0]));
  });

  ext
}

fn to_bounded_name(name: Vec<u8>) -> Result<BoundedVec<u8, AuctionsStringLimit>, Error<Test>> {
  name.try_into().map_err(|_| Error::<Test>::TooLong)
}

fn valid_general_auction_data() -> GeneralAuctionData<Test> {
	GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u64,
		end: 21u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, NFT_INSTANCE_ID_1),
		next_bid_min: 1,
	}
}

/// English auction tests
fn english_auction_object(general_data: GeneralAuctionData<Test>, specific_data: EnglishAuctionData) -> Auction<Test> {
	let auction_data = EnglishAuction {
		general_data,
		specific_data,
	};

	Auction::English(auction_data)
}

fn valid_english_specific_data() -> EnglishAuctionData {
	EnglishAuctionData {}
}

/// TopUp auction tests
fn topup_auction_object(general_data: GeneralAuctionData<Test>, specific_data: TopUpAuctionData) -> Auction<Test> {
	let auction_data = TopUpAuction {
		general_data,
		specific_data,
	};

	Auction::TopUp(auction_data)
}

fn valid_topup_specific_data() -> TopUpAuctionData {
	TopUpAuctionData {}
}

fn bid_object(amount: BalanceOf<Test>, block_number: <Test as frame_system::Config>::BlockNumber) -> Bid<Test> {
	Bid { amount, block_number }
}

fn get_auction_subaccount_id(auction_id: <Test as pallet::Config>::AuctionId) -> AccountId32 {
	<Test as pallet::Config>::PalletId::get().into_sub_account(("ac", auction_id))
}

/// Candle auction tests
fn candle_auction_object(general_data: GeneralAuctionData<Test>, specific_data: CandleAuctionData<Test>) -> Auction<Test> {
	let auction_data = CandleAuction {
		general_data,
		specific_data,
	};

	Auction::Candle(auction_data)
}

fn valid_candle_general_auction_data() -> GeneralAuctionData<Test> {
	GeneralAuctionData {
		name: to_bounded_name(b"Auction 0".to_vec()).unwrap(),
		reserve_price: None,
		last_bid: None,
		start: 10u64,
		end: 99_366u64,
		closed: false,
		owner: ALICE,
		token: (NFT_CLASS_ID_1, NFT_INSTANCE_ID_1),
		next_bid_min: 1,
	}
}

fn valid_candle_specific_data() -> CandleAuctionData<Test> {
	CandleAuctionData {
		closing_start: 27_366,
		winner: None,
		winning_closing_range: None
	}
}
