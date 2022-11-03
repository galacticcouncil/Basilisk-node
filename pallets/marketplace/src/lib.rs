// This file is part of Basilisk-node.

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, ExistenceRequirement, ReservableCurrency},
};
use frame_system::{ensure_signed, RawOrigin};
use sp_runtime::{
	traits::{CheckedDiv, CheckedMul, Saturating, StaticLookup},
	ArithmeticError, DispatchError,
};
use sp_std::convert::TryInto;

use types::*;
use weights::WeightInfo;

mod benchmarking;
pub mod migration;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type OfferOf<T> = Offer<<T as frame_system::Config>::AccountId, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;
type RoyaltyOf<T> = Royalty<<T as frame_system::Config>::AccountId>;

pub const MAX_ROYALTY: u16 = 10_000; // 100% in basis points

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn prices)]
	/// Stores token info
	pub(super) type Prices<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftCollectionId,
		Blake2_128Concat,
		T::NftItemId,
		BalanceOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn offers)]
	/// Stores offer info
	pub(super) type Offers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		(T::NftCollectionId, T::NftItemId),
		Blake2_128Concat,
		T::AccountId,
		OfferOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn marketplace_items)]
	/// Stores Marketplace info
	pub type MarketplaceItems<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::NftCollectionId, Blake2_128Concat, T::NftItemId, RoyaltyOf<T>>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type WeightInfo: WeightInfo;
		#[pallet::constant]
		type MinimumOfferAmount: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type RoyaltyBondAmount: Get<BalanceOf<Self>>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Pays a price to the current owner
		/// Transfers NFT ownership to the buyer
		/// Disables automatic sell of the NFT
		///
		/// Parameters:
		/// - `collection_id`: The identifier of a non-fungible token collection
		/// - `item_id`: The item identifier of a collection
		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		pub fn buy(origin: OriginFor<T>, collection_id: T::NftCollectionId, item_id: T::NftItemId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::do_buy(sender, collection_id, item_id, false)
		}

		/// Set trading price and allow sell
		/// Setting price to None disables auto sell
		///
		/// Parameters:
		/// - `collection_id`: The identifier of a non-fungible token collection
		/// - `item_id`: The item identifier of a collection
		/// - `new_price`: price the token will be listed for
		#[pallet::weight(<T as Config>::WeightInfo::set_price())]
		pub fn set_price(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			item_id: T::NftItemId,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				pallet_nft::Pallet::<T>::owner(collection_id, item_id) == Some(sender.clone()),
				Error::<T>::NotTheTokenOwner
			);

			Prices::<T>::mutate_exists(collection_id, item_id, |price| *price = new_price);

			Self::deposit_event(Event::TokenPriceUpdated {
				who: sender,
				collection: collection_id,
				item: item_id,
				price: new_price,
			});

			Ok(())
		}

		/// Users can indicate what price they would be willing to pay for a token
		/// Price can be lower than current listing price
		/// Token doesn't have to be currently listed
		///
		/// Parameters:
		/// - `collection_id`: The identifier of a non-fungible token collection
		/// - `item_id`: The item identifier of a collection
		/// - `amount`: The amount user is willing to pay
		/// - `expires`: The block until the current owner can accept the offer
		#[pallet::weight(<T as Config>::WeightInfo::make_offer())]
		pub fn make_offer(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			item_id: T::NftItemId,
			amount: BalanceOf<T>,
			expires: T::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(amount >= T::MinimumOfferAmount::get(), Error::<T>::OfferTooLow);
			ensure!(
				!Offers::<T>::contains_key((collection_id, item_id), sender.clone()),
				Error::<T>::AlreadyOffered
			);

			let token_id = (collection_id, item_id);

			Offers::<T>::insert(
				token_id,
				sender.clone(),
				Offer {
					maker: sender.clone(),
					amount,
					expires,
				},
			);

			<T as Config>::Currency::reserve(&sender, amount)?;

			Self::deposit_event(Event::OfferPlaced {
				who: sender,
				collection: collection_id,
				item: item_id,
				amount,
				expires,
			});

			Ok(())
		}

		/// Reverse action to make_offer
		/// Removes an offer and unreserves funds
		/// Can be done by the offer maker or owner of the token
		///
		/// Parameters:
		/// - `collection_id`: The identifier of a non-fungible token collection
		/// - `item_id`: The item identifier of a collection
		/// - `maker`: User who made the offer
		#[pallet::weight(<T as Config>::WeightInfo::withdraw_offer())]
		pub fn withdraw_offer(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			item_id: T::NftItemId,
			maker: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let token_id = (collection_id, item_id);

			Offers::<T>::try_mutate_exists(token_id, maker, |maybe_offer| -> DispatchResult {
				let offer = maybe_offer.take().ok_or(Error::<T>::UnknownOffer)?;
				let sender_is_owner = match pallet_nft::Pallet::<T>::owner(collection_id, item_id) {
					Some(owner) => sender == owner,
					None => false,
				};

				ensure!(
					sender == offer.maker || sender_is_owner,
					Error::<T>::WithdrawNotAuthorized
				);

				<T as Config>::Currency::unreserve(&offer.maker, offer.amount);

				Self::deposit_event(Event::OfferWithdrawn {
					who: sender,
					collection: collection_id,
					item: item_id,
				});
				Ok(())
			})
		}

		/// Accept an offer and process the trade
		///
		/// Parameters:
		/// - `collection_id`: The identifier of a non-fungible token collection
		/// - `item_id`: The item identifier of a collection
		/// - `maker`: User who made the offer
		#[pallet::weight(<T as Config>::WeightInfo::accept_offer())]
		pub fn accept_offer(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			item_id: T::NftItemId,
			maker: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let token_id = (collection_id, item_id);

			let owner =
				pallet_nft::Pallet::<T>::owner(collection_id, item_id).ok_or(Error::<T>::CollectionOrItemUnknown)?;

			ensure!(sender == owner, Error::<T>::AcceptNotAuthorized);

			Offers::<T>::try_mutate_exists(token_id, maker, |maybe_offer| -> DispatchResult {
				let offer = maybe_offer.take().ok_or(Error::<T>::UnknownOffer)?;

				if offer.expires > <frame_system::Pallet<T>>::block_number() {
					<T as Config>::Currency::unreserve(&offer.maker, offer.amount);
					Self::do_buy(offer.maker.clone(), collection_id, item_id, true)?;
					Self::deposit_event(Event::OfferAccepted {
						who: sender,
						collection: collection_id,
						item: item_id,
						amount: offer.amount,
						maker: offer.maker,
					});
					Ok(())
				} else {
					Err(Error::<T>::OfferExpired.into())
				}
			})
		}

		/// Add royalty feature where a cut for author is provided
		/// There is non-refundable reserve held for creating a royalty
		///
		/// Parameters:
		/// - `collection_id`: The collection of the asset to be minted.
		/// - `item_id`: The item value of the asset to be minted.
		/// - `author`: Receiver of the royalty
		/// - `royalty`: Percentage reward from each trade for the author, represented in basis points
		#[pallet::weight(<T as Config>::WeightInfo::add_royalty())]
		pub fn add_royalty(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			item_id: T::NftItemId,
			author: T::AccountId,
			royalty: u16,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				!MarketplaceItems::<T>::contains_key(collection_id, item_id),
				Error::<T>::RoyaltyAlreadySet
			);
			ensure!(royalty < MAX_ROYALTY, Error::<T>::NotInRange);
			let owner = pallet_nft::Pallet::<T>::owner(collection_id, item_id)
				.ok_or(pallet_nft::Error::<T>::CollectionUnknown)?;
			ensure!(sender == owner, pallet_nft::Error::<T>::NotPermitted);

			let royalty_bond = T::RoyaltyBondAmount::get();
			<T as Config>::Currency::reserve(&sender, royalty_bond)?;

			MarketplaceItems::<T>::insert(
				collection_id,
				item_id,
				Royalty {
					author: author.clone(),
					royalty,
				},
			);

			Self::deposit_event(Event::RoyaltyAdded {
				collection: collection_id,
				item: item_id,
				author,
				royalty,
			});

			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The price for a token was updated
		TokenPriceUpdated {
			who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			price: Option<BalanceOf<T>>,
		},
		/// Token was sold to a new owner
		TokenSold {
			owner: T::AccountId,
			buyer: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			price: BalanceOf<T>,
		},
		/// Offer was placed on a token
		OfferPlaced {
			who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			amount: BalanceOf<T>,
			expires: T::BlockNumber,
		},
		/// Offer was withdrawn
		OfferWithdrawn {
			who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
		},
		/// Offer was accepted
		OfferAccepted {
			who: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			amount: BalanceOf<T>,
			maker: T::AccountId,
		},
		/// Royalty hs been paid to the author
		RoyaltyPaid {
			collection: T::NftCollectionId,
			item: T::NftItemId,
			author: T::AccountId,
			royalty: u16,
			royalty_amount: BalanceOf<T>,
		},
		/// Marketplace data has been added
		RoyaltyAdded {
			collection: T::NftCollectionId,
			item: T::NftItemId,
			author: T::AccountId,
			royalty: u16,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account is not the owner of the token
		NotTheTokenOwner,
		/// Cannot buy a token from yourself
		BuyFromSelf,
		/// Token is currently not for sale
		NotForSale,
		/// Collection or item does not exist
		CollectionOrItemUnknown,
		/// Offer is lower than the minimum threshold
		OfferTooLow,
		/// No offer for this token found from the user
		UnknownOffer,
		/// Offer is no longer valid
		OfferExpired,
		/// User already made an offer for this token
		AlreadyOffered,
		/// User has to be offer maker or token owner to withdraw an offer
		WithdrawNotAuthorized,
		/// User has to be the token owner to accept an offer
		AcceptNotAuthorized,
		/// Royalty can be set only once
		RoyaltyAlreadySet,
		/// Royalty not in 0-9_999 range
		NotInRange,
	}
}

