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

use super::*;
use frame_support::traits::BalanceStatus;

/// Hold info about each transfer which has to be made to resolve a direct trade.
pub struct Transfer<'a, T: Config> {
	pub from: &'a T::AccountId,
	pub to: &'a T::AccountId,
	pub asset: AssetId,
	pub amount: Balance,
	pub fee_transfer: bool,
}

/// Hold info about a direct trade between two intentions.
/// After a direct trade is prepared - ```transfers``` contains all necessary transfers to complete the trade.
pub struct DirectTradeData<'a, T: Config> {
	pub intention_a: &'a Intention<T>,
	pub intention_b: &'a Intention<T>,
	pub amount_from_a: Balance,
	pub amount_from_b: Balance,
	pub transfers: Vec<Transfer<'a, T>>,
}

/// Direct trade implementation
/// Represents direct trade between two accounts
impl<'a, T: Config> DirectTradeData<'a, T> {
	/// Prepare direct trade
	/// 1. Validate balances
	/// 2. Calculate fees
	/// 3. Reserve amounts for each transfer ( including fee transfers )
	pub fn prepare(&mut self, pool_account: &'a T::AccountId) -> bool {
		if T::Currency::free_balance(self.intention_a.assets.asset_in, &self.intention_a.who) < self.amount_from_a {
			Self::send_insufficient_balance_event(self.intention_a, self.intention_a.assets.asset_in);
			return false;
		}
		if T::Currency::free_balance(self.intention_a.assets.asset_out, &self.intention_b.who) < self.amount_from_b {
			Self::send_insufficient_balance_event(self.intention_b, self.intention_a.assets.asset_out);
			return false;
		}

		// Let's handle the fees now for registered transfers.
		let default_fee = (2, 1_000); // 0.2%
		let fee_a = hydra_dx_math::fee::calculate_pool_trade_fee(self.amount_from_a, default_fee);
		let fee_b = hydra_dx_math::fee::calculate_pool_trade_fee(self.amount_from_b, default_fee);

		if fee_a.is_none() || fee_b.is_none() {
			return false;
		}

		// Unwrapping is correct as None case is handled in previous statement.
		let transfer_a_fee = fee_a.unwrap();
		let transfer_b_fee = fee_b.unwrap();
		if transfer_a_fee > self.amount_from_a || transfer_b_fee > self.amount_from_b {
			return false;
		}

		// Work out where to take a fee from.
		// There are multiple possible scenarios to consider
		// 1. SELL - SELL
		// 2. SELL - BUY
		// 3. BUY - SELL
		// 4. BUY - BUY
		// Each one is handled slightly different, hence the complicated match statement.
		match (&self.intention_a.sell_or_buy, &self.intention_b.sell_or_buy) {
			(IntentionType::SELL, IntentionType::SELL) => {
				if !Self::reserve_if_can(
					self.intention_a.assets.asset_in,
					&self.intention_a.who,
					self.amount_from_a,
				) {
					return false;
				}

				if !Self::reserve_if_can(
					self.intention_a.assets.asset_out,
					&self.intention_b.who,
					self.amount_from_b,
				) {
					return false;
				}

				let transfer = Transfer::<T> {
					from: &self.intention_a.who,
					to: &self.intention_b.who,
					asset: self.intention_a.assets.asset_in,
					amount: self.amount_from_a - transfer_a_fee,
					fee_transfer: false,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_b.who,
					to: &self.intention_a.who,
					asset: self.intention_a.assets.asset_out,
					amount: self.amount_from_b - transfer_b_fee,
					fee_transfer: false,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_a.who,
					to: pool_account,
					asset: self.intention_a.assets.asset_in,
					amount: transfer_a_fee,
					fee_transfer: true,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_b.who,
					to: pool_account,
					asset: self.intention_b.assets.asset_in,
					amount: transfer_b_fee,
					fee_transfer: true,
				};
				self.transfers.push(transfer);
			}
			(IntentionType::BUY, IntentionType::BUY) => {
				if !Self::reserve_if_can(
					self.intention_a.assets.asset_in,
					&self.intention_a.who,
					self.amount_from_a + transfer_a_fee,
				) {
					return false;
				}

				if !Self::reserve_if_can(
					self.intention_a.assets.asset_out,
					&self.intention_b.who,
					self.amount_from_b + transfer_b_fee,
				) {
					return false;
				}

				let transfer = Transfer::<T> {
					from: &self.intention_a.who,
					to: &self.intention_b.who,
					asset: self.intention_a.assets.asset_in,
					amount: self.amount_from_a,
					fee_transfer: false,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_b.who,
					to: &self.intention_a.who,
					asset: self.intention_a.assets.asset_out,
					amount: self.amount_from_b,
					fee_transfer: false,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_a.who,
					to: pool_account,
					asset: self.intention_a.assets.asset_in,
					amount: transfer_a_fee,
					fee_transfer: true,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_b.who,
					to: pool_account,
					asset: self.intention_b.assets.asset_in,
					amount: transfer_b_fee,
					fee_transfer: true,
				};
				self.transfers.push(transfer);
			}
			(IntentionType::BUY, IntentionType::SELL) => {
				if !Self::reserve_if_can(
					self.intention_a.assets.asset_in,
					&self.intention_a.who,
					self.amount_from_a + transfer_a_fee,
				) {
					return false;
				}

				if !Self::reserve_if_can(
					self.intention_a.assets.asset_out,
					&self.intention_b.who,
					self.amount_from_b,
				) {
					return false;
				}

				let transfer = Transfer::<T> {
					from: &self.intention_a.who,
					to: &self.intention_b.who,
					asset: self.intention_a.assets.asset_in,
					amount: self.amount_from_a - transfer_a_fee,
					fee_transfer: false,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_b.who,
					to: &self.intention_a.who,
					asset: self.intention_a.assets.asset_out,
					amount: self.amount_from_b,
					fee_transfer: false,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_a.who,
					to: pool_account,
					asset: self.intention_a.assets.asset_in,
					amount: 2 * transfer_a_fee,
					fee_transfer: true,
				};
				self.transfers.push(transfer);
			}
			(IntentionType::SELL, IntentionType::BUY) => {
				if !Self::reserve_if_can(
					self.intention_a.assets.asset_in,
					&self.intention_a.who,
					self.amount_from_a,
				) {
					return false;
				}

				if !Self::reserve_if_can(
					self.intention_a.assets.asset_out,
					&self.intention_b.who,
					self.amount_from_b + transfer_b_fee,
				) {
					return false;
				}

				let transfer = Transfer::<T> {
					from: &self.intention_a.who,
					to: &self.intention_b.who,
					asset: self.intention_a.assets.asset_in,
					amount: self.amount_from_a,
					fee_transfer: false,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_b.who,
					to: &self.intention_a.who,
					asset: self.intention_a.assets.asset_out,
					amount: self.amount_from_b - transfer_b_fee,
					fee_transfer: false,
				};
				self.transfers.push(transfer);

				let transfer = Transfer::<T> {
					from: &self.intention_b.who,
					to: pool_account,
					asset: self.intention_b.assets.asset_in,
					amount: 2 * transfer_b_fee,
					fee_transfer: true,
				};
				self.transfers.push(transfer);
			}
		}

		true
	}

