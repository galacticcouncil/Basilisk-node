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
//
use crate::pallet::Config;
use sp_core::H256;
use frame_support::parameter_types;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;
use crate as rewards;
use sp_arithmetics::Percent;
use orml_traits::RewardHandler;
use sp_std::cell::RefCell;
use std::collections::HashMap;


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>; 
type AccountId = u64;
type PoolId = u64;
type Balance = u128;

pub const ALICE:AccountId = 1;
pub const BOB:AccountId = 2;
pub const CHARLIE:AccountId = 3;
pub const DAVE:AccountId = 4;
pub const DOT_POOL:PoolId = 1;
pub const HDX_POOL:PoolId = 2;
pub const BSX_POOL:PoolId = 3;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Rewards: rewards::{Pallet, Storage, Call},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
}

impl system::Config for Test {
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
    type OnSetCode = ();
}

thread_local! {
	pub static RECEIVED_PAYOUT: RefCell<HashMap<(PoolId, AccountId), Balance>> = RefCell::new(HashMap::new());
}

pub struct Handler;
impl RewardHandler<AccountId> for Handler {
	type Balance = Balance;
	type PoolId = PoolId;

	fn payout(who: &AccountId, pool: &Self::PoolId, amount: Self::Balance) {
		RECEIVED_PAYOUT.with(|v| {
			let mut old_map = v.borrow().clone();
			if let Some(before) = old_map.get_mut(&(*pool, *who)) {
				*before += amount;
			} else {
				old_map.insert((*pool, *who), amount);
			};

			*v.borrow_mut() = old_map;
		});
	}
}

parameter_types! {
    pub const MaxSnapshots:u16 = 5;
    pub const LoyaltyWeightBonus: u32 = 2;
    pub const LoyaltySlash: Percent = Percent::from_percent(50);
}

impl Config for Test {
    type MaxSnapshots = MaxSnapshots; 
    type LoyaltyWeightBonus = LoyaltyWeightBonus;
    type LoyaltySlash = LoyaltySlash;
    type Handler = Handler;
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		t.into()
	}
}
