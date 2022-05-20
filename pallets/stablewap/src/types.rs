use sp_runtime::Permill;
use std::ops::Add;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_runtime::traits::{CheckedAdd, Zero};

pub type Balance = u128;

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PoolId<AssetId>(pub AssetId);

//////////////

#[derive(Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PoolInfo<AssetId, Balance> {
	pub(crate) assets: PoolAssets<AssetId>,
	pub(crate) amplification: Balance,
	pub(crate) fee: Permill,
}

//////////////

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PoolAssets<AssetId>(pub AssetId, pub AssetId);

impl<AssetId: PartialEq> PoolAssets<AssetId> {
	pub fn contains(&self, value: AssetId) -> bool {
		self.0 == value || self.1 == value
	}
}

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

impl<AssetId> From<PoolAssets<AssetId>> for Vec<AssetId> {
	fn from(assets: PoolAssets<AssetId>) -> Self {
		vec![assets.0, assets.1]
	}
}

pub struct PoolAssetIterator<AssetId> {
	iter: sp_std::vec::IntoIter<AssetId>,
}

impl<'a, AssetId: Copy> IntoIterator for &'a PoolAssets<AssetId> {
	type Item = AssetId;
	type IntoIter = PoolAssetIterator<AssetId>;

	fn into_iter(self) -> Self::IntoIter {
		let v: Vec<AssetId> = self.into();

		PoolAssetIterator { iter: v.into_iter() }
	}
}

impl<AssetId> Iterator for PoolAssetIterator<AssetId> {
	type Item = AssetId;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next()
	}
}

//////////////

#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo, Default)]
pub struct AssetAmounts<Balance>(pub Balance, pub Balance);

impl<Balance> From<(Balance, Balance)> for AssetAmounts<Balance> {
	fn from(amounts: (Balance, Balance)) -> Self {
		Self(amounts.0, amounts.1)
	}
}

impl<Balance: PartialOrd + Zero> AssetAmounts<Balance> {
	pub fn valid(&self) -> bool {
		self.0 > Balance::zero() && self.1 > Balance::zero()
	}
}

impl<Balance: CheckedAdd> Add<Self> for AssetAmounts<Balance> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0, self.1 + rhs.1)
	}
}

impl<Balance: Copy> From<&AssetAmounts<Balance>> for Vec<Balance> {
	fn from(amounts: &AssetAmounts<Balance>) -> Self {
		vec![amounts.0, amounts.1]
	}
}

impl<Balance: CheckedAdd> CheckedAdd for AssetAmounts<Balance> {
	fn checked_add(&self, v: &Self) -> Option<Self> {
		let first = self.0.checked_add(&v.0)?;
		let second = self.1.checked_add(&v.1)?;
		Some(AssetAmounts(first, second))
	}
}

pub struct AssetAmountIterator<Balance> {
	iter: sp_std::vec::IntoIter<Balance>,
}

impl<'a, Balance: Copy> IntoIterator for &'a AssetAmounts<Balance> {
	type Item = Balance;
	type IntoIter = AssetAmountIterator<Balance>;

	fn into_iter(self) -> Self::IntoIter {
		let v: Vec<Balance> = self.into();

		AssetAmountIterator { iter: v.into_iter() }
	}
}

impl<Balance> Iterator for AssetAmountIterator<Balance> {
	type Item = Balance;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next()
	}
}
