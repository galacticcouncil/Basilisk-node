use crate::{ClassDetailsFor, DepositBalanceOf, InstanceDetailsFor};
use sp_runtime::DispatchResult;

pub trait CanMint {
	fn can_mint<T: crate::pallet::Config<I>, I>(
		origin: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
	) -> DispatchResult;

	fn reserve<T: crate::pallet::Config<I>, I>(
		owner: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult;
}

pub trait CanBurn {
	fn can_burn<T: crate::pallet::Config<I>, I>(
		origin: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		instance: &InstanceDetailsFor<T, I>,
	) -> DispatchResult;

	fn unreserve<T: crate::pallet::Config<I>, I>(
		owner: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult;
}
