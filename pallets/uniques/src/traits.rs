use crate::{ClassTeam, DepositBalanceOf};
use sp_runtime::DispatchResult;

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
