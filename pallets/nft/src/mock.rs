use super::*;
use crate as pallet_nft;

use frame_support::{parameter_types, weights::Weight};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

mod nfc {
	// Re-export needed for `impl_outer_event!`.
	pub use super::super::*;
}

type AccountId = AccountId32;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		OrmlNft: orml_nft::{Pallet, Storage},
		NFT: pallet_nft::{Pallet, Call, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub ClassBondAmount: Balance = 100;
	pub MintMaxQuantity: u32 = 100_000;
	pub MaxMetadataLength: u32 = 256;
	pub MaxEmoteLength: u32 = 256;
}

impl pallet_nft::Config for Test {
	type Currency = Balances;
	type Event = Event;
	type WeightInfo = pallet_nft::weights::HydraWeight<Test>;
	type ClassBondAmount = ClassBondAmount;
	type MintMaxQuantity = MintMaxQuantity;
	type MaxEmoteLength = MaxEmoteLength;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 256;
	pub MaxTokenMetadata: u32 = 256;
}

impl orml_nft::Config for Test {
	type ClassId = u64;
	type TokenId = u64;
	type ClassData = pallet_nft::ClassData;
	type TokenData = pallet_nft::TokenData;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
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
	type ReserveIdentifier = [u8; 8];
}

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const BSX: Balance = 100_000_000_000;
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const CLASS_ID: <Test as orml_nft::Config>::ClassId = 0;
pub const EMOTE: &str = "RMRK::EMOTE::RMRK1.0.0::0aff6865bed3a66b-VALHELLO-POTION_HEAL-0000000000000001::1F389";
pub const TEST_QUANTITY: u32 = 99;
pub const TEST_PRICE: Balance = 99;
pub const TOKEN_ID: <Test as orml_nft::Config>::TokenId = 0;

pub struct ExtBuilder;
impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(ALICE, 1_000_000 * BSX), (BOB, 6_666_666 * BSX)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Test>::events()
		.pop()
		.expect("An event expected")
		.event
}
