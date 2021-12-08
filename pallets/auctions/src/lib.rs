// This file is part of Basilisk-node.

// Built with <3 for decentralisation and the kind support of Web3 Foundation Grants Program:
// https://github.com/w3f/Grants-Program/blob/master/applications/subauction.md

// Copyright (C) 2021-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//!
//! # Auctions Pallet
//!
//! ## Overview
//!
//! The Auctions pallet provides extendable auction functionality for NFTs.
//!
//! The pallet implements an NftAuction trait which allows users to extend the pallet by implementing other
//! auction types. All auction types must implement bid() and close() functions at their interface.
//! 
//! The auction types share the same store called Auctions. Auction types are represented in a struct which holds
//! two other structs with general_data (eg auction name, start, end) and specific_data for the given auction type.
//! Besides Auctions, there are are two other stores: NextAuctionId and AuctionOwnerById.
//! 
//! ## Dispatchable Functions
//! - `create` - create an auction
//! 
//! - `update` - update an auction
//! 
//! - `destroy` - destroy an auction
//! 
//! - `bid` - place a bid on an auctio
//! 
//! - `close` - close an auction after the end time has lapsed. Not done in a hook for better chain performance.
//! 
//! ## Implemented Auction types
//! 
//! ### EnglishAuction
//! 
//! In an English auction, participants place bids in a running auction. Once the auction has reached its end time,
//! the highest bid wins.
//! 
//! The implementation of English auction allows sellers to set a starting price for the object, under which it will not
//! be sold (auction.general_data.next_bid_min).
//! 
//! It also extens the end time of the auction for any last-minute bids in order to prevent auction sniping.
//! 

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

// Used for encoding/decoding into scale
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{
		tokens::nonfungibles::Inspect, Currency, ExistenceRequirement, Get, LockIdentifier, LockableCurrency,
		WithdrawReasons,
	},
	Parameter,
};
use frame_system::{ensure_signed, RawOrigin};
use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, StaticLookup,
		Zero,
	},
	Permill,
};
use sp_std::result;
pub use traits::*;
use weights::WeightInfo;
use scale_info::TypeInfo;

