use crate as duster;

use frame_support::parameter_types;
use frame_support::traits::{GenesisBuild, OnKilledAccount};

use orml_currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;
use primitives::{AssetId, Balance};

use crate::Config;
use frame_system as system;

use sp_core::H256;

use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use frame_support::weights::Weight;
use primitives::Amount;
use sp_runtime::traits::Zero;
use sp_std::cell::RefCell;
use sp_std::vec::Vec;

type AccountId = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

lazy_static::lazy_static! {
pub static ref ALICE: AccountId = 100;
pub static ref BOB: AccountId = 200;
pub static ref DUSTER: AccountId = 300;
pub static ref TREASURY: AccountId = 400;
}

parameter_types! {
	pub TreasuryAccount: AccountId = *TREASURY;
}

frame_support::construct_runtime!(
	pub enum Test where
	Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Duster: duster::{Pallet, Call, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>},
		Currencies: orml_currencies::{Pallet, Event<T>},
		Balances: pallet_balances::{Pallet,Call, Storage,Config<T>, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;

	pub const SS58Prefix: u8 = 63;
	pub const MaxLocks: u32 = 50;

	pub const NativeExistentialDeposit: u128 = 0;

	pub NativeCurrencyId: AssetId = 0;
	pub Reward: Balance = 10_000;
}

thread_local! {
	pub static KILLED: RefCell<Vec<u64>> = RefCell::new(vec![]);
}

pub struct RecordKilled;
impl OnKilledAccount<u64> for RecordKilled {
	fn on_killed_account(who: &u64) {
		KILLED.with(|r| r.borrow_mut().push(*who))
	}
}

impl system::Config for Test {
	type BaseCallFilter = ();
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
	type OnKilledAccount = RecordKilled;
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		Zero::zero()
	};
}

parameter_type_with_key! {
	pub MinDeposits: |currency_id: AssetId| -> Balance {
		match currency_id {
			0 => 1000,
			1 => 100_000,
			_ => 0
		}
	};
}

impl Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type MultiCurrency = Currencies;
	type MinCurrencyDeposits = MinDeposits;
	type Reward = Reward;
	type NativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
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
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = NativeExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
	native_balances: Vec<(AccountId, Balance)>,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![],
			native_balances: vec![(*TREASURY, 1_000_000)],
		}
	}
}

impl ExtBuilder {
	pub fn with_balance(mut self, account: AccountId, currency_id: AssetId, amount: Balance) -> Self {
		self.endowed_accounts.push((account, currency_id, amount));
		self
	}
	pub fn with_native_balance(mut self, account: AccountId, amount: Balance) -> Self {
		self.native_balances.push((account, amount));
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
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

		duster::GenesisConfig::<Test> {
			account_blacklist: vec![*TREASURY],
			reward_account: *TREASURY,
			dust_account: *TREASURY,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
