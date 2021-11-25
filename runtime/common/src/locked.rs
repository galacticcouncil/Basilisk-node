use super::*;
use primitives::LockedBalance;

pub struct GetLockedBalance<T>(sp_std::marker::PhantomData<T>);

impl<T: orml_tokens::Config + pallet_balances::Config + frame_system::Config>
	LockedBalance<AssetId, T::AccountId, Balance> for GetLockedBalance<T>
where
	AssetId: Into<<T as orml_tokens::Config>::CurrencyId>,
	Balance: From<<T as orml_tokens::Config>::Balance>,
	Balance: From<<T as pallet_balances::Config>::Balance>,
{
	fn get_by_lock(lock_id: LockIdentifier, currency_id: AssetId, who: T::AccountId) -> Balance {
		if currency_id == NativeAssetId::get() {
			match orml_tokens::Pallet::<T>::locks(who, currency_id.into())
				.into_iter()
				.find(|lock| lock.id == lock_id)
			{
				Some(lock) => lock.amount.into(),
				None => 0u128,
			}
		} else {
			match pallet_balances::Pallet::<T>::locks(who)
				.into_iter()
				.find(|lock| lock.id == lock_id)
			{
				Some(lock) => lock.amount.into(),
				None => 0u128,
			}
		}
	}
}
