use super::*;

pub const MOCKED_NFT_CLASS_ID_1: u16 = 1;
pub const MOCKED_NFT_INSTANCE_ID_1: u32 = 1_000_000;

// NFT mocks
pub fn mocked_nft_class_id_1<T: Config>() -> <T as pallet_nft::Config>::NftClassId {
	<T as pallet_nft::Config>::NftClassId::from(1_000_000u32)
}

pub fn mocked_nft_instance_id_1<T: Config>() -> <T as pallet_nft::Config>::NftInstanceId {
	<T as pallet_nft::Config>::NftInstanceId::from(1u16)
}


pub fn mocked_nft_token<T: Config>() -> (<T as pallet_nft::Config>::NftClassId, <T as pallet_nft::Config>::NftInstanceId) {
	(mocked_nft_class_id_1::<T>(), mocked_nft_instance_id_1::<T>())
}

// Candle Auction object mocks
pub fn mocked_candle_auction_object<T: Config>(
	common_data: CommonAuctionData<T>,
	specific_data: CandleAuctionData<T>,
) -> Auction<T> {
	let auction_data = CandleAuction {
		common_data,
		specific_data,
	};

	Auction::Candle(auction_data)
}

pub fn mocked_candle_common_data<T: Config>(owner: T::AccountId) -> CommonAuctionData<T> {
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
		token: mocked_nft_token::<T>(),
		next_bid_min: BalanceOf::<T>::from(1u32),
	}
}

pub fn candle_specific_data<T: Config>() -> CandleAuctionData<T> {
	CandleAuctionData {
		closing_start: 27_366u32.into(),
		winner: None,
		winning_closing_range: None,
	}
}

