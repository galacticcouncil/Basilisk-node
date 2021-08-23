#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{
	dispatch::DispatchResult,
	traits::{Currency, ReservableCurrency},
};
use weights::WeightInfo;

mod benchmarking;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn class_item_price)]
	/// Stores prices for NFT pools
	pub type TokenPrice<T: Config> =
		StorageMap<_, Blake2_128Concat, u32 /*replace with NFT ID*/, BalanceOf<T>, OptionQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: ReservableCurrency<Self::AccountId>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// NFTs trading functions
		#[pallet::weight(<T as Config>::WeightInfo::buy())]
		pub fn buy(origin: OriginFor<T>) -> DispatchResult {
			// 1. Get NFT by id from storage from NFT pallet
			// 2. Get it's current price from this pallet storage (NULL = not for sale? automatically set with every new buy)
			// 3. Call transfer of funds buyer->seller and transfer of token seller->buyer (transactionally)
			todo! {}
		}

		// Set trading price for an NFT and allow sell
		#[pallet::weight(<T as Config>::WeightInfo::allow_sell())]
		pub fn allow_sell(origin: OriginFor<T>) -> DispatchResult {
			todo! {}
		}

		// Set trading price to NULL and thus disallow trading
		#[pallet::weight(<T as Config>::WeightInfo::withdraw_from_market())]
		pub fn withdraw_from_market(origin: OriginFor<T>) -> DispatchResult {
			todo! {}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		// NFT bought
	// NFT sold
	}

	#[pallet::error]
	pub enum Error<T> {}
}

impl<T: Config> Pallet<T> {
	// Set a price for an NFT, can be done only if owner wants to sell
	fn set_price() -> DispatchResult {
		todo! {}
	}
}
