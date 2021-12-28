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

#![cfg(test)]

use crate::Config;
use frame_support::parameter_types;
use frame_support::traits::{Everything, GenesisBuild};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
};

use frame_support::weights::IdentityFee;
use hydradx_traits::AssetPairAccountIdFor;
use orml_currencies::BasicCurrencyAdapter;
use pallet_transaction_multi_payment::MultiCurrencyAdapter;
use primitives::{
	constants::chain::{MAX_IN_RATIO, MAX_OUT_RATIO, MIN_POOL_LIQUIDITY, MIN_TRADING_LIMIT},
	Amount, AssetId, Balance, Price,
};

use frame_support::traits::Get;
use std::cell::RefCell;

use frame_benchmarking::frame_support::weights::Pays;
use frame_system::EnsureSigned;

pub type AccountId = u64;

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
				 PaymentPallet: pallet_transaction_multi_payment::{Pallet, Call, Storage, Event<T>},
				 XYKPallet: pallet_xyk::{Pallet, Call, Storage, Event<T>},
				 Balances: pallet_balances::{Pallet,Call, Storage,Config<T>, Event<T>},
				 Currencies: orml_currencies::{Pallet, Event<T>},
				 Tokens: orml_tokens::{Pallet, Event<T>},
				 AssetRegistry: pallet_asset_registry::{Pallet, Storage, Event<T>},
				 TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		 }

);

parameter_types! {
	pub const BlockHashCount: u64 = 250;

	pub const HdxAssetId: u32 = 0;
	pub const ExistentialDeposit: u128 = 0;
	pub const MaxLocks: u32 = 50;
	pub const TransactionByteFee: Balance = 1;
	pub ExchangeFeeRate: (u32, u32) = (2, 1_000);
	pub PayForSetCurrency : Pays = Pays::Yes;
	pub const RegistryStringLimit: u32 = 100;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
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
	type SS58Prefix = ();
	type OnSetCode = ();
}
impl Config for Test {}

impl pallet_transaction_multi_payment::Config for Test {
	type Event = Event;
	type AcceptedCurrencyOrigin = frame_system::EnsureRoot<u64>;
	type Currencies = Currencies;
	type AMMPool = XYKPallet;
	type WeightInfo = ();
	type WithdrawFeeForSetCurrency = PayForSetCurrency;
	type WeightToFee = IdentityFee<Balance>;
}

impl pallet_asset_registry::Config for Test {
	type Event = Event;
	type RegistryOrigin = EnsureSigned<AccountId>;
	type AssetId = AssetId;
	type Balance = Balance;
	type AssetNativeLocation = u8;
	type StringLimit = RegistryStringLimit;
	type NativeAssetId = HdxAssetId;
	type WeightInfo = ();
}

impl pallet_balances::Config for Test {
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = ();
	/// The ubiquitous event type.
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

impl pallet_transaction_payment::Config for Test {
	type OnChargeTransaction = MultiCurrencyAdapter<Balances, (), PaymentPallet>;
	type TransactionByteFee = TransactionByteFee;
	type OperationalFeeMultiplier = ();
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

pub struct AssetPairAccountIdTest();

impl AssetPairAccountIdFor<AssetId, u64> for AssetPairAccountIdTest {
	fn from_assets(asset_a: AssetId, asset_b: AssetId, _: &str) -> u64 {
		let mut a = asset_a as u128;
		let mut b = asset_b as u128;
		if a > b {
			std::mem::swap(&mut a, &mut b)
		}
		(a * 1000 + b) as u64
	}
}

parameter_types! {
	pub const MinTradingLimit: Balance = MIN_TRADING_LIMIT;
	pub const MinPoolLiquidity: Balance = MIN_POOL_LIQUIDITY;
	pub const MaxInRatio: u128 = MAX_IN_RATIO;
	pub const MaxOutRatio: u128 = MAX_OUT_RATIO;
}

impl pallet_xyk::Config for Test {
	type Event = Event;
	type AssetRegistry = AssetRegistry;
	type AssetPairAccountId = AssetPairAccountIdTest;
	type Currency = Currencies;
	type NativeAssetId = HdxAssetId;
	type WeightInfo = ();
	type GetExchangeFee = ExchangeFeeRate;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type CanCreatePool = pallet_xyk::AllowAllPools;
	type AMMHandler = ();
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
	type DustRemovalWhitelist = Everything;
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
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			base_weight: 0,
			native_balances: vec![(1, 100_000)],
			endowed_accounts: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn base_weight(mut self, base_weight: u64) -> Self {
		self.base_weight = base_weight;
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

		pallet_transaction_multi_payment::GenesisConfig::<Test> {
			currencies: vec![(2, Price::from(1))],
			fallback_account: 1000,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
