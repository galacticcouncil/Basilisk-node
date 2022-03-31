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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode};

use primitive_types::U256;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

use frame_support::sp_runtime::FixedU128;

pub mod asset;
pub mod constants;

/// An index to a block.
pub type BlockNumber = u32;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Type for storing the id of an asset.
pub type AssetId = u32;

/// Type for storing the balance of an account.
pub type Balance = u128;

/// Signed version of Balance
pub type Amount = i128;

/// Price
pub type Price = FixedU128;

/// Scaled Unsigned of Balance
pub type HighPrecisionBalance = U256;
pub type LowPrecisionBalance = u128;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode, Clone, Copy, PartialEq, Eq, TypeInfo)]
pub enum IntentionType {
	SELL,
	BUY,
}

impl Default for IntentionType {
	fn default() -> IntentionType {
		IntentionType::SELL
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct ExchangeIntention<AccountId, Balance, IntentionID> {
	pub who: AccountId,
	pub assets: asset::AssetPair,
	pub amount_in: Balance,
	pub amount_out: Balance,
	pub trade_limit: Balance,
	pub discount: bool,
	pub sell_or_buy: IntentionType,
	pub intention_id: IntentionID,
}
