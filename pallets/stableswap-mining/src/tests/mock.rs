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

pub(crate) use crate as stableswap_mining;
use crate::Config;
pub(crate) use frame_support::{
	assert_ok,
	instances::Instance1,
	parameter_types,
	sp_runtime::traits::{One, Zero},
	traits::{Everything, GenesisBuild},
	PalletId,
};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSigned};
use orml_traits::parameter_type_with_key;
pub use sp_arithmetic::{FixedU128, Permill};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
	DispatchError, DispatchResult,
};
//use stableswap_mining::*;

pub use pallet_stableswap::{
	traits::ShareAccountIdFor,
	types::{PoolAssets, PoolId, PoolInfo},
};

use core::ops::RangeInclusive;
use std::cell::RefCell;
use std::collections::HashMap;

pub use primitives::{Amount, AssetId, Balance};

pub type AccountId = u128;
pub type BlockNumber = u64;
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;
pub const EVE: AccountId = 5;
pub const GC: AccountId = 6;

pub const ONE: Balance = 1_000_000_000_000;

pub const INITIAL_BALANCE: u128 = 1_000_000_000_000;

pub const BSX: AssetId = 1000;
pub const HDX: AssetId = 1001;
pub const ACA: AssetId = 1002;
pub const KSM: AssetId = 1003;
pub const DOT: AssetId = 1004;
pub const ETH: AssetId = 1005;
pub const TKN1: AssetId = 1_006;
pub const TKN2: AssetId = 1_007;
pub const DAI: AssetId = 1_008;

pub const GC_FARM: GlobalFarmId = 1;

pub const LIQ_MINING_NFT_CLASS: u128 = 1;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
	Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		WarehouseMining: warehouse_liquidity_mining::<Instance1>::{Pallet, Storage, Event<T>},
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
	pub const NFTClass: primitives::ClassId = LIQ_MINING_NFT_CLASS;
}

impl Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type CreateOrigin = EnsureRoot<AccountId>;
	type PalletId = StableMiningPalletId;
	type BlockNumberProvider = MockBlockNumberProvider;
	type NFTClassId = NFTClass;
	type NFTHandler = DummyNFT;
	type LiquidityMiningHandler = WarehouseMining;
	type WeightInfo = ();
}

parameter_types! {
	pub const LMPalletId: PalletId = PalletId(*b"TEST_lm_");
	pub const MinPlannedYieldingPeriods: BlockNumber = 100;
	pub const MinTotalFarmRewards: Balance = 1_000_000;
	pub const MaxEntriesPerDeposit: u8 = 5;
	pub const MaxYieldFarmsPerGlobalFarm: u8 = 4;
}

impl warehouse_liquidity_mining::Config<Instance1> for Test {
	type Event = Event;
	type CurrencyId = AssetId;
	type MultiCurrency = Tokens;
	type PalletId = LMPalletId;
	type MinPlannedYieldingPeriods = MinPlannedYieldingPeriods;
	type MinTotalFarmRewards = MinTotalFarmRewards;
	type BlockNumberProvider = MockBlockNumberProvider;
	type AmmPoolId = pallet_stableswap::types::PoolId<AssetId>;
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
	type OnKilledTokenAccount = ();
	type OnNewTokenAccount = ();
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

pub use hydradx_traits_stableswap::{Registry, ShareTokenRegistry};

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

pub fn set_block_number(n: u64) {
	MockBlockNumberProvider::set(n);
	System::set_block_number(n);
}

pub struct InitialLiquidity {
	pub(crate) account: AccountId,
	pub(crate) asset: AssetId,
	pub(crate) amount: Balance,
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
	registered_assets: Vec<(Vec<u8>, AssetId)>,
	created_pools: Vec<(AccountId, PoolInfo<AssetId>, InitialLiquidity)>,
	global_farms: Vec<(
		Balance,
		PeriodOf<Test>,
		BlockNumber,
		AssetId,
		AssetId,
		AccountId,
		Permill,
		Balance,
		FixedU128,
	)>,
	yield_farms: Vec<(
		AccountId,
		GlobalFarmId,
		FarmMultiplier,
		Option<LoyaltyCurve>,
		PoolId<AssetId>,
		AssetId,
		AssetId,
	)>,
	starting_block: u64,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		// If eg. tests running on one thread only, this thread local is shared.
		// let's make sure that it is empty for each  test case
		// or set to original default value
		REGISTERED_ASSETS.with(|v| {
			v.borrow_mut().clear();
		});
		ASSET_IDENTS.with(|v| {
			v.borrow_mut().clear();
		});
		POOL_IDS.with(|v| {
			v.borrow_mut().clear();
		});
		Self {
			endowed_accounts: vec![],
			registered_assets: vec![],
			created_pools: vec![],
			global_farms: vec![],
			yield_farms: vec![],
			starting_block: 1,
		}
	}
}

