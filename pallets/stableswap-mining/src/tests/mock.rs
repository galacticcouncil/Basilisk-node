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
	assert_ok, parameter_types,
	sp_runtime::traits::{One, Zero},
	traits::{Everything, GenesisBuild},
	PalletId,
};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSigned};
use orml_traits::parameter_type_with_key;
pub use sp_arithmetic::{FixedU128, Perquintill};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
	DispatchError, DispatchResult,
};

pub use pallet_stableswap::{
	traits::ShareAccountIdFor,
	types::{AssetLiquidity, PoolInfo},
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
pub const GC: AccountId = 6;

pub const ONE: Balance = 1_000_000_000_000;

pub const BSX: AssetId = 1000;
pub const HDX: AssetId = 1001;
pub const KSM: AssetId = 1002;
pub const DAI: AssetId = 1_003;

pub const GC_FARM: GlobalFarmId = 1;
pub const BOB_FARM: GlobalFarmId = 2;

pub const LIQ_MINING_NFT_CLASS: u128 = 1;

pub const GLOBAL_FARM_UNDISTRIBUTED_REWARDS: Balance = 1_000_000 * ONE;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
	Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
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
	type LiquidityMiningHandler = DummyLiquidityMining;
	type WeightInfo = ();
}

parameter_types! {
	pub const LMPalletId: PalletId = PalletId(*b"TEST_lm_");
	pub const MinPlannedYieldingPeriods: BlockNumber = 100;
	pub const MinTotalFarmRewards: Balance = 1_000_000;
	pub const MaxEntriesPerDeposit: u8 = 5;
	pub const MaxYieldFarmsPerGlobalFarm: u8 = 4;
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
	pub static POOL_IDS: RefCell<Vec<AssetId>> = RefCell::new(Vec::new());
	pub static DEPOSIT_IDS: RefCell<Vec<DepositId>> = RefCell::new(Vec::new());

	pub static GLOBAL_FARMS: RefCell<HashMap<u32, DymmyGlobalFarm>> = RefCell::new(HashMap::default());
	pub static YIELD_FARMS: RefCell<HashMap<u32, DummyYieldFarm>> = RefCell::new(HashMap::default());
	pub static DEPOSITS: RefCell<HashMap<u128, DummyDeposit>> = RefCell::new(HashMap::default());
	pub static DEPOSIT_ENTRIES: RefCell<HashMap<(DepositId, u32), DummyFarmEntry>> = RefCell::new(HashMap::default());

	pub static FARM_ID: RefCell<u32> = RefCell::new(0);
	pub static DEPOSIT_ID: RefCell<DepositId> = RefCell::new(0);
}

#[derive(Copy, Clone)]
pub struct DymmyGlobalFarm {
	_total_rewards: Balance,
	_planned_yielding_periods: PeriodOf<Test>,
	_blocks_per_period: BlockNumber,
	incentivized_asset: AssetId,
	reward_currency: AssetId,
	_owner: AccountId,
	_yield_per_period: Perquintill,
	_min_deposit: Balance,
	price_adjustment: FixedU128,
	_max_reward_per_period: Balance,
}

#[derive(Clone, Debug)]
pub struct DummyYieldFarm {
	_global_farm_id: u32,
	multiplier: FixedU128,
	amm_pool_id: AssetId,
	_assets: Vec<AssetId>,
	stopped: bool,
}

#[derive(Copy, Clone)]
pub struct DummyDeposit {
	amm_pool_id: AssetId,
	shares_amount: Balance,
	entries: u32,
}

#[derive(Copy, Clone)]
pub struct DummyFarmEntry {
	_yield_farm_id: u32,
	global_farm_id: u32,
	_incentivized_asset_balance: Balance,
	last_claimed: BlockNumber,
}

