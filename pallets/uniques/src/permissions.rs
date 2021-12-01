use crate::traits::InstanceReserve;
use crate::{ClassTeam, Config, DepositBalanceOf};
use frame_support::sp_runtime::DispatchResult;
use frame_support::traits::ReservableCurrency;

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
