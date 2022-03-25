#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::HasCompact;
use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	ensure,
	traits::{tokens::nonfungibles::*, Get, NamedReservableCurrency},
	transactional, BoundedVec,
};
use frame_system::ensure_signed;

use primitives::{nft::NftPermission, ReserveIdentifier};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, StaticLookup, Zero},
	DispatchError,
};
pub use types::*;
use weights::WeightInfo;

mod benchmarking;
pub mod types;
pub mod weights;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

pub type BoundedVecOfUnq<T> = BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>;
type ClassInfoOf<T> = ClassInfo<<T as Config>::ClassType, BoundedVecOfUnq<T>>;
pub type InstanceInfoOf<T> = InstanceInfo<BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config {
		type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = ReserveIdentifier>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type ProtocolOrigin: EnsureOrigin<Self::Origin>;
		type NftClassId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ Into<Self::ClassId>
			+ From<Self::ClassId>;
		type NftInstanceId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ Into<Self::InstanceId>
			+ From<Self::InstanceId>;
		type ClassType: Member + Parameter + Default + Copy;
		type Permissions: NftPermission<Self::ClassType>;
		/// Class IDs reserved for runtime up to the following constant
		#[pallet::constant]
		type ReserveClassIdUpTo: Get<Self::NftClassId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn classes)]
	/// Stores class info
	pub type Classes<T: Config> = StorageMap<_, Twox64Concat, T::NftClassId, ClassInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn instances)]
	/// Stores instance info
	pub type Instances<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::NftClassId, Twox64Concat, T::NftInstanceId, InstanceInfoOf<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates an NFT class of the given class
		/// and sets its metadata
		///
		/// Parameters:
		/// - `class_id`: Identifier of a class
		/// - `class_type`: The class type determines its purpose and usage
		/// - `metadata`: Arbitrary data about a class, e.g. IPFS hash or name
		///
		/// Emits ClassCreated event
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		#[transactional]
		pub fn create_class(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			class_type: T::ClassType,
			metadata: BoundedVecOfUnq<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(T::ReserveClassIdUpTo::get() < class_id, Error::<T>::IdReserved);
			ensure!(T::Permissions::can_create(&class_type), Error::<T>::NotPermitted);

			Self::do_create_class(sender, class_id, Default::default(), metadata)?;

			Ok(())
		}

		/// Mints an NFT in the specified class
		/// and sets its metadata
		///
		/// Parameters:
		/// - `class_id`: The class of the asset to be minted.
		/// - `instance_id`: The class of the asset to be minted.
		/// - `metadata`: Arbitrary data about an instance, e.g. IPFS hash or symbol
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			metadata: BoundedVecOfUnq<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let class_type = Self::classes(class_id)
				.map(|c| c.class_type)
				.ok_or(Error::<T>::ClassUnknown)?;

			ensure!(T::Permissions::can_mint(&class_type), Error::<T>::NotPermitted);

			Self::do_mint(sender, class_id, instance_id, metadata)?;

			Ok(())
		}

		/// Transfers NFT from account A to account B
		/// Only the ProtocolOrigin can send NFT to another account
		/// This is to prevent creating deposit burden for others
		///
		/// Parameters:
		/// - `class_id`: The class of the asset to be transferred.
		/// - `instance_id`: The instance of the asset to be transferred.
		/// - `dest`: The account to receive ownership of the asset.
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let dest = T::Lookup::lookup(dest)?;

			let class_type = Self::classes(class_id)
				.map(|c| c.class_type)
				.ok_or(Error::<T>::ClassUnknown)?;

			ensure!(T::Permissions::can_transfer(&class_type), Error::<T>::NotPermitted);

			Self::do_transfer(class_id, instance_id, sender, dest)?;

			Ok(())
		}

		/// Removes a token from existence
		///
		/// Parameters:
		/// - `class_id`: The class of the asset to be burned.
		/// - `instance_id`: The instance of the asset to be burned.
		#[pallet::weight(<T as Config>::WeightInfo::burn())]
		#[transactional]
		pub fn burn(origin: OriginFor<T>, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let class_type = Self::classes(class_id)
				.map(|c| c.class_type)
				.ok_or(Error::<T>::ClassUnknown)?;

			ensure!(T::Permissions::can_burn(&class_type), Error::<T>::NotPermitted);

			Self::do_burn(sender, class_id, instance_id)?;

			Ok(())
		}

		/// Removes a class from existence
		///
		/// Parameters:
		/// - `class_id`: The identifier of the asset class to be destroyed.
		#[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
		#[transactional]
		pub fn destroy_class(origin: OriginFor<T>, class_id: T::NftClassId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let class_type = Self::classes(class_id)
				.map(|c| c.class_type)
				.ok_or(Error::<T>::ClassUnknown)?;

			ensure!(T::Permissions::can_destroy(&class_type), Error::<T>::NotPermitted);

			Self::do_destroy_class(sender, class_id)?;

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A class was created \[owner, class_id, class_type\]
		ClassCreated {
			owner: T::AccountId,
			class_id: T::NftClassId,
			class_type: T::ClassType,
			metadata: BoundedVecOfUnq<T>,
		},
		/// An instance was minted \[owner, class_id, instance_id\]
		InstanceMinted {
			owner: T::AccountId,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			metadata: BoundedVecOfUnq<T>,
		},
		/// An instance was transferred \[from, to, class_id, instance_id\]
		InstanceTransferred {
			from: T::AccountId,
			to: T::AccountId,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
		},
		/// An instance was burned \[sender, class_id, instance_id\]
		InstanceBurned {
			owner: T::AccountId,
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
		},
		/// A class was destroyed \[class_id\]
		ClassDestroyed {
			owner: T::AccountId,
			class_id: T::NftClassId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Count of instances overflown
		NoAvailableInstanceId,
		/// Count of classes overflown
		NoAvailableClassId,
		/// Class still contains minted tokens
		TokenClassNotEmpty,
		/// Class does not exist
		ClassUnknown,
		/// Instance does not exist
		InstanceUnknown,
		/// Operation not permitted
		NotPermitted,
		/// ID reserved for runtime
		IdReserved,
	}
}

