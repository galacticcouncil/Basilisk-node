use crate::DepositBalanceOf;
use sp_runtime::DispatchResult;

pub trait CanMint {
	fn can_mint<T: crate::pallet::Config<I>, I: 'static>(
		origin: T::AccountId,
		class_owner: &T::AccountId,
		class_admin: &T::AccountId,
		class_issuer: &T::AccountId,
	) -> DispatchResult;
}

pub trait CanBurn {
	fn can_burn<T: crate::pallet::Config<I>, I: 'static>(
		origin: T::AccountId,
		instance_owner: &T::AccountId,
		class_owner: &T::AccountId,
		class_admin: &T::AccountId,
		class_issuer: &T::AccountId,
	) -> DispatchResult;
}

pub trait InstanceReserve {
	fn reserve<T: crate::pallet::Config<I>, I>(
		instance_owner: &T::AccountId,
		class_owner: &T::AccountId,
		admin: &T::AccountId,
		issuer: &T::AccountId,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult;

	fn unreserve<T: crate::pallet::Config<I>, I>(
		instance_owner: &T::AccountId,
		class_owner: &T::AccountId,
		admin: &T::AccountId,
		issuer: &T::AccountId,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult;
}