mod benchmarking;
pub mod traits;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Identifier for the currency lock on accounts
const AUCTION_LOCK_ID: LockIdentifier = *b"_auction";

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config + TypeInfo {
		/// Event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Balance type (used for bidding)
		type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// AuctionID type
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

		/// Limit of auction name length
		#[pallet::constant]
		type AuctionsStringLimit: Get<u32>;

		/// Increase end time to avoid sniping
		#[pallet::constant]
		type BidAddBlocks: Get<u32>;

		/// Next bid step in percent
		#[pallet::constant]
		type BidStepPerc: Get<u32>;

		/// Minimum auction duration
		#[pallet::constant]
		type MinAuctionDuration: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn auctions)]
	/// Stores on-going and future auctions (closed auctions will be destroyed)
	pub(crate) type Auctions<T: Config> = StorageMap<_, Twox64Concat, T::AuctionId, Auction<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auctions_index)]
	/// Stores the next auction ID
	pub(crate) type NextAuctionId<T: Config> = StorageValue<_, T::AuctionId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auction_owner_by_id)]
	/// Stores auction owner by ID
	pub(crate) type AuctionOwnerById<T: Config> = StorageMap<_, Twox64Concat, T::AuctionId, T::AccountId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	/// Auction events
	pub enum Event<T: Config> {
		/// An auction is created
		AuctionCreated(T::AccountId, T::AuctionId),
		/// A bid is placed
		BidPlaced(T::AuctionId, T::AccountId, BalanceOf<T>),
		/// An auction has closed
		AuctionClosed(T::AuctionId),
		/// An auction was destroyed
		AuctionDestroyed(T::AuctionId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Auction does not exist
		AuctionNotExist,
		/// Auction has not started yet
		AuctionNotStarted,
		/// Auction has already started
		AuctionAlreadyStarted,
		/// Auction is already closed (auction.general_data.closed is true)
		AuctionClosed,
		/// Auction has reached its ending time (auction.general_data.end is in the past)
		AuctionEndTimeReached,
		/// Auction end time has not been reached (auction.general_data.end is in the future)
		AuctionEndTimeNotReached,
		/// Auction.general_data.closed can only be set via close() extrinsic
		CannotSetAuctionClosed,
		/// Bid amount is invalid
		InvalidBidPrice,
		/// Auction count has reached its limit
		NoAvailableAuctionId,
		/// Auction has already started
		AuctionStartTimeAlreadyPassed,
		/// Invalid auction time configuration
		InvalidTimeConfiguration,
		/// No permissions to update/destroy auction
		NotAuctionOwner,
		/// No permission to handle token
		NotATokenOwner,
		/// Bid overflow
		BidOverflow,
		/// Cannot bid on own auction
		CannotBidOnOwnAuction,
		/// Time underflow
		TimeUnderflow,
		/// Token is frozen from transfers
		TokenFrozen,
		/// Auction name cannot be empty
		EmptyAuctionName,
		/// BoundedVec exceeds limits
		TooLong,
		/// Auction type cannot be changed
		NoChangeOfAuctionType,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 
		/// Creates a new auction for a given Auction type
		/// 
		/// - validates auction.general_data
		/// - validates logic specific to create action
		/// - creates auction
		/// - deposits AuctionCreated event
		/// 
		#[pallet::weight(<T as Config>::WeightInfo::create_auction())]
		pub fn create(origin: OriginFor<T>, auction: Auction<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			match &auction {
				Auction::English(auction_object) => {
					Self::validate_general_data(&auction_object.general_data)?;
					Self::validate_create(&auction_object.general_data)?;
					Self::handle_create(sender, &auction, &auction_object.general_data)?;
				}
			}

			Ok(())
		}

		/// 
		/// Updates an existing auction which has not yet started
		/// 
		/// - validates auction.general_data
		/// - validates write action & updates auction
		/// 
		#[pallet::weight(<T as Config>::WeightInfo::update_auction())]
		pub fn update(origin: OriginFor<T>, id: T::AuctionId, updated_auction: Auction<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			match &updated_auction {
				Auction::English(auction_object) => {
					Self::validate_general_data(&auction_object.general_data)?;
					Self::handle_update(sender, id, updated_auction.clone(), &auction_object.general_data)?;
				}
			}

			Ok(())
		}

		/// 
		/// Destroys an existing auction which has not yet started
		/// 
		/// - validates write action
		/// - destroys auction
		/// - deposits AuctionDestroyed event
		/// 
		#[pallet::weight(<T as Config>::WeightInfo::destroy_auction())]
		pub fn destroy(origin: OriginFor<T>, id: T::AuctionId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionNotExist)?;

			match &auction {
				Auction::English(auction_object) => {
					Self::validate_update(sender, &auction_object.general_data)?;
					Self::handle_destroy(id, &auction_object.general_data)?;
				}
			}

			Ok(())
		}

		/// 
		/// Places a bid on a running auction
		/// 
		/// - validates bid
		/// - calls the bid() implementation on the given Auction type
		/// - deposits BidPlaced event
		/// 
		#[pallet::weight(<T as Config>::WeightInfo::bid())]
		pub fn bid(origin: OriginFor<T>, auction_id: T::AuctionId, value: BalanceOf<T>) -> DispatchResult {
			let bidder = ensure_signed(origin)?;

			<Auctions<T>>::try_mutate(auction_id, |maybe_auction| -> DispatchResult {
				let auction = maybe_auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;

				match auction {
					Auction::English(auction_object) => {
						Self::validate_bid(&bidder, &auction_object.general_data, value)?;
						auction_object.bid(bidder.clone(), value)?;
					}
				}

				Self::deposit_event(Event::BidPlaced(auction_id, bidder, value));

				Ok(())
			})
		}

		/// 
		/// Closes an auction 
		/// 
		/// All auctions which have reached their auction end time do not accept any new bids.
		/// However, the transfer of NFT and funds happens once an auction is closed.
		/// 
		/// All auctions which have reached their auction end time must be closed by calling this exstrinsic.
		/// 
		/// The reason for not automating this in a hook is to preserve chain performance (similar to claiming
		/// staking rewards in Substrate).
		/// 
		/// - validates auction close
		/// - calls the implementation of close() on the given Auction type
		/// - deposits AuctionClosed event
		/// 
		#[pallet::weight(<T as Config>::WeightInfo::close_auction())]
		pub fn close(_origin: OriginFor<T>, auction_id: T::AuctionId) -> DispatchResult {
			<Auctions<T>>::try_mutate(auction_id, |maybe_auction| -> DispatchResult {
				let auction = maybe_auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;

				match auction {
					Auction::English(auction_object) => {
						Self::validate_close(&auction_object.general_data)?;
						auction_object.close()?;
					}
				}

				Self::deposit_event(Event::AuctionClosed(auction_id));

				Ok(())
			})
		}
	}
}

