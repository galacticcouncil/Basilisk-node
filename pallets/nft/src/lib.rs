#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{tokens::nonfungibles::*, BalanceStatus, Currency, NamedReservableCurrency, ReservableCurrency},
	transactional, BoundedVec,
};
use frame_system::ensure_signed;

use primitives::ReserveIdentifier;
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, One, StaticLookup, Zero};
use sp_std::{convert::TryInto, vec::Vec};
use types::{ClassInfo, ClassType, InstanceInfo};
use weights::WeightInfo;

use pallet_uniques::traits::InstanceReserve;
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
	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
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
		type ProtocolOrigin: EnsureOrigin<Self::Origin>;

		type NftClassId: Member + Parameter + Default + Copy + HasCompact + AtLeast32BitUnsigned + Into<Self::ClassId>;
		type NftInstanceId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ From<Self::InstanceId>
			+ Into<Self::InstanceId>;
	}

	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextClassId<T: Config> = StorageValue<_, T::NftClassId, ValueQuery>;

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	pub type NextInstanceId<T: Config> = StorageMap<_, Twox64Concat, T::NftClassId, T::NftInstanceId, ValueQuery>;

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
		/// Creates an NFT class and sets its metadata
		///
		/// Parameters:
		/// - `class_id`: The identifier of the new asset class. This must not be currently in use.
		/// - `class_type`: The class type determines its purpose and usage
		/// - `metadata`: Arbitrary data about a class, e.g. IPFS hash
		///
		/// Emits `Created` and `ClassMetadataSet` events when successful.
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, class_type: types::ClassType, metadata: Vec<u8>) -> DispatchResult {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			match class_type {
				ClassType::PoolShare => ensure!(sender.is_none(), Error::<T>::NotPermitted),
				_ => (),
			}

			let class_id = NextClassId::<T>::try_mutate(|id| -> Result<T::NftClassId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableClassId)?;
				Ok(current_id)
			})?;

			let metadata_bounded = Self::to_bounded_string(metadata)?;

			let deposit_info = match class_type {
				ClassType::PoolShare => (Zero::zero(), true),
				_ => (T::ClassDeposit::get(), false),
			};

			pallet_uniques::Pallet::<T>::do_create_class(
				class_id.into(),
				sender.clone().unwrap_or_default(),
				sender.clone().unwrap_or_default(),
				deposit_info.0,
				deposit_info.1,
				pallet_uniques::Event::Created(
					class_id.into(),
					sender.clone().unwrap_or_default(),
					sender.clone().unwrap_or_default(),
				),
			)?;

			Classes::<T>::insert(
				class_id,
				ClassInfo {
					class_type: class_type,
					metadata: metadata_bounded,
				},
			);

			Self::deposit_event(Event::ClassCreated(
				sender.clone().unwrap_or_default(),
				class_id,
				class_type,
			));

			Ok(())
		}

		/// Mints an NFT in the specified class
		/// Sets metadata and the royalty attribute
		///
		/// Parameters:
		/// - `class_id`: The class of the asset to be minted.
		/// - `instance_id`: The instance value of the asset to be minted.
		/// - `author`: Receiver of the royalty
		/// - `royalty`: Percentage reward from each trade for the author
		/// - `metadata`: Arbitrary data about an instance, e.g. IPFS hash
		///
		/// Emits `Issued` and `AttributeSet` and `MetadataSet` events when successful.
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			author: Option<T::AccountId>,
			royalty: Option<u8>,
			metadata: Option<Vec<u8>>,
		) -> DispatchResult {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			let class_type = Self::classes(class_id)
				.map(|c| c.class_type)
				.ok_or(Error::<T>::ClassUnknown)?;

			match class_type {
				ClassType::PoolShare | ClassType::Unknown => ensure!(sender.is_none(), Error::<T>::NotPermitted),
				_ => (),
			}

			if let Some(r) = royalty {
				ensure!(r < 100, Error::<T>::NotInRange);
			}

			let instance_id: T::NftInstanceId =
				NextInstanceId::<T>::try_mutate(class_id, |id| -> Result<T::NftInstanceId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableInstanceId)?;
					Ok(current_id)
				})?;

			pallet_uniques::Pallet::<T>::do_mint(
				class_id.into(),
				instance_id.into(),
				sender.clone().unwrap_or_default(),
				|_details| {
					if let Some(sender) = sender.clone() {
						<T as Config>::Currency::reserve_named(&RESERVE_ID, &sender, T::TokenDeposit::get())?;
					}
					Ok(())
				},
			)?;

			if class_type == ClassType::Marketplace {
				let metadata_bounded = Self::to_bounded_string(metadata.ok_or(Error::<T>::MetadataNotSet)?)?;
				let author = author.ok_or(Error::<T>::AuthorNotSet)?;
				let royalty = royalty.ok_or(Error::<T>::RoyaltyNotSet)?;

				Instances::<T>::insert(
					class_id,
					instance_id,
					InstanceInfo {
						author: author,
						royalty: royalty,
						metadata: metadata_bounded,
					},
				);
			}

			Self::deposit_event(Event::InstanceMinted(
				sender.clone().unwrap_or_default(),
				class_id,
				instance_id,
			));

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
			class_id: T::NftClassId,
			instance_id: T::NftInstanceId,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			let dest = T::Lookup::lookup(dest)?;

			pallet_uniques::Pallet::<T>::do_transfer(
				class_id.into(),
				instance_id.into(),
				dest,
				|_class_details, instance_details| {
					if let Some(sender) = sender {
						let is_permitted = instance_details.owner == sender;
						ensure!(is_permitted, pallet_uniques::Error::<T, ()>::NoPermission);
						Ok(())
					} else {
						Ok(())
					}
				},
			)
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
		pub fn burn(origin: OriginFor<T>, class_id: T::NftClassId, instance_id: T::NftInstanceId) -> DispatchResult {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			ensure!(
				pallet_uniques::Pallet::<T>::can_transfer(&class_id.into(), &instance_id.into()),
				Error::<T>::TokenFrozen
			);

			pallet_uniques::Pallet::<T>::do_burn(
				class_id.into(),
				instance_id.into(),
				|_class_details, instance_details| {
					if let Some(sender) = sender {
						let is_permitted = instance_details.owner == sender;
						ensure!(is_permitted, pallet_uniques::Error::<T, ()>::NoPermission);
						<T as Config>::Currency::unreserve_named(&RESERVE_ID, &sender, T::TokenDeposit::get());
						Ok(())
					} else {
						Ok(())
					}
				},
			)?;

			Instances::<T>::remove(class_id, instance_id);

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
		pub fn destroy_class(origin: OriginFor<T>, class_id: T::NftClassId) -> DispatchResultWithPostInfo {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			let witness =
				pallet_uniques::Pallet::<T>::get_destroy_witness(&class_id.into()).ok_or(Error::<T>::NoWitness)?;

			// TODO: Need to check if any instance exists somehow still
			pallet_uniques::Pallet::<T>::do_destroy_class(class_id.into(), witness, sender)?;
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
		ClassCreated(T::AccountId, T::NftClassId, ClassType),
		/// An instance was minted \[sender, class_id, instance_id\]
		InstanceMinted(T::AccountId, T::NftClassId, T::NftInstanceId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// String exceeds allowed length
		TooLong,
		/// Count of instances overflown
		NoAvailableInstanceId,
		/// Count of classes overflown
		NoAvailableClassId,
		/// No witness found for given class
		NoWitness,
		/// Class still contains minted tokens
		TokenClassNotEmpty,
		/// Class does not exist
		ClassUnknown,
		/// Royalty has to be set for marketplace
		RoyaltyNotSet,
		/// Author has to be set for marketplace
		AuthorNotSet,
		/// Metadata has to be set for marketplace
		MetadataNotSet,
		/// Cannot burn token if frozen
		TokenFrozen,
		/// Royalty not in 0-99 range
		NotInRange,
		/// Operation not permitted
		NotPermitted,
	}
}

impl<T: Config> Pallet<T> {
	fn to_bounded_string(name: Vec<u8>) -> Result<BoundedVec<u8, T::StringLimit>, Error<T>> {
		name.try_into().map_err(|_| Error::<T>::TooLong)
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
