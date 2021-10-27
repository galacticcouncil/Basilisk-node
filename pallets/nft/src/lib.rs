#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::Decode;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{
		tokens::nonfungibles::Inspect, tokens::nonfungibles::*, BalanceStatus, Currency, EnsureOrigin,
		NamedReservableCurrency, ReservableCurrency,
	},
	transactional, BoundedVec,
};
use frame_system::{ensure_signed, RawOrigin};

use primitives::ReserveIdentifier;
use sp_runtime::{
	traits::{CheckedAdd, One, StaticLookup},
	RuntimeDebug,
};
use sp_std::{convert::TryInto, vec::Vec};
use types::{ClassInfo, ClassType, InstanceInfo};
use weights::WeightInfo;

use pallet_uniques::traits::{CanBurn, CanCreateClass, CanDestroyClass, CanMint, CanTransfer, InstanceReserve};
use pallet_uniques::{ClassTeam, DepositBalanceOf};

mod benchmarking;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type ClassInfoOf<T> = ClassInfo<BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;
pub type InstanceInfoOf<T> =
	InstanceInfo<<T as frame_system::Config>::AccountId, BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	pub const RESERVE_ID: ReserveIdentifier = ReserveIdentifier::Nft;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config {
		/// Currency type for reserve balance.
		type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = ReserveIdentifier>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Amount that must be reserved for each minted NFT
		#[pallet::constant]
		type TokenDeposit: Get<BalanceOf<Self>>;
		type WeightInfo: WeightInfo;
	}

	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	pub type NextInstanceId<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, T::InstanceId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn classes)]
	/// Stores class info
	pub type Classes<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, ClassInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn instances)]
	/// Stores instance info
	pub type Instances<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::InstanceId, InstanceInfoOf<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates an NFT class and sets its metadata
		///
		/// Parameters:
		/// - `class_id`: The identifier of the new asset class. This must not be currently in use.
		/// - `class_type`: The class type determines its purpose and usage
		///
		/// Emits `Created` and `ClassMetadataSet` events when successful.
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, class_type: types::ClassType, metadata: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let admin = T::Lookup::unlookup(sender.clone());

			let class_id = NextClassId::<T>::try_mutate(|id| -> Result<T::ClassId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableClassId)?;
				Ok(current_id)
			})?;

			let metadata_bounded = Self::to_bounded_string(metadata)?;

			pallet_uniques::Pallet::<T>::create(origin.clone(), class_id, admin.clone())?;

			Classes::<T>::insert(
				class_id,
				ClassInfo {
					class_type: class_type,
					metadata: metadata_bounded,
				},
			);

			Self::deposit_event(Event::ClassCreated(sender, class_id, class_type));

			Ok(())
		}

		/// Mints an NFT in the specified class
		/// Sets metadata and the royalty attribute
		///
		/// Parameters:
		/// - `class_id`: The class of the asset to be minted.
		/// - `instance_id`: The instance value of the asset to be minted.
		/// - `owner`: The initial owner of the minted asset.
		///
		/// Emits `Issued` and `AttributeSet` and `MetadataSet` events when successful.
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			author: Option<T::AccountId>,
			royalty: Option<u8>,
			metadata: Option<Vec<u8>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let instance_id =
				NextInstanceId::<T>::try_mutate(class_id, |id| -> Result<T::InstanceId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableInstanceId)?;
					Ok(current_id)
				})?;

			let class_type = Self::classes(class_id)
				.map(|c| c.class_type)
				.ok_or(Error::<T>::ClassUnknown)?;

			pallet_uniques::Pallet::<T>::mint(origin.clone(), class_id, instance_id, sender.clone())?;

			if class_type == ClassType::Marketplace {
				Instances::<T>::try_mutate(class_id, instance_id, |maybe_info| -> DispatchResult {
					let info = maybe_info.as_mut().ok_or(Error::<T>::InstanceUnknown)?;
					let metadata_bounded = Self::to_bounded_string(metadata.ok_or(Error::<T>::MetadataNotSet)?)?;
					
					info.author = author.ok_or(Error::<T>::AuthorNotSet)?;
					info.royalty = royalty.ok_or(Error::<T>::RoyaltyNotSet)?;
					info.metadata = metadata_bounded;

					Ok(())
				})?;
			}

			<T as Config>::Currency::reserve_named(&RESERVE_ID, &sender, T::TokenDeposit::get())?;

			Self::deposit_event(Event::InstanceMinted(sender, class_id, instance_id));

			Ok(())
		}

		/// Transfers NFT from account A to account B
		/// Only the owner can send their NFT to another account
		///
		/// Parameters:
		/// - `class_id`: The class of the asset to be transferred.
		/// - `instance_id`: The instance of the asset to be transferred.
		/// - `dest`: The account to receive ownership of the asset.
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			instance_id: T::InstanceId,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			ensure_signed(origin.clone())?;

			pallet_uniques::Pallet::<T>::transfer(origin, class_id, instance_id, dest)?;

			Ok(())
		}

		/// Removes a token from existence
		///
		/// Parameters:
		/// - `class_id`: The class of the asset to be burned.
		/// - `instance_id`: The instance of the asset to be burned.
		/// - `check_owner`: If `Some` then the operation will fail with `WrongOwner` unless the
		///   asset is owned by this value.
		///
		/// Emits `Burned` with the actual amount burned.
		#[pallet::weight(<T as Config>::WeightInfo::burn())]
		#[transactional]
		pub fn burn(origin: OriginFor<T>, class_id: T::ClassId, instance_id: T::InstanceId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(
				pallet_uniques::Pallet::<T>::can_transfer(&class_id, &instance_id),
				Error::<T>::TokenFrozen
			);

			pallet_uniques::Pallet::<T>::burn(origin, class_id, instance_id, None)?;
			Instances::<T>::remove(class_id, instance_id);
			<T as Config>::Currency::unreserve_named(&RESERVE_ID, &sender, T::TokenDeposit::get());

			Ok(())
		}

		/// Removes a class from existence
		///
		/// Parameters:
		/// - `class_id`: The identifier of the asset class to be destroyed.
		///
		/// Emits `Destroyed` event when successful.
		#[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
		#[transactional]
		pub fn destroy_class(origin: OriginFor<T>, class_id: T::ClassId) -> DispatchResultWithPostInfo {
			ensure_signed(origin.clone())?;

			let witness = pallet_uniques::Pallet::<T>::get_destroy_witness(&class_id).ok_or(Error::<T>::NoWitness)?;

			pallet_uniques::Pallet::<T>::destroy(origin, class_id, witness)?;
			Classes::<T>::remove(class_id);

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Class was created \[sender, class_id, class_type\]
		ClassCreated(T::AccountId, T::ClassId, ClassType),
		/// An instance was minted \[sender, class_id, instance_id\]
		InstanceMinted(T::AccountId, T::ClassId, T::InstanceId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// String exceeds allowed length
		TooLong,
		/// Count of instances overflown
		NoAvailableInstanceId,
		/// Count of classes overflown
		NoAvailableClassId,
		/// Witness not available
		WitnessUnavailable,
		/// Cannot burn token if frozen
		TokenFrozen,
		/// No witness found for given class
		NoWitness,
		/// Class still contains minted tokens
		TokenClassNotEmpty,
		/// This class type is not known
		UnsupportedClassType,
		/// Class does not exist
		ClassUnknown,
		/// Instance does not exist
		InstanceUnknown,
		/// Royalty has to be set for marketplace
		RoyaltyNotSet,
		/// Author has to be set for marketplace
		AuthorNotSet,
		/// Metadata has to be set for marketplace
		MetadataNotSet,
	}
}

