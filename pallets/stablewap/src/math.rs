use crate::types::{AssetAmounts, Balance, FixedBalance, PoolInfo};

pub(crate) struct AssetAmountChanges<Balance> {
	pub delta_amounts: AssetAmounts<Balance>,
	pub share_amount: Balance,
}

pub(crate) fn calculate_add_liquidity_changes<AssetId>(
	_pool: &PoolInfo<AssetId, Balance, FixedBalance>,
	_added_amounts: &AssetAmounts<Balance>,
) -> Option<AssetAmountChanges<Balance>> {
	None
}
