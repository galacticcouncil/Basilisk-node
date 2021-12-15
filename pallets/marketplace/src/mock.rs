use crate as pallet_marketplace;
use frame_support::{parameter_types, traits::Everything};
use frame_system as system;
use primitives::{ClassType, ReserveIdentifier};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use system::EnsureRoot;

use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use pallet_nft::NftPermission;
use scale_info::TypeInfo;

mod marketplace {
	// Re-export needed for `impl_outer_event!`.
	pub use super::super::*;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = AccountId32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Marketplace: pallet_marketplace::{Pallet, Call, Storage, Event<T>},
		NFT: pallet_nft::{Pallet, Call, Event<T>, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
	}
);

/// Balance of an account.
pub type Balance = u128;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

parameter_types! {
	pub const MinimumOfferAmount: Balance = 20 * BSX;
}

impl pallet_marketplace::Config for Test {
	type Event = Event;
	type WeightInfo = pallet_marketplace::weights::BasiliskWeight<Test>;
	type MinimumOfferAmount = MinimumOfferAmount;
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftPermissions;

impl NftPermission<ClassType> for NftPermissions {
	fn can_create(class_type: &ClassType) -> bool {
		match *class_type {
			ClassType::Plain => true,
			ClassType::Marketplace => true,
			_ => false,
		}
	}

	fn can_mint(class_type: &ClassType) -> bool {
		match *class_type {
			ClassType::Plain => true,
			ClassType::Marketplace => true,
			_ => false,
		}
	}

	fn can_transfer(class_type: &ClassType) -> bool {
		match *class_type {
			ClassType::Plain => true,
			ClassType::Marketplace => true,
			_ => false,
		}
	}

	fn can_burn(class_type: &ClassType) -> bool {
		match *class_type {
			ClassType::Plain => true,
			ClassType::Marketplace => true,
			_ => false,
		}
	}

	fn can_destroy(class_type: &ClassType) -> bool {
		match *class_type {
			ClassType::Plain => true,
			ClassType::Marketplace => true,
			_ => false,
		}
	}

	fn has_deposit(class_type: &ClassType) -> bool {
		match *class_type {
			ClassType::Plain => true,
			ClassType::Marketplace => true,
			_ => false,
		}
	}
}
impl pallet_nft::Config for Test {
	type Currency = Balances;
	type Event = Event;
	type WeightInfo = pallet_nft::weights::BasiliskWeight<Test>;
	type TokenDeposit = InstanceDeposit;
	type NftClassId = u32;
	type NftInstanceId = u32;
	type ProtocolOrigin = EnsureRoot<AccountId>;
	type ClassType = ClassType;
	type Permissions = NftPermissions;
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
	type ReserveIdentifier = ReserveIdentifier;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
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
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ClassDeposit: Balance = 10_000 * BSX; // 1 UNIT deposit to create asset class
	pub const InstanceDeposit: Balance = 100 * BSX; // 1/100 UNIT deposit to create asset instance
	pub const KeyLimit: u32 = 32;	// Max 32 bytes per key
	pub const ValueLimit: u32 = 64;	// Max 64 bytes per value
	pub const UniquesMetadataDepositBase: Balance = 100 * BSX;
	pub const AttributeDepositBase: Balance = 10 * BSX;
	pub const DepositPerByte: Balance = BSX;
	pub const UniquesStringLimit: u32 = 128;
}

impl pallet_uniques::Config for Test {
	type Event = Event;
	type ClassId = u32;
	type InstanceId = u32;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type ClassDeposit = ClassDeposit;
	type InstanceDeposit = InstanceDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = ();
	type InstanceReserveStrategy = ();
}

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([4u8; 32]);

pub const BSX: Balance = 100_000_000_000;

pub const CLASS_ID_0: <Test as pallet_uniques::Config>::ClassId = 0;
pub const CLASS_ID_1: <Test as pallet_uniques::Config>::ClassId = 1;

pub const INSTANCE_ID_0: <Test as pallet_uniques::Config>::InstanceId = 0;
pub const INSTANCE_ID_1: <Test as pallet_uniques::Config>::InstanceId = 1;

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
			balances: vec![
				(ALICE, 200_000 * BSX),
				(BOB, 15_000 * BSX),
				(CHARLIE, 150_000 * BSX),
				(DAVE, 200_000 * BSX),
			],
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
