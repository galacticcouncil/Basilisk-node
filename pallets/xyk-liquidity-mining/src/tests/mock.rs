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
use super::*;

use crate as liq_mining;
use crate::Config;
use frame_support::weights::RuntimeDbWeight;
use frame_support::{
	parameter_types,
	traits::{Everything, GenesisBuild, Nothing},
	PalletId,
};

use frame_system as system;
use hydradx_traits::{nft::CreateTypedCollection, pools::DustRemovalAccountWhitelist, AMM};
use orml_traits::parameter_type_with_key;
use pallet_liquidity_mining::{FarmMultiplier, YieldFarmId};
use pallet_nft::CollectionType;
use primitives::{asset::AssetPair, Amount, AssetId, Balance};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
};
use sp_std::convert::TryFrom;
use std::{cell::RefCell, collections::HashMap};

pub type AccountId = u128;
pub type BlockNumber = u64;
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const ZERO_REWARDS_USER: AccountId = 4;

pub const ONE: Balance = 1_000_000_000_000;

pub const BSX_ACA_SHARE_ID: AssetId = 100;
pub const BSX_KSM_SHARE_ID: AssetId = 101;

pub const BSX: AssetId = 1000;
pub const ACA: AssetId = 3000;
pub const KSM: AssetId = 4000;
pub const DOT: AssetId = 5000;

pub const BSX_ACA_AMM: AccountId = 11_000;
pub const BSX_KSM_AMM: AccountId = 11_001;
pub const DEFAULT_AMM: AccountId = 11_007;

pub const BSX_FARM: YieldFarmId = 1;
pub const KSM_FARM: YieldFarmId = 2;

pub const INITIAL_READ_WEIGHT: u64 = 1;
pub const INITIAL_WRITE_WEIGHT: u64 = 1;

pub const LM_NFT_COLLECTION: primitives::CollectionId = 1;

pub const BSX_KSM_ASSET_PAIR: AssetPair = AssetPair {
	asset_in: BSX,
	asset_out: KSM,
};

pub const BSX_DOT_ASSET_PAIR: AssetPair = AssetPair {
	asset_in: BSX,
	asset_out: DOT,
};

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
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
	pub static MockBlockNumberProvider: u64 = 0;
	pub const DbWeight: RuntimeDbWeight = RuntimeDbWeight{
		read: INITIAL_READ_WEIGHT, write: INITIAL_WRITE_WEIGHT
	};
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
	type DbWeight = DbWeight;
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

thread_local! {
	pub static NFT_COLLECTION: RefCell<(u128, u128, u128)>= RefCell::new((0,0,0));

	pub static AMM_POOLS: RefCell<HashMap<AccountId, (AssetId, AssetPair)>> = RefCell::new(HashMap::new());
	pub static NFTS: RefCell<HashMap<pallet_liquidity_mining::DepositId, AccountId>> = RefCell::new(HashMap::default());
	pub static DEPOSIT_IDS: RefCell<Vec<DepositId>> = RefCell::new(Vec::new());

	pub static GLOBAL_FARMS: RefCell<HashMap<u32, DymmyGlobalFarm>> = RefCell::new(HashMap::default());
	pub static YIELD_FARMS: RefCell<HashMap<u32, DummyYieldFarm>> = RefCell::new(HashMap::default());
	pub static DEPOSITS: RefCell<HashMap<u128, DummyDeposit>> = RefCell::new(HashMap::default());
	pub static DEPOSIT_ENTRIES: RefCell<HashMap<(DepositId, u32), DummyFarmEntry>> = RefCell::new(HashMap::default());

	pub static FARM_ID: RefCell<u32> = RefCell::new(0);
	pub static DEPOSIT_ID: RefCell<DepositId> = RefCell::new(0);

	pub static DUSTER_WHITELIST: RefCell<Vec<AccountId>>= RefCell::new(Vec::new());
}
#[derive(Copy, Clone)]
pub struct DymmyGlobalFarm {
	total_rewards: Balance,
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
	multiplier: FarmMultiplier,
	amm_pool_id: AccountId,
	_assets: Vec<AssetId>,
	stopped: bool,
}

#[derive(Copy, Clone)]
pub struct DummyDeposit {
	amm_pool_id: AccountId,
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

pub struct DummyAMM;

impl AMM<AccountId, AssetId, AssetPair, Balance> for DummyAMM {
	fn get_max_out_ratio() -> u128 {
		0_u32.into()
	}

	fn get_fee(_pool_account_id: &AccountId) -> (u32, u32) {
		(0, 0)
	}

	fn get_max_in_ratio() -> u128 {
		0_u32.into()
	}

	fn get_pool_assets(pool_id: &AccountId) -> Option<Vec<AssetId>> {
		AMM_POOLS.with(|v| match v.borrow().get(pool_id) {
			Some((_, pair)) => Some(vec![pair.asset_in, pair.asset_out]),
			_ => None,
		})
	}

	fn get_spot_price_unchecked(_asset_a: AssetId, _asset_b: AssetId, _amount: Balance) -> Balance {
		Balance::from(0_u32)
	}

