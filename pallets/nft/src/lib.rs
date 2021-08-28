#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
	ensure,
	traits::{Currency, ExistenceRequirement, ReservableCurrency},
	transactional,
};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{StaticLookup, Zero},
	RuntimeDebug,
};
use sp_std::vec::Vec;
use weights::WeightInfo;

mod benchmarking;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct ClassData {
	pub is_pool: bool, // NFT pools for tokenized merch
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct TokenData {
	pub locked: bool, // token locking will be used in the nft auctions
}

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
	pub type PoolItemPrice<T: Config> = StorageMap<_, Blake2_128Concat, ClassIdOf<T>, BalanceOf<T>, ValueQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config<ClassData = ClassData, TokenData = TokenData> {
		type Currency: ReservableCurrency<Self::AccountId>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		// How much will be bonded
		#[pallet::constant]
		type ClassBondAmount: Get<BalanceOf<Self>>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///////////////////////////////////////////////////
		//
		// Generic methods for NFT handling
		//
		///////////////////////////////////////////////////

		/// Creates an NFT class
		/// This is necessary as the first step, because tokens will be minted as part of this class
		/// An amount X (ClassBondAmount) is reserved
		///
		/// Parameters:
		/// - `metadata`: Arbitrary info/description of a class
		/// - `data`: Field(s) defined in the ClassData struct
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, metadata: Vec<u8>, data: T::ClassData) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			T::Currency::reserve(&sender, T::ClassBondAmount::get())?;
			let class_id = orml_nft::Pallet::<T>::create_class(&sender, metadata, data)?;

