use crate as price_oracle;
use crate::Config;
use frame_support::parameter_types;
use frame_support::traits::OnInitialize;
use frame_system;
use primitives::{AssetId, Balance, Price};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
};
use price_oracle::PriceEntry;

pub type AccountId = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ASSET_PAIR_A: AccountId = 1_000;
pub const ASSET_PAIR_B: AccountId = 2_000;

pub const PRICE_ENTRY_1: PriceEntry = PriceEntry {price: Price::from_inner(2000000000000000000), amount: 1_000, liq_amount: 2_000};
pub const PRICE_ENTRY_2: PriceEntry = PriceEntry {price: Price::from_inner(5000000000000000000), amount: 3_000, liq_amount: 4_000};

frame_support::construct_runtime!(
	pub enum Test where
	 Block = Block,
	 NodeBlock = Block,
	 UncheckedExtrinsic = UncheckedExtrinsic,
	 {
		 System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		 PriceOracle: price_oracle::{Pallet, Call, Storage, Event<T>},
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

parameter_types! {
	pub const BucketLength: u32 = 10;
	pub const BucketDepth: u32 = 4;
	pub const MaxAssetCount: u32 = 5;
}

impl Config for Test {
	type Event = Event;
	type BucketLength = BucketLength;
	type BucketDepth = BucketDepth;
	type MaxAssetCount = MaxAssetCount;
}

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		sp_io::TestExternalities::from(storage)
	}
}

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	PriceOracle::on_initialize(System::block_number());
}
