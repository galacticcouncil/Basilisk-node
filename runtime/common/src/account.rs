use crate::{AccountId, AssetId};
use frame_support::sp_runtime::traits::Hash;
use pallet_stableswap::traits::ShareAccountIdFor;
use pallet_stableswap::types::PoolAssets;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::traits::BlakeTwo256;
use sp_std::vec::Vec;

/// Account id constructor for given assets and an identifier.
/// Used to construct account id for `Assets` which uniquely identifies a pool (xyk, stableswap)
pub struct AccountIdForStableswap;

impl ShareAccountIdFor<PoolAssets<AssetId>> for AccountIdForStableswap {
	type AccountId = AccountId;

	fn from_assets(assets: &PoolAssets<AssetId>, identifier: Option<&[u8]>) -> Self::AccountId {
		AccountId::unchecked_from(BlakeTwo256::hash(&Self::name(assets, identifier)[..]))
	}

	/// Create a name to uniquely identify a share account id for given assets and an identifier.
	fn name(assets: &PoolAssets<AssetId>, identifier: Option<&[u8]>) -> Vec<u8> {
		let mut buf: Vec<u8> = identifier.unwrap_or(b"").to_vec();
		buf.extend_from_slice(&assets.0.to_le_bytes());
		buf.extend_from_slice(&assets.1.to_le_bytes());
		buf
	}
}
