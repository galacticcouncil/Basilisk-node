use crate::types::{AssetAmounts, Balance};

/// Given amount of shares and asset reserves, calculate corresponding amounts of each asset to be withdrawn.
pub(crate) fn calculate_remove_liquidity_amounts(
	reserves: &AssetAmounts<Balance>,
	shares: Balance,
	share_asset_issuance: Balance,
) -> Option<AssetAmounts<Balance>> {
	let result = hydra_dx_math::stableswap::calculate_remove_liquidity_amounts(
		&[reserves.0, reserves.1],
		shares,
		share_asset_issuance,
	)?;

	Some((result[0], result[1]).into())
}
