use crate::traits::{CanBurn, CanMint};
use crate::{ClassDetailsFor, Config, DepositBalanceOf, Error, InstanceDetailsFor};
use frame_benchmarking::frame_support::sp_runtime::DispatchResult;
use frame_support::ensure;
use frame_support::traits::ReservableCurrency;

pub struct IsIssuer();

impl CanMint for IsIssuer {
	fn can_mint<T: Config<I>, I>(origin: T::AccountId, class_details: &ClassDetailsFor<T, I>) -> DispatchResult {
		ensure!(class_details.issuer == origin, Error::<T, I>::NoPermission);
		Ok(())
	}

	fn reserve<T: Config<I>, I>(
		owner: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		T::Currency::reserve(&class_details.owner, deposit)
	}
}

pub struct AdminOrOwner();

impl CanBurn for AdminOrOwner {
	fn can_burn<T: Config<I>, I>(
		origin: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		instance: &InstanceDetailsFor<T, I>,
	) -> DispatchResult {
		let is_permitted = class_details.admin == origin || instance.owner == origin;
		ensure!(is_permitted, Error::<T, I>::NoPermission);
		Ok(())
	}

	fn unreserve<T: Config<I>, I>(
		owner: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		T::Currency::unreserve(&class_details.owner, deposit);
		Ok(())
	}
}

// Default behaviour

impl CanMint for () {
	fn can_mint<T: Config<I>, I>(origin: T::AccountId, class_details: &ClassDetailsFor<T, I>) -> DispatchResult {
		IsIssuer::can_mint::<T, I>(origin, class_details)
	}

	fn reserve<T: Config<I>, I>(
		owner: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		IsIssuer::reserve::<T, I>(owner, class_details, deposit)
	}
}

impl CanBurn for () {
	fn can_burn<T: Config<I>, I>(
		origin: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		instance: &InstanceDetailsFor<T, I>,
	) -> DispatchResult {
		AdminOrOwner::can_burn::<T, I>(origin, class_details, instance)
	}

	fn unreserve<T: Config<I>, I>(
		owner: T::AccountId,
		class_details: &ClassDetailsFor<T, I>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		AdminOrOwner::unreserve::<T, I>(owner, class_details, deposit)
	}
}