impl<T: Config> Pallet<T> {
	fn to_bounded_string(name: Vec<u8>) -> Result<BoundedVec<u8, T::StringLimit>, Error<T>> {
		name.try_into().map_err(|_| Error::<T>::TooLong)
	}
}

impl<P: Config> CanMint for Pallet<P> {
	fn can_mint<T: pallet_uniques::Config<I>, I: 'static>(
		_sender: T::AccountId,
		_class_team: &ClassTeam<T::AccountId>,
		class_id: &T::ClassId,
	) -> DispatchResult {
		// let class_type = Classes::get(*class_id).map(|c| c.class_type).ok_or(Error::<T, I>::ClassUnknown)?;
		// match class_type {
		// 	ClassType::PoolShare => {
		// 		//T::ForceOrigin::ensure_origin(origin)?;
		// 		Ok(())
		// 	}
		// 	_ => Ok(()),
		// }
		Ok(())
	}
}

impl<P: Config> CanBurn for Pallet<P> {
	fn can_burn<T: pallet_uniques::Config<I>, I: 'static>(
		sender: T::AccountId,
		instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		// let type_bounded = Self::to_bounded_key(b"type".to_vec())?;
		// let class_type = pallet_uniques::Pallet::<T, I>::class_attribute(class_id, &type_bounded)
		// 	.ok_or(pallet_uniques::Error::<T, I>::Unknown)?;
		// let class_type_decoded = ClassType::decode(&mut class_type.as_slice()).unwrap_or(Default::default());
		// match class_type_decoded {
		// 	ClassType::PoolShare => {
		// 		//T::ForceOrigin::ensure_origin(origin)?;
		// 		Ok(())
		// 	}
		// 	_ => {
		// 		let is_permitted = *instance_owner == sender;
		// 		ensure!(is_permitted, pallet_uniques::Error::<T, I>::NoPermission);
		// 		Ok(())
		// 	}
		// }
		Ok(())
	}
}

