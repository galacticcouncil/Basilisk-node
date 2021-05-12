#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
	ensure,
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

pub type Balance = u128;
pub type ClassData = u32;
pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct TokenData {
	pub locked: bool,
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

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config<ClassData = ClassData, TokenData = TokenData> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		pub fn create_class(origin: OriginFor<T>, metadata: Vec<u8>, data: T::ClassData) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let class_id = orml_nft::Module::<T>::create_class(&sender, metadata, data)?;
			Self::deposit_event(Event::NFTTokenClassCreated(sender, class_id));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: <T as orml_nft::Config>::ClassId,
			metadata: Vec<u8>,
			token_data: TokenData,
			quantity: u32,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(quantity > Zero::zero(), Error::<T>::InvalidQuantity);
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(sender == class_info.owner, Error::<T>::NoPermission);
			let mut data = token_data;
			data.locked = false;
			for _ in 0..quantity {
				orml_nft::Module::<T>::mint(&sender, class_id, metadata.clone(), data.clone())?;
			}
			Self::deposit_event(Event::NFTTokenMinted(sender, class_id, quantity));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			token: (T::ClassId, T::TokenId),
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let _class_info = orml_nft::Module::<T>::classes(token.0).ok_or(Error::<T>::ClassNotFound)?;
			let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			ensure!(sender == token_info.owner, Error::<T>::NoPermission);
			ensure!(!token_info.data.locked, Error::<T>::TokenLocked);
			let to: T::AccountId = T::Lookup::lookup(dest)?;
			orml_nft::Module::<T>::transfer(&sender, &to, token)?;
			Self::deposit_event(Event::NFTTokenTransferred(sender, to, token.0, token.1));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::burn())]
		pub fn burn(origin: OriginFor<T>, token: (T::ClassId, T::TokenId)) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let _class_info = orml_nft::Module::<T>::classes(token.0).ok_or(Error::<T>::ClassNotFound)?;
			let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			ensure!(sender == token_info.owner, Error::<T>::NoPermission);
			ensure!(!token_info.data.locked, Error::<T>::TokenLocked);
			orml_nft::Module::<T>::burn(&sender, token)?;
			Self::deposit_event(Event::NFTTokenBurned(sender, token.0, token.1));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
		pub fn destroy_class(origin: OriginFor<T>, class_id: T::ClassId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(sender == class_info.owner, Error::<T>::NoPermission);
			ensure!(
				class_info.total_issuance == Zero::zero(),
				Error::<T>::CannotDestroyClass
			);
			orml_nft::Module::<T>::destroy_class(&sender, class_id)?;
			Self::deposit_event(Event::NFTTokenClassDestroyed(sender, class_id));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		NFTTokenClassCreated(T::AccountId, T::ClassId),
		NFTTokenMinted(T::AccountId, T::ClassId, u32),
		NFTTokenMintedLockToggled(T::AccountId, T::ClassId, T::TokenId, bool),
		NFTTokenTransferred(T::AccountId, T::AccountId, T::ClassId, T::TokenId),
		NFTTokenBurned(T::AccountId, T::ClassId, T::TokenId),
		NFTTokenClassDestroyed(T::AccountId, T::ClassId),
	}

	#[pallet::error]
	pub enum Error<T> {
		ClassNotFound,
		TokenNotFound,
		NoPermission,
		CannotDestroyClass,
		TokenLocked,
		InvalidQuantity,
	}
}

impl<T: Config> Pallet<T> {
	pub fn is_owner(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
		orml_nft::Module::<T>::is_owner(account, token)
	}

	pub fn is_locked(token: (T::ClassId, T::TokenId)) -> Result<bool, DispatchError> {
		let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
		Ok(token_info.data.locked)
	}

	pub fn toggle_lock(account: &T::AccountId, token_id: (T::ClassId, T::TokenId)) -> DispatchResult {
		let _class_info = orml_nft::Module::<T>::classes(token_id.0).ok_or(Error::<T>::ClassNotFound)?;
		orml_nft::Tokens::<T>::mutate_exists(token_id.0, token_id.1, |token| -> DispatchResult {
			if let Some(ref mut token) = token {
				ensure!(*account == token.owner, Error::<T>::NoPermission);
				token.data.locked ^= true; // Toggle
						   // fix clone
				Self::deposit_event(Event::NFTTokenMintedLockToggled(
					account.clone(),
					token_id.0,
					token_id.1,
					token.data.locked,
				));
			}
			Ok(())
		})?;
		Ok(())
	}
}