pub struct DummyRegistry<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Registry<T::AssetId, Vec<u8>, Balance, DispatchError> for DummyRegistry<T>
where
	T::AssetId: Into<AssetId> + From<u32>,
{
	fn exists(asset_id: T::AssetId) -> bool {
		let asset = REGISTERED_ASSETS.with(|v| v.borrow().get(&asset_id).copied());
		matches!(asset, Some(_))
	}

	fn retrieve_asset(name: &Vec<u8>) -> Result<T::AssetId, DispatchError> {
		let asset_id = ASSET_IDENTS.with(|v| v.borrow().get(name).copied());
		if let Some(id) = asset_id {
			Ok(id)
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

		Ok(assigned)
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

impl ShareAccountIdFor<Vec<u32>> for AccountIdConstructor {
	type AccountId = AccountId;

	fn from_assets(assets: &Vec<u32>, _identifier: Option<&[u8]>) -> Self::AccountId {
		let sum: u32 = assets.iter().sum();

		(sum * 1000) as u128
	}

	fn name(assets: &Vec<u32>, identifier: Option<&[u8]>) -> Vec<u8> {
		let mut buf: Vec<u8> = if let Some(ident) = identifier {
			ident.to_vec()
		} else {
			vec![]
		};
		for asset in assets.iter() {
			buf.extend_from_slice(&(asset).to_le_bytes());
		}
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

pub struct DummyLiquidityMining {}

impl DummyLiquidityMining {}

impl hydradx_traits::liquidity_mining::Mutate<AccountId, AssetId, BlockNumber> for DummyLiquidityMining {
	type Error = DispatchError;

	type AmmPoolId = AssetId;
	type Balance = Balance;
	type Period = PeriodOf<Test>;
	type LoyaltyCurve = LoyaltyCurve;

	fn create_global_farm(
		total_rewards: Self::Balance,
		planned_yielding_periods: Self::Period,
		blocks_per_period: BlockNumber,
		incentivized_asset: AssetId,
		reward_currency: AssetId,
		owner: AccountId,
		yield_per_period: Perquintill,
		min_deposit: Self::Balance,
		price_adjustment: FixedU128,
	) -> Result<(u32, Self::Balance), Self::Error> {
		let max_reward_per_period = total_rewards.checked_div(planned_yielding_periods.into()).unwrap();
		let farm_id = get_next_farm_id();

		GLOBAL_FARMS.with(|v| {
			v.borrow_mut().insert(
				farm_id,
				DymmyGlobalFarm {
					_total_rewards: total_rewards,
					_planned_yielding_periods: planned_yielding_periods,
					_blocks_per_period: blocks_per_period,
					incentivized_asset,
					reward_currency,
					_owner: owner,
					_yield_per_period: yield_per_period,
					_min_deposit: min_deposit,
					price_adjustment,
					_max_reward_per_period: max_reward_per_period,
				},
			);
		});

		Ok((farm_id, max_reward_per_period))
	}

	fn update_global_farm_price_adjustment(
		_who: AccountId,
		global_farm_id: u32,
		price_adjustment: FixedU128,
	) -> Result<(), Self::Error> {
		GLOBAL_FARMS.with(|v| {
			let mut p = v.borrow_mut();

			let global_farm = p.get_mut(&global_farm_id).unwrap();

			global_farm.price_adjustment = price_adjustment;

			Ok(())
		})
	}

	fn destroy_global_farm(
		who: AccountId,
		global_farm_id: u32,
	) -> Result<(AssetId, Self::Balance, AccountId), Self::Error> {
		GLOBAL_FARMS.with(|v| {
			let g_f = v.borrow_mut().remove_entry(&global_farm_id).unwrap().1;

			Ok((g_f.reward_currency, GLOBAL_FARM_UNDISTRIBUTED_REWARDS, who))
		})
	}

	fn create_yield_farm(
		_who: AccountId,
		global_farm_id: u32,
		multiplier: FixedU128,
		_loyalty_curve: Option<Self::LoyaltyCurve>,
		amm_pool_id: Self::AmmPoolId,
		assets: Vec<AssetId>,
	) -> Result<u32, Self::Error> {
		let farm_id = get_next_farm_id();

		YIELD_FARMS.with(|v| {
			v.borrow_mut().insert(
				farm_id,
				DummyYieldFarm {
					_global_farm_id: global_farm_id,
					multiplier,
					amm_pool_id,
					_assets: assets,
					stopped: false,
				},
			);
		});

		Ok(farm_id)
	}

	fn update_yield_farm_multiplier(
		_who: AccountId,
		_global_farm_id: u32,
		amm_pool_id: Self::AmmPoolId,
		multiplier: FixedU128,
	) -> Result<u32, Self::Error> {
		YIELD_FARMS.with(|v| {
			let mut p = v.borrow_mut();

			let (id, yield_farm) = p.iter_mut().find(|(_, farm)| farm.amm_pool_id == amm_pool_id).unwrap();

			yield_farm.multiplier = multiplier;

			Ok(*id)
		})
	}

	fn stop_yield_farm(
		_who: AccountId,
		_global_farm_id: u32,
		amm_pool_id: Self::AmmPoolId,
	) -> Result<u32, Self::Error> {
		YIELD_FARMS.with(|v| {
			let mut p = v.borrow_mut();

			let (id, yield_farm) = p.iter_mut().find(|(_, farm)| farm.amm_pool_id == amm_pool_id).unwrap();

			yield_farm.stopped = true;

			Ok(*id)
		})
	}

	fn resume_yield_farm(
		_who: AccountId,
		_global_farm_id: u32,
		yield_farm_id: u32,
		_amm_pool_id: Self::AmmPoolId,
		multiplier: FixedU128,
	) -> Result<(), Self::Error> {
		YIELD_FARMS.with(|v| {
			let mut p = v.borrow_mut();

			let yield_farm = p.get_mut(&yield_farm_id).unwrap();

			yield_farm.stopped = true;
			yield_farm.multiplier = multiplier;

			Ok(())
		})
	}

	fn destroy_yield_farm(
		_who: AccountId,
		_global_farm_id: u32,
		yield_farm_id: u32,
		_amm_pool_id: Self::AmmPoolId,
	) -> Result<(), Self::Error> {
		YIELD_FARMS.with(|v| {
			let _ = v.borrow_mut().remove_entry(&yield_farm_id).unwrap().1;
		});

		Ok(())
	}

	fn deposit_lp_shares(
		global_farm_id: u32,
		yield_farm_id: u32,
		amm_pool_id: Self::AmmPoolId,
		shares_amount: Self::Balance,
		get_balance_in_amm: fn(AssetId, Self::AmmPoolId) -> Result<Self::Balance, Self::Error>,
	) -> Result<u128, Self::Error> {
		let deposit_id = get_next_deposit_id();

		let incentivized_asset = GLOBAL_FARMS.with(|v| v.borrow().get(&global_farm_id).unwrap().incentivized_asset);

		let incentivized_asset_balance = get_balance_in_amm(incentivized_asset, amm_pool_id).unwrap();

		DEPOSITS.with(|v| {
			v.borrow_mut().insert(
				deposit_id,
				DummyDeposit {
					amm_pool_id,
					shares_amount,
					entries: 1,
				},
			);
		});

		DEPOSIT_ENTRIES.with(|v| {
			v.borrow_mut().insert(
				(deposit_id, yield_farm_id),
				DummyFarmEntry {
					global_farm_id,
					_yield_farm_id: yield_farm_id,
					_incentivized_asset_balance: incentivized_asset_balance,
					last_claimed: MockBlockNumberProvider::get(),
				},
			);
		});

		Ok(deposit_id)
	}

	fn redeposit_lp_shares(
		global_farm_id: u32,
		yield_farm_id: u32,
		deposit_id: u128,
		get_balance_in_amm: fn(AssetId, Self::AmmPoolId) -> Result<Self::Balance, Self::Error>,
	) -> Result<Self::Balance, Self::Error> {
		let deposit = DEPOSITS.with(|v| {
			let mut p = v.borrow_mut();
			let mut deposit = p.get_mut(&deposit_id).unwrap();

			deposit.entries += 1;

			*deposit
		});

		let incentivized_asset = GLOBAL_FARMS.with(|v| v.borrow().get(&global_farm_id).unwrap().incentivized_asset);
		let amm_pool_id = deposit.amm_pool_id;

		let incentivized_asset_balance = get_balance_in_amm(incentivized_asset, amm_pool_id).unwrap();

		DEPOSIT_ENTRIES.with(|v| {
			v.borrow_mut().insert(
				(deposit_id, yield_farm_id),
				DummyFarmEntry {
					_yield_farm_id: yield_farm_id,
					global_farm_id,
					_incentivized_asset_balance: incentivized_asset_balance,
					last_claimed: MockBlockNumberProvider::get(),
				},
			)
		});

		Ok(deposit.shares_amount)
	}

	fn claim_rewards(
		_who: AccountId,
		deposit_id: u128,
		yield_farm_id: u32,
		fail_on_doubleclaim: bool,
	) -> Result<(u32, AssetId, Self::Balance, Self::Balance), Self::Error> {
		let deposit = DEPOSITS.with(|v| *v.borrow().get(&deposit_id).unwrap());

		DEPOSIT_ENTRIES.with(|v| {
			let mut p = v.borrow_mut();
			let yield_farm_entry = p.get_mut(&(deposit_id, yield_farm_id)).unwrap();

			if yield_farm_entry.last_claimed == MockBlockNumberProvider::get() && fail_on_doubleclaim {
				return Err("Dummy Double Claim".into());
			}

			let reward_currency = GLOBAL_FARMS.with(|v| {
				v.borrow()
					.get(&yield_farm_entry.global_farm_id)
					.unwrap()
					.reward_currency
			});

			let mut claimed = 20_000_000 * ONE;
			let mut unclaimable = 10_000 * ONE;
			if deposit.shares_amount.is_zero() {
				claimed = 0;
				unclaimable = 200_000 * ONE;
			}

			if yield_farm_entry.last_claimed == MockBlockNumberProvider::get() {
				claimed = 0;
			}

			yield_farm_entry.last_claimed = MockBlockNumberProvider::get();

			Ok((yield_farm_entry.global_farm_id, reward_currency, claimed, unclaimable))
		})
	}

	fn withdraw_lp_shares(
		deposit_id: u128,
		yield_farm_id: u32,
		_unclaimable_rewards: Self::Balance,
	) -> Result<(u32, Self::Balance, bool), Self::Error> {
		let deposit = DEPOSITS.with(|v| {
			let mut p = v.borrow_mut();
			let mut deposit = p.get_mut(&deposit_id).unwrap();

			deposit.entries -= 1;

			*deposit
		});

		let global_farm_id = DEPOSIT_ENTRIES
			.with(|v| v.borrow_mut().remove(&(deposit_id, yield_farm_id)))
			.unwrap()
			.global_farm_id;
		let withdrawn_amount = deposit.shares_amount;

		let mut destroyed = false;
		if deposit.entries.is_zero() {
			DEPOSITS.with(|v| v.borrow_mut().remove(&deposit_id));
			destroyed = true;
		}

		Ok((global_farm_id, withdrawn_amount, destroyed))
	}

	fn is_yield_farm_claimable(_global_farm_id: u32, yield_farm_id: u32, _amm_pool_id: Self::AmmPoolId) -> bool {
		!YIELD_FARMS.with(|v| v.borrow().get(&yield_farm_id).unwrap().stopped)
	}

	fn get_global_farm_id(deposit_id: u128, yield_farm_id: u32) -> Option<u32> {
		DEPOSIT_ENTRIES.with(|v| Some(v.borrow().get(&(deposit_id, yield_farm_id)).unwrap().global_farm_id))
	}
}

pub fn set_block_number(n: u64) {
	MockBlockNumberProvider::set(n);
	System::set_block_number(n);
}

pub struct InitialLiquidity {
	pub(crate) account: AccountId,
	pub(crate) assets: Vec<AssetLiquidity<AssetId>>,
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
	registered_assets: Vec<(Vec<u8>, AssetId)>,
	created_pools: Vec<(AccountId, PoolInfo<AssetId>, InitialLiquidity)>,
	#[allow(clippy::too_many_arguments, clippy::type_complexity)]
	global_farms: Vec<(
		Balance,
		PeriodOf<Test>,
		BlockNumber,
		AssetId,
		AssetId,
		AccountId,
		Perquintill,
		Balance,
		FixedU128,
	)>,
	#[allow(clippy::too_many_arguments, clippy::type_complexity)]
	yield_farms: Vec<(
		AccountId,
		GlobalFarmId,
		FarmMultiplier,
		Option<LoyaltyCurve>,
		AssetId,
		Vec<AssetId>,
	)>,
	deposits: Vec<(AccountId, GlobalFarmId, YieldFarmId, AssetId, Balance)>,
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
		GLOBAL_FARMS.with(|v| {
			v.borrow_mut().clear();
		});
		YIELD_FARMS.with(|v| {
			v.borrow_mut().clear();
		});
		DEPOSITS.with(|v| {
			v.borrow_mut().clear();
		});
		DEPOSIT_ENTRIES.with(|v| {
			v.borrow_mut().clear();
		});

		FARM_ID.with(|v| {
			*v.borrow_mut() = 0;
		});
		DEPOSIT_ID.with(|v| {
			*v.borrow_mut() = 0;
		});

		Self {
			endowed_accounts: vec![],
			registered_assets: vec![],
			created_pools: vec![],
			global_farms: vec![],
			yield_farms: vec![],
			deposits: vec![],
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

	pub fn _start_from_block(mut self, block_number: u64) -> Self {
		self.starting_block = block_number;

		self
	}

	#[allow(clippy::too_many_arguments)]
	pub fn with_global_farm(
		mut self,
		total_rewards: Balance,
		planned_yielding_periods: PeriodOf<Test>,
		blocks_per_period: BlockNumber,
		incentivized_asset: AssetId,
		reward_currency: AssetId,
		owner: AccountId,
		yield_per_period: Perquintill,
		min_deposit: Balance,
		price_adjustment: FixedU128,
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
			price_adjustment,
		));

		self
	}

	pub fn with_yield_farm(
		mut self,
		who: AccountId,
		global_farm_id: GlobalFarmId,
		multiplier: FarmMultiplier,
		loyalty_curve: Option<LoyaltyCurve>,
		pool_id: AssetId,
		assets: Vec<AssetId>,
	) -> Self {
		self.yield_farms
			.push((who, global_farm_id, multiplier, loyalty_curve, pool_id, assets));

		self
	}

	pub fn with_deposit(
		mut self,
		owner: AccountId,
		global_farm_id: GlobalFarmId,
		yield_farm_id: YieldFarmId,
		pool_id: AssetId,
		amount: Balance,
	) -> Self {
		self.deposits
			.push((owner, global_farm_id, yield_farm_id, pool_id, amount));

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

			for (who, pool, initial_liquid) in self.created_pools {
				let pool_id = retrieve_current_asset_id();
				assert_ok!(Stableswap::create_pool(
					Origin::signed(who),
					pool.assets.clone().into(),
					pool.amplification,
					pool.trade_fee,
					pool.withdraw_fee,
				));
				POOL_IDS.with(|v| {
					v.borrow_mut().push(pool_id);
				});

				if initial_liquid.assets.len() as u128 > Balance::zero() {
					assert_ok!(Stableswap::add_liquidity(
						Origin::signed(initial_liquid.account),
						pool_id,
						initial_liquid.assets,
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
				price_adjustment,
			) in self.global_farms
			{
				let _ = DummyLiquidityMining::create_global_farm(
					total_rewards,
					planned_yielding_periods,
					blocks_per_period,
					incentivized_asset,
					reward_currency,
					owner,
					yield_per_period,
					min_deposit,
					price_adjustment,
				);
			}

			//Create yield farms
			for (who, global_farm_id, multiplier, loyalty_curve, amm_pool_id, assets) in self.yield_farms {
				let _ = DummyLiquidityMining::create_yield_farm(
					who,
					global_farm_id,
					multiplier,
					loyalty_curve,
					amm_pool_id,
					assets,
				);
			}

			//Create deposits
			let mut i: DepositId = 1;
			for (owner, global_farm_id, yield_farm_id, pool_id, amount) in self.deposits {
				assert_ok!(StableswapMining::deposit_lp_shares(
					Origin::signed(owner),
					global_farm_id,
					yield_farm_id,
					pool_id,
					amount
				));

				DEPOSIT_IDS.with(|v| {
					v.borrow_mut().push(i);
				});
				i += 1;
			}
		});

		r
	}
}

pub(crate) fn retrieve_current_asset_id() -> AssetId {
	REGISTERED_ASSETS.with(|v| v.borrow().len() as AssetId)
}

pub(crate) fn get_pool_id_at(idx: usize) -> AssetId {
	POOL_IDS.with(|v| v.borrow()[idx])
}

pub(crate) fn get_deposit_id_at(idx: usize) -> DepositId {
	DEPOSIT_IDS.with(|v| v.borrow()[idx])
}

fn get_next_farm_id() -> u32 {
	FARM_ID.with(|v| {
		*v.borrow_mut() += 1;

		*v.borrow()
	})
}

fn get_next_deposit_id() -> DepositId {
	DEPOSIT_ID.with(|v| {
		*v.borrow_mut() += 1;

		*v.borrow()
	})
}
