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
mod assets;
mod governance;
mod system;
pub mod xcm;

pub use assets::*;
pub use governance::origins::pallet_custom_origins;
pub use governance::*;
pub use system::*;
pub use xcm::*;

pub use primitives::{
	constants::time::SLOT_DURATION, AccountId, Amount, AssetId, Balance, BlockNumber, CollectionId, Hash, Index,
	ItemId, Price, Signature,
};

use frame_support::sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, BlakeTwo256, Block as BlockT},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_api::impl_runtime_apis;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{ConstU32, OpaqueMetadata};
use sp_std::{convert::From, marker::PhantomData, prelude::*, vec};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
use frame_support::{
	construct_runtime,
	genesis_builder_helper::{build_config, create_default_config},
	parameter_types,
	weights::Weight,
};

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
	spec_version: 118,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
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
		let maybe_data = cumulus_pallet_parachain_system::Pallet::<T>::validation_data();

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
		//NOTE: 6 - is used by Scheduler which must be after cumulus_pallet_parachain_system
		Democracy: pallet_democracy exclude_parts { Config } = 7,
		Elections: pallet_elections_phragmen = 8,
		Council: pallet_collective::<Instance1> = 9,
		TechnicalCommittee: pallet_collective::<Instance2> = 10,
		Vesting: orml_vesting = 11,
		Proxy: pallet_proxy = 12,
		Tips: pallet_tips = 13,

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
	(
		frame_support::migrations::RemovePallet<DmpQueuePalletName, <Runtime as frame_system::Config>::DbWeight>,
		frame_support::migrations::RemovePallet<XcmRateLimiterPalletName, <Runtime as frame_system::Config>::DbWeight>,
		cumulus_pallet_xcmp_queue::migration::v4::MigrationToV4<Runtime>,
		pallet_identity::migration::versioned::V0ToV1<Runtime, 200u64>, // We have currently 89 identities in basllisk, so limit of 200 should be enough
	),
>;

parameter_types! {
	pub const DmpQueuePalletName: &'static str = "DmpQueue";
	pub const XcmRateLimiterPalletName: &'static str = "XcmRateLimiter";
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(SLOT_DURATION)
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect,
		) -> Weight {
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}


	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}

		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
				included_hash: <Block as BlockT>::Hash,
				slot: cumulus_primitives_aura::Slot,
		) -> bool {
				ConsensusHook::can_build_upon(included_hash, slot)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use orml_benchmarking::list_benchmark as orml_list_benchmark;

			use frame_system_benchmarking::Pallet as SystemBench;
			use pallet_xyk_liquidity_mining_benchmarking::Pallet as XYKLiquidityMiningBench;
			use pallet_xcm::benchmarking::Pallet as PalletXcmExtrinsiscsBenchmark;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			orml_list_benchmark!(list, extra, pallet_currencies, benchmarking::currencies);
			orml_list_benchmark!(list, extra, pallet_xyk, benchmarking::xyk);
			orml_list_benchmark!(list, extra, orml_tokens, benchmarking::tokens);
			orml_list_benchmark!(list, extra, orml_vesting, benchmarking::vesting);
			orml_list_benchmark!(list, extra, pallet_duster, benchmarking::duster);
			orml_list_benchmark!(list, extra, pallet_transaction_multi_payment, benchmarking::multi_payment);
			orml_list_benchmark!(list, extra, pallet_route_executor, benchmarking::route_executor);
			orml_list_benchmark!(list, extra, pallet_marketplace, benchmarking::marketplace);
			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{BenchmarkError, Benchmarking, BenchmarkBatch};
			use frame_support::traits::TrackedStorageKey;
			use sp_core::Get;
			use sp_std::sync::Arc;
			use primitives::constants::chain::CORE_ASSET_ID;

			use orml_benchmarking::add_benchmark as orml_add_benchmark;

			use frame_system_benchmarking::Pallet as SystemBench;
			use pallet_xyk_liquidity_mining_benchmarking::Pallet as XYKLiquidityMiningBench;
			use pallet_xcm::benchmarking::Pallet as PalletXcmExtrinsiscsBenchmark;

			impl frame_system_benchmarking::Config for Runtime {
				fn setup_set_code_requirements(code: &sp_std::vec::Vec<u8>) -> Result<(), BenchmarkError> {
					ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					Ok(())
				}

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}

			parameter_types! {
				pub const RandomParaId: ParaId = ParaId::new(22222222);
				pub const ExistentialDeposit: u128= 1_000_000_000_000;
				pub AssetLocation: Location = Location::new(1, cumulus_primitives_core::Junctions::X2(
					Arc::new([cumulus_primitives_core::Junction::Parachain(ParachainInfo::get().into()),
						cumulus_primitives_core::Junction::GeneralIndex(CORE_ASSET_ID.into())
						])
				));
			}

			use cumulus_primitives_core::ParaId;
			use polkadot_xcm::latest::prelude::{Location, AssetId, Fungible, Asset, ParentThen, Parachain, Parent};

			impl pallet_xcm::benchmarking::Config for Runtime {
				fn reachable_dest() -> Option<Location> {
					Some(Parent.into())
				}

				fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
					Some((
						Asset {
							fun: Fungible(ExistentialDeposit::get()),
							id: AssetId(AssetLocation::get())
						},
						Parent.into(),
					))
				}

				fn reserve_transferable_asset_and_dest() -> Option<(Asset, Location)> {
					Some((
						Asset {
							fun: Fungible(ExistentialDeposit::get()),
							id: AssetId(AssetLocation::get())
						},
						ParentThen(Parachain(RandomParaId::get().into()).into()).into(),
					))
				}
			}

			impl pallet_xyk_liquidity_mining_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
				// Treasury Account
				frame_system::Account::<Runtime>::hashed_key_for(Treasury::account_id()).into()
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			orml_add_benchmark!(params, batches, pallet_xyk, benchmarking::xyk);
			orml_add_benchmark!(params, batches, pallet_currencies, benchmarking::currencies);
			orml_add_benchmark!(params, batches, orml_tokens, benchmarking::tokens);
			orml_add_benchmark!(params, batches, orml_vesting, benchmarking::vesting);
			orml_add_benchmark!(params, batches, pallet_duster, benchmarking::duster);
			orml_add_benchmark!(params, batches, pallet_transaction_multi_payment, benchmarking::multi_payment);
			orml_add_benchmark!(params, batches, pallet_route_executor, benchmarking::route_executor);
			orml_add_benchmark!(params, batches, pallet_marketplace, benchmarking::marketplace);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_support::parameter_types! {
		pub const BenchmarkMaxBalance: crate::Balance = crate::Balance::max_value();
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
		[pallet_elections_phragmen, Elections]
		[pallet_treasury, Treasury]
		[pallet_scheduler, Scheduler]
		[pallet_utility, Utility]
		[pallet_tips, Tips]
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

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
			relay_chain_slot,
			sp_std::time::Duration::from_secs(6),
		)
		.create_inherent_data()
		.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
