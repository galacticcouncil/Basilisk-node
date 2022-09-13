use crate::{Config, MAX_ASSETS_IN_POOL, POOL_IDENTIFIER};
use sp_runtime::Permill;
use sp_std::prelude::*;

use crate::traits::ShareAccountIdFor;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::ConstU32;
use frame_support::BoundedVec;
use orml_traits::MultiCurrency;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

pub(crate) type Balance = u128;

/// Pool properties for 2-asset pool (v1)
/// `assets`: pool assets
/// `amplification`: amp parameter
/// `fee`: trade fee to be withdrawn on sell/buy
#[derive(Clone, Encode, Decode, PartialEq, MaxEncodedLen, TypeInfo, RuntimeDebug)]
pub struct PoolInfo<AssetId> {
	pub assets: BoundedVec<AssetId, ConstU32<MAX_ASSETS_IN_POOL>>,
	pub amplification: u16,
	pub trade_fee: Permill,
	pub withdraw_fee: Permill,
}

impl<AssetId> PoolInfo<AssetId>
where
	AssetId: Ord + Copy,
{
	pub(crate) fn find_asset(&self, asset: AssetId) -> Option<usize> {
		self.assets.iter().position(|v| *v == asset)
	}

	pub(crate) fn pool_account<T: Config>(&self) -> T::AccountId
	where
		T::ShareAccountId: ShareAccountIdFor<Vec<AssetId>, AccountId = T::AccountId>,
	{
		T::ShareAccountId::from_assets(&self.assets, Some(POOL_IDENTIFIER))
	}

	pub(crate) fn balances<T: Config>(&self) -> Vec<Balance>
	where
		T::ShareAccountId: ShareAccountIdFor<Vec<AssetId>, AccountId = T::AccountId>,
		T::AssetId: From<AssetId>,
	{
		let acc = self.pool_account::<T>();
		self.assets
			.iter()
			.map(|asset| T::Currency::free_balance((*asset).into(), &acc))
			.collect()
	}
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, TypeInfo)]
pub struct AssetLiquidity<AssetId> {
	pub asset_id: AssetId,
	pub amount: Balance,
}
