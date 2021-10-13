use crate::traits::{CanBurn, CanMint};
use crate::{ClassDetailsFor, Config, Error, InstanceDetailsFor};
use frame_benchmarking::frame_support::sp_runtime::DispatchResult;
use frame_support::ensure;

pub struct IsIssuer();

impl CanMint for IsIssuer {
	fn can_mint<T: Config<I>, I>(origin: T::AccountId, class_details: &ClassDetailsFor<T, I>) -> DispatchResult {
		ensure!(class_details.issuer == origin, Error::<T, I>::NoPermission);
		Ok(())
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
}

// Default behaviour

impl CanMint for () {
	fn can_mint<T: Config<I>, I>(origin: T::AccountId, class_details: &ClassDetailsFor<T, I>) -> DispatchResult {
		IsIssuer::can_mint::<T, I>(origin, class_details)
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
}
