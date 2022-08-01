//      ---_ ......._-_--.        ,adPPYba, 8b,dPPYba,    ,adPPYba,  88   ,d8
//     (|\ /      / /| \  \       I8[    "" 88P'   `"8a  a8P_____88  88 ,a8"
//     /  /     .'  -=-'   `.      `"Y8ba,  88       88  8PP"""""""  8888[
//    /  /    .'             )    aa    ]8I 88       88  "8b,   ,aa  88`"Yba,
//  _/  /   .'        _.)   /     `"YbbdP"' 88       88   `"Ybbd8"'  88   `Y8a
//  / o   o        _.-' /  .'
//  \          _.-'    / .'*|
//  \______.-'//    .'.' \*|      This file is part of Basilisk-node.
//   \|  \ | //   .'.' _ |*|      Built with <3 for decentralisation.
//    `   \|//  .'.'_ _ _|*|
//     .  .// .'.' | _ _ \*|      Copyright (C) 2021-2022  Intergalactic, Limited (GIB).
//     \`-|\_/ /    \ _ _ \*\     SPDX-License-Identifier: Apache-2.0
//      `/'\__/      \ _ _ \*\    Licensed under the Apache License, Version 2.0 (the "License");
//     /^|            \ _ _ \*    you may not use this file except in compliance with the License.
//    '  `             \ _ _ \    http://www.apache.org/licenses/LICENSE-2.0
//     '  `             \ _ _ \

use crate::{self as pallet};

use frame_support::{parameter_types, traits::Everything, PalletId};
use frame_system as system;
use primitives::nft::{ClassType, NftPermissions};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use system::EnsureRoot;

mod auction {
	// Re-export needed for `impl_outer_event!`.
	pub use super::super::*;
}

pub use crate::mock::Event as TestEvent;

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
		Auctions: pallet::{Pallet, Call, Storage, Event<T>},
		Nft: pallet_nft::{Pallet, Call, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
	}
);

/// Balance of an account.
pub type Balance = u128;

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([4u8; 32]);
pub const EVE: AccountId = AccountId::new([5u8; 32]);
pub const BSX: Balance = 100_000_000_000;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

parameter_types! {
	pub ReserveClassIdUpTo: u32 = 999;
}

impl pallet_nft::Config for Test {
	type Event = Event;
	type WeightInfo = pallet_nft::weights::BasiliskWeight<Test>;
	type NftClassId = u32;
	type NftInstanceId = u32;
	type ProtocolOrigin = EnsureRoot<AccountId>;
	type ClassType = ClassType;
	type Permissions = NftPermissions;
	type ReserveClassIdUpTo = ReserveClassIdUpTo;
}

parameter_types! {
	pub const AuctionsStringLimit: u32 = 128;
	pub const BidAddBlocks: u32 = 10;
	pub const BidStepPerc: u32 = 10;
	pub const MinAuctionDuration: u32 = 10;
	pub const BidMinAmount: u32 = 1;
	pub const AuctionsPalletId: PalletId = PalletId(*b"auctions");
	pub const CandleDefaultDuration: u32 = 99_356;
	pub const CandleDefaultClosingPeriodDuration: u32 = 72_000;
	pub const CandleDefaultClosingRangesCount: u32 = 100;
}

pub struct TestRandomness<T>(sp_std::marker::PhantomData<T>);

impl<Output: codec::Decode + Default, T> frame_support::traits::Randomness<Output, T::BlockNumber> for TestRandomness<T>
where
	T: frame_system::Config,
{
	fn random(subject: &[u8]) -> (Output, T::BlockNumber) {
		use sp_runtime::traits::TrailingZeroInput;

		(
			Output::decode(&mut TrailingZeroInput::new(subject)).unwrap_or_default(),
			frame_system::Pallet::<T>::block_number(),
		)
	}
}

impl pallet::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type AuctionId = u64;
	type Currency = Balances;
	type Randomness = TestRandomness<Test>;
	type WeightInfo = pallet::weights::BasiliskWeight<Test>;
	type AuctionsStringLimit = AuctionsStringLimit;
	type BidAddBlocks = BidAddBlocks;
	type BidStepPerc = BidStepPerc;
	type MinAuctionDuration = MinAuctionDuration;
	type BidMinAmount = BidMinAmount;
	type PalletId = AuctionsPalletId;
	type CandleDefaultDuration = CandleDefaultDuration;
	type CandleDefaultClosingPeriodDuration = CandleDefaultClosingPeriodDuration;
	type CandleDefaultClosingRangesCount = CandleDefaultClosingRangesCount;
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
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ClassDeposit: Balance = 9_900 * BSX; // 1 UNIT deposit to create asset class
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
}

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
				(ALICE, 40_000 * BSX),
				(BOB, 2_000 * BSX),
				(CHARLIE, 4_000 * BSX),
				(DAVE, 4_000 * BSX),
				(EVE, 4_000 * BSX),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
