use crate::traits::{CanBurn, CanMint, InstanceReserve};
use crate::{Config, DepositBalanceOf, Error};
use frame_benchmarking::frame_support::sp_runtime::DispatchResult;
use frame_support::ensure;
use frame_support::traits::ReservableCurrency;

pub struct IsIssuer();

impl CanMint for IsIssuer {
	fn can_mint<T: Config<I>, I: 'static>(
		origin: T::AccountId,
		_class_owner: &T::AccountId,
		_class_admin: &T::AccountId,
		class_issuer: &T::AccountId,
	) -> DispatchResult {
		ensure!(*class_issuer == origin, Error::<T, I>::NoPermission);
		Ok(())
	}
}

pub struct AdminOrOwner();

impl CanBurn for AdminOrOwner {
	fn can_burn<T: Config<I>, I: 'static>(
		origin: T::AccountId,
		instance_owner: &T::AccountId,
		_class_owner: &T::AccountId,
		class_admin: &T::AccountId,
		_class_issuer: &T::AccountId,
	) -> DispatchResult {
		let is_permitted = *class_admin == origin || *instance_owner == origin;
		ensure!(is_permitted, Error::<T, I>::NoPermission);
		Ok(())
	}
}

// Default behaviour

impl CanMint for () {
	fn can_mint<T: Config<I>, I: 'static>(
		origin: T::AccountId,
		class_owner: &T::AccountId,
		class_admin: &T::AccountId,
		class_issuer: &T::AccountId,
	) -> DispatchResult {
		IsIssuer::can_mint::<T, I>(origin, class_owner, class_admin, class_issuer)
	}
}

impl CanBurn for () {
	fn can_burn<T: Config<I>, I: 'static>(
		origin: T::AccountId,
		instance_owner: &T::AccountId,
		class_owner: &T::AccountId,
		class_admin: &T::AccountId,
		class_issuer: &T::AccountId,
	) -> DispatchResult {
		AdminOrOwner::can_burn::<T, I>(origin, instance_owner, class_owner, class_admin, class_issuer)
	}
}

impl InstanceReserve for () {
	fn reserve<T: Config<I>, I>(
		_instance_owner: &T::AccountId,
		class_owner: &T::AccountId,
		_admin: &T::AccountId,
		_issuer: &T::AccountId,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		T::Currency::reserve(class_owner, deposit)
	}

	fn unreserve<T: Config<I>, I>(
		_instance_owner: &T::AccountId,
		class_owner: &T::AccountId,
		_admin: &T::AccountId,
		_issuer: &T::AccountId,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		T::Currency::unreserve(class_owner, deposit);
		Ok(())
	}
}
