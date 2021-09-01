#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{
	dispatch::DispatchResult,
	traits::{Currency, ReservableCurrency},
	transactional,
};
use sp_runtime::traits::StaticLookup;
use frame_system::ensure_signed;
use weights::WeightInfo;

mod benchmarking;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> = <<T as pallet_nft::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NftClassIdOf<T> = pallet_nft::ClassIdOf<T>;
pub type NftTokenIdOf<T> = pallet_nft::TokenIdOf<T>;


// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::ExistenceRequirement};
	use frame_system::{ensure_signed, pallet_prelude::OriginFor};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn class_item_price)]
	/// Stores prices for NFT pools
	pub type TokenPrices<T: Config> =
		StorageDoubleMap<_, Twox64Concat, NftClassIdOf<T>, Twox64Concat, NftTokenIdOf<T>, BalanceOf<T>, OptionQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/*
		- quantity mint NFT
		- set price for NFT
		- make offer
		- set royalty
		- buy NFT
		- Accept offer 
		*/

		/// Pays a price to the current owner 
		/// Pays a royalty to the creator
		/// Transfers NFT ownership to the buyer
		/// Unlists the NFT
		///
		/// Parameters:
		/// - `owner`: The destination account a token will be sent to
		/// - `token`: unique identificator of a token
		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		#[transactional]
		pub fn buy(origin: OriginFor<T>, owner: T::AccountId, token: (NftClassIdOf<T>, NftTokenIdOf<T>)) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(sender != owner, Error::<T>::BuyFromSelf);

			TokenPrices::<T>::try_mutate_exists(token.0, token.1, |price| -> DispatchResult {
				let price = price.take().ok_or(Error::<T>::NotForSale)?;

				T::Currency::transfer(&sender, &owner, price, ExistenceRequirement::KeepAlive)?;
				
				let to = T::Lookup::unlookup(sender.clone());

				pallet_nft::Pallet::<T>::transfer(origin, to, token)?;
				
				Self::deposit_event(Event::TokenSold(owner, sender, token.0, token.1, price));
				Ok(())
			})
		}

		// Set trading price and allow sell
		// Setting to NULL will delist the token
		#[pallet::weight(<T as Config>::WeightInfo::set_price())]
		#[transactional]
		pub fn set_price(origin: OriginFor<T>, token: (NftClassIdOf<T>, NftTokenIdOf<T>), new_price: Option<BalanceOf<T>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(pallet_nft::Pallet::<T>::is_owner(&sender, token), Error::<T>::NotATokenOwner);

			TokenPrices::<T>::mutate_exists(token.0, token.1, |price| *price = new_price);

			Self::deposit_event(Event::TokenPriceUpdated(sender, token.0, token.1, new_price));

			Ok(())
		}

		// Set trading price to NULL and thus disallow trading
		#[pallet::weight(<T as Config>::WeightInfo::withdraw_from_market())]
		pub fn withdraw_from_market(origin: OriginFor<T>) -> DispatchResult {
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The price for a token was updated
		TokenPriceUpdated(T::AccountId, NftClassIdOf<T>, NftTokenIdOf<T>, Option<BalanceOf<T>>),
		/// Token was sold to a new owner
		TokenSold(T::AccountId, T::AccountId, NftClassIdOf<T>, NftTokenIdOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account is not the owner of the token
		NotATokenOwner,
		/// Trying to buy under the current price
		PriceTooLow,
		/// Cannot buy a token from yourself
		BuyFromSelf,
		/// Token is currently not for sale
		NotForSale,
	}
}

impl<T: Config> Pallet<T> {
	// Set a price for an NFT, can be done only if owner wants to sell
	fn do_set_price() -> DispatchResult {
		Ok(())
	}
}
