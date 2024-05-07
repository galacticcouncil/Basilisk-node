#![cfg(test)]
use crate::kusama_test_net::*;

use frame_support::{assert_ok, dispatch::GetDispatchInfo};
use sp_runtime::codec::Encode;

use polkadot_xcm::v4::prelude::*;

use xcm_emulator::TestExt;

use sp_std::sync::Arc;
#[test]
fn allowed_transact_call_should_pass_filter() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::Currencies::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			BSX,
			1_000 * UNITS,
		));
	});

	OtherParachain::execute_with(|| {
		// allowed by SafeCallFilter and the runtime call filter
		let call = pallet_currencies::Call::<basilisk_runtime::Runtime>::transfer {
			dest: BOB.into(),
			currency_id: 0,
			amount: UNITS,
		};
		let bsx_loc = Location::new(
			1,
			cumulus_primitives_core::Junctions::X2(Arc::new(
				vec![
					cumulus_primitives_core::Junction::Parachain(BASILISK_PARA_ID),
					cumulus_primitives_core::Junction::GeneralIndex(0),
				]
				.try_into()
				.unwrap(),
			)),
		);
		let asset_to_withdraw: Asset = Asset {
			id: cumulus_primitives_core::AssetId(bsx_loc.clone()),
			fun: Fungible(900 * UNITS),
		};

		let asset_for_buy_execution: Asset = Asset {
			id: cumulus_primitives_core::AssetId(bsx_loc),
			fun: Fungible(800 * UNITS),
		};

		let message = Xcm(vec![
			WithdrawAsset(asset_to_withdraw.into()),
			BuyExecution {
				fees: asset_for_buy_execution.into(),
				weight_limit: Unlimited,
			},
			Transact {
				require_weight_at_most: call.get_dispatch_info().weight,
				origin_kind: OriginKind::SovereignAccount,
				call: basilisk_runtime::RuntimeCall::Currencies(call).encode().into(),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				beneficiary: cumulus_primitives_core::Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		// Act
		let dest = Location::new(
			1,
			cumulus_primitives_core::Junctions::X1(Arc::new(
				vec![cumulus_primitives_core::Junction::Parachain(BASILISK_PARA_ID)]
					.try_into()
					.unwrap(),
			)),
		);

		assert_ok!(basilisk_runtime::PolkadotXcm::send_xcm(
			cumulus_primitives_core::Junctions::Here,
			dest,
			message
		));
	});

	// Assert
	Basilisk::execute_with(|| {
		assert_xcm_message_processing_passed();
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(BOB)),
			BOB_INITIAL_BSX_BALANCE + UNITS
		);
	});
}

#[test]
fn blocked_transact_calls_should_not_pass_filter() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::Currencies::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			BSX,
			1_000 * UNITS,
		));
	});

	OtherParachain::execute_with(|| {
		// filtered by SafeCallFilter
		let call = pallet_tips::Call::<basilisk_runtime::Runtime>::report_awesome {
			reason: vec![0, 10],
			who: BOB.into(),
		};
		let bsx_loc = Location::new(
			1,
			cumulus_primitives_core::Junctions::X2(Arc::new(
				vec![
					cumulus_primitives_core::Junction::Parachain(BASILISK_PARA_ID),
					cumulus_primitives_core::Junction::GeneralIndex(0),
				]
				.try_into()
				.unwrap(),
			)),
		);
		let asset_to_withdraw: Asset = Asset {
			id: cumulus_primitives_core::AssetId(bsx_loc.clone()),
			fun: Fungible(900 * UNITS),
		};

		let asset_for_buy_execution: Asset = Asset {
			id: cumulus_primitives_core::AssetId(bsx_loc),
			fun: Fungible(800 * UNITS),
		};
		let message = Xcm(vec![
			WithdrawAsset(asset_to_withdraw.into()),
			BuyExecution {
				fees: asset_for_buy_execution,
				weight_limit: Unlimited,
			},
			Transact {
				require_weight_at_most: Weight::from_parts(10_000_000_000, 0u64),
				origin_kind: OriginKind::Native,
				call: basilisk_runtime::RuntimeCall::Tips(call).encode().into(),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				beneficiary: cumulus_primitives_core::Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		let dest_basilisk = Location::new(
			1,
			cumulus_primitives_core::Junctions::X1(Arc::new(
				vec![cumulus_primitives_core::Junction::Parachain(BASILISK_PARA_ID)]
					.try_into()
					.unwrap(),
			)),
		);

		// Act
		assert_ok!(basilisk_runtime::PolkadotXcm::send_xcm(Here, dest_basilisk, message));
	});

	Basilisk::execute_with(|| {
		// Assert
		assert_xcm_message_processing_failed()
	});
}

#[test]
fn safe_call_filter_should_respect_runtime_call_filter() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::Currencies::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			BSX,
			1_000 * UNITS,
		));
	});

	OtherParachain::execute_with(|| {
		// filtered by the runtime call filter
		let call = pallet_uniques::Call::<basilisk_runtime::Runtime>::create {
			collection: 1u128,
			admin: ALICE.into(),
		};
		let bsx_loc = Location::new(
			1,
			cumulus_primitives_core::Junctions::X2(Arc::new(
				vec![
					cumulus_primitives_core::Junction::Parachain(BASILISK_PARA_ID),
					cumulus_primitives_core::Junction::GeneralIndex(0),
				]
				.try_into()
				.unwrap(),
			)),
		);
		let asset_to_withdraw: Asset = Asset {
			id: cumulus_primitives_core::AssetId(bsx_loc.clone()),
			fun: Fungible(900 * UNITS),
		};

		let asset_for_buy_execution: Asset = Asset {
			id: cumulus_primitives_core::AssetId(bsx_loc),
			fun: Fungible(800 * UNITS),
		};

		let message = Xcm(vec![
			WithdrawAsset(asset_to_withdraw.into()),
			BuyExecution {
				fees: asset_for_buy_execution,
				weight_limit: Unlimited,
			},
			Transact {
				require_weight_at_most: Weight::from_parts(1_000_000_000, 2653u64),
				origin_kind: OriginKind::Native,
				call: basilisk_runtime::RuntimeCall::Uniques(call).encode().into(),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				beneficiary: cumulus_primitives_core::Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		let dest_basilisk = Location::new(
			1,
			cumulus_primitives_core::Junctions::X1(Arc::new(
				vec![cumulus_primitives_core::Junction::Parachain(BASILISK_PARA_ID)]
					.try_into()
					.unwrap(),
			)),
		);

		// Act
		assert_ok!(basilisk_runtime::PolkadotXcm::send_xcm(Here, dest_basilisk, message));
	});

	//Assert
	Basilisk::execute_with(|| assert_xcm_message_processing_failed());
}
