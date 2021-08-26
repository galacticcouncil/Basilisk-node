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
use crate as multi_payment;
use crate::{Config, MultiCurrencyAdapter};
use frame_support::{parameter_types, weights::DispatchClass};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
	Perbill,
};

use frame_support::weights::IdentityFee;
use frame_support::weights::Weight;
use orml_currencies::BasicCurrencyAdapter;
use primitives::{Amount, AssetId, Balance, Price};

use pallet_xyk::AssetPairAccountIdFor;
use std::cell::RefCell;

use frame_support::traits::{GenesisBuild, Get};
use primitives::fee;

pub type AccountId = u64;

pub const INITIAL_BALANCE: Balance = 1_000_000_000_000_000u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const FALLBACK_ACCOUNT: AccountId = 300;

pub const HDX: AssetId = 0;
pub const SUPPORTED_CURRENCY_NO_BALANCE: AssetId = 2000;
pub const SUPPORTED_CURRENCY_WITH_BALANCE: AssetId = 3000;
pub const NOT_SUPPORTED_CURRENCY: AssetId = 4000;

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAX_BLOCK_WEIGHT: Weight = 1024;

thread_local! {
		static EXTRINSIC_BASE_WEIGHT: RefCell<u64> = RefCell::new(0);
}

pub struct ExtrinsicBaseWeight;
impl Get<u64> for ExtrinsicBaseWeight {
	fn get() -> u64 {
		EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow())
	}
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
	 Block = Block,
	 NodeBlock = Block,
	 UncheckedExtrinsic = UncheckedExtrinsic,
	 {
		 System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		 PaymentPallet: multi_payment::{Pallet, Call, Storage, Event<T>},
		 transaction_payment: pallet_transaction_payment::{Pallet, Storage},
		 XYKPallet: pallet_xyk::{Pallet, Call, Storage, Event<T>},
		 Balances: pallet_balances::{Pallet,Call, Storage,Config<T>, Event<T>},
		 Currencies: orml_currencies::{Pallet, Event<T>},
		 AssetRegistry: pallet_asset_registry::{Pallet, Storage},
		 Tokens: orml_tokens::{Pallet, Event<T>},
	 }

);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;

	pub const HdxAssetId: u32 = 0;
	pub const ExistentialDeposit: u128 = 0;
	pub const MaxLocks: u32 = 50;
	pub const TransactionByteFee: Balance = 1;

	pub RuntimeBlockWeights: system::limits::BlockWeights = system::limits::BlockWeights::builder()
		.base_block(10)
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAX_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAX_BLOCK_WEIGHT);
			weights.reserved = Some(
				MAX_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAX_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(Perbill::from_percent(0))
		.build_or_panic();

	pub ExchangeFeeRate: fee::Fee = fee::Fee::default();
	pub PayForSetCurrency : Pays = Pays::Yes;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

impl Config for Test {
	type Event = Event;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type AMMPool = XYKPallet;
	type WeightInfo = ();
	type WithdrawFeeForSetCurrency = PayForSetCurrency;
	type WeightToFee = IdentityFee<Balance>;
}

impl pallet_asset_registry::Config for Test {
	type AssetId = AssetId;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

impl pallet_transaction_payment::Config for Test {
	type OnChargeTransaction = MultiCurrencyAdapter<Balances, (), PaymentPallet>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}
pub struct AssetPairAccountIdTest();

impl AssetPairAccountIdFor<AssetId, u64> for AssetPairAccountIdTest {
	fn from_assets(asset_a: AssetId, asset_b: AssetId) -> u64 {
		let mut a = asset_a as u128;
		let mut b = asset_b as u128;
		if a > b {
			let tmp = a;
			a = b;
			b = tmp;
		}
		return (a * 1000 + b) as u64;
	}
}

impl pallet_xyk::Config for Test {
	type Event = Event;
	type AssetPairAccountId = AssetPairAccountIdTest;
	type Currency = Currencies;
	type NativeAssetId = HdxAssetId;
	type WeightInfo = ();
	type GetExchangeFee = ExchangeFeeRate;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		Zero::zero()
	};
}

impl orml_tokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = ();
}

impl orml_currencies::Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Test, Balances, Amount, u32>;
	type GetNativeCurrencyId = HdxAssetId;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	base_weight: u64,
	native_balances: Vec<(AccountId, Balance)>,
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
	payment_authority: AccountId,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			base_weight: 0,
			payment_authority: BOB,
			native_balances: vec![(ALICE, INITIAL_BALANCE), (BOB, 0)],
			endowed_accounts: vec![
				(ALICE, HDX, INITIAL_BALANCE),
				(ALICE, SUPPORTED_CURRENCY_NO_BALANCE, 0u128), // Used for insufficient balance testing
				(ALICE, SUPPORTED_CURRENCY_WITH_BALANCE, INITIAL_BALANCE),
			],
		}
	}
}

impl ExtBuilder {
	pub fn base_weight(mut self, base_weight: u64) -> Self {
		self.base_weight = base_weight;
		self
	}
	pub fn account_native_balance(mut self, account: AccountId, balance: Balance) -> Self {
		self.native_balances.push((account, balance));
		self
	}
	pub fn account_tokens(mut self, account: AccountId, asset: AssetId, balance: Balance) -> Self {
		self.endowed_accounts.push((account, asset, balance));
		self
	}
	fn set_constants(&self) {
		EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow_mut() = self.base_weight);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_constants();
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: self.native_balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Test> {
			balances: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let core_asset: u32 = 0;
		let mut buf: Vec<u8> = Vec::new();

		buf.extend_from_slice(&core_asset.to_le_bytes());
		buf.extend_from_slice(b"HDT");
		buf.extend_from_slice(&core_asset.to_le_bytes());

		pallet_asset_registry::GenesisConfig::<Test> {
			core_asset_id: 0,
			next_asset_id: 2,
			asset_ids: vec![(buf.to_vec(), 1)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		crate::GenesisConfig::<Test> {
			currencies: vec![
				(SUPPORTED_CURRENCY_NO_BALANCE, Price::from(1)),
				(SUPPORTED_CURRENCY_WITH_BALANCE, Price::from_float(1.5)),
			],
			authorities: vec![self.payment_authority],
			fallback_account: FALLBACK_ACCOUNT,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
