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
use sp_std::vec::Vec;

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
	fn resolve_matched_intentions(pair_account: &AccountId, intention: &Intention, matched: &[&Intention]);
}

pub trait Registry<AssetId, AssetName, Balance, Error> {
	fn exists(name: AssetId) -> bool;

	fn retrieve_asset(name: &AssetName) -> Result<AssetId, Error>;

	fn create_asset(name: &AssetName, existential_deposit: Balance) -> Result<AssetId, Error>;

	fn get_or_create_asset(name: AssetName, existential_deposit: Balance) -> Result<AssetId, Error> {
		if let Ok(asset_id) = Self::retrieve_asset(&name) {
			Ok(asset_id)
		} else {
			Self::create_asset(&name, existential_deposit)
		}
	}
}

pub trait ShareTokenRegistry<AssetId, AssetName, Balance, Error>: Registry<AssetId, AssetName, Balance, Error> {
	fn retrieve_shared_asset(name: &AssetName, assets: &Vec<AssetId>) -> Result<AssetId, Error>;

	fn create_shared_asset(name: &AssetName, assets: &Vec<AssetId>, existential_deposit: Balance) -> Result<AssetId, Error>;

	fn get_or_create_shared_asset(name: AssetName, assets: Vec<AssetId>, existential_deposit: Balance) -> Result<AssetId, Error> {
		if let Ok(asset_id) = Self::retrieve_shared_asset(&name, &assets) {
			Ok(asset_id)
		} else {
			Self::create_shared_asset(&name, &assets, existential_deposit)
		}
	}
}
