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

use codec::{Decode, Encode, MaxEncodedLen};

use primitive_types::U256;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

use frame_support::sp_runtime::FixedU128;
use sp_runtime::RuntimeDebug;

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

/// NFT Class ID
pub type ClassId = u128;

/// NFT Instance ID
pub type InstanceId = u128;

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

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, PartialOrd, Ord, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[repr(u8)]
pub enum ReserveIdentifier {
	Nft,
	Marketplace,
	Auction,

	// always the last, indicate number of variants
	Count,
}

pub mod nft {
	use super::*;

	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum ClassType {
		Marketplace = 0_isize,
		LiquidityMining = 1_isize,
		Redeemable = 2_isize,
		Auction = 3_isize,
		HydraHeads = 4_isize,
	}

	impl Default for ClassType {
		fn default() -> Self {
			ClassType::Marketplace
		}
	}

	pub trait NftPermission<InnerClassType> {
		fn can_create(class_type: &InnerClassType) -> bool;
		fn can_mint(class_type: &InnerClassType) -> bool;
		fn can_transfer(class_type: &InnerClassType) -> bool;
		fn can_burn(class_type: &InnerClassType) -> bool;
		fn can_destroy(class_type: &InnerClassType) -> bool;
		fn has_deposit(class_type: &InnerClassType) -> bool;
	}

	#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct NftPermissions;

	impl NftPermission<ClassType> for NftPermissions {
		fn can_create(class_type: &ClassType) -> bool {
			matches!(*class_type, ClassType::Marketplace)
		}

		fn can_mint(class_type: &ClassType) -> bool {
			matches!(*class_type, ClassType::Marketplace)
		}

		fn can_transfer(class_type: &ClassType) -> bool {
			matches!(*class_type, ClassType::Marketplace | ClassType::LiquidityMining)
		}

		fn can_burn(class_type: &ClassType) -> bool {
			matches!(*class_type, ClassType::Marketplace)
		}

		fn can_destroy(class_type: &ClassType) -> bool {
			matches!(*class_type, ClassType::Marketplace)
		}

		fn has_deposit(class_type: &ClassType) -> bool {
			matches!(*class_type, ClassType::Marketplace)
		}
	}
}
