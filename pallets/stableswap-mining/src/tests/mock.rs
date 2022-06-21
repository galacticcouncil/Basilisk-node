// This file is part of galacticcouncil/warehouse.

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg(test)]
use super::*;

use crate as stableswap_mining;
use crate::Config;
use frame_support::{
	parameter_types,
	traits::{Everything, GenesisBuild},
	PalletId,
    instances::Instance1,
};
use frame_system as system;
use frame_system::{EnsureSigned, EnsureRoot};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
    DispatchError, DispatchResult
};

use pallet_stableswap::{types::{PoolId, PoolAssets}, traits::ShareAccountIdFor};

use std::cell::RefCell;
use std::collections::HashMap;
use core::ops::RangeInclusive;

pub type Balance = u128;
pub type AssetId = u32;
pub type Amount = i128;

pub type AccountId = u128;
pub type FarmId = warehouse_liquidity_mining::FarmId;
pub type BlockNumber = u64;
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;
pub const EVE: AccountId = 5;

pub const INITIAL_BALANCE: u128 = 1_000_000_000_000;

pub const BSX: AssetId = 1000;
pub const HDX: AssetId = 1001;
pub const ACA: AssetId = 1002;
pub const KSM: AssetId = 1003;
pub const DOT: AssetId = 1004;
pub const ETH: AssetId = 1005;
pub const TKN1: AssetId = 1_006;
pub const TKN2: AssetId = 1_007;

pub const LIQ_MINING_NFT_CLASS: u128 = 1;

#[derive(Eq, PartialEq, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum ReserveIdentifier {
    Nft,
    Marketplace,
    // always the last, indicate number of variants
    Count,
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
        WarehouseMining: warehouse_liquidity_mining::<Instance1>::{Pallet, Storage},
		Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>},
        Stableswap: pallet_stableswap::{Pallet, Call, Storage, Event<T>},
        StableswapMining: stableswap_mining::{Pallet, Call, Event<T>}
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const StableMiningPalletId: PalletId = PalletId(*b"STSP##LM");
    pub const NFTClass: u128 = LIQ_MINING_NFT_CLASS;
}

impl Config for Test {
    type Event = Event;
    type MultiCurrency = Tokens;
    type CreateOrigin = EnsureRoot<AccountId>;
    type PalletId = StableMiningPalletId;
	type BlockNumberProvider = MockBlockNumberProvider;
    type NFTClassId = NFTClass;
    type NFTHandler = DummyNFT; 
    //type LiquidityMiningInstance =;
    type WeightInfo = ();
}

parameter_types! {
	pub const LMPalletId: PalletId = PalletId(*b"TEST_lm_");
	pub const MinPlannedYieldingPeriods: BlockNumber = 100;
	pub const MinTotalFarmRewards: Balance = 1_000_000;
	#[derive(PartialEq)]
	pub const MaxEntriesPerDeposit: u8 = 5;
	pub const MaxYieldFarmsPerGlobalFarm: u8 = 4;
}

impl warehouse_liquidity_mining::Config<Instance1> for Test {
	type CurrencyId = AssetId;
	type MultiCurrency = Tokens;
	type PalletId = LMPalletId;
	type MinPlannedYieldingPeriods = MinPlannedYieldingPeriods;
	type MinTotalFarmRewards = MinTotalFarmRewards;
	type BlockNumberProvider = MockBlockNumberProvider;
	type AmmPoolId = AccountId;
	type LiquidityMiningHandler = StableswapMining;
	type MaxFarmEntriesPerDeposit = MaxEntriesPerDeposit;
	type MaxYieldFarmsPerGlobalFarm = MaxYieldFarmsPerGlobalFarm;
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
	type MaxLocks = ();
	type DustRemovalWhitelist = Everything;
}

parameter_types! {
	pub const Precision: Balance = 1;
	pub const MinimumLiquidity: Balance = 1000;
	pub const MinimumTradingLimit: Balance = 1000;
	pub const AmplificationRange: RangeInclusive<u16> = RangeInclusive::new(2, 10_000);
}

impl pallet_stableswap::Config for Test {
	type Event = Event;
	type AssetId = AssetId;
	type Currency = Tokens;
	type ShareAccountId = AccountIdConstructor;
	type AssetRegistry = DummyRegistry<Test>;
	type CreatePoolOrigin = EnsureSigned<AccountId>;
	type Precision = Precision;
	type MinPoolLiquidity = MinimumLiquidity;
	type AmplificationRange = AmplificationRange;
	type MinTradingLimit = MinimumTradingLimit;
	type WeightInfo = ();
}

use hydradx_traits::{Registry, ShareTokenRegistry};

thread_local! {
    pub static NFTS: RefCell<HashMap<warehouse_liquidity_mining::DepositId, AccountId>> = RefCell::new(HashMap::default());
	pub static REGISTERED_ASSETS: RefCell<HashMap<AssetId, u32>> = RefCell::new(HashMap::default());
	pub static ASSET_IDENTS: RefCell<HashMap<Vec<u8>, u32>> = RefCell::new(HashMap::default());
	pub static POOL_IDS: RefCell<Vec<PoolId<AssetId>>> = RefCell::new(Vec::new());
}

