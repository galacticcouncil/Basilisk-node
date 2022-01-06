#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::convert::TryInto;

use frame_support::{dispatch::DispatchResult, ensure, transactional, traits::{Currency, Get}, BoundedVec};
use frame_system::ensure_signed;
use primitives::nft::ClassType;
use sp_runtime::{SaturatedConversion, traits::Zero, DispatchError};

use types::{RedeemablesClassInfo, BondingCurve};
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
	use crate::types::RedeemablesClassInfo;

use super::*;
	use frame_support::{pallet_prelude::*, PalletId, traits::ExistenceRequirement};
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config<ClassType = primitives::nft::ClassType> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		#[pallet::constant]
    	type PalletId: Get<PalletId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn classes_redeemables)]
	/// Stores class info
	pub type ClassesRedeemables<T: Config> = StorageMap<_, Twox64Concat, T::NftClassId, RedeemablesClassInfo>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Buys from a pool
		///
		#[pallet::weight(0)]
		#[transactional]
		pub fn buy(origin: OriginFor<T>, class_id: T::NftClassId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::mint_for_redeemables(sender.clone(), class_id)?;

			let class_info = Self::classes_redeemables(class_id)
				.ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

			let price = class_info.price().saturated_into();
			
			#[cfg(test)]
			println!("Bought! issuance #: {:?} / {:?}, price: {:?} BSX", class_info.issued, class_info.max_supply, price);

			<T as pallet_nft::Config>::Currency::transfer(
				&sender,
				&Self::module_account(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;
			
			Self::deposit_event(Event::Bought(sender, class_id, price));

			Ok(())
		}

		/// Sells into a bonding curve.
		///
		#[pallet::weight(0)]
		#[transactional]
		pub fn sell(origin: OriginFor<T>, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			//pallet_nft::Pallet::<T>::burn_for_redeemables(sender.clone(), class_id, instance_id, false)?;

			ClassesRedeemables::<T>::try_mutate(class_id, |maybe_info| -> DispatchResult {
				let info = maybe_info.as_mut().ok_or(Error::<T>::ClassUnknown)?;

				info.issued = info.issued.saturating_sub(One::one());

				Ok(())
			})?;

			let class_info = Self::classes_redeemables(class_id)
				.ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

			let price = class_info.price().saturated_into();

			#[cfg(test)]
			println!("Sold! issuance #: {:?} / {:?}, price: {:?} BSX", class_info.issued, class_info.max_supply, price);

			<T as pallet_nft::Config>::Currency::transfer(
				&Self::module_account(),
				&sender,
				price,
				ExistenceRequirement::KeepAlive,
			)?;
			
			Self::deposit_event(Event::Sold(sender, class_id, price));

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
			//name, street + number, city, county, zip, country: BoundedVec,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			pallet_nft::Pallet::<T>::burn(origin, class_id, instance_id)?;

			#[cfg(test)]
			println!("Redeemed! issuance #: {:?} / {:?}", class_id, instance_id);

			ClassesRedeemables::<T>::try_mutate(class_id, |maybe_info| -> DispatchResult {
				let info = maybe_info.as_mut().ok_or(Error::<T>::ClassUnknown)?;

				info.redeemed = info.redeemed.saturating_add(One::one());

				Ok(())
			})?;
			
			
			Self::deposit_event(Event::Redeemed(sender, class_id, instance_id));

			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		///
		Bought(T::AccountId, T::NftClassId, BalanceOf<T>),
		///
		Sold(T::AccountId, T::NftClassId, BalanceOf<T>),
		///
		Redeemed(T::AccountId, T::NftClassId, T::NftInstanceId),
	}

	#[pallet::error]
	pub enum Error<T> {
		Overflow,
		ClassUnknown,
		ReachedMaxSupply,
	}
}

impl<T: Config> Pallet<T> {
	fn module_account() -> T::AccountId {
        T::PalletId::get().into_account()
    }

	pub fn create_class_for_redeemables(max_supply: u32, curve: BondingCurve) -> Result<T::NftClassId, DispatchError> {
		let (class_id, class_type) = pallet_nft::Pallet::<T>::do_create_class(Default::default(), ClassType::Redeemable, Default::default())?;
	
		ClassesRedeemables::<T>::insert(
			class_id,
			RedeemablesClassInfo {
				class_type,
				max_supply,
				redeemed: Zero::zero(),
				issued: Zero::zero(),
				curve,
			},
		);
		
		Ok(class_id)
	}
	
	pub fn mint_for_redeemables(
		owner: T::AccountId,
		class_id: T::NftClassId,
	) -> Result<T::NftInstanceId, DispatchError> {
		let class_type = Self::classes_redeemables(class_id)
			.map(|c| c.class_type)
			.ok_or(Error::<T>::ClassUnknown)?;
		
		ClassesRedeemables::<T>::try_mutate(class_id, |maybe_info| -> DispatchResult {
			let info = maybe_info.as_mut().ok_or(Error::<T>::ClassUnknown)?;
	
			ensure!(
				info.issued.saturating_add(1) < info.max_supply,
				Error::<T>::ReachedMaxSupply
			);
	
			info.issued = info.issued.saturating_add(One::one());
	
			Ok(())
		})?;
	
		let metadata: BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit> =
			b"metadata".to_vec().try_into().unwrap();
	
		let instance_id = pallet_nft::Pallet::<T>::do_mint(owner.clone(), class_id, metadata)?;
		
		Ok(instance_id)
	}
	
	pub fn burn_for_redeemables (
		sender: T::AccountId,
		class_id: T::NftClassId,
		instance_id: T::NftInstanceId,
		redeem: bool,
	) -> DispatchResult {
	
		pallet_nft::Pallet::<T>::do_burn(
			sender,
			class_id,
			instance_id,
		)?;
	
		ClassesRedeemables::<T>::try_mutate(class_id, |maybe_info| -> DispatchResult {
			let info = maybe_info.as_mut().ok_or(Error::<T>::ClassUnknown)?;
	
			if redeem {
				info.redeemed = info.redeemed.saturating_add(One::one())
			} else {
				info.issued = info.issued.saturating_sub(One::one())
			}
	
			Ok(())
		})?;
		
		Ok(())
	}
}