	fn validate_sell(
		_origin: &AccountId,
		_assets: AssetPair,
		_amount: Balance,
		_min_bought: Balance,
		_discount: bool,
	) -> Result<
		hydradx_traits::AMMTransfer<AccountId, AssetId, AssetPair, Balance>,
		frame_support::sp_runtime::DispatchError,
	> {
		Err(sp_runtime::DispatchError::Other("NotImplemented"))
	}

	fn execute_buy(
		_transfer: &hydradx_traits::AMMTransfer<AccountId, AssetId, AssetPair, Balance>,
	) -> frame_support::dispatch::DispatchResult {
		Err(sp_runtime::DispatchError::Other("NotImplemented"))
	}

	fn execute_sell(
		_transfer: &hydradx_traits::AMMTransfer<AccountId, AssetId, AssetPair, Balance>,
	) -> frame_support::dispatch::DispatchResult {
		Err(sp_runtime::DispatchError::Other("NotImplemented"))
	}

	fn validate_buy(
		_origin: &AccountId,
		_assets: AssetPair,
		_amount: Balance,
		_max_limit: Balance,
		_discount: bool,
	) -> Result<
		hydradx_traits::AMMTransfer<AccountId, AssetId, AssetPair, Balance>,
		frame_support::sp_runtime::DispatchError,
	> {
		Err(sp_runtime::DispatchError::Other("NotImplemented"))
	}

	fn get_min_pool_liquidity() -> Balance {
		Balance::from(0_u32)
	}

	fn get_min_trading_limit() -> Balance {
		Balance::from(0_u32)
	}

	// Fn bellow are used by liq. mining pallet
	fn exists(assets: AssetPair) -> bool {
		AMM_POOLS.with(|v| {
			let p = v.borrow();

			p.iter().any(|(_, v)| v.1 == assets)
		})
	}

	fn get_pair_id(assets: AssetPair) -> AccountId {
		AMM_POOLS.with(|v| {
			let p = v.borrow();

			match p.iter().find(|(_, v)| v.1 == assets) {
				Some((pair_id, _)) => *pair_id,
				None => DEFAULT_AMM,
			}
		})
	}

	fn get_share_token(assets: AssetPair) -> AssetId {
		AMM_POOLS.with(|v| {
			let p = v.borrow();

			match p.iter().find(|(_, v)| v.1 == assets) {
				Some((_, v)) => v.0,
				None => BSX,
			}
		})
	}
}

parameter_types! {
	pub const WarehouseLMPalletId: PalletId = PalletId(*b"WhouseLm");
	pub const MinDeposit: Balance = 1;
	pub const MaxLocks: u32 = 1;
	pub const LMPalletId: PalletId = PalletId(*b"TEST_lm_");
	#[derive(PartialEq, Eq)]
	pub const MaxEntriesPerDeposit: u8 = 10;
	pub const MaxYieldFarmsPerGlobalFarm: u8 = 5;
	pub const NftCollectionId: primitives::CollectionId = LM_NFT_COLLECTION;
	pub const ReserveClassIdUpTo: u128 = 2;
}

impl Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type CreateOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
	type PalletId = LMPalletId;
	type BlockNumberProvider = MockBlockNumberProvider;
	type AMM = DummyAMM;
	type NftCollectionId = NftCollectionId;
	type NFTHandler = DummyNFT;
	type LiquidityMiningHandler = DummyLiquidityMining;
	type NonDustableWhitelistHandler = Whitelist;
}

pub struct DummyNFT;

impl<AccountId: From<u128>> Inspect<AccountId> for DummyNFT {
	type ItemId = u128;
	type CollectionId = u128;

	fn owner(_collection: &Self::CollectionId, item: &Self::ItemId) -> Option<AccountId> {
		let mut owner: Option<AccountId> = None;

		NFTS.with(|v| {
			if let Some(o) = v.borrow().get(item) {
				owner = Some((*o).into());
			}
		});
		owner
	}
}

impl<AccountId: From<u128> + Into<u128> + Copy> Create<AccountId> for DummyNFT {
	fn create_collection(collection: &Self::CollectionId, who: &AccountId, admin: &AccountId) -> DispatchResult {
		NFT_COLLECTION.with(|v| {
			v.replace((*collection, (*who).into(), (*admin).into()));
		});
		Ok(())
	}
}

impl<AccountId: From<u128> + Into<u128> + Copy> Mutate<AccountId> for DummyNFT {
	fn mint_into(_collection: &Self::CollectionId, item: &Self::ItemId, who: &AccountId) -> DispatchResult {
		NFTS.with(|v| {
			let mut m = v.borrow_mut();
			m.insert(*item, (*who).into());
		});
		Ok(())
	}

	fn burn(
		_collection: &Self::CollectionId,
		item: &Self::ItemId,
		_maybe_check_ower: Option<&AccountId>,
	) -> DispatchResult {
		NFTS.with(|v| {
			let mut m = v.borrow_mut();
			m.remove(item);
		});
		Ok(())
	}
}