impl ExtBuilder {
	pub fn with_endowed_accounts(mut self, accounts: Vec<(AccountId, AssetId, Balance)>) -> Self {
		self.endowed_accounts = accounts;
		self
	}

	pub fn with_registered_asset(mut self, name: Vec<u8>, asset: AssetId) -> Self {
		self.registered_assets.push((name, asset));

		self
	}

	pub fn with_pool(mut self, who: AccountId, pool: PoolInfo<AssetId>, initial_liquidity: InitialLiquidity) -> Self {
		self.created_pools.push((who, pool, initial_liquidity));

		self
	}

	pub fn start_from_block(mut self, block_number: u64) -> Self {
		self.starting_block = block_number;

		self
	}

	pub fn with_global_farms(
		mut self,
		total_rewards: Balance,
		planned_yielding_periods: PeriodOf<Test>,
		blocks_per_period: BlockNumber,
		incentivized_asset: AssetId,
		reward_currency: AssetId,
		owner: AccountId,
		yield_per_period: Permill,
		min_deposit: Balance,
		price_adujustment: FixedU128,
	) -> Self {
		self.global_farms.push((
			total_rewards,
			planned_yielding_periods,
			blocks_per_period,
			incentivized_asset,
			reward_currency,
			owner,
			yield_per_period,
			min_deposit,
			price_adujustment,
		));

		self
	}

	pub fn with_yield_farms(
		mut self,
		who: AccountId,
		global_farm_id: GlobalFarmId,
		multiplier: FarmMultiplier,
		loyalty_curve: Option<LoyaltyCurve>,
		pool_id: PoolId<AssetId>,
		assets: (AssetId, AssetId),
	) -> Self {
		self.yield_farms.push((
			who,
			global_farm_id,
			multiplier,
			loyalty_curve,
			pool_id,
			assets.0,
			assets.1,
		));

		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let mut all_assets: Vec<(Vec<u8>, AssetId)> = vec![(b"DAI".to_vec(), DAI), (b"HDX".to_vec(), HDX)];
		all_assets.extend(self.registered_assets);

		for (name, asset) in all_assets.into_iter() {
			REGISTERED_ASSETS.with(|v| {
				v.borrow_mut().insert(asset, asset);
			});

			ASSET_IDENTS.with(|v| {
				v.borrow_mut().insert(name, asset);
			})
		}

		orml_tokens::GenesisConfig::<Test> {
			balances: self
				.endowed_accounts
				.iter()
				.flat_map(|(x, asset, amount)| vec![(*x, *asset, *amount)])
				.collect(),
		}
		.assimilate_storage(&mut t)
		.unwrap();
		let mut r: sp_io::TestExternalities = t.into();

		r.execute_with(|| {
			set_block_number(self.starting_block);

			for (who, pool, initial) in self.created_pools {
				let pool_id = PoolId(retrieve_current_asset_id());
				assert_ok!(Stableswap::create_pool(
					Origin::signed(who),
					(pool.assets.0, pool.assets.1),
					pool.amplification,
					pool.fee,
				));
				POOL_IDS.with(|v| {
					v.borrow_mut().push(pool_id);
				});

				if initial.amount > Balance::zero() {
					assert_ok!(Stableswap::add_liquidity(
						Origin::signed(initial.account),
						pool_id,
						initial.asset,
						initial.amount,
					));
				}
			}

			//Create global farms
			for (
				total_rewards,
				planned_yielding_periods,
				blocks_per_period,
				incentivized_asset,
				reward_currency,
				owner,
				yield_per_period,
				min_deposit,
				price_adujustment,
			) in self.global_farms
			{
				assert_ok!(WarehouseMining::create_global_farm(
					total_rewards,
					planned_yielding_periods,
					blocks_per_period,
					incentivized_asset,
					reward_currency,
					owner,
					yield_per_period,
					min_deposit,
					price_adujustment
				));
			}

			//Create yield farms
			for (who, global_farm_id, multiplier, loyalty_curve, amm_pool_id, asset_a, asset_b) in self.yield_farms {
				assert_ok!(WarehouseMining::create_yield_farm(
					who,
					global_farm_id,
					multiplier,
					loyalty_curve,
					amm_pool_id,
					asset_a,
					asset_b
				));
			}
		});

		r
	}
}

pub(crate) fn retrieve_current_asset_id() -> AssetId {
	REGISTERED_ASSETS.with(|v| v.borrow().len() as AssetId)
}

pub(crate) fn get_pool_id_at(idx: usize) -> PoolId<AssetId> {
	POOL_IDS.with(|v| v.borrow()[idx])
}
