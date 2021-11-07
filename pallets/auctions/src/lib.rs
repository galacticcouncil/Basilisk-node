#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

// Used for encoding/decoding into scale
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{
		tokens::nonfungibles::Inspect, Currency, ExistenceRequirement, LockIdentifier, LockableCurrency,
		WithdrawReasons,
	},
	Parameter,
	BoundedVec,
};
use frame_system::{ensure_signed, RawOrigin};
use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, StaticLookup,
		Zero,
	},
	Permill,
};
use sp_std::{
	fmt::{Debug,Display, Formatter},
	convert::TryInto,
	result
};
pub use traits::*;
use weights::WeightInfo;
use codec::{Decode, Encode};

mod benchmarking;
pub mod weights;

pub mod traits;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Identifier for the currency lock on accounts
const AUCTION_LOCK_ID: LockIdentifier = *b"_auction";
/// Set in percent how much next bid has to be raised
const BID_STEP_PERC: u32 = 10;
/// Increase endtime to avoid sniping
const BID_ADD_BLOCKS: u32 = 10;
/// Minimal auction duration
const MIN_AUCTION_DUR: u32 = 10;

pub trait NftAuction<AccountId, BlockNumber, NftClassId, NftTokenId> {
	fn validate() -> bool;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum AuctionType {
	English,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum Auction<T: Config> {
	English(
		EnglishAuction<T>
	),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct EnglishAuction<T: Config> {
	pub common_data: CommonAuctionDataOf<T>,
	pub specific_data: EnglishAuctionData<T>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct EnglishAuctionData<T: Config> {
	pub reserve_price: BalanceOf<T>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct CommonAuctionData<AccountId, Balance, BlockNumber, NftClassId, NftTokenId, BoundedVec> {
	// TODO: Replace Vec with BoundedVec
	pub name: BoundedVec,
	pub last_bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub owner: AccountId,
	// pub auction_type: AuctionType,
	pub token: (NftClassId, NftTokenId),
	pub minimal_bid: Balance,
	// pub no_identity_allowed: bool,
	// pub starting_price: Balance,
	// pub private: bool,
	// pub max_participants: u32,
}

/// Define type aliases for better readability
pub type CommonAuctionDataOf<T> = CommonAuctionData<
	<T as frame_system::Config>::AccountId,
	BalanceOf<T>,
	<T as frame_system::Config>::BlockNumber,
	<T as pallet_uniques::Config>::ClassId,
	<T as pallet_uniques::Config>::InstanceId,
	BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>,
>;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

impl<T: Config> sp_std::fmt::Debug for Auction<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "Auction")
	}
}

// impl Display<T: Config> for AuctionType<T> {
// 	fn fmt(&self, f: &mut Formatter) -> sp_std::fmt::Result {
// 		write!(f, "{:?}", self)
// 	}
// }

// impl Default for AuctionType<T: Config> {
// 	fn default() -> Self {
// 		AuctionType::English
// 	}
// }

impl<T: Config> NftAuction<T::AccountId, T::BlockNumber, T::ClassId, T::InstanceId> for EnglishAuction<T> {
	fn validate() -> bool {
		true
	}
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The balance type for bidding
		type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// The auction ID type
		type AuctionId: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Bounded
			+ CheckedAdd;

		/// Single type currency (TODO multiple currencies)
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		/// Weights
		type WeightInfo: WeightInfo;

		// This type is needed to convert from Currency to Balance
		type CurrencyBalance: From<Self::Balance>
			+ Into<<<Self as crate::Config>::Currency as Currency<<Self as frame_system::Config>::AccountId>>::Balance>;
	}

	#[pallet::storage]
	#[pallet::getter(fn auctions)]
	/// Stores on-going and future auctions. Closed auction are removed.
	pub type Auctions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, AuctionType, Twox64Concat, T::AuctionId, Auction<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auctions_index)]
	/// Track the next auction ID.
	pub type NextAuctionId<T: Config> = StorageValue<_, T::AuctionId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auction_end_time)]
	/// Index auctions by end time.
	pub type AuctionEndTime<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::BlockNumber, Twox64Concat, T::AuctionId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auction_owner_by_id)]
	/// Auction owner by ID
	pub type AuctionOwnerById<T: Config> = StorageMap<_, Twox64Concat, T::AuctionId, T::AccountId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Auction created
		AuctionCreated(T::AccountId, T::AuctionId),
		/// A bid is placed
		Bid(T::AuctionId, T::AccountId, BalanceOf<T>),
		/// Auction ended
		AuctionConcluded(T::AuctionId),
		/// Auction removed
		AuctionRemoved(T::AuctionId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Auction does not exist
		AuctionNotExist,
		/// Auction has not started yet
		AuctionNotStarted,
		/// Auction has already started
		AuctionAlreadyStarted,
		/// Bid amount is invalid
		InvalidBidPrice,
		/// Auction count has reached it's limit
		NoAvailableAuctionId,
		/// Auction has already started
		AuctionStartTimeAlreadyPassed,
		/// Invalid auction type
		NonExistingAuctionType,
		/// Invalid auction time configuration
		InvalidTimeConfiguration,
		/// No permissions to update/delete auction
		NotAuctionOwner,
		/// No permission to handle token
		NotATokenOwner,
		/// Auction has already ended
		AuctionAlreadyConcluded,
		/// Bid overflow
		BidOverflow,
		/// Can't bid on own auction
		BidOnOwnAuction,
		/// Time underflow
		TimeUnderflow,
		/// Token is frozen from transfers
		TokenFrozen,
		/// Auction name cannot be empty
		EmptyAuctionName,
		/// BoundedVec exceeds limits
		TooLong,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_auction())]
		pub fn create_auction(origin: OriginFor<T>, auction: Auction<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			match &auction {
				Auction::English(data) => {
					Self::validate_common(&data.common_data)?;
					Self::handle_auction_create(sender, &auction, AuctionType::English, &data.common_data)?;
				}
			}

			Ok(().into())
		}

		// #[pallet::weight(<T as Config>::WeightInfo::update_auction())]
		// pub fn update_auction(origin: OriginFor<T>, id: T::AuctionId, auction_info: AuctionInfoOf<T>) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;
		// 	<Auctions<T>>::try_mutate(id, |auction| -> DispatchResult {
		// 		if let Some(auction_object) = auction {
		// 			Self::validate_update_delete(&sender, &auction_object.owner, auction_object.start)?;
		// 			*auction = Some(auction_info);
		// 			Ok(())
		// 		} else {
		// 			Err(Error::<T>::AuctionNotExist.into())
		// 		}
		// 	})
		// }

		// #[pallet::weight(<T as Config>::WeightInfo::bid_value())]
		// pub fn bid_value(origin: OriginFor<T>, id: T::AuctionId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
		// 	let sender = ensure_signed(origin)?;

		// 	Self::bid(sender.clone(), id, value)?;
		// 	Self::deposit_event(Event::Bid(id, sender, value));
		// 	Ok(().into())
		// }

		// #[pallet::weight(<T as Config>::WeightInfo::delete_auction())]
		// pub fn delete_auction(origin: OriginFor<T>, id: T::AuctionId) -> DispatchResultWithPostInfo {
		// 	let sender = ensure_signed(origin)?;
		// 	let auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionNotExist)?;
		// 	Self::validate_update_delete(&sender, &auction.owner, auction.start)?;

		// 	pallet_uniques::Pallet::<T>::thaw(
		// 		RawOrigin::Signed(auction.owner.clone()).into(),
		// 		auction.token.0,
		// 		auction.token.1,
		// 	)?;

		// 	<AuctionOwnerById<T>>::remove(id);
		// 	<Auctions<T>>::remove(id);

		// 	Self::deposit_event(Event::AuctionRemoved(id));
		// 	Ok(().into())
		// }
	}

	// #[pallet::hooks]
	// impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
	// 	fn on_finalize(now: T::BlockNumber) {
	// 		Self::conclude_auction(now).unwrap_or_default();
	// 	}
	// }
}

