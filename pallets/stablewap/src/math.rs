use crate::types::{AssetAmounts, Balance, FixedBalance, PoolInfo};
use sp_runtime::traits::Zero;

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

pub(crate) struct TradeChanges<AssetId> {
	pub pool: PoolInfo<AssetId, Balance, FixedBalance>,
	pub delta_amount_out: Balance,
}

pub(crate) fn calculate_sell_changes<AssetId>(
	pool: &PoolInfo<AssetId, Balance, FixedBalance>,
	_asset_in: AssetId,
	_asset_out: AssetId,
	_amount: Balance,
) -> Option<TradeChanges<AssetId>>
where
	AssetId: Clone,
{
	let new_pool = (*pool).clone();

	Some(TradeChanges {
		pool: new_pool,
		delta_amount_out: Balance::zero(),
	})
}

pub(crate) fn calculate_buy_changes<AssetId>(
	pool: &PoolInfo<AssetId, Balance, FixedBalance>,
	_asset_in: AssetId,
	_asset_out: AssetId,
	_amount: Balance,
) -> Option<TradeChanges<AssetId>>
where
	AssetId: Clone,
{
	let new_pool = (*pool).clone();

	Some(TradeChanges {
		pool: new_pool,
		delta_amount_out: Balance::zero(),
	})
}