pub struct DummyRegistry<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Registry<T::AssetId, Vec<u8>, Balance, DispatchError> for DummyRegistry<T>
where
	T::AssetId: Into<AssetId> + From<u32>,
{
	fn exists(asset_id: T::AssetId) -> bool {
		let asset = REGISTERED_ASSETS.with(|v| v.borrow().get(&(asset_id.into())).copied());
		matches!(asset, Some(_))
	}

	fn retrieve_asset(name: &Vec<u8>) -> Result<T::AssetId, DispatchError> {
		let asset_id = ASSET_IDENTS.with(|v| v.borrow().get(name).copied());
		if let Some(id) = asset_id {
			Ok(id.into())
		} else {
			Err(pallet_stableswap::Error::<Test>::AssetNotRegistered.into())
		}
	}

	fn create_asset(name: &Vec<u8>, _existential_deposit: Balance) -> Result<T::AssetId, DispatchError> {
		let assigned = REGISTERED_ASSETS.with(|v| {
			let l = v.borrow().len();
			v.borrow_mut().insert(l as u32, l as u32);
			l as u32
		});

		ASSET_IDENTS.with(|v| v.borrow_mut().insert(name.clone(), assigned));

		Ok(T::AssetId::from(assigned))
	}
}


impl<T: Config> ShareTokenRegistry<T::AssetId, Vec<u8>, Balance, DispatchError> for DummyRegistry<T>
where
	T::AssetId: Into<AssetId> + From<u32>,
{
	fn retrieve_shared_asset(name: &Vec<u8>, _assets: &[T::AssetId]) -> Result<T::AssetId, DispatchError> {
		Self::retrieve_asset(name)
	}

	fn create_shared_asset(
		name: &Vec<u8>,
		_assets: &[T::AssetId],
		existential_deposit: Balance,
	) -> Result<T::AssetId, DispatchError> {
		Self::get_or_create_asset(name.clone(), existential_deposit)
	}
}
pub struct AccountIdConstructor;

impl ShareAccountIdFor<PoolAssets<u32>> for AccountIdConstructor {
	type AccountId = AccountId;

	fn from_assets(assets: &PoolAssets<u32>, _identifier: Option<&[u8]>) -> Self::AccountId {
		let mut a = assets.0;
		let mut b = assets.1;
		if a > b {
			std::mem::swap(&mut a, &mut b)
		}
		(a * 1000 + b) as u128
	}

	fn name(assets: &PoolAssets<u32>, identifier: Option<&[u8]>) -> Vec<u8> {
		let mut buf: Vec<u8> = if let Some(ident) = identifier {
			ident.to_vec()
		} else {
			vec![]
		};
		buf.extend_from_slice(&(assets.0).to_le_bytes());
		buf.extend_from_slice(&(assets.1).to_le_bytes());

		buf
	}
}

use frame_support::traits::tokens::nonfungibles::{Create, Inspect, Mutate};
pub struct DummyNFT;

impl<AccountId: From<u128>> Inspect<AccountId> for DummyNFT {
	type InstanceId = warehouse_liquidity_mining::DepositId;
	type ClassId = NFTClass;

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

impl<AccountId: From<u128>> Create<AccountId> for DummyNFT {
	fn create_class(_class: &Self::ClassId, _who: &AccountId, _admin: &AccountId) -> DispatchResult {
		Ok(())
	}
}

impl<AccountId: From<u128> + Into<u128> + Copy> Mutate<AccountId> for DummyNFT {
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

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, BSX, INITIAL_BALANCE),
				(ALICE, ACA, INITIAL_BALANCE),
				(ALICE, HDX, INITIAL_BALANCE),
				(ALICE, KSM, INITIAL_BALANCE),
				(ALICE, DOT, INITIAL_BALANCE),
				
                (BOB, BSX, INITIAL_BALANCE),
				(BOB, ACA, INITIAL_BALANCE),
				(BOB, HDX, INITIAL_BALANCE),
				(BOB, KSM, INITIAL_BALANCE),
				(BOB, DOT, INITIAL_BALANCE),
                
                (CHARLIE, BSX, INITIAL_BALANCE),
				(CHARLIE, ACA, INITIAL_BALANCE),
				(CHARLIE, HDX, INITIAL_BALANCE),
				(CHARLIE, KSM, INITIAL_BALANCE),
				(CHARLIE, DOT, INITIAL_BALANCE),

                (DAVE, BSX, INITIAL_BALANCE),
				(DAVE, ACA, INITIAL_BALANCE),
				(DAVE, HDX, INITIAL_BALANCE),
				(DAVE, KSM, INITIAL_BALANCE),
				(DAVE, DOT, INITIAL_BALANCE),
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

pub fn set_block_number(n: u64) {
	MockBlockNumberProvider::set(n);
	System::set_block_number(n);
}
