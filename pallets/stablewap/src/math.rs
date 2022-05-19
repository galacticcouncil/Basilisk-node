use crate::types::{Balance, FixedBalance, PoolInfo};
use sp_runtime::traits::Zero;

pub(crate) struct AssetAmountChanges<Balance> {
	pub share_amount: Balance,
}

pub(crate) fn calculate_add_liquidity_changes<AssetId>(
	_pool: &PoolInfo<AssetId, FixedBalance>,
	_asset: AssetId,
	_reserve: Balance,
	_amount: Balance,
) -> Option<AssetAmountChanges<Balance>> {
	None
}

pub(crate) struct TradeChanges {
	pub delta_amount_out: Balance,
}

pub(crate) fn calculate_sell_changes<AssetId>(
	_pool: &PoolInfo<AssetId, FixedBalance>,
	_asset_in: AssetId,
	_asset_out: AssetId,
	_amount: Balance,
) -> Option<TradeChanges> {
	Some(TradeChanges {
		delta_amount_out: Balance::zero(),
	})
}

pub(crate) fn calculate_buy_changes<AssetId>(
	_pool: &PoolInfo<AssetId, FixedBalance>,
	_asset_in: AssetId,
	_asset_out: AssetId,
	_amount: Balance,
) -> Option<TradeChanges> {
	Some(TradeChanges {
		delta_amount_out: Balance::zero(),
	})
}
