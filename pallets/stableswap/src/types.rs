use crate::MAX_ASSETS_IN_POOL;
use sp_runtime::Permill;
use sp_std::prelude::*;
use std::collections::BTreeSet;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::ConstU32;
use frame_support::BoundedVec;
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
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct PoolInfo<AssetId> {
	pub assets: BoundedVec<AssetId, ConstU32<MAX_ASSETS_IN_POOL>>,
	pub amplification: u16,
	pub trade_fee: Permill,
	pub withdraw_fee: Permill,
}

fn has_unique_elements<T>(iter: &mut T) -> bool
where
	T: Iterator,
	T::Item: Ord,
{
	let mut uniq = BTreeSet::new();
	iter.all(move |x| uniq.insert(x))
}

impl<AssetId> PoolInfo<AssetId>
where
	AssetId: Ord,
{
	pub(crate) fn find_asset(&self, asset: AssetId) -> Option<usize> {
		self.assets.iter().position(|v| *v == asset)
	}

	pub(crate) fn is_valid(&self) -> bool {
		has_unique_elements(&mut self.assets.iter())
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

#[derive(Debug, Clone, Encode, Decode, PartialEq, TypeInfo)]
pub struct AssetLiquidity<AssetId> {
	pub asset_id: AssetId,
	pub amount: Balance,
}