impl<T: Config> Pallet<T> {
	fn validate_common(common_data: &CommonAuctionDataOf<T>) -> DispatchResult {
		let current_block_number = frame_system::Pallet::<T>::block_number();
		ensure!(
			common_data.start >= current_block_number,
			Error::<T>::AuctionStartTimeAlreadyPassed
		);
		ensure!(
			common_data.start >= Zero::zero() && common_data.end > Zero::zero() && common_data.end > common_data.start + MIN_AUCTION_DUR.into(),
			Error::<T>::InvalidTimeConfiguration
		);
		ensure!(!common_data.name.is_empty(), Error::<T>::EmptyAuctionName);
		let token_owner = pallet_uniques::Pallet::<T>::owner(common_data.token.0, common_data.token.1);
		ensure!(token_owner == Some(common_data.owner.clone()), Error::<T>::NotATokenOwner);
		let is_transferrable = pallet_uniques::Pallet::<T>::can_transfer(&common_data.token.0, &common_data.token.1);
		ensure!(is_transferrable, Error::<T>::TokenFrozen);
		Ok(())
	}

	fn handle_auction_create(sender: <T>::AccountId, auction: &Auction<T>, auction_type: AuctionType, common_data: &CommonAuctionDataOf<T>) -> DispatchResult {
		let auction_id = <NextAuctionId<T>>::try_mutate(|next_id| -> result::Result<<T>::AuctionId, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id
				.checked_add(&One::one())
				.ok_or(Error::<T>::NoAvailableAuctionId)?;
			Ok(current_id)
		})?;

		<Auctions<T>>::insert(auction_type, auction_id, auction.clone());
		<AuctionOwnerById<T>>::insert(auction_id, &sender);
		<AuctionEndTime<T>>::insert(common_data.end, auction_id, ());

		pallet_uniques::Pallet::<T>::freeze(RawOrigin::Signed(sender.clone()).into(), common_data.token.0, common_data.token.1)?;

		Self::deposit_event(Event::AuctionCreated(sender, auction_id));

