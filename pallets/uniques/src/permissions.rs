use crate::traits::{CanBurn, CanDestroyClass, CanCreateClass, CanMint, CanTransfer, InstanceReserve};
use crate::{ClassTeam, Config, DepositBalanceOf, Error};
use frame_support::ensure;
use frame_support::sp_runtime::DispatchResult;
use frame_support::traits::ReservableCurrency;

pub struct IsIssuer();

impl CanMint for IsIssuer {
	fn can_mint<T: Config<I>, I: 'static>(
		sender: T::AccountId,
		class_team: &ClassTeam<T::AccountId>,
		_class_id: &T::ClassId,
	) -> DispatchResult {
		ensure!(class_team.issuer == sender, Error::<T, I>::NoPermission);
		Ok(())
	}
}

pub struct AdminOrOwner();

impl CanBurn for AdminOrOwner {
	fn can_burn<T: Config<I>, I: 'static>(
		sender: T::AccountId,
		instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		_class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		let is_permitted = class_team.admin == sender || *instance_owner == sender;
		ensure!(is_permitted, Error::<T, I>::NoPermission);
		Ok(())
	}
}

// Default behaviour

impl CanMint for () {
	fn can_mint<T: Config<I>, I: 'static>(
		sender: T::AccountId,
		class_team: &ClassTeam<T::AccountId>,
		class_id: &T::ClassId,
	) -> DispatchResult {
		IsIssuer::can_mint::<T, I>(sender, class_team, class_id)
	}
}

impl CanBurn for () {
	fn can_burn<T: Config<I>, I: 'static>(
		sender: T::AccountId,
		instance_owner: &T::AccountId,
		instance_id: &T::InstanceId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		AdminOrOwner::can_burn::<T, I>(sender, instance_owner, instance_id, class_id, class_team)
	}
}

impl CanTransfer for () {
	fn can_transfer<T: Config<I>, I: 'static>(
		sender: T::AccountId,
		instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		_class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		let is_permitted = class_team.admin == sender || *instance_owner == sender;
		ensure!(is_permitted, Error::<T, I>::NoPermission);
		Ok(())
	}
}

impl CanDestroyClass for () {
	fn can_destroy_class<T: Config<I>, I: 'static>(
		sender: &T::AccountId,
		_class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		ensure!(class_team.owner == *sender, Error::<T, I>::NoPermission);
		Ok(())
	}

	fn can_destroy_instances<T: Config<I>, I: 'static>(
		_sender: &T::AccountId,
		_class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		Ok(())
	}
}

impl CanCreateClass for () {
	fn can_create_class<T: Config<I>, I: 'static>(
		_sender: &T::AccountId,
		_class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult {
		Ok(())
	}
}

impl InstanceReserve for () {
	fn reserve<T: Config<I>, I>(
		_instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		_class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		T::Currency::reserve(&class_team.owner, deposit)
	}

	fn unreserve<T: Config<I>, I>(
		_instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		_class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		T::Currency::unreserve(&class_team.owner, deposit);
		Ok(())
	}

	fn repatriate<T: Config<I>, I>(
		_dest: &T::AccountId,
		_instance_owner: &T::AccountId,
		_instance_id: &T::InstanceId,
		_class_id: &T::ClassId,
		_class_team: &ClassTeam<T::AccountId>,
		_deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult {
		// Nothing to do here, because instances reserve are always on class owner
		// This becomes useful when instance reserve are on instance owner,
		// and instance is transferred
		Ok(())
	}
}
