use sp_runtime::{FixedU128, Permill};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

pub type Balance = u128;
pub type FixedBalance = FixedU128;

#[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PoolInfo<AssetId, Balance, FixedBalance> {
	pub(crate) share_asset: AssetId,
	pub(crate) amplification: FixedBalance,
	pub(crate) balances: PoolBalances<Balance>,
	pub(crate) fee: Permill,
}

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PoolAssets<AssetId>(AssetId, AssetId);

#[derive(Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Default)]
pub struct PoolBalances<Balance>(pub Balance, pub Balance);

impl<AssetId: PartialOrd> From<(AssetId, AssetId)> for PoolAssets<AssetId> {
	fn from(assets: (AssetId, AssetId)) -> Self {
		if assets.0 < assets.1 {
			Self(assets.0, assets.1)
		} else {
			Self(assets.1, assets.0)
		}
	}
}

impl<AssetId: PartialEq> PoolAssets<AssetId> {
	pub fn is_valid(&self) -> bool {
		self.0 != self.1
	}
}

impl<AssetId: Copy> From<&PoolAssets<AssetId>> for Vec<AssetId> {
	fn from(assets: &PoolAssets<AssetId>) -> Self {
		vec![assets.0, assets.1]
	}
}
