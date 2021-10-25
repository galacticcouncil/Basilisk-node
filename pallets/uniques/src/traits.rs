use crate::{ClassTeam, DepositBalanceOf};
use sp_runtime::DispatchResult;

pub trait CanMint {
	fn can_mint<T: crate::pallet::Config<I>, I: 'static>(
		sender: T::AccountId,
		class_team: &ClassTeam<T::AccountId>,
		class_id: &T::ClassId,
	) -> DispatchResult;
}

pub trait CanBurn {
	fn can_burn<T: crate::pallet::Config<I>, I: 'static>(
		sender: T::AccountId,
		instance_owner: &T::AccountId,
		instance_id: &T::InstanceId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult;
}

pub trait CanTransfer {
	fn can_transfer<T: crate::pallet::Config<I>, I: 'static>(
		sender: T::AccountId,
		instance_owner: &T::AccountId,
		instance_id: &T::InstanceId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult;
}

pub trait CanCreateClass {
	fn can_create_class<T: crate::pallet::Config<I>, I: 'static>(
		sender: &T::AccountId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult;
}

pub trait CanDestroyClass {
	fn can_destroy_class<T: crate::pallet::Config<I>, I: 'static>(
		sender: &T::AccountId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult;

	fn can_destroy_instances<T: crate::pallet::Config<I>, I: 'static>(
		sender: &T::AccountId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
	) -> DispatchResult;
}

pub trait InstanceReserve {
	fn reserve<T: crate::pallet::Config<I>, I>(
		instance_owner: &T::AccountId,
		instance_id: &T::InstanceId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult;

	fn unreserve<T: crate::pallet::Config<I>, I>(
		instance_owner: &T::AccountId,
		instance_id: &T::InstanceId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult;

	fn repatriate<T: crate::pallet::Config<I>, I>(
		dest: &T::AccountId,
		instance_owner: &T::AccountId,
		instance_id: &T::InstanceId,
		class_id: &T::ClassId,
		class_team: &ClassTeam<T::AccountId>,
		deposit: DepositBalanceOf<T, I>,
	) -> DispatchResult;
}
