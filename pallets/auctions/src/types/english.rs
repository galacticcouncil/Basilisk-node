//      ---_ ......._-_--.        ,adPPYba, 8b,dPPYba,    ,adPPYba,  88   ,d8
//     (|\ /      / /| \  \       I8[    "" 88P'   `"8a  a8P_____88  88 ,a8"
//     /  /     .'  -=-'   `.      `"Y8ba,  88       88  8PP"""""""  8888[
//    /  /    .'             )    aa    ]8I 88       88  "8b,   ,aa  88`"Yba,
//  _/  /   .'        _.)   /     `"YbbdP"' 88       88   `"Ybbd8"'  88   `Y8a
//  / o   o        _.-' /  .'
//  \          _.-'    / .'*|
//  \______.-'//    .'.' \*|      This file is part of Basilisk-node.
//   \|  \ | //   .'.' _ |*|      Built with <3 for decentralisation.
//    `   \|//  .'.'_ _ _|*|
//     .  .// .'.' | _ _ \*|      Copyright (C) 2021-2022  Intergalactic, Limited (GIB).
//     \`-|\_/ /    \ _ _ \*\     SPDX-License-Identifier: Apache-2.0
//      `/'\__/      \ _ _ \*\    Licensed under the Apache License, Version 2.0 (the "License");
//     /^|            \ _ _ \*    you may not use this file except in compliance with the License.
//    '  `             \ _ _ \    http://www.apache.org/licenses/LICENSE-2.0
//     '  `             \ _ _ \

use super::*;

///
/// Implementation of EnglishAuction
///
impl<T: Config> NftAuction<T::AccountId, T::AuctionId, BalanceOf<T>, Auction<T>, Bid<T>> for EnglishAuction<T> {
	#[require_transactional]
	fn create(&self, sender: T::AccountId, auction: &Auction<T>) -> DispatchResult {
		Pallet::<T>::validate_create_permissions(sender.clone(), &self.common_data)?;
		self.validate_data(sender.clone())?;
		Pallet::<T>::handle_create(sender, auction, &self.common_data)?;

		Ok(())
	}

	#[require_transactional]
	fn update(self, sender: T::AccountId, auction_id: T::AuctionId) -> DispatchResult {
		self.validate_data(sender.clone())?;

		<Auctions<T>>::try_mutate(auction_id, |maybe_auction| -> DispatchResult {
			let auction_result = maybe_auction.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

			if let Auction::English(english_auction) = auction_result {
				Pallet::<T>::validate_update_destroy_permissions(sender, &english_auction.common_data)?;
				*english_auction = self;

				Ok(())
			} else {
				Err(Error::<T>::NoChangeOfAuctionType.into())
			}
		})
	}

	///
	/// Places a bid on an EnglishAuction
	///
	/// - removes lock on auction.common_data.last_bid
	/// - sets lock on new bid
	/// - updates auction.common_data.last_bid and auction.common_data.next_bid_min
	/// - if necessary, increases auction end time to prevent sniping
	///
	#[require_transactional]
	fn bid(&mut self, auction_id: T::AuctionId, bidder: T::AccountId, bid: &Bid<T>) -> DispatchResult {
		// Unreserve funds
		if let Some(current_bid) = &self.common_data.last_bid {
			<<T as crate::Config>::Currency as Currency<T::AccountId>>::transfer(
				&Pallet::<T>::get_auction_subaccount_id(auction_id),
				&current_bid.0,
				current_bid.1,
				ExistenceRequirement::AllowDeath,
			)?;
		}

		// Reserve funds by transferring to the subaccount of the auction
		<<T as crate::Config>::Currency as Currency<T::AccountId>>::transfer(
			&bidder,
			&Pallet::<T>::get_auction_subaccount_id(auction_id),
			bid.amount,
			ExistenceRequirement::KeepAlive,
		)?;

		self.common_data.last_bid = Some((bidder, bid.amount));
		// Set next minimal bid
		Pallet::<T>::set_next_bid_min(&mut self.common_data, bid.amount)?;

		// Avoid auction sniping
		Pallet::<T>::avoid_auction_sniping(&mut self.common_data)?;

		Ok(())
	}

	///
	/// Closes an EnglishAuction
	///
	/// - removes lock on NFT
	/// - transfers NFT to winning bidder
	/// - removes lock on auction.common_data.last_bid
	/// - transfers the amount of the bid from the account of the bidder to the owner of the auction
	/// - sets auction.common_data.closed to true
	///
	#[require_transactional]
	fn close(&mut self, auction_id: T::AuctionId) -> Result<bool, DispatchError> {
		Pallet::<T>::unfreeze_nft(&self.common_data)?;

		// In case of a winning bid:
		// - transfer NFT
		// - transfer reserved funds from the auction subaccount
		if let Some(winning_bid) = &self.common_data.last_bid {
			let dest = T::Lookup::unlookup(winning_bid.0.clone());
			let source = T::Origin::from(frame_system::RawOrigin::Signed(self.common_data.owner.clone()));
			pallet_nft::Pallet::<T>::transfer(source, self.common_data.token.0, self.common_data.token.1, dest)?;

			<<T as crate::Config>::Currency as Currency<T::AccountId>>::transfer(
				&Pallet::<T>::get_auction_subaccount_id(auction_id),
				&self.common_data.owner,
				winning_bid.1,
				ExistenceRequirement::AllowDeath,
			)?;
		}

		self.common_data.closed = true;

		Ok(true)
	}

	/// English auctions do not implement reserved amounts and there are no claims
	fn claim(
		&self,
		_auction_id: T::AuctionId,
		_bidder: T::AccountId,
		_amount: BalanceOf<T>,
	) -> Result<bool, DispatchError> {
		Err(Error::<T>::ClaimsNotSupportedForThisAuctionType.into())
	}

	///
	/// Validates common and specific auction data
	///
	fn validate_data(&self, sender: T::AccountId) -> DispatchResult {
		Pallet::<T>::validate_common_data(sender, &self.common_data)?;

		if let Some(reserve_price) = self.common_data.reserve_price {
			// If a reserve_price is set, it must be equal to next_bid_min
			ensure!(
				reserve_price == self.common_data.next_bid_min,
				Error::<T>::InvalidNextBidMin
			);
		} else {
			ensure!(
				self.common_data.next_bid_min == T::BidMinAmount::get().into(),
				Error::<T>::InvalidNextBidMin
			);
		}

		Ok(())
	}
}
