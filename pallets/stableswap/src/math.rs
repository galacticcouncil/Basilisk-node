use crate::types::{AssetAmounts, Balance};

const D_ITERATIONS: u8 = 128;
const Y_ITERATIONS: u8 = 64;

/// Calculate shares amount after liquidity is added to the pool.
///
/// No fee applied. Currently is expected that liquidity of both assets are added to the pool.
///
/// share_amount = share_supply * ( d1 - d0 ) / d0
///
/// Returns `Some(shares)` when successful.
pub(crate) fn calculate_add_liquidity_shares(
	initial_reserves: &AssetAmounts<Balance>,
	updated_reserves: &AssetAmounts<Balance>,
	precision: Balance,
	amplification: Balance,
	share_issuance: Balance,
) -> Option<Balance> {
	hydra_dx_math::stableswap::calculate_add_liquidity_shares::<D_ITERATIONS>(
		&[initial_reserves.0, initial_reserves.1],
		&[updated_reserves.0, updated_reserves.1],
		amplification,
		precision,
		share_issuance,
	)
}

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

	Some(result.into())
}

pub(crate) fn calculate_out_given_in(
	reserve_in: Balance,
	reserve_out: Balance,
	amount_in: Balance,
	precision: Balance,
	amplification: Balance,
) -> Option<Balance> {
	hydra_dx_math::stableswap::calculate_out_given_in::<D_ITERATIONS, Y_ITERATIONS>(
		reserve_in,
		reserve_out,
		amount_in,
		amplification,
		precision,
	)
}

pub(crate) fn calculate_in_given_out(
	reserve_in: Balance,
	reserve_out: Balance,
	amount_out: Balance,
	precision: Balance,
	amplification: Balance,
) -> Option<Balance> {
	hydra_dx_math::stableswap::calculate_in_given_out::<D_ITERATIONS, Y_ITERATIONS>(
		reserve_in,
		reserve_out,
		amount_out,
		amplification,
		precision,
	)
}
