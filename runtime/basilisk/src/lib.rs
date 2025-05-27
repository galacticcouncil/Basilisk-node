// This file is part of Basilisk-node.

// Copyright (C) 2020-2023  Intergalactic, Limited (GIB).
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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 512.
#![recursion_limit = "512"]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::from_over_into)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::items_after_test_module)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(test)]
mod tests;

mod benchmarking;
pub mod weights;

mod adapter;
pub mod apis;
mod assets;
mod governance;
mod system;
pub mod xcm;

pub use assets::*;
pub use governance::origins::pallet_custom_origins;
pub use governance::*;
pub use system::*;
pub use xcm::*;

use frame_support::sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, BlakeTwo256, Block as BlockT},
};
use frame_system::pallet_prelude::BlockNumberFor;
pub use primitives::{
	constants::time::SLOT_DURATION, AccountId, Amount, AssetId, Balance, BlockNumber, CollectionId, Hash, Index,
	ItemId, Price, Signature,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::ConstU32;
use sp_std::{convert::From, marker::PhantomData, prelude::*, vec};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
use frame_support::{construct_runtime, weights::Weight};

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
		}
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("basilisk"),
	impl_name: create_runtime_str!("basilisk"),
	authoring_version: 1,
	spec_version: 126,
	impl_version: 0,
	apis: apis::RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

pub fn get_all_module_accounts() -> vec::Vec<AccountId> {
	vec![
		TreasuryPalletId::get().into_account_truncating(),
		VestingPalletId::get().into_account_truncating(),
	]
}

use sp_runtime::traits::BlockNumberProvider;

// Relay chain Block number provider.
// Reason why the implementation is different for benchmarks is that it is not possible
// to set or change the block number in a benchmark using parachain system pallet.
// That's why we revert to using the system pallet in the benchmark.
pub struct RelayChainBlockNumberProvider<T>(PhantomData<T>);

