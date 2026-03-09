#![cfg_attr(not(feature = "std"), no_std)]

pub mod oracle;
pub mod router;

use codec::{Decode, Encode};
use frame_support::dispatch::{self};
use frame_support::sp_runtime::{traits::Zero, DispatchError, RuntimeDebug};
use frame_support::weights::Weight;
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

/// Identifier for oracle data sources.
pub type Source = [u8; 8];

/// Hold information to perform amm transfer
/// Contains also exact amount which will be sold/bought
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct AMMTransfer<AccountId, AssetId, AssetPair, Balance> {
	pub origin: AccountId,
	pub assets: AssetPair,
	pub amount: Balance,
	pub amount_b: Balance,
	pub discount: bool,
	pub discount_amount: Balance,
	pub fee: (AssetId, Balance),
}

/// Traits for handling AMM Pool trades.
pub trait AMM<AccountId, AssetId, AssetPair, Amount: Zero> {
	/// Check if both assets exist in a pool.
	fn exists(assets: AssetPair) -> bool;

	/// Return pair account.
	fn get_pair_id(assets: AssetPair) -> AccountId;

	/// Return share token for assets.
	fn get_share_token(assets: AssetPair) -> AssetId;

	/// Return list of active assets in a given pool.
	fn get_pool_assets(pool_account_id: &AccountId) -> Option<Vec<AssetId>>;

	/// Calculate spot price for asset a and b.
	fn get_spot_price_unchecked(asset_a: AssetId, asset_b: AssetId, amount: Amount) -> Amount;

	/// Sell trade validation
	/// Perform all necessary checks to validate an intended sale.
	fn validate_sell(
		origin: &AccountId,
		assets: AssetPair,
		amount: Amount,
		min_bought: Amount,
		discount: bool,
	) -> Result<AMMTransfer<AccountId, AssetId, AssetPair, Amount>, frame_support::sp_runtime::DispatchError>;

	/// Execute buy for given validated transfer.
	fn execute_sell(transfer: &AMMTransfer<AccountId, AssetId, AssetPair, Amount>) -> dispatch::DispatchResult;

	/// Perform asset swap.
	/// Call execute following the validation.
	fn sell(
		origin: &AccountId,
		assets: AssetPair,
		amount: Amount,
		min_bought: Amount,
		discount: bool,
	) -> dispatch::DispatchResult {
		Self::execute_sell(&Self::validate_sell(origin, assets, amount, min_bought, discount)?)?;

		Ok(())
	}

	/// Buy trade validation
	/// Perform all necessary checks to validate an intended buy.
	fn validate_buy(
		origin: &AccountId,
		assets: AssetPair,
		amount: Amount,
		max_limit: Amount,
		discount: bool,
	) -> Result<AMMTransfer<AccountId, AssetId, AssetPair, Amount>, frame_support::sp_runtime::DispatchError>;

	/// Execute buy for given validated transfer.
	fn execute_buy(
		transfer: &AMMTransfer<AccountId, AssetId, AssetPair, Amount>,
		destination: Option<&AccountId>,
	) -> dispatch::DispatchResult;

	/// Perform asset swap.
	fn buy(
		origin: &AccountId,
		assets: AssetPair,
		amount: Amount,
		max_limit: Amount,
		discount: bool,
	) -> dispatch::DispatchResult {
		Self::execute_buy(&Self::validate_buy(origin, assets, amount, max_limit, discount)?, None)?;

		Ok(())
	}

	/// Perform asset swap and send bought assets to the destination account.
	fn buy_for(
		origin: &AccountId,
		assets: AssetPair,
		amount: Amount,
		max_limit: Amount,
		discount: bool,
		dest: &AccountId,
	) -> dispatch::DispatchResult {
		Self::execute_buy(
			&Self::validate_buy(origin, assets, amount, max_limit, discount)?,
			Some(dest),
		)?;
		Ok(())
	}
	fn get_min_trading_limit() -> Amount;

	fn get_min_pool_liquidity() -> Amount;

	fn get_max_in_ratio() -> u128;

	fn get_max_out_ratio() -> u128;

	fn get_fee(pool_account_id: &AccountId) -> (u32, u32);
}

/// Handler used by AMM pools to perform some tasks when a new pool is created.
pub trait OnCreatePoolHandler<AssetId> {
	/// Register an asset to be handled by price-oracle pallet.
	/// If an asset is not registered, calling `on_trade` results in populating the price buffer in the price oracle pallet,
	/// but the entries are ignored and the average price for the asset is not calculated.
	fn on_create_pool(asset_a: AssetId, asset_b: AssetId) -> dispatch::DispatchResult;
}

impl<AssetId> OnCreatePoolHandler<AssetId> for () {
	fn on_create_pool(_asset_a: AssetId, _asset_b: AssetId) -> dispatch::DispatchResult {
		Ok(())
	}
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

/// Provides account's fee payment asset
pub trait AccountFeeCurrency<AccountId> {
	type AssetId;
	fn get(a: &AccountId) -> Self::AssetId;
}
