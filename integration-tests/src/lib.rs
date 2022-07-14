mod cross_chain_transfer;
mod fees;
mod kusama_test_net;
mod nft_marketplace;
mod non_native_fee;

use polkadot_xcm::{latest::prelude::*, VersionedMultiAssets};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};

// Determine the hash for assets expected to be have been trapped.
#[allow(unused)]
pub(crate) fn determine_hash<M>(origin: &MultiLocation, assets: M) -> H256
where
	M: Into<MultiAssets>,
{
	let versioned = VersionedMultiAssets::from(assets.into());
	BlakeTwo256::hash_of(&(origin, &versioned))
}
