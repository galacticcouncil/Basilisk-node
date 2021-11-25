use super::*;
use frame_support::sp_runtime::traits::Zero;

use hydradx_traits::LockedBalance;
use frame_support::traits::LockIdentifier;

pub struct MultiCurrencyLockedBalance<T>(sp_std::marker::PhantomData<T>);

impl<T: orml_tokens::Config + pallet_balances::Config + frame_system::Config>
	LockedBalance<AssetId, T::AccountId, Balance> for MultiCurrencyLockedBalance<T>
where
	AssetId: Into<<T as orml_tokens::Config>::CurrencyId>,
	Balance: From<<T as orml_tokens::Config>::Balance>,
	Balance: From<<T as pallet_balances::Config>::Balance>,
{
	fn get_by_lock(lock_id: LockIdentifier, currency_id: AssetId, who: T::AccountId) -> Balance {
		if currency_id == NativeAssetId::get() {
            match pallet_balances::Pallet::<T>::locks(who)
                .into_iter()
                .find(|lock| lock.id == lock_id)
            {
                Some(lock) => lock.amount.into(),
                None => Zero::zero(),
            }
		} else {
            match orml_tokens::Pallet::<T>::locks(who, currency_id.into())
                .into_iter()
                .find(|lock| lock.id == lock_id)
            {
                Some(lock) => lock.amount.into(),
                None => Zero::zero(),
            }
		}
	}
}
