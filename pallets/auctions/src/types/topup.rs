use super::*;

///
/// Implementation of TopUpAuction
///
impl<T: Config> NftAuction<T::AccountId, T::AuctionId, BalanceOf<T>, Auction<T>, Bid<T>> for TopUpAuction<T> {
	///
	/// Creates a TopUp Auction
	///
	fn create(&self, sender: T::AccountId, auction: &Auction<T>) -> DispatchResult {
		self.validate_data()?;
		Pallet::<T>::validate_create(&self.common_data)?;
		Pallet::<T>::handle_create(sender, auction, &self.common_data)?;

		Ok(())
	}

	///
	/// Updates a TopUp Auction
	///
	fn update(self, sender: T::AccountId, auction_id: T::AuctionId) -> DispatchResult {
		self.validate_data()?;

		<Auctions<T>>::try_mutate(auction_id, |maybe_auction| -> DispatchResult {
			let auction_result = maybe_auction.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

			if let Auction::TopUp(topup_auction) = auction_result {
				Pallet::<T>::validate_update(sender, &topup_auction.common_data)?;
				*topup_auction = self;

				Ok(())
			} else {
				Err(Error::<T>::NoChangeOfAuctionType.into())
			}
		})
	}

	///
	/// Places a bid on an TopUpAuction
	///
	fn bid(&mut self, auction_id: T::AuctionId, bidder: T::AccountId, bid: &Bid<T>) -> DispatchResult {
		// Trasnfer funds to the subaccount of the auction
		<<T as crate::Config>::Currency as Currency<T::AccountId>>::transfer(
			&bidder,
			&Pallet::<T>::get_auction_subaccount_id(auction_id),
			bid.amount,
			ExistenceRequirement::KeepAlive,
		)?;

		self.common_data.last_bid = Some((bidder.clone(), bid.amount));

		// Set next minimal bid
		Pallet::<T>::set_next_bid_min(&mut self.common_data, bid.amount)?;

		<ReservedAmounts<T>>::try_mutate(&bidder, auction_id, |locked_amount| -> DispatchResult {
			*locked_amount = locked_amount
				.checked_add(&bid.amount)
				.ok_or(Error::<T>::InvalidReservedAmount)?;

			Ok(())
		})?;

		// Avoid auction sniping
		Pallet::<T>::avoid_auction_sniping(&mut self.common_data)?;

		Ok(())
	}

	///
	/// Closes a TopUpAuction
	///
	fn close(&mut self, auction_id: T::AuctionId) -> Result<bool, DispatchError> {
		let mut destroy_auction_data = false;

		Pallet::<T>::unfreeze_nft(&self.common_data)?;

		if let Some(winner) = &self.common_data.last_bid {
			if Pallet::<T>::is_auction_won(&self.common_data) {
				let dest = T::Lookup::unlookup(winner.0.clone());
				let source = T::Origin::from(frame_system::RawOrigin::Signed(self.common_data.owner.clone()));
				pallet_nft::Pallet::<T>::transfer(
					source,
					self.common_data.token.0.into(),
					self.common_data.token.1.into(),
					dest,
				)?;

				let auction_account = &Pallet::<T>::get_auction_subaccount_id(auction_id);
				let transfer_amount =
					<<T as crate::Config>::Currency as Currency<T::AccountId>>::free_balance(auction_account);

				<<T as crate::Config>::Currency as Currency<T::AccountId>>::transfer(
					auction_account,
					&self.common_data.owner,
					transfer_amount,
					ExistenceRequirement::AllowDeath,
				)?;

				// Auction and related data can be pruned
				destroy_auction_data = true;
			}
		} else {
			destroy_auction_data = true;
		}

		self.common_data.closed = true;

		Ok(destroy_auction_data)
	}

	///
	/// Claims reserved amounts which were bid on a TopUp auction
	/// 
	fn claim(&self, auction_id: T::AuctionId, bidder: T::AccountId, amount: BalanceOf<T>) -> Result<bool, DispatchError> {
		let mut destroy_auction_data = false;

		Pallet::<T>::validate_claim(&self.common_data)?;

		ensure!(
			!Pallet::<T>::is_auction_won(&self.common_data),
			Error::<T>::CannotClaimWonAuction
		);

		Pallet::<T>::handle_claim(bidder, auction_id, amount)?;

		let auction_account = &Pallet::<T>::get_auction_subaccount_id(auction_id);
		let auction_balance = <<T as crate::Config>::Currency as Currency<T::AccountId>>::free_balance(auction_account);

		// Auction and related data can be pruned
		if auction_balance.is_zero() {
			destroy_auction_data = true;
		}

		Ok(destroy_auction_data)
	}

	///
	/// Validates common auction data
	///
	fn validate_data(&self) -> DispatchResult {
		Pallet::<T>::validate_common_data(&self.common_data)
	}
}