			Self::deposit_event(Event::TokenClassCreated(sender, class_id));
			Ok(().into())
		}

		/// NFT is minted in the specified class
		///
		/// Parameters:
		/// - `class_id`: identificator of a class
		/// - `metadata`: Arbitrary info/description of a token
		/// - `data`: Field(s) defined in the TokenData struct
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
			metadata: Vec<u8>,
			token_data: TokenData
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(sender == class_info.owner, Error::<T>::NotClassOwner);
			let data = token_data;
			let token_id = orml_nft::Pallet::<T>::mint(&sender, class_id, metadata.clone(), data.clone())?;

			Self::deposit_event(Event::TokenMinted(sender, class_id, token_id));
			Ok(().into())
		}

		/// Transfers NFT from account A to account B
		/// Only the owner can send their NFT to another account
		///
		/// Parameters:
		/// - `dest`: The destination account a token will be sent to
		/// - `token`: unique identificator of a token
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let to: T::AccountId = T::Lookup::lookup(dest)?;
			ensure!(sender != to, Error::<T>::CannotSendToSelf);
			let token_info = orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			ensure!(sender == token_info.owner, Error::<T>::NotTokenOwner);
			ensure!(!token_info.data.locked, Error::<T>::TokenLocked);
			orml_nft::Pallet::<T>::transfer(&sender, &to, token)?;
			Self::deposit_event(Event::TokenTransferred(sender, to, token.0, token.1));
			Ok(().into())
		}

		/// Removes a token from existence
		///
		/// Parameters:
		/// - `token`: unique identificator of a token
		#[pallet::weight(<T as Config>::WeightInfo::burn())]
		#[transactional]
		pub fn burn(origin: OriginFor<T>, token: (T::ClassId, T::TokenId)) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let token_info = orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			ensure!(sender == token_info.owner, Error::<T>::NotTokenOwner);
			ensure!(!token_info.data.locked, Error::<T>::TokenLocked);
			orml_nft::Pallet::<T>::burn(&sender, token)?;
			Self::deposit_event(Event::TokenBurned(sender, token.0, token.1));
			Ok(().into())
		}

		/// Removes a class from existence
		/// Returns the bond amount
		///
		/// Parameters:
		/// - `class_id`: unique identificator of a class
		#[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
		#[transactional]
		pub fn destroy_class(origin: OriginFor<T>, class_id: ClassIdOf<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(class_info.total_issuance == Zero::zero(), Error::<T>::NonZeroIssuance);
			orml_nft::Pallet::<T>::destroy_class(&sender, class_id)?;
			T::Currency::unreserve(&sender, T::ClassBondAmount::get());
			Self::deposit_event(Event::TokenClassDestroyed(sender, class_id));
			Ok(().into())
		}

		///////////////////////////////////////////////////
		//
		// Basilisk-specific methods for NFT handling
		//
		///////////////////////////////////////////////////

		/// Similar method to create_/destroy_class
		/// The difference between a pool and a class in this case is that
		/// a price has to be specified for each pool. Any NFT within this class
		/// will have this exact constant price
		///
		/// Parameters:
		/// - `metadata`: Arbitrary info/description of a pool
		/// - `data`: Field(s) defined in the ClassData struct
		/// - `price`: Price of each individual NFT
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			metadata: Vec<u8>,
			data: T::ClassData,
			price: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			T::Currency::reserve(&sender, T::ClassBondAmount::get())?;
			let class_id = orml_nft::Pallet::<T>::create_class(&sender, metadata, data)?;
			PoolItemPrice::<T>::insert(class_id, price);

			Self::deposit_event(Event::TokenPoolCreated(sender, class_id));
			Ok(().into())
		}

		/// Removes a pool from existence
		/// Returns the bond amount
		///
		/// Parameters:
		/// - `class_id`: unique identificator of a class
		#[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
		#[transactional]
		pub fn destroy_pool(origin: OriginFor<T>, class_id: ClassIdOf<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(class_info.total_issuance == Zero::zero(), Error::<T>::NonZeroIssuance);
			orml_nft::Pallet::<T>::destroy_class(&sender, class_id)?;
			PoolItemPrice::<T>::remove(class_id);
			T::Currency::unreserve(&sender, T::ClassBondAmount::get());
			Self::deposit_event(Event::TokenPoolDestroyed(sender, class_id));
			Ok(().into())
		}

		/// NFTs can be bought from a pool for a constant price
		///
		/// Parameters:
		/// - `token`: unique identificator of a token
		#[pallet::weight(<T as Config>::WeightInfo::buy_from_pool())]
		#[transactional]
		pub fn buy_from_pool(origin: OriginFor<T>, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Pallet::<T>::classes(token.0).ok_or(Error::<T>::ClassNotFound)?;
			let token_info = orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			let price = Self::class_item_price(token.0);
			ensure!(class_info.data.is_pool, Error::<T>::NotAPool);
			ensure!(class_info.owner == token_info.owner, Error::<T>::TokenAlreadyHasAnOwner);
			ensure!(sender != token_info.owner, Error::<T>::CannotBuyOwnToken);
			ensure!(!token_info.data.locked, Error::<T>::TokenLocked);

			orml_nft::Pallet::<T>::transfer(&token_info.owner, &sender, token)?;
			T::Currency::transfer(&sender, &token_info.owner, price, ExistenceRequirement::KeepAlive)?;
			Self::deposit_event(Event::BoughtFromPool(class_info.owner, sender, token.0, token.1));
			Ok(())
		}

		/// Owned NFTs can be sold back to the pool for the original price
		///
		/// Parameters:
		/// - `token`: unique identificator of a token
		#[pallet::weight(<T as Config>::WeightInfo::sell_to_pool())]
		#[transactional]
		pub fn sell_to_pool(origin: OriginFor<T>, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Pallet::<T>::classes(token.0).ok_or(Error::<T>::ClassNotFound)?;
			let token_info = orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			let price = Self::class_item_price(token.0);
			ensure!(class_info.data.is_pool, Error::<T>::NotAPool);
			ensure!(class_info.owner != token_info.owner, Error::<T>::CannotSellPoolToken);
			ensure!(sender == token_info.owner, Error::<T>::NotTokenOwner);
			ensure!(!token_info.data.locked, Error::<T>::TokenLocked);

			orml_nft::Pallet::<T>::transfer(&sender, &class_info.owner, token)?;
			T::Currency::transfer(&class_info.owner, &sender, price, ExistenceRequirement::KeepAlive)?;
			Self::deposit_event(Event::SoldToPool(sender, class_info.owner, token.0, token.1));
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenClassCreated(T::AccountId, T::ClassId),
		TokenMinted(T::AccountId, T::ClassId, T::TokenId),
		TokenMintedLockToggled(T::AccountId, T::ClassId, T::TokenId, bool),
		TokenTransferred(T::AccountId, T::AccountId, T::ClassId, T::TokenId),
		TokenBurned(T::AccountId, T::ClassId, T::TokenId),
		TokenClassDestroyed(T::AccountId, T::ClassId),
		BoughtFromPool(T::AccountId, T::AccountId, T::ClassId, T::TokenId),
		SoldToPool(T::AccountId, T::AccountId, T::ClassId, T::TokenId),
		TokenPoolCreated(T::AccountId, T::ClassId),
		TokenPoolDestroyed(T::AccountId, T::ClassId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The class does not exist
		ClassNotFound,
		/// The token does not exist
		TokenNotFound,
		/// Not the class owner
		NotClassOwner,
		/// Not the token owner
		NotTokenOwner,
		/// The token class is not empty
		NonZeroIssuance,
		/// Token is currently locked
		TokenLocked,
		/// A token can not be transferred to self
		CannotSendToSelf,
		/// A user cannot buy already owned token
		CannotBuyOwnToken,
		/// Token has been already bought from a pool
		TokenAlreadyHasAnOwner,
		/// A token still owned by class owner
		CannotSellPoolToken,
		/// Class wasn't created as a pool
		NotAPool,
		/// Metadata exceed the allowed length
		MetadataTooLong,
	}
}

impl<T: Config> Pallet<T> {
	pub fn is_owner(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
		orml_nft::Pallet::<T>::is_owner(account, token)
	}

	pub fn is_locked(token: (T::ClassId, T::TokenId)) -> Result<bool, DispatchError> {
		let token_info = orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
		Ok(token_info.data.locked)
	}

	pub fn toggle_lock(account: &T::AccountId, token_id: (T::ClassId, T::TokenId)) -> DispatchResult {
		let _class_info = orml_nft::Pallet::<T>::classes(token_id.0).ok_or(Error::<T>::ClassNotFound)?;
		orml_nft::Tokens::<T>::mutate_exists(token_id.0, token_id.1, |token| -> DispatchResult {
			if let Some(ref mut token) = token {
				ensure!(*account == token.owner, Error::<T>::NotTokenOwner);
				token.data.locked ^= true; // Toggle
				Self::deposit_event(Event::TokenMintedLockToggled(
					account.clone(),
					token_id.0,
					token_id.1,
					token.data.locked,
				));
			}
			Ok(())
		})
	}
}
