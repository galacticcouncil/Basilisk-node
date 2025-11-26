#![cfg_attr(not(feature = "std"), no_std)]

pub mod oracle;
pub mod router;


use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use frame_support::sp_runtime::{traits::Zero, DispatchError, RuntimeDebug};
use frame_support::weights::Weight;
use frame_support::dispatch::{self};

/// Identifier for oracle data sources.
pub type Source = [u8; 8];

/// Handler used by AMM pools to perform some tasks when a new pool is created.
pub trait OnCreatePoolHandler<AssetId> {
    /// Register an asset to be handled by price-oracle pallet.
    /// If an asset is not registered, calling `on_trade` results in populating the price buffer in the price oracle pallet,
    /// but the entries are ignored and the average price for the asset is not calculated.
    fn on_create_pool(asset_a: AssetId, asset_b: AssetId) -> dispatch::DispatchResult;
}

/// Handler used by AMM pools to perform some tasks when a trade is executed.
pub trait OnTradeHandler<AssetId, Balance, Price> {
    /// Include a trade in the average price calculation of the price-oracle pallet.
    #[allow(clippy::too_many_arguments)]
    fn on_trade(
        source: Source,
        asset_a: AssetId,
        asset_b: AssetId,
        amount_a: Balance,
        amount_b: Balance,
        liquidity_a: Balance,
        liquidity_b: Balance,
        price: Price,
    ) -> Result<Weight, (Weight, DispatchError)>;
    /// Known overhead for a trade in `on_initialize/on_finalize`.
    /// Needs to be specified here if we don't want to make AMM pools tightly coupled with the price oracle pallet, otherwise we can't access the weight.
    /// Add this weight to an extrinsic from which you call `on_trade`.
    fn on_trade_weight() -> Weight;
}

impl<AssetId, Balance, Price> OnTradeHandler<AssetId, Balance, Price> for () {
    fn on_trade(
        _source: Source,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount_a: Balance,
        _amount_b: Balance,
        _liquidity_a: Balance,
        _liquidity_b: Balance,
        _price: Price,
    ) -> Result<Weight, (Weight, DispatchError)> {
        Ok(Weight::zero())
    }
    fn on_trade_weight() -> Weight {
        Weight::zero()
    }
}

/// Handler used by AMM pools to perform some tasks when liquidity changes outside of trades.
pub trait OnLiquidityChangedHandler<AssetId, Balance, Price> {
    /// Notify that the liquidity for a pair of assets has changed.
    #[allow(clippy::too_many_arguments)]
    fn on_liquidity_changed(
        source: Source,
        asset_a: AssetId,
        asset_b: AssetId,
        amount_a: Balance,
        amount_b: Balance,
        liquidity_a: Balance,
        liquidity_b: Balance,
        price: Price,
    ) -> Result<Weight, (Weight, DispatchError)>;
    /// Known overhead for a liquidity change in `on_initialize/on_finalize`.
    /// Needs to be specified here if we don't want to make AMM pools tightly coupled with the price oracle pallet, otherwise we can't access the weight.
    /// Add this weight to an extrinsic from which you call `on_liquidity_changed`.
    fn on_liquidity_changed_weight() -> Weight;
}

impl<AssetId, Balance, Price> OnLiquidityChangedHandler<AssetId, Balance, Price> for () {
    fn on_liquidity_changed(
        _source: Source,
        _a: AssetId,
        _b: AssetId,
        _amount_a: Balance,
        _amount_b: Balance,
        _liq_a: Balance,
        _liq_b: Balance,
        _price: Price,
    ) -> Result<Weight, (Weight, DispatchError)> {
        Ok(Weight::zero())
    }

    fn on_liquidity_changed_weight() -> Weight {
        Weight::zero()
    }
}