#[cfg(not(feature = "runtime-benchmarks"))]
impl<T: cumulus_pallet_parachain_system::Config + orml_tokens::Config> BlockNumberProvider
	for RelayChainBlockNumberProvider<T>
{
	type BlockNumber = BlockNumberFor<T>;

	fn current_block_number() -> Self::BlockNumber {
		let maybe_data = cumulus_pallet_parachain_system::ValidationData::<T>::get();

		if let Some(data) = maybe_data {
			data.relay_parent_number.into()
		} else {
			Self::BlockNumber::default()
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<T: frame_system::Config> BlockNumberProvider for RelayChainBlockNumberProvider<T> {
	type BlockNumber = BlockNumberFor<T>;

	fn current_block_number() -> Self::BlockNumber {
		frame_system::Pallet::<T>::current_block_number()
	}
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime
	{
		// Substrate
		System: frame_system exclude_parts { Origin } = 0,
		Timestamp: pallet_timestamp = 1,
		Balances: pallet_balances = 2,
		TransactionPayment: pallet_transaction_payment exclude_parts { Config } = 3,
		// due to multi payment pallet prices, this needs to be initialized at the very beginning
		MultiTransactionPayment: pallet_transaction_multi_payment = 106,
		Treasury: pallet_treasury = 4,
		Utility: pallet_utility = 5,
		// NOTE: 6 - is used by Scheduler which must be after cumulus_pallet_parachain_system
		Democracy: pallet_democracy exclude_parts { Config } = 7,
		// NOTE 7, 8, 9 are retired (used by gov v1)
		TechnicalCommittee: pallet_collective::<Instance2> = 10,
		Vesting: orml_vesting = 11,
		Proxy: pallet_proxy = 12,

		// The order of next 4 is important, and it cannot change.
		Authorship: pallet_authorship = 14,
		CollatorSelection: pallet_collator_selection = 15,
		Session: pallet_session = 16,
		Aura: pallet_aura = 17,
		AuraExt: cumulus_pallet_aura_ext exclude_parts { Storage } = 18,
		Preimage: pallet_preimage = 19,
		Uniques: pallet_uniques = 20,
		Identity: pallet_identity = 21,
		Multisig: pallet_multisig = 22,
		StateTrieMigration: pallet_state_trie_migration = 23,

		// OpenGov
		ConvictionVoting: pallet_conviction_voting::{Pallet, Call, Storage, Event<T>} = 24,
		Referenda: pallet_referenda::{Pallet, Call, Storage, Event<T>} = 25,
		Origins: pallet_custom_origins::{Origin} = 26,
		Whitelist: pallet_whitelist::{Pallet, Call, Storage, Event<T>} = 27,

		// Parachain and XCM - starts at index 50
		// The order of next 3 pallest is important
		RelayChainInfo: pallet_relaychain_info = 108,
		Scheduler: pallet_scheduler = 6,
		ParachainSystem: cumulus_pallet_parachain_system exclude_parts { Config } = 50,
		ParachainInfo: staging_parachain_info = 51,

		PolkadotXcm: pallet_xcm = 52,
		CumulusXcm: cumulus_pallet_xcm = 53,
		XcmpQueue: cumulus_pallet_xcmp_queue exclude_parts { Call } = 54,
		// 55 was used by DmpQueue which is now replaced by MessageQueue
		MessageQueue: pallet_message_queue = 56,

		// Basilisk - runtime module index for basilisk's pallets starts at 100
		AssetRegistry: pallet_asset_registry = 100,
		XYK: pallet_xyk = 101,
		Duster: pallet_duster = 102,
		LBP: pallet_lbp = 104,
		NFT: pallet_nft = 105,
		Marketplace: pallet_marketplace = 109,
		TransactionPause: pallet_transaction_pause = 110,
		Router: pallet_route_executor = 111,
		XYKLiquidityMining: pallet_xyk_liquidity_mining = 112,
		XYKWarehouseLM: warehouse_liquidity_mining::<Instance1> = 113,
		CollatorRewards: pallet_collator_rewards = 114,
		// Note: 115 was used by rate limiter which is now removed
		Broadcast: pallet_broadcast = 116,

		EmaOracle: pallet_ema_oracle = 120,

		// ORML related modules - runtime module index for orml starts at 150
		Currencies: pallet_currencies = 150,
		Tokens: orml_tokens = 151,

		// ORML XCM
		OrmlXcm: orml_xcm = 153,
		XTokens: orml_xtokens = 154,
		UnknownTokens: orml_unknown_tokens = 155,
	}
);

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	migrations::Migrations,
>;

pub mod migrations {
	use super::*;

	impl cumulus_pallet_xcmp_queue::migration::v5::V5Config for Runtime {
		type ChannelList = ParachainSystem;
	}

	pub type Migrations = (cumulus_pallet_xcmp_queue::migration::v5::MigrateV4ToV5<Runtime>,);
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_support::parameter_types! {
		pub const BenchmarkMaxBalance: crate::Balance = crate::Balance::MAX;
	}
	frame_benchmarking::define_benchmarks!(
		[pallet_lbp, LBP]
		[pallet_nft, NFT]
		[pallet_asset_registry, AssetRegistry]
		[pallet_xyk_liquidity_mining, XYKLiquidityMiningBench::<Runtime>]
		[pallet_transaction_pause, TransactionPause]
		[pallet_ema_oracle, EmaOracle]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_timestamp, Timestamp]
		[pallet_democracy, Democracy]
		[pallet_treasury, Treasury]
		[pallet_scheduler, Scheduler]
		[pallet_utility, Utility]
		[pallet_identity, Identity]
		[pallet_collective, TechnicalCommittee]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
		[pallet_message_queue, MessageQueue]
		[pallet_preimage, Preimage]
		[pallet_multisig, Multisig]
		[pallet_proxy, Proxy]
		[cumulus_pallet_parachain_system, ParachainSystem]
		[pallet_state_trie_migration, StateTrieMigration]
		[pallet_collator_selection, CollatorSelection]
		[pallet_xcm, PalletXcmExtrinsiscsBenchmark::<Runtime>]
		[pallet_conviction_voting, ConvictionVoting]
		[pallet_referenda, Referenda]
		[pallet_whitelist, Whitelist]
	);
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
