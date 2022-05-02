#![cfg(test)]
use super::*;

use crate as lbp;
use crate::{AssetPairAccountIdFor, Config};
use frame_support::parameter_types;
use frame_support::traits::{Everything, GenesisBuild, LockIdentifier, Nothing};
use hydradx_traits::LockedBalance;
use orml_traits::parameter_type_with_key;
use primitives::constants::chain::{
	AssetId, Balance, CORE_ASSET_ID, MAX_IN_RATIO, MAX_OUT_RATIO, MIN_POOL_LIQUIDITY, MIN_TRADING_LIMIT,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use frame_system::EnsureSigned;
use std::collections::BTreeMap;

pub type Amount = i128;
pub type AccountId = u64;
pub type BlockNumber = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const INITIAL_BALANCE: Balance = 1_000_000_000_000_000u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;

pub const HDX: AssetId = CORE_ASSET_ID;
pub const KUSD: AssetId = 2_000;
pub const BSX: AssetId = 3_000;
pub const ETH: AssetId = 4_000;

pub const EXISTENTIAL_DEPOSIT: Balance = 100;
pub const SALE_START: Option<BlockNumber> = Some(10);
pub const SALE_END: Option<BlockNumber> = Some(40);

pub const HDX_BSX_POOL_ID: AccountId = 3_000;
pub const KUSD_BSX_POOL_ID: AccountId = 2_003_000;

pub const DEFAULT_FEE: (u32, u32) = (2, 1_000);

pub const SAMPLE_POOL_DATA: Pool<AccountId, BlockNumber> = Pool {
	owner: ALICE,
	start: SALE_START,
	end: SALE_END,
	assets: (KUSD, BSX),
	initial_weight: 10_000_000,
	final_weight: 90_000_000,
	weight_curve: WeightCurveType::Linear,
	fee: DEFAULT_FEE,
	fee_collector: CHARLIE,
	repay_target: 0,
};

pub const SAMPLE_AMM_TRANSFER: AMMTransfer<AccountId, AssetId, AssetPair, Balance> = AMMTransfer {
	origin: ALICE,
	assets: AssetPair {
		asset_in: KUSD,
		asset_out: BSX,
	},
	amount: 1000,
	amount_out: 10000,
	discount: false,
	discount_amount: 0_u128,
	fee: (KUSD, 200),
};

frame_support::construct_runtime!(
	pub enum Test where
	 Block = Block,
	 NodeBlock = Block,
	 UncheckedExtrinsic = UncheckedExtrinsic,
	 {
		 System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		 LBPPallet: lbp::{Pallet, Call, Storage, Event<T>},
		 Currency: orml_tokens::{Pallet, Event<T>},
		 XYKPallet: pallet_xyk::{Pallet, Call, Storage, Event<T>},
		 AssetRegistry: pallet_asset_registry::{Pallet, Storage, Event<T>},
	 }

);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		EXISTENTIAL_DEPOSIT
	};
}

