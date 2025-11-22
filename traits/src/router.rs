use hydradx_traits::router::{ExecutorError, PoolType};
use sp_arithmetic::FixedU128;

/// All AMMs used in the router are required to implement this trait.
pub trait TradeExecution<Origin, AccountId, AssetId, Balance> {
    type Error;

    fn calculate_sell(
        pool_type: PoolType<AssetId>,
        asset_in: AssetId,
        asset_out: AssetId,
        amount_in: Balance,
    ) -> Result<Balance, ExecutorError<Self::Error>>;

    fn calculate_buy(
        pool_type: PoolType<AssetId>,
        asset_in: AssetId,
        asset_out: AssetId,
        amount_out: Balance,
    ) -> Result<Balance, ExecutorError<Self::Error>>;

    fn execute_sell(
        who: Origin,
        pool_type: PoolType<AssetId>,
        asset_in: AssetId,
        asset_out: AssetId,
        amount_in: Balance,
        min_limit: Balance,
    ) -> Result<(), ExecutorError<Self::Error>>;

    fn execute_buy(
        who: Origin,
        pool_type: PoolType<AssetId>,
        asset_in: AssetId,
        asset_out: AssetId,
        amount_out: Balance,
        max_limit: Balance,
    ) -> Result<(), ExecutorError<Self::Error>>;

    fn get_liquidity_depth(
        pool_type: PoolType<AssetId>,
        asset_a: AssetId,
        asset_b: AssetId,
    ) -> Result<Balance, ExecutorError<Self::Error>>;

    fn calculate_spot_price_with_fee(
        pool_type: PoolType<AssetId>,
        asset_a: AssetId,
        asset_b: AssetId,
    ) -> Result<FixedU128, ExecutorError<Self::Error>>;
}