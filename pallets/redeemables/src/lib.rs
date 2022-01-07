#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::convert::TryInto;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, Get},
	transactional,
};
use frame_system::ensure_signed;
use primitives::nft::ClassType;
use sp_runtime::{traits::Zero, SaturatedConversion};

use types::BondingCurve;
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
use sp_runtime::traits::{AccountIdConversion, One};

#[frame_support::pallet]
pub mod pallet {
	use crate::types::RedeemablesClassInfo;

	use super::*;
	use frame_support::{pallet_prelude::*, traits::ExistenceRequirement, PalletId};
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

			let pallet_account = Self::pallet_account();

			ClassesRedeemables::<T>::try_mutate(class_id, |maybe_info| -> DispatchResult {
				let info = maybe_info.as_mut().ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

				ensure!(
					info.issued.saturating_add(One::one()) < info.max_supply,
					Error::<T>::ReachedMaxSupply
				);

				info.issued = info.issued.saturating_add(One::one());

				Ok(())
			})?;

			let metadata: pallet_nft::BoundedVecOfUnq<T> = b"metadata".to_vec().try_into().unwrap();

			let instance_id = pallet_nft::Pallet::<T>::do_mint(pallet_account.clone(), class_id, metadata)?;

			let class_info = Self::classes_redeemables(class_id).ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

			let price = class_info.price().saturated_into();

			#[cfg(test)]
			println!(
				"Bought! issuance #: {:?} / {:?}, price: {:?} BSX",
				class_info.issued, class_info.max_supply, price
			);

			<T as pallet_nft::Config>::Currency::transfer(
				&sender,
				&Self::pallet_account(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			pallet_nft::Pallet::<T>::do_transfer(class_id, instance_id, pallet_account, sender.clone())?;

			Self::deposit_event(Event::Bought(sender, class_id, price));

			Ok(())
		}

		/// Sells into a bonding curve.
		///
		#[pallet::weight(0)]
		#[transactional]
		pub fn sell(origin: OriginFor<T>, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ClassesRedeemables::<T>::try_mutate(class_id, |maybe_info| -> DispatchResult {
				let info = maybe_info.as_mut().ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

				info.issued = info.issued.saturating_sub(One::one());

				Ok(())
			})?;

			let class_info = Self::classes_redeemables(class_id).ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

			let price = class_info.price().saturated_into();

			pallet_nft::Pallet::<T>::do_burn(sender.clone(), class_id, instance_id)?;

			#[cfg(test)]
			println!(
				"Sold! issuance #: {:?} / {:?}, price: {:?} BSX",
				class_info.issued, class_info.max_supply, price
			);

			<T as pallet_nft::Config>::Currency::transfer(
				&Self::pallet_account(),
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

			pallet_nft::Pallet::<T>::do_burn(sender.clone(), class_id, instance_id)?;

			ClassesRedeemables::<T>::try_mutate(class_id, |maybe_info| -> DispatchResult {
				let info = maybe_info.as_mut().ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

				info.redeemed = info.redeemed.saturating_add(One::one());

				#[cfg(test)]
				println!(
					"Redeemed! issuance #: {:?} / {:?} redeemed (ooc): {:?}",
					info.issued, info.max_supply, info.redeemed
				);

				Ok(())
			})?;

			Self::deposit_event(Event::Redeemed(sender, class_id, instance_id));

			Ok(())
		}

		/// Adds class details for redeemables pool
		///
		#[pallet::weight(0)]
		#[transactional]
		pub fn add_redeemables_class_info(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			max_supply: u32,
			curve: BondingCurve,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let class_type = pallet_nft::Classes::<T>::get(class_id)
				.map(|c| c.class_type)
				.ok_or(pallet_nft::Error::<T>::ClassUnknown)?;

			ensure!(class_type == ClassType::Redeemable, Error::<T>::WrongClassType);

			let class_info = RedeemablesClassInfo {
				class_type,
				max_supply,
				redeemed: Zero::zero(),
				issued: Zero::zero(),
				curve,
			};

			ClassesRedeemables::<T>::insert(class_id, class_info.clone());

			Self::deposit_event(Event::ClassInfoAdded(sender, class_id, class_info));

			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// NFT item bought from pool \[who, class, amount\]
		Bought(T::AccountId, T::NftClassId, BalanceOf<T>),
		/// NFT item sold to pool \[who, class, amount\]
		Sold(T::AccountId, T::NftClassId, BalanceOf<T>),
		/// NFT item redeemed from pool \[who, class, instance\]
		Redeemed(T::AccountId, T::NftClassId, T::NftInstanceId),
		/// Info to redeemable class added
		ClassInfoAdded(T::AccountId, T::NftClassId, RedeemablesClassInfo),
	}

	#[pallet::error]
	pub enum Error<T> {
		// Cannot mint more than is the maximum pool supply
		ReachedMaxSupply,
		// Wrong class type
		WrongClassType,
	}
}

impl<T: Config> Pallet<T> {
	fn pallet_account() -> T::AccountId {
		T::PalletId::get().into_account()
	}
}
