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
//
use crate as liq_mining;
use crate::Config;
use frame_support::{ 
    parameter_types, 
    PalletId,
    traits::{GenesisBuild, Nothing}
};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, AssetId, Balance};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, BlockNumberProvider},
};

type AccountId = u128;
pub type PoolId = crate::PoolId;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;
pub const TREASURY: AccountId = 5;
pub const ACC_1M: AccountId = 6;

pub const INITIAL_BALANCE: u128 = 1_000_000_000_000;

pub const BSX_ACA_POOL: PoolId = 1;
pub const BSX_KSM_POOL: PoolId = 2;
pub const BSX_DOT_POOL: PoolId = 3;

pub const BSX_ACA_SHARE_ID: AssetId = 100;
pub const BSX_KSM_SHARE_ID: AssetId = 101;
pub const BSX_DOT_SHARE_ID: AssetId = 102;


pub const BSX: AssetId = 1000;
pub const HDX: AssetId = 2000;
pub const ACA: AssetId = 3000;
pub const KSM: AssetId = 4000;

pub const BSX_ACA_LM_POOL: PoolId = 12_000;
pub const BSX_KSM_LM_POOL: PoolId = 12_001;
pub const BSX_DOT_LM_POOL: PoolId = 12_002;

pub const BSX_FARM: PoolId = 11_000;
pub const KSM_FARM: PoolId = 11_001;
pub const ACA_FARM: PoolId = 11_002;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
frame_support::construct_runtime!(
	pub enum Test where
	Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		LiquidityMining: liq_mining::{Pallet, Call, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
	pub static MockBlockNumberProvider: u64 = 0;
}

impl BlockNumberProvider for MockBlockNumberProvider {
	type BlockNumber = u64;

	fn current_block_number() -> Self::BlockNumber {
		Self::get()
	}
}
impl system::Config for Test {
	type BaseCallFilter = ();
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
}

parameter_types! {
	pub NativeCurrencyId: AssetId = 0;

    pub AccumulatePeriod: BlockNumber = 10;     // 10 blocks
	pub const MaxLocks: u32 = 1;
    pub const LMPalletId: PalletId = PalletId(*b"TEST_lm_");
    pub const MinPlannedYieldingPeriods: BlockNumber = 100;
    pub const MinTotalFarmRewards: Balance = 1_000_000;
}

impl Config for Test {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = AssetId;
	type MultiCurrency = Tokens;
    type CreateOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
    type PalletId = LMPalletId;
    type MinPlannedYieldingPeriods = MinPlannedYieldingPeriods;
    type MinTotalFarmRewards = MinTotalFarmRewards;
	type BlockNumberProvider = MockBlockNumberProvider;
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
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = Nothing;
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, BSX_ACA_SHARE_ID, INITIAL_BALANCE),
				(ALICE, BSX_DOT_SHARE_ID, INITIAL_BALANCE),
				(ALICE, BSX_KSM_SHARE_ID, INITIAL_BALANCE),
				(ALICE, BSX, INITIAL_BALANCE),
                (ACC_1M, BSX, 1_000_000),
				(BOB, BSX_ACA_SHARE_ID, INITIAL_BALANCE),
				(BOB, BSX_DOT_SHARE_ID, INITIAL_BALANCE),
				(BOB, BSX_KSM_SHARE_ID, INITIAL_BALANCE),
				(CHARLIE, BSX_ACA_SHARE_ID, INITIAL_BALANCE),
				(CHARLIE, BSX_DOT_SHARE_ID, INITIAL_BALANCE),
				(CHARLIE, BSX_KSM_SHARE_ID, INITIAL_BALANCE),
				(DAVE, BSX_ACA_SHARE_ID, INITIAL_BALANCE),
				(DAVE, BSX_DOT_SHARE_ID, INITIAL_BALANCE),
				(DAVE, BSX_KSM_SHARE_ID, INITIAL_BALANCE),
                (TREASURY, BSX, 1_000_000_000_000_000_000),
                (TREASURY, ACA, 1_000_000_000_000_000_000),
                (TREASURY, HDX, 1_000_000_000_000_000_000),
                (TREASURY, KSM, 1_000_000_000_000_000_000),
			],
		}
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

pub fn run_to_block(n: u64) {
	MockBlockNumberProvider::set(n);
	System::set_block_number(System::block_number() + 1);
}
