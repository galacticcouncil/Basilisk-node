#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::Decode;
use frame_support::{
	dispatch::DispatchResult,
	traits::{tokens::nonfungibles::Inspect, Currency},
	transactional,
};
use frame_system::{ensure_signed, RawOrigin};
use sp_runtime::{
	traits::{Saturating, StaticLookup},
	Percent,
};
use sp_std::{convert::TryInto, vec::Vec};

use types::TokenInfo;
use weights::WeightInfo;

use pallet_nft::types::ClassType;

mod benchmarking;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> =
	<<T as pallet_nft::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type TokenInfoOf<T> = TokenInfo<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	/// Stores marketplace token info
	pub type Tokens<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::InstanceId, TokenInfoOf<T>, OptionQuery>;

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
		/// - `owner`: The destination account a token will be sent to
		/// - `class_id`: The class of the asset
		/// - `instance_id`: The instance of the asset
		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		pub fn buy(origin: OriginFor<T>, class_id: T::ClassId, instance_id: T::InstanceId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let owner = pallet_uniques::Pallet::<T>::owner(class_id, instance_id).ok_or(Error::<T>::Unknown)?;

			ensure!(sender != owner, Error::<T>::BuyFromSelf);

			let to = T::Lookup::unlookup(sender.clone());
			let owner_origin = T::Origin::from(RawOrigin::Signed(owner.clone()));
			let class_owner = pallet_uniques::Pallet::<T>::class_owner(&class_id).ok_or(Error::<T>::Unknown)?;
			let class_owner_origin = T::Origin::from(RawOrigin::Signed(class_owner.clone()));

			pallet_uniques::Pallet::<T>::thaw(class_owner_origin.clone(), class_id, instance_id)?;

			Tokens::<T>::try_mutate(class_id, instance_id, |maybe_token_info| -> DispatchResult {
				let token_info = maybe_token_info.as_mut().ok_or(Error::<T>::Unknown)?;

				let mut price = token_info.price.take().ok_or(Error::<T>::NotForSale)?;

				// Calculate royalty and subtract from price if author different from buyer
				if owner != token_info.author && token_info.royalty != 0u8 {
					let royalty_perc = Percent::from_percent(token_info.royalty);
					let royalty_amount = royalty_perc * price;
					price = price.saturating_sub(royalty_amount);

					// Send royalty to author
					<T as pallet_nft::Config>::Currency::transfer(
						&sender,
						&token_info.author,
						royalty_amount,
						ExistenceRequirement::KeepAlive,
					)?;
				}

				// Send the net price from current to the previous owner
				<T as pallet_nft::Config>::Currency::transfer(&sender, &owner, price, ExistenceRequirement::KeepAlive)?;

				pallet_nft::Pallet::<T>::transfer(owner_origin.clone(), class_id, instance_id, to)?;

				pallet_uniques::Pallet::<T>::freeze(class_owner_origin, class_id, instance_id)?;

				Self::deposit_event(Event::TokenSold(owner, sender, class_id, instance_id, price));
				Ok(())
			})
		}

		/// Set trading price and allow sell
		/// Setting to NULL will delist the token
		///
		/// Parameters:
		/// - `class_id`: The class of the asset
		/// - `instance_id`: The instance of the asset
		/// - `new_price`: price the token will be listed for
		#[pallet::weight(<T as Config>::WeightInfo::set_price())]
		#[transactional]
		pub fn set_price(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			instance_id: T::InstanceId,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Tokens::<T>::contains_key(class_id, instance_id), Error::<T>::NotListed);

			ensure!(
				pallet_uniques::Pallet::<T>::owner(class_id, instance_id) == Some(sender.clone()),
				Error::<T>::NotTheTokenOwner
			);

			Tokens::<T>::try_mutate(class_id, instance_id, |maybe_token_info| -> DispatchResult {
				let token_info = maybe_token_info.as_mut().ok_or(Error::<T>::Unknown)?;

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
		/// - `class_id`: The class of the asset
		/// - `instance_id`: The instance of the asset
		#[pallet::weight(<T as Config>::WeightInfo::list())]
		#[transactional]
		pub fn list(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			instance_id: T::InstanceId,
			author: T::AccountId,
			royalty: u8,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(
				!Tokens::<T>::contains_key(class_id, instance_id),
				Error::<T>::AlreadyListed
			);

			ensure!(
				pallet_uniques::Pallet::<T>::owner(class_id, instance_id) == Some(sender.clone()),
				Error::<T>::NotTheTokenOwner
			);

			// Check if class type can be decoded to one of available types
			Self::get_class_type(class_id)?;

			Tokens::<T>::insert(
				class_id,
				instance_id,
				TokenInfo {
					author: author,
					royalty: royalty,
					price: None,
				},
			);

			pallet_uniques::Pallet::<T>::freeze(origin.clone(), class_id, instance_id)?;

			Self::deposit_event(Event::TokenListed(sender, class_id, instance_id));

			Ok(())
		}

		/// Unlists the token from Marketplace
		/// unfreezes the NFT from transfers
		/// and other modifications
		///
		/// Parameters:
		/// - `class_id`: The class of the asset
		/// - `instance_id`: The instance of the asset
		#[pallet::weight(<T as Config>::WeightInfo::unlist())]
		#[transactional]
		pub fn unlist(origin: OriginFor<T>, class_id: T::ClassId, instance_id: T::InstanceId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(Tokens::<T>::contains_key(class_id, instance_id), Error::<T>::NotListed);

			ensure!(
				pallet_uniques::Pallet::<T>::owner(class_id, instance_id) == Some(sender.clone()),
				Error::<T>::NotTheTokenOwner
			);

			let class_owner = pallet_uniques::Pallet::<T>::class_owner(&class_id).ok_or(Error::<T>::Unknown)?;
			let class_owner_origin = T::Origin::from(RawOrigin::Signed(class_owner.clone()));

			Tokens::<T>::remove(class_id, instance_id);

			pallet_uniques::Pallet::<T>::thaw(class_owner_origin.clone(), class_id, instance_id)?;

			Self::deposit_event(Event::TokenListed(sender, class_id, instance_id));

			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The price for a token was updated
		TokenPriceUpdated(T::AccountId, T::ClassId, T::InstanceId, Option<BalanceOf<T>>),
		/// Token was sold to a new owner
		TokenSold(T::AccountId, T::AccountId, T::ClassId, T::InstanceId, BalanceOf<T>),
		/// Token listed on Marketplace
		TokenListed(T::AccountId, T::ClassId, T::InstanceId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account is not the owner of the token
		NotTheTokenOwner,
		/// Cannot buy a token from yourself
		BuyFromSelf,
		/// Token is currently not for sale
		NotForSale,
		/// String exceeds allowed length
		TooLong,
		/// This class type cannot be listed on Marketplace
		UnsupportedClassType,
		/// Royalty not in 0-99 range
		NotInRange,
		/// Token is not listed on Marketplace
		NotListed,
		/// Token info does not exist
		Unknown,
		/// Token already listed on marketplace
		AlreadyListed,
	}
}

impl<T: Config> Pallet<T> {
	fn get_class_type(class_id: T::ClassId) -> Result<ClassType, Error<T>> {
		let mut class_type_vec: &[u8] =
			&pallet_uniques::Pallet::<T>::class_attribute(&class_id, b"type").unwrap_or(b"Unknown".to_vec());

		if let Some(class_type) = ClassType::decode(&mut class_type_vec).ok() {
			Ok(class_type)
		} else {
			Err(Error::<T>::UnsupportedClassType)
		}
	}
}
