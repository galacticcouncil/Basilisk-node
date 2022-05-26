// This file is part of Basilisk-node.

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test environment for Assets pallet.

use std::cell::RefCell;
use std::collections::HashMap;

use crate as pallet_stableswap;

use crate::Config;

use frame_support::traits::{Everything, GenesisBuild};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64},
};
use frame_system::EnsureSigned;
use orml_traits::parameter_type_with_key;
pub use orml_traits::MultiCurrency;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	DispatchError,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type Balance = u128;
pub type AssetId = u32;

pub const HDX: AssetId = 0;
pub const DAI: AssetId = 2;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;

pub const ONE: Balance = 1_000_000_000_000;

#[macro_export]
macro_rules! assert_balance {
	( $x:expr, $y:expr, $z:expr) => {{
		assert_eq!(Tokens::free_balance($y, &$x), $z);
	}};
}

thread_local! {
	pub static REGISTERED_ASSETS: RefCell<HashMap<AssetId, u32>> = RefCell::new(HashMap::default());
	pub static ASSET_IDENTS: RefCell<HashMap<Vec<u8>, u32>> = RefCell::new(HashMap::default());
}

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Event<T>},
		Stableswap: pallet_stableswap::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
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
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		0
	};
}

impl orml_tokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = i128;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = ();
	type DustRemovalWhitelist = Everything;
}

parameter_types! {
	pub const HDXAssetId: AssetId = HDX;
	pub const DAIAssetId: AssetId = DAI;
	pub const Precision: Balance = 1;
	pub const MinimumLiquidity: Balance = 1000;
	pub const MinimumTradingLimit: Balance = 1000;
}

impl Config for Test {
	type Event = Event;
	type AssetId = AssetId;
	type Currency = Tokens;
	type ShareAccountId = AccountIdConstructor;
	type AssetRegistry = DummyRegistry<Test>;
	type CreatePoolOrigin = EnsureSigned<u64>;
	type Precision = Precision;
	type MinimumLiquidity = MinimumLiquidity;
	type MinimumTradingLimit = MinimumTradingLimit;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(u64, AssetId, Balance)>,
	registered_assets: Vec<(Vec<u8>, AssetId)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		// If eg. tests running on one thread only, this thread local is shared.
		// let's make sure that it is empty for each  test case
		// or set to original default value
		REGISTERED_ASSETS.with(|v| {
			v.borrow_mut().clear();
		});
		ASSET_IDENTS.with(|v| {
			v.borrow_mut().clear();
		});
		Self {
			endowed_accounts: vec![],
			registered_assets: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn with_endowed_accounts(mut self, accounts: Vec<(u64, AssetId, Balance)>) -> Self {
		self.endowed_accounts = accounts;
		self
	}

	pub fn with_registered_asset(mut self, name: Vec<u8>, asset: AssetId) -> Self {
		self.registered_assets.push((name, asset));
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let mut all_assets: Vec<(Vec<u8>, AssetId)> =
			vec![("DAI".as_bytes().to_vec(), DAI), ("HDX".as_bytes().to_vec(), HDX)];
		all_assets.extend(self.registered_assets);

		for (name, asset) in all_assets.into_iter() {
			REGISTERED_ASSETS.with(|v| {
				v.borrow_mut().insert(asset, asset);
			});

			ASSET_IDENTS.with(|v| {
				v.borrow_mut().insert(name, asset);
			})
		}

		orml_tokens::GenesisConfig::<Test> {
			balances: self
				.endowed_accounts
				.iter()
				.flat_map(|(x, asset, amount)| vec![(*x, *asset, *amount)])
				.collect(),
		}
		.assimilate_storage(&mut t)
		.unwrap();
		t.into()
	}
}

use crate::traits::ShareAccountIdFor;
use crate::types::PoolAssets;
use hydradx_traits::{Registry, ShareTokenRegistry};

pub struct DummyRegistry<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Registry<T::AssetId, Vec<u8>, Balance, DispatchError> for DummyRegistry<T>
where
	T::AssetId: Into<AssetId> + From<u32>,
{
	fn exists(asset_id: T::AssetId) -> bool {
		let asset = REGISTERED_ASSETS.with(|v| v.borrow().get(&(asset_id.into())).copied());
		matches!(asset, Some(_))
	}

	fn retrieve_asset(name: &Vec<u8>) -> Result<T::AssetId, DispatchError> {
		let asset_id = ASSET_IDENTS.with(|v| v.borrow().get(name).copied());
		if let Some(id) = asset_id {
			Ok(id.into())
		} else {
			Err(pallet_stableswap::Error::<Test>::AssetNotRegistered.into())
		}
	}

	fn create_asset(name: &Vec<u8>, _existential_deposit: Balance) -> Result<T::AssetId, DispatchError> {
		let assigned = REGISTERED_ASSETS.with(|v| {
			let l = v.borrow().len();
			v.borrow_mut().insert(l as u32, l as u32);
			l as u32
		});

		ASSET_IDENTS.with(|v| v.borrow_mut().insert(name.clone(), assigned));

		Ok(T::AssetId::from(assigned))
	}
}

impl<T: Config> ShareTokenRegistry<T::AssetId, Vec<u8>, Balance, DispatchError> for DummyRegistry<T>
where
	T::AssetId: Into<AssetId> + From<u32>,
{
	fn retrieve_shared_asset(name: &Vec<u8>, _assets: &[T::AssetId]) -> Result<T::AssetId, DispatchError> {
		Self::retrieve_asset(name)
	}

	fn create_shared_asset(
		name: &Vec<u8>,
		_assets: &[T::AssetId],
		existential_deposit: Balance,
	) -> Result<T::AssetId, DispatchError> {
		Self::get_or_create_asset(name.clone(), existential_deposit)
	}
}

pub struct AccountIdConstructor;

impl ShareAccountIdFor<PoolAssets<u32>> for AccountIdConstructor {
	type AccountId = u64;

	fn from_assets(assets: &PoolAssets<u32>, _identifier: Option<&str>) -> Self::AccountId {
		let mut a = assets.0;
		let mut b = assets.1;
		if a > b {
			std::mem::swap(&mut a, &mut b)
		}
		(a * 1000 + b) as u64
	}

	fn name(assets: &PoolAssets<u32>, identifier: Option<&str>) -> Vec<u8> {
		let mut buf: Vec<u8> = if let Some(ident) = identifier {
			ident.as_bytes().to_vec()
		} else {
			vec![]
		};
		buf.extend_from_slice(&(assets.0).to_le_bytes());
		buf.extend_from_slice(&(assets.1).to_le_bytes());

		buf
	}
}

pub(crate) fn retrieve_current_asset_id() -> AssetId {
	REGISTERED_ASSETS.with(|v| v.borrow().len() as AssetId)
}
