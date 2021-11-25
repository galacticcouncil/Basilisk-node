#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::too_many_arguments)]

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

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type ClassInfoOf<T> = ClassInfo<BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;
type InstanceInfoOf<T> = InstanceInfo<
	<T as frame_system::Config>::AccountId,
	BalanceOf<T>,
	BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>,
>;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::OriginFor;

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
			+ Into<Self::InstanceId>
			+ From<Self::InstanceId>;
	}

	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub(super) type NextClassId<T: Config> = StorageValue<_, T::NftClassId, ValueQuery>;

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_instance_id)]
	pub(super) type NextInstanceId<T: Config> =
		StorageMap<_, Twox64Concat, T::NftClassId, T::NftInstanceId, ValueQuery>;

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
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, class_type: types::ClassType, metadata: Vec<u8>) -> DispatchResult {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			if class_type == ClassType::PoolShare {
				ensure!(sender.is_none(), Error::<T>::NotPermitted)
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
					class_type,
					metadata: metadata_bounded,
				},
			);

			Self::deposit_event(Event::ClassCreated(sender.unwrap_or_default(), class_id, class_type));

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
		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			owner: T::AccountId,
			class_id: T::NftClassId,
			author: Option<T::AccountId>,
			royalty: Option<u8>,
			metadata: Option<Vec<u8>>,
			shares: Option<BalanceOf<T>>,
			accrps: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			let class_type = Self::classes(class_id)
				.map(|c| c.class_type)
				.ok_or(Error::<T>::ClassUnknown)?;

			if let Some(r) = royalty {
				ensure!(r < 100, Error::<T>::NotInRange);
			}

			let instance_id: T::NftInstanceId =
				NextInstanceId::<T>::try_mutate(class_id, |id| -> Result<T::NftInstanceId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableInstanceId)?;
					Ok(current_id)
				})?;

			pallet_uniques::Pallet::<T>::do_mint(class_id.into(), instance_id.into(), owner, |_details| {
				if sender.is_some() {
					ensure!(class_type == ClassType::Marketplace, Error::<T>::NotPermitted)
				}
				Ok(())
			})?;

			if class_type == ClassType::Marketplace {
				let metadata_bounded = Self::to_bounded_string(metadata.ok_or(Error::<T>::MetadataNotSet)?)?;
				let author = author.ok_or(Error::<T>::AuthorNotSet)?;
				let royalty = royalty.ok_or(Error::<T>::RoyaltyNotSet)?;
				let shares = shares.ok_or(Error::<T>::SharesNotSet)?;
				let accrps = accrps.ok_or(Error::<T>::AccrpsNotSet)?;

				Instances::<T>::insert(
					class_id,
					instance_id,
					InstanceInfo {
						author,
						royalty,
						metadata: metadata_bounded,
						shares,
						accrps,
					},
				);
			}

			Self::deposit_event(Event::InstanceMinted(sender.unwrap_or_default(), class_id, instance_id));

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

			if sender == dest {
				return Ok(());
			}

			pallet_uniques::Pallet::<T>::do_transfer(
				class_id.into(),
				instance_id.into(),
				dest.clone(),
				|_class_details, instance_details| {
					let is_permitted = instance_details.owner == sender;
					ensure!(is_permitted, Error::<T>::NotPermitted);
					Ok(())
				},
			)?;

			Self::deposit_event(Event::InstanceTransferred(sender, dest, class_id, instance_id));

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

			pallet_uniques::Pallet::<T>::do_burn(
				class_id.into(),
				instance_id.into(),
				|_class_details, instance_details| {
					let is_permitted = instance_details.owner == sender;
					ensure!(is_permitted, Error::<T>::NotPermitted);
					Ok(())
				},
			)?;

			Instances::<T>::remove(class_id, instance_id);

			Self::deposit_event(Event::InstanceBurned(sender, class_id, instance_id));

			Ok(())
		}

		/// Removes a class from existence
		///
		/// Parameters:
		/// - `class_id`: The identifier of the asset class to be destroyed.
		#[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
		#[transactional]
		pub fn destroy_class(origin: OriginFor<T>, class_id: T::NftClassId) -> DispatchResultWithPostInfo {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			let class_type = Self::classes(class_id)
				.map(|c| c.class_type)
				.ok_or(Error::<T>::ClassUnknown)?;

			if class_type == ClassType::PoolShare {
				ensure!(sender.is_none(), Error::<T>::NotPermitted)
			}

			let witness =
				pallet_uniques::Pallet::<T>::get_destroy_witness(&class_id.into()).ok_or(Error::<T>::NoWitness)?;

			ensure!(witness.instances == 0u32, Error::<T>::TokenClassNotEmpty);
			pallet_uniques::Pallet::<T>::do_destroy_class(class_id.into(), witness, sender.clone())?;
			Classes::<T>::remove(class_id);

			Self::deposit_event(Event::ClassDestroyed(sender.unwrap_or_default(), class_id));

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A class was created \[sender, class_id, class_type\]
		ClassCreated(T::AccountId, T::NftClassId, ClassType),
		/// An instance was minted \[sender, class_id, instance_id\]
		InstanceMinted(T::AccountId, T::NftClassId, T::NftInstanceId),
		/// An instance was transferred \[from, to, class_id, instance_id\]
		InstanceTransferred(T::AccountId, T::AccountId, T::NftClassId, T::NftInstanceId),
		/// An instance was burned \[sender, class_id, instance_id\]
		InstanceBurned(T::AccountId, T::NftClassId, T::NftInstanceId),
		/// A class was destroyed \[sender, class_id\]
		ClassDestroyed(T::AccountId, T::NftClassId),
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
		/// Shares has to be set for liquidity mining
		SharesNotSet,
		/// Accumulated reward per share has to be set for liquidity mining
		AccrpsNotSet,
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

	fn _get_instance_owner(class_id: T::NftClassId, instance_id: T::NftInstanceId) -> Option<T::AccountId> {
		pallet_uniques::Pallet::<T>::owner(class_id.into(), instance_id.into())
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
		T::Currency::repatriate_reserved(instance_owner, dest, deposit, BalanceStatus::Reserved).map(|_| ())
	}
}
