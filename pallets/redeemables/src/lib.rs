#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, ensure, traits::Currency, transactional};
use frame_system::ensure_signed;

use weights::WeightInfo;

use pallet_nft::{types::ClassType};

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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Next available curve ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub(super) type NextCurveId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn classes)]
	/// Stores class info
	pub type Curves<T: Config> = StorageMap<_, Blake2_128Concat, u64, Option<BondingCurve>>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		
	}

	#[pallet::error]
	pub enum Error<T> {
		Overflow,
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_curve_id() -> Result<u64, Error<T>> {
		NextCurveId::<T>::try_mutate(|id| {
			let current_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::Overflow)?;
			Ok(current_id)
		})
	}
}
