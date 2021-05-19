use crate as lbp;
use crate::{AssetPairPoolIdFor, Config};
use frame_support::parameter_types;
use frame_support::traits::GenesisBuild;
use frame_system;
use orml_traits::parameter_type_with_key;
use primitives::{fee, AssetId, Balance, CORE_ASSET_ID};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
};

pub type Amount = i128;
pub type AccountId = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const INITIAL_BALANCE: Balance = 1_000_000_000_000_000u128;
pub const POOL_DEPOSIT: Balance = 10u128.pow(14);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub const HDX: AssetId = CORE_ASSET_ID;
pub const ACA: AssetId = 2_000;
pub const DOT: AssetId = 3_000;
pub const ETH: AssetId = 4_000;

pub const HDX_DOT_POOL_ID: AccountId = 3_000;
pub const ACA_DOT_POOL_ID: AccountId = 2_003_000;

frame_support::construct_runtime!(
	pub enum Test where
	 Block = Block,
	 NodeBlock = Block,
	 UncheckedExtrinsic = UncheckedExtrinsic,
	 {
		 System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		 LBPPallet: lbp::{Pallet, Call, Storage, Event<T>},
		 Currency: orml_tokens::{Pallet, Event<T>},
	 }

);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
}

impl frame_system::Config for Test {
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
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
}

pub struct AssetPairPoolIdTest();

impl AssetPairPoolIdFor<AssetId, u64> for AssetPairPoolIdTest {
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

parameter_types! {
	pub PoolDeposit: Balance = POOL_DEPOSIT;
	pub ExchangeFee: fee::Fee  = fee::Fee::default();
	pub const NativeAssetId: AssetId = CORE_ASSET_ID;
}

impl Config for Test {
	type Event = Event;
	type MultiCurrency = Currency;
	type NativeAssetId = NativeAssetId;
	type CreatePoolOrigin = frame_system::EnsureRoot<u64>;
	type LBPWeightFunction = lbp::LBPWeightFunction;
	type AssetPairPoolId = AssetPairPoolIdTest;
	type PoolDeposit = PoolDeposit;
	type ExchangeFee = ExchangeFee;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, HDX, INITIAL_BALANCE),
				(ALICE, DOT, INITIAL_BALANCE),
				(ALICE, ACA, INITIAL_BALANCE),
				(ALICE, ETH, INITIAL_BALANCE),
				(BOB, HDX, INITIAL_BALANCE),
				(BOB, DOT, INITIAL_BALANCE),
				(BOB, ACA, INITIAL_BALANCE),
				(BOB, ETH, INITIAL_BALANCE),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		orml_tokens::GenesisConfig::<Test> {
			endowed_accounts: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
	}
}
