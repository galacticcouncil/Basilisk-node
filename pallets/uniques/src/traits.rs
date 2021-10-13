use crate::{ClassDetailsFor, InstanceDetailsFor};
use sp_runtime::DispatchResult;

pub trait CanMint {
	fn can_mint<T: crate::pallet::Config<I>, I>(
		origin: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
	) -> DispatchResult;
}

pub trait CanBurn {
	fn can_burn<T: crate::pallet::Config<I>, I>(
		origin: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		instance: &InstanceDetailsFor<T, I>,
	) -> DispatchResult;
}
