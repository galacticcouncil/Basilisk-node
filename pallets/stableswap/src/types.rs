use sp_runtime::Permill;
use sp_std::vec;
use sp_std::vec::Vec;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_runtime::traits::Zero;

pub(crate) type Balance = u128;

/// Pool identifier. Share Asset id is used as pool identifier.
/// Share asset is unique token for each pool. That means using share asset as pool identifier
/// does not require additional "tracking" id for newly created pools.
#[derive(Clone, Copy, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Eq)]
pub struct PoolId<AssetId>(pub AssetId);

/// Pool properties for 2-asset pool (v1)
/// `assets`: pool assets
/// `amplification`: amp parameter
/// `fee`: trade fee to be withdrawn on sell/buy
#[derive(Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PoolInfo<AssetId> {
	pub assets: PoolAssets<AssetId>,
	pub amplification: u16,
	pub fee: Permill,
}

impl<AssetId> PoolInfo<AssetId>
where
	AssetId: PartialOrd,
{
	/// Check if an asset is in the pool
	pub(crate) fn contains_asset(&self, asset: AssetId) -> bool {
		self.assets.contains(asset)
	}
}

/// Assets in a pool.
/// Supports 2-asset pools.
/// Asset's tuple is ordered by id where first asset id < second asset id.
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PoolAssets<AssetId>(pub AssetId, pub AssetId);

impl<AssetId: PartialOrd> PoolAssets<AssetId> {
	pub fn new(asset_a: AssetId, asset_b: AssetId) -> Self {
		(asset_a, asset_b).into()
	}

	pub fn contains(&self, value: AssetId) -> bool {
		self.0 == value || self.1 == value
	}

	/// PoolAssets is valid only if assets are not equal
	pub fn is_valid(&self) -> bool {
		self.0 != self.1
	}
}

impl<AssetId: PartialOrd> From<(AssetId, AssetId)> for PoolAssets<AssetId> {
	fn from(assets: (AssetId, AssetId)) -> Self {
		// Order assets by id
		if assets.0 < assets.1 {
			Self(assets.0, assets.1)
		} else {
			Self(assets.1, assets.0)
		}
	}
}

impl<AssetId: Copy> From<&PoolAssets<AssetId>> for Vec<AssetId> {
	fn from(assets: &PoolAssets<AssetId>) -> Self {
		vec![assets.0, assets.1]
	}
}

/// Pool asset's reserve amounts.
/// Used together with `PoolAssets` where first reserve is for `PoolAssets.0`
#[derive(Clone, PartialEq, Default)]
pub struct AssetAmounts<Balance>(pub Balance, pub Balance);

impl<Balance> From<(Balance, Balance)> for AssetAmounts<Balance> {
	fn from(amounts: (Balance, Balance)) -> Self {
		Self(amounts.0, amounts.1)
	}
}

impl<Balance: PartialOrd + Zero> AssetAmounts<Balance> {
	pub fn is_valid(&self) -> bool {
		self.0 > Balance::zero() && self.1 > Balance::zero()
	}
}

impl<Balance: Copy> From<&AssetAmounts<Balance>> for Vec<Balance> {
	fn from(amounts: &AssetAmounts<Balance>) -> Self {
		vec![amounts.0, amounts.1]
	}
}