impl<T: Config> Pallet<T> {
	pub fn class_owner(class_id: T::NftClassId) -> Option<T::AccountId> {
		pallet_uniques::Pallet::<T>::class_owner(&class_id.into())
	}

	pub fn owner(class_id: T::NftClassId, instance_id: T::NftInstanceId) -> Option<T::AccountId> {
		pallet_uniques::Pallet::<T>::owner(class_id.into(), instance_id.into())
	}

	pub fn do_create_class(
		owner: T::AccountId,
		class_id: T::NftClassId,
		class_type: T::ClassType,
		metadata: BoundedVecOfUnq<T>,
	) -> Result<(T::NftClassId, T::ClassType), DispatchError> {
		let deposit_info = match T::Permissions::has_deposit(&class_type) {
			false => (Zero::zero(), true),
			true => (T::ClassDeposit::get(), false),
		};

		pallet_uniques::Pallet::<T>::do_create_class(
			class_id.into(),
			owner.clone(),
			owner.clone(),
			deposit_info.0,
			deposit_info.1,
			pallet_uniques::Event::Created {
				class: class_id.into(),
				creator: owner.clone(),
				owner: owner.clone(),
			},
		)?;

		Classes::<T>::insert(
			class_id,
			ClassInfo {
				class_type,
				metadata: metadata.clone(),
			},
		);

		Self::deposit_event(Event::ClassCreated {
			owner,
			class_id,
			class_type,
			metadata,
		});

		Ok((class_id, class_type))
	}

	pub fn do_mint(
		owner: T::AccountId,
		class_id: T::NftClassId,
		instance_id: T::NftInstanceId,
		metadata: BoundedVecOfUnq<T>,
	) -> Result<T::NftInstanceId, DispatchError> {
		pallet_uniques::Pallet::<T>::do_mint(class_id.into(), instance_id.into(), owner.clone(), |_details| Ok(()))?;

		Instances::<T>::insert(
			class_id,
			instance_id,
			InstanceInfo {
				metadata: metadata.clone(),
			},
		);

		Self::deposit_event(Event::InstanceMinted {
			owner,
			class_id,
			instance_id,
			metadata,
		});

		Ok(instance_id)
	}

	pub fn do_transfer(
		class_id: T::NftClassId,
		instance_id: T::NftInstanceId,
		from: T::AccountId,
		to: T::AccountId,
	) -> DispatchResult {
		if from == to {
			return Ok(());
		}

		pallet_uniques::Pallet::<T>::do_transfer(
			class_id.into(),
			instance_id.into(),
			to.clone(),
			|_class_details, _instance_details| {
				let owner = Self::owner(class_id, instance_id).ok_or(Error::<T>::InstanceUnknown)?;
				ensure!(owner == from, Error::<T>::NotPermitted);
				Self::deposit_event(Event::InstanceTransferred {
					from,
					to,
					class_id,
					instance_id,
				});
				Ok(())
			},
		)
	}

	pub fn do_burn(owner: T::AccountId, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
		pallet_uniques::Pallet::<T>::do_burn(
			class_id.into(),
			instance_id.into(),
			|_class_details, _instance_details| {
				let iowner = Self::owner(class_id, instance_id).ok_or(Error::<T>::InstanceUnknown)?;
				ensure!(owner == iowner, Error::<T>::NotPermitted);
				Ok(())
			},
		)?;

		Instances::<T>::remove(class_id, instance_id);

		Self::deposit_event(Event::InstanceBurned {
			owner,
			class_id,
			instance_id,
		});

		Ok(())
	}

	pub fn do_destroy_class(owner: T::AccountId, class_id: T::NftClassId) -> DispatchResultWithPostInfo {
		let witness =
			pallet_uniques::Pallet::<T>::get_destroy_witness(&class_id.into()).ok_or(Error::<T>::ClassUnknown)?;

		ensure!(witness.instances == 0u32, Error::<T>::TokenClassNotEmpty);
		pallet_uniques::Pallet::<T>::do_destroy_class(class_id.into(), witness, Some(owner.clone()))?;
		Classes::<T>::remove(class_id);

		Self::deposit_event(Event::ClassDestroyed { owner, class_id });
		Ok(().into())
	}
}