	/// Execute direct trade.
	/// Trade must be prepared first. Execute all transfers.
	pub fn execute(&self) -> bool {
		self.send_direct_trade_resolve_event();

		for transfer in &self.transfers {
			if T::Currency::repatriate_reserved(
				transfer.asset,
				transfer.from,
				transfer.to,
				transfer.amount,
				BalanceStatus::Free,
			)
			.is_err()
			{
				return false;
			}

			if transfer.fee_transfer {
				let intention = if transfer.from == &self.intention_a.who {
					self.intention_a
				} else {
					self.intention_b
				};
				Self::send_trade_fee_event(transfer.from, intention, transfer.to, transfer.asset, transfer.amount);
			}
		}
		true
	}

	/// Revert all reserved amounts.
	/// This does NOT revert transfers, only reserved amounts. So it can be only called if a preparation fails.
	pub fn revert(&mut self) {
		for transfer in &self.transfers {
			T::Currency::unreserve(transfer.asset, transfer.from, transfer.amount);
		}
	}

	/// Send pallet event in case of insufficient balance.
	fn send_insufficient_balance_event(intention: &Intention<T>, asset: AssetId) {
		Pallet::<T>::deposit_event(Event::InsufficientAssetBalanceEvent(
			intention.who.clone(),
			asset,
			intention.sell_or_buy,
			intention.intention_id,
			Error::<T>::InsufficientAssetBalance.into(),
		));
	}

	/// Send pallet event after a fee is transferred.
	fn send_trade_fee_event(
		from: &T::AccountId,
		intention: &Intention<T>,
		to: &T::AccountId,
		asset: AssetId,
		amount: Balance,
	) {
		Pallet::<T>::deposit_event(Event::IntentionResolvedDirectTradeFees(
			from.clone(),
			intention.intention_id,
			to.clone(),
			asset,
			amount,
		));
	}

	/// Send event after successful direct trade.
	fn send_direct_trade_resolve_event(&self) {
		Pallet::<T>::deposit_event(Event::IntentionResolvedDirectTrade(
			self.intention_a.who.clone(),
			self.intention_b.who.clone(),
			self.intention_a.intention_id,
			self.intention_b.intention_id,
			self.amount_from_a,
			self.amount_from_b,
		));
	}

	/// Reserve amount.
	fn reserve_if_can(asset: AssetId, who: &T::AccountId, amount: Balance) -> bool {
		T::Currency::reserve(asset, who, amount).is_ok()
	}
}
