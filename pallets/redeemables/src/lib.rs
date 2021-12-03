#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, transactional, traits::{Currency, Get}};
use frame_system::ensure_signed;
use sp_runtime::SaturatedConversion;

use weights::WeightInfo;

mod benchmarking;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as pallet_nft::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;
use sp_runtime::traits::{One, AccountIdConversion};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		#[pallet::constant]
    	type PalletId: Get<PalletId>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Buys from a pool
		///
		#[pallet::weight(0)]
		#[transactional]
		pub fn buy(origin: OriginFor<T>, class_id: T::NftClassId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			pallet_nft::Pallet::<T>::mint_for_redeemables(sender.clone(), class_id)?;

			let class_info = pallet_nft::Pallet::<T>::classes_redeemables(class_id)
				.ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

			let price = class_info.price();

			println!("issuance #: {:?} / {:?}, price: {:?} BSX", class_info.issued, class_info.max_supply, price);

			// <T as pallet_nft::Config>::Currency::transfer(
			// 	&sender,
			// 	&Self::module_account(),
			// 	price,
			// 	ExistenceRequirement::KeepAlive,
			// )?;
			
			//Self::deposit_event(Event::Bought(sender, class_id, price));

			Ok(())
		}

		/// Sells into a bonding curve.
		///
		#[pallet::weight(0)]
		#[transactional]
		pub fn sell(origin: OriginFor<T>, class_id: T::NftClassId, curve_id: u32) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::deposit_event(Event::Sold(sender, class_id, curve_id));

			Ok(())
		}

		/// Redeem token = burn + remark with postal address
		///
		#[pallet::weight(0)]
		#[transactional]
		pub fn redeem(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			curve_id: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::deposit_event(Event::Redeemed(sender, curve_id, class_id, instance_id));

			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		///
		Bought(T::AccountId, T::NftClassId, BalanceOf<T>),
		///
		Sold(T::AccountId, T::NftClassId, u32),
		///
		Redeemed(T::AccountId, u32, T::NftClassId, T::NftInstanceId),
	}

	#[pallet::error]
	pub enum Error<T> {
		Overflow,
	}
}

impl<T: Config> Pallet<T> {
	fn _module_account() -> T::AccountId {
        T::PalletId::get().into_account()
    }
}