parameter_types! {
	pub const MaxLocks: u32 = 1;
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

pub struct LbpAssetPairAccountIdTest;

impl AssetPairAccountIdFor<AssetId, u64> for LbpAssetPairAccountIdTest {
	fn from_assets(asset_a: AssetId, asset_b: AssetId, _: &str) -> u64 {
		let mut a = asset_a as u128;
		let mut b = asset_b as u128;
		if a > b {
			std::mem::swap(&mut a, &mut b);
		}
		(a * 1_000 + b) as u64
	}
}

parameter_types! {
	pub const NativeAssetId: AssetId = CORE_ASSET_ID;
	pub const MinTradingLimit: Balance = MIN_TRADING_LIMIT;
	pub const MinPoolLiquidity: Balance = MIN_POOL_LIQUIDITY;
	pub const MaxInRatio: u128 = MAX_IN_RATIO;
	pub const MaxOutRatio: u128 = MAX_OUT_RATIO;
}

pub struct MultiLockedBalance();

impl LockedBalance<AssetId, AccountId, Balance> for MultiLockedBalance {
	fn get_by_lock(lock_id: LockIdentifier, asset: AssetId, account: AccountId) -> Balance {
		if asset == NativeAssetId::get() {
			match Currency::locks(account, asset)
				.into_iter()
				.find(|lock| lock.id == lock_id)
			{
				Some(lock) => lock.amount,
				None => Zero::zero(),
			}
		} else {
			match Currency::locks(account, asset)
				.into_iter()
				.find(|lock| lock.id == lock_id)
			{
				Some(lock) => lock.amount,
				None => Zero::zero(),
			}
		}
	}
}

pub struct XykAssetPairAccountIdTest;

impl AssetPairAccountIdFor<AssetId, u64> for XykAssetPairAccountIdTest {
	fn from_assets(asset_a: AssetId, asset_b: AssetId, _: &str) -> u64 {
		let mut a = asset_a as u128;
		let mut b = asset_b as u128;
		if a > b {
			std::mem::swap(&mut a, &mut b);
		}
		(a * 10_000 + b) as u64
	}
}

impl pallet_xyk::Config for Test {
	type Event = Event;
	type AssetRegistry = AssetRegistry;
	type AssetPairAccountId = XykAssetPairAccountIdTest;
	type Currency = Currency;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = ();
	type GetExchangeFee = ExchangeFeeRate;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type CanCreatePool = pallet_xyk::AllowAllPools;
	type AMMHandler = ();
}

parameter_types! {
	pub ExchangeFeeRate: (u32, u32) = (2, 1_000);
	pub RegistryStringLimit: u32 = 100;
}

impl pallet_asset_registry::Config for Test {
	type Event = Event;
	type RegistryOrigin = EnsureSigned<AccountId>;
	type AssetId = AssetId;
	type Balance = Balance;
	type AssetNativeLocation = u8;
	type StringLimit = RegistryStringLimit;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = ();
}

impl Config for Test {
	type Event = Event;
	type MultiCurrency = Currency;
	type LockedBalance = MultiLockedBalance;
	type CreatePoolOrigin = frame_system::EnsureRoot<u64>;
	type LBPWeightFunction = lbp::LBPWeightFunction;
	type AssetPairAccountId = LbpAssetPairAccountIdTest;
	type WeightInfo = ();
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type BlockNumberProvider = System;
	type OnRemoveLiquidity = pallet_xyk::CreatePool<Self>;
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, HDX, INITIAL_BALANCE),
				(ALICE, BSX, INITIAL_BALANCE),
				(ALICE, KUSD, INITIAL_BALANCE),
				(ALICE, ETH, INITIAL_BALANCE),
				(BOB, HDX, INITIAL_BALANCE),
				(BOB, BSX, INITIAL_BALANCE),
				(BOB, KUSD, INITIAL_BALANCE),
				(BOB, ETH, INITIAL_BALANCE),
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

		pallet_asset_registry::GenesisConfig::<Test> {
			asset_names: vec![
				(b"KUSD".to_vec(), 1_000_000u128),
				(b"BSX".to_vec(), 1_000_000u128),
				(b"ETH".to_vec(), 1_000_000u128),],
			native_asset_name: b"HDX".to_vec(),
			native_existential_deposit: EXISTENTIAL_DEPOSIT,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}

pub fn set_block_number<T: frame_system::Config<BlockNumber = u64>>(n: u64) {
	frame_system::Pallet::<T>::set_block_number(n);
}

pub fn run_to_sale_start() {
	set_block_number::<Test>(SALE_START.unwrap());
}

pub fn run_to_sale_end() {
	set_block_number::<Test>(SALE_END.unwrap() + 1);
}

pub fn generate_trades(
	start: BlockNumber,
	end: BlockNumber,
	sale_rate: u128,
	sell_ratio: u128,
) -> BTreeMap<BlockNumber, (bool, u128)> {
	let mut trades = BTreeMap::new();
	let intervals: u64 = 72;

	let buy_amount = sale_rate / 24;
	let sell_amount = sale_rate / sell_ratio / 24;

	let skip = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
	let sells = vec![19, 20, 21, 33, 34, 35, 48, 49, 50, 62, 63, 64];
	for i in 0..=intervals {
		let block_num = start + (i * ((end - start) / intervals));

		if skip.contains(&i) {
			continue;
		}

		let (is_buy, amount) = if sells.contains(&i) {
			(false, sell_amount)
		} else {
			(true, buy_amount)
		};

		trades.insert(block_num, (is_buy, amount));
	}
	trades
}
