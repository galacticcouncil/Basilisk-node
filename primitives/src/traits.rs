// This file is part of Basilisk-node.

// Copyright (C) 2020-2021  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::upper_case_acronyms)]

use frame_support::dispatch;
use frame_support::sp_runtime::traits::CheckedDiv;
use sp_std::vec::Vec;
use crate::{Balance, Price};
use crate::asset::AssetPair;

/// Hold information to perform amm transfer
/// Contains also exact amount which will be sold/bought
pub struct AMMTransfer<AccountId, AssetId, AssetPair, Balance> {
	pub origin: AccountId,
	pub assets: AssetPair,
	pub amount: Balance,
	pub amount_out: Balance,
	pub discount: bool,
	pub discount_amount: Balance,
	pub fee: (AssetId, Balance),
}

impl<AccountId, AssetId> AMMTransfer<AccountId, AssetId, AssetPair, Balance>
where
	Balance: Copy
{
	pub fn normalize_price(&self) -> Option<(Price, Balance)> {
		let ordered_asset_pair = self.assets.ordered_pair();
		let (balance_a, balance_b) = if ordered_asset_pair.0 == self.assets.asset_in {
			(self.amount, self.amount_out)
		} else {
			(self.amount_out, self.amount)
		};

        let price_a = Price::from(balance_a);
		let price_b = Price::from(balance_b);
		let price = price_a.checked_div(&price_b);
        price.map(|p| (p, balance_a))
	}
}

/// Traits for handling AMM Pool trades.
pub trait AMM<AccountId, AssetId, AssetPair, Amount> {
	/// Check if both assets exist in a pool.
	fn exists(assets: AssetPair) -> bool;

	/// Return pair account.
	fn get_pair_id(assets: AssetPair) -> AccountId;

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
	fn execute_buy(transfer: &AMMTransfer<AccountId, AssetId, AssetPair, Amount>) -> dispatch::DispatchResult;

	/// Perform asset swap.
	fn buy(
		origin: &AccountId,
		assets: AssetPair,
		amount: Amount,
		max_limit: Amount,
		discount: bool,
	) -> dispatch::DispatchResult {
		Self::execute_buy(&Self::validate_buy(origin, assets, amount, max_limit, discount)?)?;
		Ok(())
	}
}

pub trait Resolver<AccountId, Intention, E> {
	/// Resolve an intention directl via AMM pool.
	fn resolve_single_intention(intention: &Intention);

	/// Resolve intentions by either directly trading with each other or via AMM pool.
	/// Intention ```intention``` must be validated prior to call this function.
	fn resolve_matched_intentions(pair_account: &AccountId, intention: &Intention, matched: &[Intention]);
}

pub trait AMMHandlers<AccountId, AssetId, AssetPair, Balance>
{
	fn on_create_pool(asset_pair: AssetPair);
	// fn on_add_liquidity(asset_pair: AssetPair, amount: Balance);
	// fn on_remove_liquidity(asset_pair: AssetPair, amount: Balance);
	fn on_trade(amm_transfer: &AMMTransfer<AccountId, AssetId, AssetPair, Balance>, liq_amount: Balance);
}

impl<AccountId, AssetId, AssetPair, Balance> AMMHandlers<AccountId, AssetId, AssetPair, Balance> for () {
	fn on_create_pool(_asset_pair: AssetPair) {}
	fn on_trade(_amm_transfer: &AMMTransfer<AccountId, AssetId, AssetPair, Balance>, _liq_amount: Balance) {}
}

pub trait AssetPairAccountIdFor<AssetId: Sized, AccountId: Sized> {
	fn from_assets(asset_a: AssetId, asset_b: AssetId) -> AccountId;
}
