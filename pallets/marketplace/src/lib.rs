#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, ExistenceRequirement, NamedReservableCurrency, OriginTrait},
	transactional,
};
use frame_system::{ensure_signed, RawOrigin};
use sp_runtime::{
	traits::{Saturating, StaticLookup, Zero},
	Percent,
};

use types::TokenInfo;
use weights::WeightInfo;

use pallet_nft::{types::ClassType, Instances};
use primitives::ReserveIdentifier;

mod benchmarking;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> =
	<<T as pallet_nft::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type TokenInfoOf<T> =
	TokenInfo<<T as frame_system::Config>::AccountId, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	pub const RESERVE_ID: ReserveIdentifier = ReserveIdentifier::Marketplace;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	/// Stores marketplace token info
	pub type Tokens<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::NftClassId, Twox64Concat, T::NftInstanceId, TokenInfoOf<T>, OptionQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
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
		#[transactional]
		pub fn buy(origin: OriginFor<T>, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			Self::do_buy(sender, class_id, instance_id, false)
		}

		/// Set trading price and allow sell
		/// Setting to NULL will delist the token
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		/// - `new_price`: price the token will be listed for
		#[pallet::weight(<T as Config>::WeightInfo::set_price())]
		#[transactional]
		pub fn set_price(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				pallet_uniques::Pallet::<T>::owner(class_id.into(), instance_id.into()) == Some(sender.clone()),
				Error::<T>::NotTheTokenOwner
			);

			Tokens::<T>::try_mutate(class_id, instance_id, |maybe_token_info| -> DispatchResult {
				let token_info = maybe_token_info.as_mut().ok_or(Error::<T>::NotListed)?;

				token_info.price = new_price;

				Ok(())
			})?;

			Self::deposit_event(Event::TokenPriceUpdated(sender, class_id, instance_id, new_price));

			Ok(())
		}

		/// Lists the token on Marketplace
		/// freezes the NFT from transfers
		/// and other modifications
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		#[pallet::weight(<T as Config>::WeightInfo::list())]
		#[transactional]
		pub fn list(origin: OriginFor<T>, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let class_type = pallet_nft::Classes::<T>::get(class_id)
				.map(|c| c.class_type)
				.ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

			ensure!(
				!Tokens::<T>::contains_key(class_id, instance_id),
				Error::<T>::AlreadyListed
			);
			ensure!(class_type == ClassType::Marketplace, Error::<T>::UnsupportedClassType);

			let royalty = Instances::<T>::get(class_id, instance_id)
				.map(|i| i.royalty)
				.ok_or(pallet_nft::Error::<T>::RoyaltyNotSet)?;
			let author = Instances::<T>::get(class_id, instance_id)
				.map(|i| i.author)
				.ok_or(pallet_nft::Error::<T>::AuthorNotSet)?;

			// Only token owner can list
			ensure!(
				pallet_uniques::Pallet::<T>::owner(class_id.into(), instance_id.into()) == Some(sender.clone()),
				Error::<T>::NotTheTokenOwner
			);

			Tokens::<T>::insert(
				class_id,
				instance_id,
				TokenInfo {
					price: None,
					offer: None,
				},
			);

			pallet_uniques::Pallet::<T>::freeze(origin.clone(), class_id.into(), instance_id.into())?;

			Self::deposit_event(Event::TokenListed(sender, class_id, instance_id, author, royalty));

			Ok(())
		}

		/// Unlists the token from Marketplace
		/// unfreezes the NFT from transfers
		/// and other modifications
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		#[pallet::weight(<T as Config>::WeightInfo::unlist())]
		#[transactional]
		pub fn unlist(origin: OriginFor<T>, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(Tokens::<T>::contains_key(class_id, instance_id), Error::<T>::NotListed);

			ensure!(
				pallet_uniques::Pallet::<T>::owner(class_id.into(), instance_id.into()) == Some(sender),
				Error::<T>::NotTheTokenOwner
			);

			Tokens::<T>::remove(class_id, instance_id);

			let owner = pallet_uniques::Pallet::<T>::owner(class_id.into(), instance_id.into())
				.ok_or(Error::<T>::ClassOrInstanceUnknown)?;

			pallet_uniques::Pallet::<T>::thaw(
				T::Origin::from(RawOrigin::Signed(owner)),
				class_id.into(),
				instance_id.into(),
			)?;

			Self::deposit_event(Event::TokenUnlisted(class_id, instance_id));

			Ok(())
		}

		/// Users can indicate what price they would be willing to pay for a token
		/// Price can be lower than current listing price
		/// Token does have to be listed on Marketplace but
		/// it doesn't have to be currently available for sale
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		/// - `amount`: The amount user is willing to pay
		/// - `expires`: The block until the current owner can accept the offer
		#[pallet::weight(<T as Config>::WeightInfo::make_offer())]
		#[transactional]
		pub fn make_offer(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			amount: BalanceOf<T>,
			expires: T::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(amount > Zero::zero(), Error::<T>::InvalidOffer);

			Tokens::<T>::try_mutate(class_id, instance_id, |maybe_token_info| -> DispatchResult {
				let token_info = maybe_token_info.as_mut().ok_or(Error::<T>::NotListed)?;

				if let Some(current_offer) = &token_info.offer {
					if amount < current_offer.1 {
						return Err(Error::<T>::InvalidOffer.into());
					}
				}

				<T as pallet_nft::Config>::Currency::reserve_named(&RESERVE_ID, &sender, amount)?;
				token_info.offer = Some((sender.clone(), amount, expires));

				Ok(())
			})?;

			Self::deposit_event(Event::OfferPlaced(sender, class_id, instance_id, amount));

			Ok(())
		}

		/// Reverse action to make_offer
		/// Removes an offer and unreserves funds
		/// Can be done by the offer maker or owner of the token
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		#[pallet::weight(<T as Config>::WeightInfo::withdraw_offer())]
		#[transactional]
		pub fn withdraw_offer(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			Tokens::<T>::try_mutate(class_id, instance_id, |maybe_token_info| -> DispatchResult {
				let token_info = maybe_token_info.as_mut().ok_or(Error::<T>::NotListed)?;
				let owner = pallet_uniques::Pallet::<T>::owner(class_id.into(), instance_id.into())
					.ok_or(Error::<T>::ClassOrInstanceUnknown)?;

				if let Some(current_offer) = &token_info.offer {
					if sender == current_offer.0 || sender == owner {
						<T as pallet_nft::Config>::Currency::unreserve_named(
							&RESERVE_ID,
							&current_offer.0,
							current_offer.1,
						);
						token_info.offer = None;
						Self::deposit_event(Event::OfferWithdrawn(sender, class_id, instance_id));
						Ok(())
					} else {
						Err(Error::<T>::UnknownOffer.into())
					}
				} else {
					Err(Error::<T>::InvalidOffer.into())
				}
			})
		}

		/// Accept an offer and process the trade
		///
		/// Parameters:
		/// - `class_id`: The identifier of a non-fungible token class
		/// - `instance_id`: The instance identifier of a class
		#[pallet::weight(<T as Config>::WeightInfo::accept_offer())]
		#[transactional]
		pub fn accept_offer(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			Tokens::<T>::try_mutate(class_id, instance_id, |maybe_token_info| -> DispatchResult {
				let token_info = maybe_token_info.as_mut().ok_or(Error::<T>::NotListed)?;

				if let Some(current_offer) = &token_info.offer {
					if current_offer.2 > <frame_system::Pallet<T>>::block_number() {
						<T as pallet_nft::Config>::Currency::unreserve_named(
							&RESERVE_ID,
							&current_offer.0,
							current_offer.1,
						);
						Self::do_buy(current_offer.0.clone(), class_id, instance_id, true)?;
						token_info.offer = None;
						Self::deposit_event(Event::OfferAccepted(sender, class_id, instance_id));
						Ok(())
					} else {
						return Err(Error::<T>::OfferExpired.into());
					}
				} else {
					return Err(Error::<T>::InvalidOffer.into());
				}
			})
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The price for a token was updated \[owner, class_id, instance_id, price\]
		TokenPriceUpdated(T::AccountId, T::NftClassId, T::NftInstanceId, Option<BalanceOf<T>>),
		/// Token was sold to a new owner \[owner, buyer, class_id, instance_id, price, author, royalty, royalty_amount\]
		TokenSold(
			T::AccountId,
			T::AccountId,
			T::NftClassId,
			T::NftInstanceId,
			BalanceOf<T>,
			Option<(T::AccountId, u8)>,
			BalanceOf<T>,
		),
		/// Token listed on Marketplace \[owner, class_id, instance_id, author royalty\]
		TokenListed(T::AccountId, T::NftClassId, T::NftInstanceId, T::AccountId, u8),
		/// Token listed on Marketplace \[class_id, instance_id\]
		TokenUnlisted(T::NftClassId, T::NftInstanceId),
		/// Offer was placed on a token \[offerer, class_id, instance_id, price\]
		OfferPlaced(T::AccountId, T::NftClassId, T::NftInstanceId, BalanceOf<T>),
		/// Offer was withdrawn \[sender, class_id, instance_id\]
		OfferWithdrawn(T::AccountId, T::NftClassId, T::NftInstanceId),
		/// Offer was accepted \[sender, class_id, instance_id\]
		OfferAccepted(T::AccountId, T::NftClassId, T::NftInstanceId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account is not the owner of the token
		NotTheTokenOwner,
		/// Cannot buy a token from yourself
		BuyFromSelf,
		/// Token is currently not for sale
		NotForSale,
		/// This class type cannot be listed on Marketplace
		UnsupportedClassType,
		/// Token is not listed on Marketplace
		NotListed,
		/// Class or instance does not exist
		ClassOrInstanceUnknown,
		/// Token already listed on marketplace
		AlreadyListed,
		/// Offer is None, zero or lower than the current one
		InvalidOffer,
		/// No offer for this token found from the user
		UnknownOffer,
		/// Offer is no longer valid
		OfferExpired,
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
		let owner = pallet_uniques::Pallet::<T>::owner(class_id.into(), instance_id.into())
			.ok_or(Error::<T>::ClassOrInstanceUnknown)?;
		ensure!(buyer != owner, Error::<T>::BuyFromSelf);

		let owner_origin = T::Origin::from(RawOrigin::Signed(owner.clone()));

		pallet_uniques::Pallet::<T>::thaw(owner_origin, class_id.into(), instance_id.into())?;

		Tokens::<T>::try_mutate(class_id, instance_id, |maybe_token_info| -> DispatchResult {
			let token_info = maybe_token_info.as_mut().ok_or(Error::<T>::NotListed)?;

			let mut price;

			if is_offer {
				if let Some(offer) = &token_info.offer {
					price = offer.1;
				} else {
					return Err(Error::<T>::InvalidOffer.into());
				}
			} else {
				price = token_info.price.take().ok_or(Error::<T>::NotForSale)?;
			}

			let royalty = Instances::<T>::get(class_id, instance_id)
				.map(|i| i.royalty)
				.ok_or(pallet_nft::Error::<T>::RoyaltyNotSet)?;
			let author = Instances::<T>::get(class_id, instance_id)
				.map(|i| i.author)
				.ok_or(pallet_nft::Error::<T>::AuthorNotSet)?;

			// Calculate royalty and subtract from price if author different from buyer
			let royalty_perc = Percent::from_percent(royalty);
			let royalty_amount = royalty_perc * price;

			if owner != author && royalty != 0u8 {
				price = price.saturating_sub(royalty_amount);

				// Send royalty to author
				<T as pallet_nft::Config>::Currency::transfer(
					&buyer,
					&author,
					royalty_amount,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			// Send the net price from current to the previous owner
			<T as pallet_nft::Config>::Currency::transfer(&buyer, &owner, price, ExistenceRequirement::KeepAlive)?;

			let to = T::Lookup::unlookup(buyer.clone());
			pallet_nft::Pallet::<T>::transfer(T::Origin::root(), class_id, instance_id, to)?;

			pallet_uniques::Pallet::<T>::freeze(
				T::Origin::from(RawOrigin::Signed(buyer.clone())),
				class_id.into(),
				instance_id.into(),
			)?;

			Self::deposit_event(Event::TokenSold(
				owner,
				buyer,
				class_id,
				instance_id,
				price,
				Some((author, royalty)),
				royalty_amount,
			));
			Ok(())
		})
	}
}
