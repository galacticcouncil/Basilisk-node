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
/// Implementation of TopUpAuction
///
impl<T: Config> NftAuction<T::AccountId, T::AuctionId, BalanceOf<T>, Auction<T>, Bid<T>> for TopUpAuction<T> {
	///
	/// Creates a TopUp Auction
	///
	#[require_transactional]
	fn create(&self, sender: T::AccountId, auction: Auction<T>) -> DispatchResult {
		Pallet::<T>::validate_create_permissions(sender.clone(), &self.common_data)?;
		self.validate_data(sender.clone())?;
		Pallet::<T>::handle_create(sender, auction, &self.common_data)?;

		Ok(())
	}

	///
	/// Updates a TopUp Auction
	///
	#[require_transactional]
	fn update(self, sender: T::AccountId, auction_id: T::AuctionId) -> DispatchResult {
		self.validate_data(sender.clone())?;

		<Auctions<T>>::try_mutate(auction_id, |maybe_auction| -> DispatchResult {
			let auction_result = maybe_auction.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

			if let Auction::TopUp(existing_auction) = auction_result {
				Pallet::<T>::validate_update_destroy_permissions(sender, &existing_auction.common_data)?;
				Pallet::<T>::validate_update(&existing_auction.common_data, &self.common_data)?;

				*existing_auction = self.clone();

				Ok(())
			} else {
				Err(Error::<T>::CannotChangeAuctionType.into())
			}
		})
	}

	///
	/// Destroys a TopUp Auction
	///
	#[require_transactional]
	fn destroy(self, sender: T::AccountId, id: T::AuctionId) -> DispatchResult {
		Pallet::<T>::validate_update_destroy_permissions(sender, &self.common_data)?;
		Pallet::<T>::handle_destroy(id)?;
		Pallet::<T>::unfreeze_nft(&self.common_data)?;

		Ok(())
	}

	///
	/// Places a bid on an TopUpAuction
	///
	#[require_transactional]
	fn bid(&mut self, auction_id: T::AuctionId, bidder: T::AccountId, bid: &Bid<T>) -> DispatchResult {
		let past_bids_amount = <ReservedAmounts<T>>::get(bidder.clone(), auction_id);
		let amount_to_reserve = bid.amount.saturating_sub(past_bids_amount);

		// Reserve bid amount by transferring it to the subaccount of the auction
		Pallet::<T>::reserve_bid_amount(auction_id, bidder.clone(), amount_to_reserve)?;

		self.common_data.last_bid = Some((bidder, bid.amount));

		// Set next minimal bid
		Pallet::<T>::set_next_bid_min(&mut self.common_data, bid.amount)?;

		// Avoid auction sniping
		Pallet::<T>::avoid_auction_sniping(&mut self.common_data)?;

		Ok(())
	}

	///
	/// Closes a TopUpAuction
	///
	#[require_transactional]
	fn close(&mut self, auction_id: T::AuctionId) -> Result<bool, DispatchError> {
		let mut destroy_auction_data = false;
		let mut auction_winner: Option<T::AccountId> = None;

		Pallet::<T>::unfreeze_nft(&self.common_data)?;

		if let Some(winner) = &self.common_data.last_bid {
			if Pallet::<T>::is_auction_won(&self.common_data) {
				let dest = T::Lookup::unlookup(winner.0.clone());
				let source = T::Origin::from(frame_system::RawOrigin::Signed(self.common_data.owner.clone()));
				pallet_nft::Pallet::<T>::transfer(source, self.common_data.token.0, self.common_data.token.1, dest)?;

				let auction_account = Pallet::<T>::get_auction_subaccount_id(auction_id)
					.ok_or(Error::<T>::CannotGenerateAuctionAccount)?;
				let transfer_amount =
					<<T as crate::Config>::Currency as Currency<T::AccountId>>::free_balance(&auction_account);

				<<T as crate::Config>::Currency as Currency<T::AccountId>>::transfer(
					&auction_account,
					&self.common_data.owner,
					transfer_amount,
					ExistenceRequirement::AllowDeath,
				)?;

				auction_winner = Some(winner.0.clone());
				// Auction and related data can be pruned
				destroy_auction_data = true;
			}
		} else {
			destroy_auction_data = true;
		}

		self.common_data.closed = true;

		Pallet::<T>::deposit_event(Event::AuctionClosed {
			auction_id,
			auction_winner,
		});

		Ok(destroy_auction_data)
	}

	///
	/// Claims reserved amounts which were bid on a TopUp auction
	///
	#[require_transactional]
	fn claim(
		&self,
		auction_id: T::AuctionId,
		bidder: T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<bool, DispatchError> {
		let mut destroy_auction_data = false;

		Pallet::<T>::validate_claim(&self.common_data)?;

		ensure!(
			!Pallet::<T>::is_auction_won(&self.common_data),
			Error::<T>::CannotClaimWonAuction
		);

		Pallet::<T>::unreserve_bid_amount(auction_id, bidder.clone(), amount, bidder)?;

		let auction_account =
			Pallet::<T>::get_auction_subaccount_id(auction_id).ok_or(Error::<T>::CannotGenerateAuctionAccount)?;
		let auction_balance =
			<<T as crate::Config>::Currency as Currency<T::AccountId>>::free_balance(&auction_account);

		// Auction and related data can be pruned
		if auction_balance.is_zero() {
			destroy_auction_data = true;
		}

		Ok(destroy_auction_data)
	}

	///
	/// Validates common auction data
	///
	fn validate_data(&self, sender: T::AccountId) -> DispatchResult {
		Pallet::<T>::validate_common_data(sender, &self.common_data)?;
		Ok(())
	}
}
