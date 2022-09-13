// This file is part of Basilisk-node.

// Copyright (C) 2020-2021  Intergalactic, Limited (GIB).
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

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn prices)]
	/// Stores token info
	pub(super) type Prices<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftClassId,
		Blake2_128Concat,
		T::NftInstanceId,
		BalanceOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn offers)]
	/// Stores offer info
	pub(super) type Offers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		(T::NftClassId, T::NftInstanceId),
		Blake2_128Concat,
		T::AccountId,
		OfferOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn marketplace_instances)]
	/// Stores Marketplace info
	pub type MarketplaceInstances<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::NftClassId, Twox64Concat, T::NftInstanceId, RoyaltyOf<T>>;

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
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		pub fn buy(origin: OriginFor<T>, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::do_buy(sender, class_id, instance_id, false)
		}

		/// Set trading price and allow sell
		/// Setting price to None disables auto sell
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		/// - `new_price`: price the token will be listed for
		#[pallet::weight(<T as Config>::WeightInfo::set_price())]
		pub fn set_price(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				pallet_nft::Pallet::<T>::owner(class_id, instance_id) == Some(sender.clone()),
				Error::<T>::NotTheTokenOwner
			);

			Prices::<T>::mutate_exists(class_id, instance_id, |price| *price = new_price);

			Self::deposit_event(Event::TokenPriceUpdated {
				who: sender,
				class: class_id,
				instance: instance_id,
				price: new_price,
			});

			Ok(())
		}

		/// Users can indicate what price they would be willing to pay for a token
		/// Price can be lower than current listing price
		/// Token doesn't have to be currently listed
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		/// - `amount`: The amount user is willing to pay
		/// - `expires`: The block until the current owner can accept the offer
		#[pallet::weight(<T as Config>::WeightInfo::make_offer())]
		pub fn make_offer(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			amount: BalanceOf<T>,
			expires: T::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(amount >= T::MinimumOfferAmount::get(), Error::<T>::OfferTooLow);
			ensure!(
				!Offers::<T>::contains_key((class_id, instance_id), sender.clone()),
				Error::<T>::AlreadyOffered
			);

			let token_id = (class_id, instance_id);

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
				class: class_id,
				instance: instance_id,
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
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		/// - `maker`: User who made the offer
		#[pallet::weight(<T as Config>::WeightInfo::withdraw_offer())]
		pub fn withdraw_offer(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			maker: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let token_id = (class_id, instance_id);

			Offers::<T>::try_mutate_exists(token_id, maker, |maybe_offer| -> DispatchResult {
				let offer = maybe_offer.take().ok_or(Error::<T>::UnknownOffer)?;
				let sender_is_owner = match pallet_nft::Pallet::<T>::owner(class_id, instance_id) {
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
					class: class_id,
					instance: instance_id,
				});
				Ok(())
			})
		}

		/// Accept an offer and process the trade
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		/// - `maker`: User who made the offer
		#[pallet::weight(<T as Config>::WeightInfo::accept_offer())]
		pub fn accept_offer(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			maker: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let token_id = (class_id, instance_id);

			let owner =
				pallet_nft::Pallet::<T>::owner(class_id, instance_id).ok_or(Error::<T>::ClassOrInstanceUnknown)?;

			ensure!(sender == owner, Error::<T>::AcceptNotAuthorized);

			Offers::<T>::try_mutate_exists(token_id, maker, |maybe_offer| -> DispatchResult {
				let offer = maybe_offer.take().ok_or(Error::<T>::UnknownOffer)?;

				if offer.expires > <frame_system::Pallet<T>>::block_number() {
					<T as Config>::Currency::unreserve(&offer.maker, offer.amount);
					Self::do_buy(offer.maker.clone(), class_id, instance_id, true)?;
					Self::deposit_event(Event::OfferAccepted {
						who: sender,
						class: class_id,
						instance: instance_id,
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
		/// - `class_id`: The class of the asset to be minted.
		/// - `instance_id`: The instance value of the asset to be minted.
		/// - `author`: Receiver of the royalty
		/// - `royalty`: Percentage reward from each trade for the author, represented in basis points
		#[pallet::weight(<T as Config>::WeightInfo::add_royalty())]
		pub fn add_royalty(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			author: T::AccountId,
			royalty: u16,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				!MarketplaceInstances::<T>::contains_key(class_id, instance_id),
				Error::<T>::RoyaltyAlreadySet
			);
			ensure!(royalty < MAX_ROYALTY, Error::<T>::NotInRange);
			let owner =
				pallet_nft::Pallet::<T>::owner(class_id, instance_id).ok_or(pallet_nft::Error::<T>::ClassUnknown)?;
			ensure!(sender == owner, pallet_nft::Error::<T>::NotPermitted);

			let royalty_bond = T::RoyaltyBondAmount::get();
			<T as Config>::Currency::reserve(&sender, royalty_bond)?;

			MarketplaceInstances::<T>::insert(
				class_id,
				instance_id,
				Royalty {
					author: author.clone(),
					royalty,
				},
			);

			Self::deposit_event(Event::RoyaltyAdded {
				class: class_id,
				instance: instance_id,
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
			class: T::NftClassId,
			instance: T::NftInstanceId,
			price: Option<BalanceOf<T>>,
		},
		/// Token was sold to a new owner
		TokenSold {
			owner: T::AccountId,
			buyer: T::AccountId,
			class: T::NftClassId,
			instance: T::NftInstanceId,
			price: BalanceOf<T>,
		},
		/// Offer was placed on a token
		OfferPlaced {
			who: T::AccountId,
			class: T::NftClassId,
			instance: T::NftInstanceId,
			amount: BalanceOf<T>,
			expires: T::BlockNumber,
		},
		/// Offer was withdrawn
		OfferWithdrawn {
			who: T::AccountId,
			class: T::NftClassId,
			instance: T::NftInstanceId,
		},
		/// Offer was accepted
		OfferAccepted {
			who: T::AccountId,
			class: T::NftClassId,
			instance: T::NftInstanceId,
			amount: BalanceOf<T>,
			maker: T::AccountId,
		},
		/// Royalty hs been paid to the author
		RoyaltyPaid {
			class: T::NftClassId,
			instance: T::NftInstanceId,
			author: T::AccountId,
			royalty: u16,
			royalty_amount: BalanceOf<T>,
		},
		/// Marketplace data has been added
		RoyaltyAdded {
			class: T::NftClassId,
			instance: T::NftInstanceId,
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
		/// Class or instance does not exist
		ClassOrInstanceUnknown,
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
		class_id: T::NftClassId,
		instance_id: T::NftInstanceId,
		is_offer: bool,
	) -> DispatchResult {
		let owner = pallet_nft::Pallet::<T>::owner(class_id, instance_id).ok_or(Error::<T>::ClassOrInstanceUnknown)?;
		ensure!(buyer != owner, Error::<T>::BuyFromSelf);

		let owner_origin = T::Origin::from(RawOrigin::Signed(owner.clone()));

		let token_id = (class_id, instance_id);

		Prices::<T>::try_mutate(class_id, instance_id, |price| -> DispatchResult {
			let mut price = if is_offer {
				Offers::<T>::get(token_id, buyer.clone())
					.map(|o| o.amount)
					.ok_or(Error::<T>::UnknownOffer)?
			} else {
				price.take().ok_or(Error::<T>::NotForSale)?
			};

			// Settle royalty if set
			if let Some(instance_info) = MarketplaceInstances::<T>::get(class_id, instance_id) {
				let royalty = instance_info.royalty;
				let author = instance_info.author;

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
						class: class_id,
						instance: instance_id,
						author,
						royalty,
						royalty_amount,
					});
				}
			}

			// Send the net price from current to the previous owner
			<T as Config>::Currency::transfer(&buyer, &owner, price, ExistenceRequirement::KeepAlive)?;

			let to = T::Lookup::unlookup(buyer.clone());
			pallet_nft::Pallet::<T>::transfer(owner_origin, class_id, instance_id, to)?;

			Self::deposit_event(Event::TokenSold {
				owner,
				buyer,
				class: class_id,
				instance: instance_id,
				price,
			});
			Ok(())
		})
	}
}