impl<T: Config> Pallet<T> {
	/// 
	/// Validates auction.general_data
	/// 
	/// Called during create and update.
	/// 
	fn validate_general_data(general_data: &GeneralAuctionData<T>) -> DispatchResult {
		let current_block_number = frame_system::Pallet::<T>::block_number();
		ensure!(
			general_data.start >= current_block_number,
			Error::<T>::AuctionStartTimeAlreadyPassed
		);
		ensure!(
			general_data.start >= Zero::zero()
				&& general_data.end > Zero::zero()
				&& general_data.end > general_data.start + T::MinAuctionDuration::get().into(),
			Error::<T>::InvalidTimeConfiguration
		);
		ensure!(!general_data.name.is_empty(), Error::<T>::EmptyAuctionName);
		let token_owner = pallet_uniques::Pallet::<T>::owner(general_data.token.0, general_data.token.1);
		ensure!(
			token_owner == Some(general_data.owner.clone()),
			Error::<T>::NotATokenOwner
		);
		ensure!(!&general_data.closed, Error::<T>::CannotSetAuctionClosed);

		Ok(())
	}

	/// 
	/// Validates certain aspects relevant to the create action
	/// 
	fn validate_create(general_data: &GeneralAuctionData<T>) -> DispatchResult {
		let is_transferrable = pallet_uniques::Pallet::<T>::can_transfer(&general_data.token.0, &general_data.token.1);
		ensure!(is_transferrable, Error::<T>::TokenFrozen);

		Ok(())
	}

	/// 
	/// Handles auction create
	/// 
	/// - fetches next auction_id
	/// - inserts the Auction object in Auctions store
	/// - inserts a new record in AuctionOwnerById
	/// - freezes NFT
	/// - deposits AuctionCreated event
	/// 
	fn handle_create(
		sender: <T>::AccountId,
		auction: &Auction<T>,
		general_data: &GeneralAuctionData<T>,
	) -> DispatchResult {
		let auction_id = <NextAuctionId<T>>::try_mutate(|next_id| -> result::Result<<T>::AuctionId, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id
				.checked_add(&One::one())
				.ok_or(Error::<T>::NoAvailableAuctionId)?;
			Ok(current_id)
		})?;

		<Auctions<T>>::insert(auction_id, auction.clone());
		<AuctionOwnerById<T>>::insert(auction_id, &sender);

		pallet_uniques::Pallet::<T>::freeze(
			RawOrigin::Signed(sender.clone()).into(),
			general_data.token.0,
			general_data.token.1,
		)?;

		Self::deposit_event(Event::AuctionCreated(sender, auction_id));

		Ok(())
	}

	/// 
	/// Validates certain aspects relevant to the update action
	/// 
	fn validate_update(sender: <T>::AccountId, general_data: &GeneralAuctionData<T>) -> DispatchResult {
		ensure!(general_data.owner == sender, Error::<T>::NotAuctionOwner);

		let current_block_number = frame_system::Pallet::<T>::block_number();
		ensure!(
			current_block_number < general_data.start,
			Error::<T>::AuctionAlreadyStarted
		);

		Ok(())
	}

	/// 
	/// Handles auction update
	/// 
	fn handle_update(
		sender: <T>::AccountId,
		auction_id: T::AuctionId,
		updated_auction: Auction<T>,
		general_data: &GeneralAuctionData<T>,
	) -> DispatchResult {
		<Auctions<T>>::try_mutate(auction_id, |auction_result| -> DispatchResult {
			if auction_result.is_some() {
				Self::validate_update(sender, general_data)?;
				*auction_result = Some(updated_auction);
				Ok(())
			} else {
				Err(Error::<T>::AuctionNotExist.into())
			}
		})
	}

	/// 
	/// Handles auction destroy
	/// 
	/// - unfreezes NFT
	/// - removes record from AuctionOwnerById
	/// - removes record from Auctions
	/// - deposits AuctionDestroyed event
	/// 
	fn handle_destroy(auction_id: T::AuctionId, general_data: &GeneralAuctionData<T>) -> DispatchResult {
		pallet_uniques::Pallet::<T>::thaw(
			RawOrigin::Signed(general_data.owner.clone()).into(),
			general_data.token.0,
			general_data.token.1,
		)?;

		<AuctionOwnerById<T>>::remove(auction_id);
		<Auctions<T>>::remove(auction_id);

		Self::deposit_event(Event::AuctionDestroyed(auction_id));

		Ok(())
	}

	/// 
	/// Validates certain aspects relevant to the bid action
	/// 
	fn validate_bid(
		bidder: &<T>::AccountId,
		general_auction_data: &GeneralAuctionData<T>,
		value: BalanceOf<T>,
	) -> DispatchResult {
		let block_number = <frame_system::Pallet<T>>::block_number();
		ensure!(bidder != &general_auction_data.owner, Error::<T>::CannotBidOnOwnAuction);
		ensure!(block_number > general_auction_data.start, Error::<T>::AuctionNotStarted);
		ensure!(
			block_number < general_auction_data.end,
			Error::<T>::AuctionEndTimeReached
		);
		ensure!(value >= general_auction_data.next_bid_min, Error::<T>::InvalidBidPrice);

		if let Some(current_bid) = &general_auction_data.last_bid {
			ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
		} else {
			ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
		}

		Ok(())
	}
	
	/// 
	/// Validates certain aspects relevant to the close action
	/// 
	fn validate_close(general_auction_data: &GeneralAuctionData<T>) -> DispatchResult {
		let block_number = <frame_system::Pallet<T>>::block_number();
		ensure!(!general_auction_data.closed, Error::<T>::AuctionClosed);
		ensure!(block_number >= general_auction_data.end, Error::<T>::AuctionEndTimeNotReached);

		Ok(())
	}
}