impl<P: Config> CanTransfer for Pallet<P> {
	fn can_transfer<T: pallet_uniques::Config<I>, I: 'static>(
		sender: T::AccountId,
		instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		// let type_bounded = Self::to_bounded_key(b"type".to_vec())?;
		// let class_type = pallet_uniques::Pallet::<T, I>::class_attribute(class_id, &type_bounded)
		// 	.ok_or(pallet_uniques::Error::<T, I>::Unknown)?;
		// let class_type_decoded = ClassType::decode(&mut class_type.as_slice()).unwrap_or(Default::default());
		// match class_type_decoded {
		// 	ClassType::PoolShare => {
		// 		//T::ForceOrigin::ensure_origin(origin)?;
		// 		Ok(())
		// 	}
		// 	_ => {
		// 		let is_permitted = *instance_owner == sender;
		// 		ensure!(is_permitted, pallet_uniques::Error::<T, I>::NoPermission);
		// 		Ok(())
		// 	}
		// }
		Ok(())
	}
}

impl<P: Config> CanCreateClass for Pallet<P> {
	fn can_create_class<T: pallet_uniques::Config<I>, I: 'static>(
		_sender: &T::AccountId,
		class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		// let type_bounded = Self::to_bounded_key(b"type".to_vec())?;
		// let class_type = pallet_uniques::Pallet::<T, I>::class_attribute(class_id, &type_bounded)
		// 	.ok_or(pallet_uniques::Error::<T, I>::Unknown)?;
		// let class_type_decoded = ClassType::decode(&mut class_type.as_slice()).unwrap_or(Default::default());
		// match class_type_decoded {
		// 	ClassType::PoolShare => {
		// 		//T::ForceOrigin::ensure_origin(T::Origin::from(RawOrigin::Signed(sender.to_owned())))?;
		// 		Ok(())
		// 	}
		// 	_ => Ok(()),
		// }
		Ok(())
	}
}

impl<P: Config> CanDestroyClass for Pallet<P> {
	fn can_destroy_class<T: pallet_uniques::Config<I>, I: 'static>(
		sender: &T::AccountId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		// let type_bounded = Self::to_bounded_key(b"type".to_vec())?;
		// let class_type = pallet_uniques::Pallet::<T, I>::class_attribute(class_id, &type_bounded)
		// 	.ok_or(pallet_uniques::Error::<T, I>::Unknown)?;
		// let class_type_decoded = ClassType::decode(&mut class_type.as_slice()).unwrap_or(Default::default());
		// match class_type_decoded {
		// 	ClassType::PoolShare => {
		// 		//T::ForceOrigin::ensure_origin(T::Origin::from(RawOrigin::Signed(sender.to_owned())))?;
		// 		Ok(())
		// 	}
		// 	_ => {
		// 		ensure!(class_team.owner == *sender, pallet_uniques::Error::<T, I>::NoPermission);
		// 		Ok(())
		// 	}
		// }
		Ok(())
	}

	fn can_destroy_instances<T: pallet_uniques::Config<I>, I: 'static>(
		_sender: &T::AccountId,
		_class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		// Is called only where are existing instances
		// Not allowed to destroy calls in such case
		Err(pallet_uniques::Error::<T, I>::NoPermission.into())
	}
}

impl<P: Config> InstanceReserve for Pallet<P> {
	fn reserve<T: pallet_uniques::Config<I>, I>(
		instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		_class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
		deposit: pallet_uniques::DepositBalanceOf<T, I>,
	) -> sp_runtime::DispatchResult {
		T::Currency::reserve(instance_owner, deposit)
	}

	fn unreserve<T: pallet_uniques::Config<I>, I>(
		instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		_class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
		deposit: pallet_uniques::DepositBalanceOf<T, I>,
	) -> sp_runtime::DispatchResult {
		T::Currency::unreserve(instance_owner, deposit);
		Ok(())
	}

	fn repatriate<T: pallet_uniques::Config<I>, I>(
		dest: &T::AccountId,
		instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		_class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
		deposit: DepositBalanceOf<T, I>,
	) -> sp_runtime::DispatchResult {
		T::Currency::repatriate_reserved(dest, instance_owner, deposit, BalanceStatus::Reserved).map(|_| ())
	}
}