impl<T: Config> Pallet<T> {
	// Call extrinsic helper function used by `buy` and `accept_offer` functions
	fn do_buy(
		buyer: T::AccountId,
		collection_id: T::NftCollectionId,
		item_id: T::NftItemId,
		is_offer: bool,
	) -> DispatchResult {
		let owner =
			pallet_nft::Pallet::<T>::owner(collection_id, item_id).ok_or(Error::<T>::CollectionOrItemUnknown)?;
		ensure!(buyer != owner, Error::<T>::BuyFromSelf);

		let owner_origin = T::Origin::from(RawOrigin::Signed(owner.clone()));

		let token_id = (collection_id, item_id);

		Prices::<T>::try_mutate(collection_id, item_id, |price| -> DispatchResult {
			let mut price = if is_offer {
				Offers::<T>::get(token_id, buyer.clone())
					.map(|o| o.amount)
					.ok_or(Error::<T>::UnknownOffer)?
			} else {
				price.take().ok_or(Error::<T>::NotForSale)?
			};

			// Settle royalty if set
			if let Some(item_info) = MarketplaceItems::<T>::get(collection_id, item_id) {
				let royalty = item_info.royalty;
				let author = item_info.author;

				// Calculate royalty and subtract from price if author different from buyer
				let royalty_amount = price
					.checked_mul(&BalanceOf::<T>::from(royalty))
					.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?
					.checked_div(&BalanceOf::<T>::from(MAX_ROYALTY))
					.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

				if owner != author && royalty != 0u16 {
					price = price.saturating_sub(royalty_amount);

					// Send royalty to author
					<T as Config>::Currency::transfer(
						&buyer,
						&author,
						royalty_amount,
						ExistenceRequirement::KeepAlive,
					)?;

					Self::deposit_event(Event::RoyaltyPaid {
						collection: collection_id,
						item: item_id,
						author,
						royalty,
						royalty_amount,
					});
				}
			}

			// Send the net price from current to the previous owner
			<T as Config>::Currency::transfer(&buyer, &owner, price, ExistenceRequirement::KeepAlive)?;

			let to = T::Lookup::unlookup(buyer.clone());
			pallet_nft::Pallet::<T>::transfer(owner_origin, collection_id, item_id, to)?;

			Self::deposit_event(Event::TokenSold {
				owner,
				buyer,
				collection: collection_id,
				item: item_id,
				price,
			});
			Ok(())
		})
	}
}
