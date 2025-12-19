use crate as pallet_marketplace;
use frame_support::{
	assert_ok, parameter_types,
	traits::{AsEnsureOriginWithArg, Everything, NeverEnsureOrigin},
	BoundedVec,
};
use frame_system as system;
use pallet_nft::{CollectionType, NftPermissions};
use sp_core::storage::Storage;
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};
use sp_std::convert::{TryFrom, TryInto};
use system::EnsureRoot;

type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = AccountId32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Marketplace: pallet_marketplace,
		NFT: pallet_nft,
		Balances: pallet_balances,
		Uniques: pallet_uniques,
	}
);

/// Balance of an account.
pub type Balance = u128;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

parameter_types! {
	pub const MinimumOfferAmount: Balance = 50 * UNITS;
	pub const RoyaltyBondAmount: Balance = 200 * UNITS;
}

impl pallet_marketplace::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = pallet_marketplace::weights::BasiliskWeight<Test>;
	type MinimumOfferAmount = MinimumOfferAmount;
	type RoyaltyBondAmount = RoyaltyBondAmount;
}

parameter_types! {
	pub ReserveCollectionIdUpTo: u32 = 999;
}

impl pallet_nft::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type NftCollectionId = u32;
	type NftItemId = u32;
	type CollectionType = CollectionType;
	type Permissions = NftPermissions;
	type ReserveCollectionIdUpTo = ReserveCollectionIdUpTo;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type DoneSlashHandler = ();
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = RuntimeTask;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
	type ExtensionsWeightInfo = ();
}

parameter_types! {
	pub const CollectionDeposit: Balance = 10_000 * UNITS; // 1 UNIT deposit to create asset collection
	pub const ItemDeposit: Balance = 100 * UNITS; // 1/100 UNIT deposit to create asset item
	pub const KeyLimit: u32 = 32;	// Max 32 bytes per key
	pub const ValueLimit: u32 = 64;	// Max 64 bytes per value
	pub const UniquesMetadataDepositBase: Balance = 100 * UNITS;
	pub const AttributeDepositBase: Balance = 10 * UNITS;
	pub const DepositPerByte: Balance = UNITS;
	pub const UniquesStringLimit: u32 = 128;
}

impl pallet_uniques::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = ();
	type Locker = ();
	type CreateOrigin = AsEnsureOriginWithArg<NeverEnsureOrigin<AccountId>>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([4u8; 32]);

pub const UNITS: Balance = 100_000_000_000;

pub const COLLECTION_ID_0: <Test as pallet_uniques::Config>::CollectionId = 1000;
pub const COLLECTION_ID_1: <Test as pallet_uniques::Config>::CollectionId = 1001;
pub const COLLECTION_ID_2: <Test as pallet_uniques::Config>::CollectionId = 1002;

pub const ITEM_ID_0: <Test as pallet_uniques::Config>::ItemId = 0;
pub const ITEM_ID_1: <Test as pallet_uniques::Config>::ItemId = 1;

#[derive(Default)]
pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, Balance)>,
	minted_nfts: Vec<(
		AccountId,
		<Test as pallet_uniques::Config>::CollectionId,
		<Test as pallet_uniques::Config>::ItemId,
	)>,
}

impl ExtBuilder {
	pub fn with_endowed_accounts(mut self, accounts: Vec<(AccountId, Balance)>) -> Self {
		self.endowed_accounts = accounts;
		self
	}

	pub fn with_minted_nft(
		mut self,
		nft: (
			AccountId,
			<Test as pallet_uniques::Config>::CollectionId,
			<Test as pallet_uniques::Config>::ItemId,
		),
	) -> Self {
		self.minted_nfts.push(nft);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		self.add_account_with_balances(&mut t);

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| self.create_nft());
		ext
	}

	fn add_account_with_balances(&self, t: &mut Storage) {
		pallet_balances::GenesisConfig::<Test> {
			balances: self
				.endowed_accounts
				.clone()
				.iter()
				.flat_map(|(x, asset)| vec![(x.clone(), *asset)])
				.collect(),
			dev_accounts: Default::default(),
		}
		.assimilate_storage(t)
		.unwrap();
	}

	fn create_nft(&self) {
		for nft in &self.minted_nfts {
			let metadata: BoundedVec<u8, <Test as pallet_uniques::Config>::StringLimit> =
				b"metadata".to_vec().try_into().unwrap();
			assert_ok!(NFT::create_collection(
				RuntimeOrigin::signed(nft.0.clone()),
				nft.1,
				Default::default(),
				metadata.clone()
			));
			assert_ok!(NFT::mint(RuntimeOrigin::signed(nft.0.clone()), nft.1, nft.2, metadata));
		}
	}
}

pub fn last_event() -> RuntimeEvent {
	frame_system::Pallet::<Test>::events()
		.pop()
		.expect("An event expected")
		.event
}

pub fn expect_events(e: Vec<RuntimeEvent>) {
	e.into_iter().for_each(frame_system::Pallet::<Test>::assert_has_event);
}
