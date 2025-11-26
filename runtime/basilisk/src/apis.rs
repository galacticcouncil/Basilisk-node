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

use crate::*;
use frame_support::{
	genesis_builder_helper::{build_state, get_preset},
	sp_runtime::{
		traits::Convert,
		transaction_validity::{TransactionSource, TransactionValidity},
		ApplyExtrinsicResult, ExtrinsicInclusionMode, FixedPointNumber,
	},
	weights::WeightToFee as _,
};
use basilisk_traits::oracle::NativePriceOracle;
use polkadot_xcm::{IntoVersion, VersionedAssetId, VersionedAssets, VersionedLocation, VersionedXcm};
use polkadot_xcm::prelude::XcmVersion;
use primitives::constants::chain::CORE_ASSET_ID;
use sp_api::impl_runtime_apis;
use sp_core::{Get, OpaqueMetadata};
use xcm_runtime_apis::{
	dry_run::{CallDryRunEffects, Error as XcmDryRunApiError, XcmDryRunEffects},
	fees::Error as XcmPaymentApiError,
};

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> ExtrinsicInclusionMode {
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
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
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

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			Default::default()
		}
	}

	impl xcm_runtime_apis::fees::XcmPaymentApi<Block> for Runtime {
		fn query_acceptable_payment_assets(xcm_version: polkadot_xcm::Version) -> Result<Vec<VersionedAssetId>, XcmPaymentApiError> {
			if !matches!(xcm_version, 3 | 4 | 5) {
				return Err(XcmPaymentApiError::UnhandledXcmVersion);
			}

			let mut asset_locations = vec![
		AssetLocation(polkadot_xcm::v5::Location {
				parents: 1,
				interior: [
					polkadot_xcm::v5::Junction::Parachain(ParachainInfo::get().into()),
					polkadot_xcm::v5::Junction::GeneralIndex(CORE_ASSET_ID.into()),
				]
				.into(),
			}),
			AssetLocation(polkadot_xcm::v5::Location {
				parents: 0,
				interior: [
					polkadot_xcm::v5::Junction::GeneralIndex(CORE_ASSET_ID.into()),
				]
				.into(),
			})];

			let mut asset_registry_locations: Vec<AssetLocation> = pallet_asset_registry::LocationAssets::<Runtime>::iter_keys().collect();
			asset_locations.append(&mut asset_registry_locations);

			let versioned_locations = asset_locations.iter().map(|loc| VersionedAssetId::V5(polkadot_xcm::v5::AssetId(loc.0.clone())));

			Ok(versioned_locations
				.filter_map(|asset| asset.into_version(xcm_version).ok())
				.collect())
		}

		fn query_weight_to_asset_fee(weight: Weight, asset: VersionedAssetId) -> Result<u128, XcmPaymentApiError> {
			let v5_xcm_asset_id = asset.into_version(5).map_err(|_| XcmPaymentApiError::VersionedConversionFailed)?;

			// get nested polkadot_xcm::AssetId type
			let xcm_asset_id: &polkadot_xcm::v5::AssetId = v5_xcm_asset_id.try_as().map_err(|_| XcmPaymentApiError::WeightNotComputable)?;

			let asset_id: AssetId = CurrencyIdConvert::convert(xcm_asset_id.clone().0).ok_or(XcmPaymentApiError::AssetNotFound)?;

			let price = MultiTransactionPayment::price(asset_id).ok_or(XcmPaymentApiError::WeightNotComputable)?;

			let fee = WeightToFee::weight_to_fee(&weight);

			let converted_fee = price.checked_mul_int(fee).ok_or(XcmPaymentApiError::WeightNotComputable)?;

			Ok(converted_fee)
		}

		fn query_xcm_weight(message: VersionedXcm<()>) -> Result<Weight, XcmPaymentApiError> {
			PolkadotXcm::query_xcm_weight(message)
		}

		fn query_delivery_fees(destination: VersionedLocation, message: VersionedXcm<()>) -> Result<VersionedAssets, XcmPaymentApiError> {
			PolkadotXcm::query_delivery_fees(destination, message)
		}
	}

	impl xcm_runtime_apis::dry_run::DryRunApi<Block, RuntimeCall, RuntimeEvent, OriginCaller> for Runtime {
		fn dry_run_call(
						origin: OriginCaller,
						call: RuntimeCall,
						result_xcms_version: XcmVersion
					) -> Result<CallDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
						PolkadotXcm::dry_run_call::<
							Runtime,
							xcm::XcmRouter,
							OriginCaller,
							RuntimeCall>(origin, call, result_xcms_version)
					}

		fn dry_run_xcm(origin_location: VersionedLocation, xcm: VersionedXcm<RuntimeCall>) -> Result<XcmDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
			PolkadotXcm::dry_run_xcm::<Runtime, xcm::XcmRouter, RuntimeCall, xcm::XcmConfig>(origin_location, xcm)
		}
	}

	impl xcm_runtime_apis::conversions::LocationToAccountApi<Block, AccountId> for Runtime {
		fn convert_location(location: VersionedLocation) -> Result<
			AccountId,
			xcm_runtime_apis::conversions::Error
		> {
			xcm_runtime_apis::conversions::LocationToAccountHelper::<
				AccountId,
				xcm::LocationToAccountId,
			>::convert_location(location)
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
			use sp_std::sync::Arc;

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

			frame_support::parameter_types! {
				pub const RandomParaId: ParaId = ParaId::new(22_222_222);
				pub const ExistentialDeposit: u128 = 1_000_000_000_000;
				pub AssetLocation: Location = Location::new(0, cumulus_primitives_core::Junctions::X1(
					Arc::new([
						cumulus_primitives_core::Junction::GeneralIndex(CORE_ASSET_ID.into())
						])
				));
			}

			use cumulus_primitives_core::ParaId;
			use polkadot_xcm::prelude::XcmVersion;
			use polkadot_xcm::latest::prelude::{Location, AssetId, Fungible, Asset, Assets, ParentThen, Parachain, Parent};

			impl pallet_xcm::benchmarking::Config for Runtime {
				type DeliveryHelper = ();

				fn reachable_dest() -> Option<Location> {
					Some(Parent.into())
				}

				fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
					None
				}

				fn reserve_transferable_asset_and_dest() -> Option<(Asset, Location)> {
					ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(
								RandomParaId::get()
							);

					Some((
						Asset {
							fun: Fungible(ExistentialDeposit::get()),
							id: AssetId(AssetLocation::get())
						},
						ParentThen(Parachain(RandomParaId::get().into()).into()).into(),
					))
				}

				fn set_up_complex_asset_transfer() -> Option<(Assets, u32, Location, Box<dyn FnOnce()>)> {
					ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(
								RandomParaId::get()
							);

					let destination = ParentThen(Parachain(RandomParaId::get().into()).into()).into();

					let fee_asset: Asset = (
						   AssetLocation::get(),
						   ExistentialDeposit::get(),
					 ).into();

					let who = frame_benchmarking::whitelisted_caller();
					let balance = 10 * ExistentialDeposit::get();
					let _ = <Balances as frame_support::traits::Currency<_>>::make_free_balance_be(&who, balance );

					assert_eq!(Balances::free_balance(&who), balance);

					let transfer_asset: Asset = (
						   AssetLocation::get(),
						   ExistentialDeposit::get(),
					 ).into();

					let assets: Assets = vec![fee_asset.clone(), transfer_asset].into();

					let fee_index: u32 = 0;

					let verify: Box<dyn FnOnce()> = Box::new(move || {
						assert!(Balances::free_balance(&who) <= balance - ExistentialDeposit::get());
					});

					Some((assets, fee_index, destination, verify))
			   }

				fn get_asset() -> Asset {
					Asset {
						id: AssetId(Location::here()),
						fun: Fungible(ExistentialDeposit::get()),
					}
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
}