		Ok(())
	}

	// fn validate_update_delete(
	// 	sender: &<T>::AccountId,
	// 	auction_owner: &<T>::AccountId,
	// 	auction_start: <T>::BlockNumber
	// ) -> DispatchResult {
	// 	ensure!(auction_owner == sender, Error::<T>::NotAuctionOwner);

	// 	let current_block_number = frame_system::Pallet::<T>::block_number();
	// 	ensure!(current_block_number < auction_start, Error::<T>::AuctionAlreadyStarted);

	// 	Ok(())
	// }

	// fn conclude_auction(now: T::BlockNumber) -> DispatchResult {
	// 	for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
	// 		if let Some(auction) = Self::auctions(auction_id) {
	// 			pallet_uniques::Pallet::<T>::thaw(
	// 				RawOrigin::Signed(auction.owner.clone()).into(),
	// 				auction.token.0,
	// 				auction.token.1,
	// 			)
	// 			.unwrap_or_default();
	// 			// there is a bid so let's determine a winner and transfer tokens
	// 			if let Some(ref winner) = auction.last_bid {
	// 				let dest = T::Lookup::unlookup(winner.0.clone());
	// 				let source = T::Origin::from(frame_system::RawOrigin::Signed(auction.owner.clone()));
	// 				pallet_nft::Pallet::<T>::transfer(source, auction.token.0, auction.token.1, dest)
	// 					.unwrap_or_default();
	// 				<T as crate::Config>::Currency::remove_lock(AUCTION_LOCK_ID, &winner.0);
	// 				<<T as crate::Config>::Currency as Currency<T::AccountId>>::transfer(
	// 					&winner.0,
	// 					&auction.owner,
	// 					winner.1,
	// 					ExistenceRequirement::KeepAlive,
	// 				)
	// 				.unwrap_or_default();
	// 			}
	// 		}
	// 	}

	// 	Ok(())
	// }
}

// impl<T: Config> Auction<T::AccountId, T::BlockNumber, T::ClassId, T::InstanceId> for Pallet<T> {
// 	impl<T: Config> Pallet<T> {

// impl<T: Config> Auction
// 	type AuctionId = T::AuctionId;
// 	type Balance = BalanceOf<T>;
// 	type AccountId = T::AccountId;

// 	fn new_auction(info: AuctionInfoOf<T>) -> result::Result<<T>::AuctionId, DispatchError> {
// 		// Basic checks before an auction is created
// 		Self::check_new_auction(&info)?;
// 		let auction_id = <NextAuctionId<T>>::try_mutate(|next_id| -> result::Result<Self::AuctionId, DispatchError> {
// 			let current_id = *next_id;
// 			*next_id = next_id
// 				.checked_add(&One::one())
// 				.ok_or(Error::<T>::NoAvailableAuctionId)?;
// 			Ok(current_id)
// 		})?;

// 		<Auctions<T>>::insert(auction_id, info.clone());
// 		<AuctionOwnerById<T>>::insert(auction_id, &info.owner);
// 		<AuctionEndTime<T>>::insert(info.end, auction_id, ());

// 		pallet_uniques::Pallet::<T>::freeze(RawOrigin::Signed(info.owner.clone()).into(), info.token.0, info.token.1)?;

// 		Ok(auction_id)
// 	}

// 	fn bid(bidder: Self::AccountId, id: Self::AuctionId, value: Self::Balance) -> DispatchResult {
// 		<Auctions<T>>::try_mutate_exists(id, |auction| -> DispatchResult {
// 			// Basic checks before a bid can be made
// 			let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;
// 			let block_number = <frame_system::Pallet<T>>::block_number();
// 			ensure!(bidder != auction.owner, Error::<T>::BidOnOwnAuction);
// 			ensure!(block_number > auction.start, Error::<T>::AuctionNotStarted);
// 			ensure!(block_number < auction.end, Error::<T>::AuctionAlreadyConcluded);
// 			ensure!(value >= auction.minimal_bid, Error::<T>::InvalidBidPrice);
// 			if let Some(ref current_bid) = auction.last_bid {
// 				ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
// 				// Unlock funds from the previous bid
// 				<T as crate::Config>::Currency::remove_lock(AUCTION_LOCK_ID, &current_bid.0);
// 			} else {
// 				ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
// 			}
// 			// Lock funds
// 			<T as crate::Config>::Currency::set_lock(AUCTION_LOCK_ID, &bidder, value, WithdrawReasons::all());
// 			auction.last_bid = Some((bidder, value));
// 			// Set next minimal bid
// 			let minimal_bid_step = Permill::from_percent(BID_STEP_PERC).mul_floor(value);
// 			auction.minimal_bid = value.checked_add(&minimal_bid_step).ok_or(Error::<T>::BidOverflow)?;
// 			// Avoid auction sniping
// 			let time_left = auction
// 				.end
// 				.checked_sub(&block_number)
// 				.ok_or(Error::<T>::TimeUnderflow)?;
// 			if time_left < BID_ADD_BLOCKS.into() {
// 				auction.end = block_number + BID_ADD_BLOCKS.into();
// 			}
// 			Ok(())
// 		})
// 	}
// }
