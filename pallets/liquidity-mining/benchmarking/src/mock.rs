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

use crate::dispatch::DispatchError;
use frame_support::{
	instances::Instance1,
	parameter_types,
	traits::{Everything, GenesisBuild, Nothing},
	PalletId,
};
use std::cell::RefCell;
use std::collections::HashMap;

use frame_system as system;
use frame_system::EnsureSigned;
use hydradx_traits::{pools::DustRemovalAccountWhitelist, AssetPairAccountIdFor};
use orml_traits::parameter_type_with_key;
use primitives::{
	constants::{
		chain::{DISCOUNTED_FEE, MAX_IN_RATIO, MAX_OUT_RATIO, MIN_POOL_LIQUIDITY, MIN_TRADING_LIMIT},
		currency::NATIVE_EXISTENTIAL_DEPOSIT,
	},
	Amount, AssetId, Balance,
};

use primitives::nft::{ClassType, NftPermissions};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
	DispatchResult,
};

pub const UNITS: Balance = 1_000_000_000_000;

pub type AccountId = u128;
pub type BlockNumber = u64;
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;

pub const INITIAL_BALANCE: u128 = 1_000_000_000_000;

pub const BSX: AssetId = 1000;
pub const KSM: AssetId = 4000;

pub const LIQ_MINING_NFT_CLASS: primitives::ClassId = 1;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
	Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		XYK: pallet_xyk::{Pallet, Call, Storage, Event<T>},
		LiquidityMining: pallet_liquidity_mining::{Pallet, Call, Storage, Event<T>},
		NFT: pallet_nft::{Pallet, Call, Event<T>, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
		Currency: orml_tokens::{Pallet, Event<T>},
		AssetRegistry: pallet_asset_registry::{Pallet, Storage, Event<T>},
		WarehouseLM: warehouse_liquidity_mining::<Instance1>::{Pallet, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
	pub static MockBlockNumberProvider: u64 = 0;
	pub const BSXAssetId: AssetId = BSX;
	pub ExchangeFeeRate: (u32, u32) = (2, 1_000);
	pub RegistryStringLimit: u32 = 100;
}

impl BlockNumberProvider for MockBlockNumberProvider {
	type BlockNumber = u64;

	fn current_block_number() -> Self::BlockNumber {
		System::block_number()
	}
}
impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl crate::Config for Test {}

parameter_types! {
	pub const WarehouseLMPalletId: PalletId = PalletId(*b"WhouseLm");
	pub const MinDeposit: Balance = 1;
	pub const MaxEntriesPerDeposit: u8 = 10;
}

impl warehouse_liquidity_mining::Config<Instance1> for Test {
	type CurrencyId = AssetId;
	type MultiCurrency = Currency;
	type PalletId = WarehouseLMPalletId;
	type MinTotalFarmRewards = MinTotalFarmRewards;
	type MinPlannedYieldingPeriods = MinPlannedYieldingPeriods;
	type BlockNumberProvider = MockBlockNumberProvider;
	type AmmPoolId = AccountId;
	type MaxFarmEntriesPerDeposit = MaxEntriesPerDeposit;
	type MaxYieldFarmsPerGlobalFarm = MaxYieldFarmsPerGlobalFarm;
	type Event = Event;
}

parameter_types! {
	pub const MaxLocks: u32 = 1;
	pub const LMPalletId: PalletId = PalletId(*b"LiqMinId");
	pub const MinPlannedYieldingPeriods: BlockNumber = 100;
	pub const MinTotalFarmRewards: Balance = 1_000_000;
	pub const NftClass: primitives::ClassId = LIQ_MINING_NFT_CLASS;
	pub const ReserveClassIdUpTo: u128 = 9999;
	pub const MaxYieldFarmsPerGlobalFarm: u8 = 5;
}

impl pallet_liquidity_mining::Config for Test {
	type Event = Event;
	type MultiCurrency = Currency;
	type CreateOrigin = frame_system::EnsureRoot<AccountId>;
	type PalletId = LMPalletId;
	type MinPlannedYieldingPeriods = MinPlannedYieldingPeriods;
	type MinTotalFarmRewards = MinTotalFarmRewards;
	type BlockNumberProvider = MockBlockNumberProvider;
	type NftClassId = NftClass;
	type AMM = XYK;
	type WeightInfo = ();
	type NFTHandler = NFT;
	type LiquidityMiningHandler = WarehouseLM;
}

impl pallet_nft::Config for Test {
	type Event = Event;
	type WeightInfo = pallet_nft::weights::BasiliskWeight<Test>;
	type NftClassId = primitives::ClassId;
	type NftInstanceId = primitives::InstanceId;
	type ProtocolOrigin = frame_system::EnsureRoot<AccountId>;
	type ClassType = ClassType;
	type Permissions = NftPermissions;
	type ReserveClassIdUpTo = ReserveClassIdUpTo;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = NATIVE_EXISTENTIAL_DEPOSIT;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
}

parameter_types! {
	pub const ClassDeposit: Balance = 100 * UNITS; // 100 UNITS deposit to create asset class
	pub const InstanceDeposit: Balance = 100 * UNITS; // 100 UNITS deposit to create asset instance
	pub const KeyLimit: u32 = 256;	// Max 256 bytes per key
	pub const ValueLimit: u32 = 1024;	// Max 1024 bytes per value
	pub const UniquesMetadataDepositBase: Balance = 100 * UNITS;
	pub const AttributeDepositBase: Balance = 10 * UNITS;
	pub const DepositPerByte: Balance = UNITS;
	pub const UniquesStringLimit: u32 = 60;
}

impl pallet_uniques::Config for Test {
	type Event = Event;
	type ClassId = primitives::ClassId;
	type InstanceId = primitives::InstanceId;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type ClassDeposit = ClassDeposit;
	type InstanceDeposit = InstanceDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		1u128
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
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = Nothing;
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
}

pub struct AssetPairAccountIdTest();

impl AssetPairAccountIdFor<AssetId, AccountId> for AssetPairAccountIdTest {
	fn from_assets(asset_a: AssetId, asset_b: AssetId, _: &str) -> AccountId {
		let mut a = asset_a as u128;
		let mut b = asset_b as u128;
		if a > b {
			std::mem::swap(&mut a, &mut b)
		}
		(a * 1000 + b) as AccountId
	}
}

impl pallet_asset_registry::Config for Test {
	type Event = Event;
	type RegistryOrigin = EnsureSigned<AccountId>;
	type AssetId = AssetId;
	type Balance = Balance;
	type AssetNativeLocation = u8;
	type StringLimit = RegistryStringLimit;
	type NativeAssetId = BSXAssetId;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinTradingLimit: Balance = MIN_TRADING_LIMIT;
	pub const MinPoolLiquidity: Balance = MIN_POOL_LIQUIDITY;
	pub const MaxInRatio: u128 = MAX_IN_RATIO;
	pub const MaxOutRatio: u128 = MAX_OUT_RATIO;
	pub const DiscountedFee: (u32, u32) = DISCOUNTED_FEE;
}

impl pallet_xyk::Config for Test {
	type Event = Event;
	type AssetRegistry = AssetRegistry;
	type AssetPairAccountId = AssetPairAccountIdTest;
	type Currency = Currency;
	type NativeAssetId = BSXAssetId;
	type WeightInfo = ();
	type GetExchangeFee = ExchangeFeeRate;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type CanCreatePool = pallet_xyk::AllowAllPools;
	type AMMHandler = ();
	type DiscountedFee = DiscountedFee;
	type NonDustableWhitelistHandler = Whitelist;
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, BSX, INITIAL_BALANCE),
				(BOB, BSX, INITIAL_BALANCE),
				(BOB, KSM, INITIAL_BALANCE),
				(CHARLIE, BSX, INITIAL_BALANCE),
				(DAVE, BSX, INITIAL_BALANCE),
			],
		}
	}
}