/// 
/// Implementation of EnglishAuction
/// 
impl<T: Config> NftAuction<T::AccountId, T::AuctionId, BalanceOf<T>, Auction<T>> for EnglishAuction<T> {
	/// 
	/// Places a bid on an EnglishAuction
	/// 
	/// - removes lock on auction.general_data.last_bid
	/// - sets lock on new bid
	/// - updates auction.general_data.last_bid and auction.general_data.next_bid_min
	/// - if necessary, increases auction end time to prevent sniping
	/// 
	fn bid(&mut self, bidder: T::AccountId, value: BalanceOf<T>) -> DispatchResult {
		// Lock / Unlock funds
		if let Some(current_bid) = &self.general_data.last_bid {
			<T as crate::Config>::Currency::remove_lock(AUCTION_LOCK_ID, &current_bid.0);
		}
		<T as crate::Config>::Currency::set_lock(AUCTION_LOCK_ID, &bidder, value, WithdrawReasons::all());

		self.general_data.last_bid = Some((bidder, value));
		// Set next minimal bid
		let bid_step = Permill::from_percent(<T as crate::Config>::BidStepPerc::get()).mul_floor(value);
		self.general_data.next_bid_min = value.checked_add(&bid_step).ok_or(Error::<T>::BidOverflow)?;

		// Avoid auction sniping
		let block_number = <frame_system::Pallet<T>>::block_number();
		let time_left = self
			.general_data
			.end
			.checked_sub(&block_number)
			.ok_or(Error::<T>::TimeUnderflow)?;
		if time_left < <T as crate::Config>::BidAddBlocks::get().into() {
			self.general_data.end = block_number + <T as crate::Config>::BidAddBlocks::get().into();
		}

		Ok(())
	}

	/// 
	/// Closes an EnglishAuction
	/// 
	/// - removes lock on NFT
	/// - transfers NFT to winning bidder
	/// - removes lock on auction.general_data.last_bid
	/// - transfers the amount of the bid from the account of the bidder to the owner of the auction
	/// - sets auction.general_data.closed to true
	/// 
	fn close(&mut self) -> DispatchResult {
		pallet_uniques::Pallet::<T>::thaw(
			RawOrigin::Signed(self.general_data.owner.clone()).into(),
			self.general_data.token.0,
			self.general_data.token.1,
		)?;
		// there is a bid so let's determine a winner and transfer tokens
		if let Some(winner) = &self.general_data.last_bid {
			let dest = T::Lookup::unlookup(winner.0.clone());
			let source = T::Origin::from(frame_system::RawOrigin::Signed(self.general_data.owner.clone()));
			pallet_nft::Pallet::<T>::transfer(source, self.general_data.token.0, self.general_data.token.1, dest)?;
			<T as crate::Config>::Currency::remove_lock(AUCTION_LOCK_ID, &winner.0);
			<<T as crate::Config>::Currency as Currency<T::AccountId>>::transfer(
				&winner.0,
				&self.general_data.owner,
				winner.1,
				ExistenceRequirement::KeepAlive,
			)?;
		}

		self.general_data.closed = true;

		Ok(())
	}
}