impl CreateTypedCollection<AccountId, primitives::CollectionId, CollectionType> for DummyNFT {
	fn create_typed_collection(
		owner: AccountId,
		collection_id: primitives::CollectionId,
		_collection_type: CollectionType,
	) -> DispatchResult {
		NFT_COLLECTION.with(|v| {
			v.replace((collection_id, owner, owner));
		});
		Ok(())
	}
}

pub struct DummyLiquidityMining {}

impl DummyLiquidityMining {}

impl hydradx_traits_lm::liquidity_mining::Mutate<AccountId, AssetId, BlockNumber> for DummyLiquidityMining {
	type Error = DispatchError;

	type AmmPoolId = AccountId;
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
					total_rewards,
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

			Ok((g_f.reward_currency, g_f.total_rewards, who))
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
		who: AccountId,
		deposit_id: u128,
		yield_farm_id: u32,
		fail_on_doubleclaim: bool,
	) -> Result<(u32, AssetId, Self::Balance, Self::Balance), Self::Error> {
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
			if yield_farm_entry.last_claimed == MockBlockNumberProvider::get() {
				claimed = 0;
				unclaimable = 200_000 * ONE;
			}

			if yield_farm_entry.last_claimed == MockBlockNumberProvider::get() {
				claimed = 0;
			}

			yield_farm_entry.last_claimed = MockBlockNumberProvider::get();

			if who == ZERO_REWARDS_USER {
				claimed = 0;
				unclaimable = 0;
			}

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
		DEPOSIT_ENTRIES.with(|v| v.borrow().get(&(deposit_id, yield_farm_id)).map(|d| d.global_farm_id))
	}
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
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
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
	type MaxReserves = ConstU32<100_000>;
	type ReserveIdentifier = ();
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,

	amm_pools: Vec<(AccountId, AssetId, AssetPair)>,

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
	yield_farms: Vec<(AccountId, GlobalFarmId, FarmMultiplier, Option<LoyaltyCurve>, AssetPair)>,
	deposits: Vec<(AccountId, GlobalFarmId, YieldFarmId, AssetPair, Balance)>,
	starting_block: u64,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		// If eg. tests running on one thread only, this thread local is shared.
		// let's make sure that it is empty for each  test case
		// or set to original default value
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
		NFTS.with(|v| {
			v.borrow_mut().clear();
		});
		AMM_POOLS.with(|v| {
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
			global_farms: vec![],
			yield_farms: vec![],
			deposits: vec![],
			amm_pools: vec![],
			starting_block: 1,
		}
	}
}

impl ExtBuilder {
	pub fn with_endowed_accounts(mut self, accounts: Vec<(AccountId, AssetId, Balance)>) -> Self {
		self.endowed_accounts = accounts;
		self
	}

	pub fn _start_from_block(mut self, block_number: u64) -> Self {
		self.starting_block = block_number;

		self
	}

	pub fn with_amm_pool(mut self, amm_id: AccountId, lp_token: AssetId, asset_pair: AssetPair) -> Self {
		self.amm_pools.push((amm_id, lp_token, asset_pair));

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
		assets: AssetPair,
	) -> Self {
		self.yield_farms
			.push((who, global_farm_id, multiplier, loyalty_curve, assets));

		self
	}

	pub fn with_deposit(
		mut self,
		owner: AccountId,
		global_farm_id: GlobalFarmId,
		yield_farm_id: YieldFarmId,
		assets: AssetPair,
		amount: Balance,
	) -> Self {
		self.deposits
			.push((owner, global_farm_id, yield_farm_id, assets, amount));

		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

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

			//Initialize amm pools
			for (amm_id, lp_token, asset_pair) in self.amm_pools {
				AMM_POOLS.with(|v| {
					v.borrow_mut().insert(amm_id, (lp_token, asset_pair));
				});
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
			for (who, global_farm_id, multiplier, loyalty_curve, asset_pair) in self.yield_farms {
				let amm_pool_id = DummyAMM::get_pair_id(asset_pair);

				assert!(amm_pool_id != DEFAULT_AMM, "get_pair_id() returned DEFAULT_AMM");

				let _ = DummyLiquidityMining::create_yield_farm(
					who,
					global_farm_id,
					multiplier,
					loyalty_curve,
					amm_pool_id,
					vec![asset_pair.asset_in, asset_pair.asset_out],
				);
			}

			//Create deposits
			let mut i: DepositId = 1;
			for (owner, global_farm_id, yield_farm_id, asset_pair, amount) in self.deposits {
				assert_ok!(LiquidityMining::deposit_shares(
					Origin::signed(owner),
					global_farm_id,
					yield_farm_id,
					asset_pair,
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

pub struct Whitelist;

impl DustRemovalAccountWhitelist<AccountId> for Whitelist {
	type Error = DispatchError;

	fn add_account(account: &AccountId) -> Result<(), Self::Error> {
		DEPOSIT_IDS.with(|v| {
			v.borrow_mut().push(*account);
		});

		Ok(())
	}

	fn remove_account(_account: &AccountId) -> Result<(), Self::Error> {
		Err(sp_runtime::DispatchError::Other("Not implemented"))
	}
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

pub fn set_block_number(n: u64) {
	MockBlockNumberProvider::set(n);
	System::set_block_number(n);
}