thread_local! {
	pub static NFTS: RefCell<HashMap<primitives::InstanceId, AccountId>> = RefCell::new(HashMap::default());
}

use frame_support::traits::tokens::nonfungibles::{Create, Inspect, Mutate};
pub struct NftHandlerStub;

impl<AccountId: From<u128>> Inspect<AccountId> for NftHandlerStub {
	type InstanceId = primitives::InstanceId;
	type ClassId = primitives::ClassId;

	fn owner(_class: &Self::ClassId, instance: &Self::InstanceId) -> Option<AccountId> {
		let mut owner: Option<AccountId> = None;

		NFTS.with(|v| {
			if let Some(o) = v.borrow().get(instance) {
				owner = Some((*o).into());
			}
		});
		owner
	}
}

impl<AccountId: From<u128>> Create<AccountId> for NftHandlerStub {
	fn create_class(_class: &Self::ClassId, _who: &AccountId, _admin: &AccountId) -> DispatchResult {
		Ok(())
	}
}

impl<AccountId: From<u128> + Into<u128> + Copy> Mutate<AccountId> for NftHandlerStub {
	fn mint_into(_class: &Self::ClassId, _instance: &Self::InstanceId, _who: &AccountId) -> DispatchResult {
		NFTS.with(|v| {
			let mut m = v.borrow_mut();
			m.insert(*_instance, (*_who).into());
		});
		Ok(())
	}

	fn burn_from(_class: &Self::ClassId, instance: &Self::InstanceId) -> DispatchResult {
		NFTS.with(|v| {
			let mut m = v.borrow_mut();
			m.remove(instance);
		});
		Ok(())
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		orml_tokens::GenesisConfig::<Test> {
			balances: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub struct Whitelist;

impl DustRemovalAccountWhitelist<AccountId> for Whitelist {
	type Error = DispatchError;

	fn add_account(_account: &AccountId) -> Result<(), Self::Error> {
		Ok(())
	}

	fn remove_account(_account: &AccountId) -> Result<(), Self::Error> {
		Ok(())
	}
}
