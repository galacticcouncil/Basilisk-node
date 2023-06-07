#![cfg(test)]
use crate::kusama_test_net::*;

use frame_support::{assert_ok, weights::Weight};
use sp_runtime::codec::Encode;

use polkadot_xcm::latest::prelude::*;
use xcm_emulator::TestExt;

#[test]
fn allowed_transact_call_should_pass_filter() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::Balances::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			1_000 * UNITS,
		));
	});

	OtherParachain::execute_with(|| {
		// allowed by SafeCallFilter and the runtime call filter
		let call = pallet_balances::Call::<basilisk_runtime::Runtime>::transfer {
			dest: BOB.into(),
			value: UNITS,
		};
		let message = Xcm(vec![
			WithdrawAsset(
				(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(BASILISK_PARA_ID), GeneralIndex(0)),
					},
					900 * UNITS,
				)
					.into(),
			),
			BuyExecution {
				fees: (
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(BASILISK_PARA_ID), GeneralIndex(0)),
					},
					800 * UNITS,
				)
					.into(),
				weight_limit: Unlimited,
			},
			Transact {
				require_weight_at_most: Weight::from_parts(10_000_000_000, 0u64),
				origin_kind: OriginKind::SovereignAccount,
				call: basilisk_runtime::RuntimeCall::Balances(call).encode().into(),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				beneficiary: Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		// Act
		assert_ok!(parachain_runtime_mock::PolkadotXcm::send_xcm(
			Here,
			MultiLocation::new(1, X1(Parachain(BASILISK_PARA_ID))),
			message
		));
	});

	Basilisk::execute_with(|| {
		// Assert
		assert!(basilisk_runtime::System::events().iter().any(|r| matches!(
			r.event,
			basilisk_runtime::RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. })
		)));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(BOB)),
			BOB_INITIAL_BSX_BALANCE + UNITS
		);
	});
}

#[test]
fn blocked_transact_calls_should_not_pass_filter() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::Balances::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			1_000 * UNITS,
		));
	});

	OtherParachain::execute_with(|| {
		// filtered by SafeCallFilter
		let call = pallet_tips::Call::<basilisk_runtime::Runtime>::report_awesome {
			reason: vec![0, 10],
			who: BOB.into(),
		};
		let message = Xcm(vec![
			WithdrawAsset(
				(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(BASILISK_PARA_ID), GeneralIndex(0)),
					},
					900 * UNITS,
				)
					.into(),
			),
			BuyExecution {
				fees: (
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(BASILISK_PARA_ID), GeneralIndex(0)),
					},
					800 * UNITS,
				)
					.into(),
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
				beneficiary: Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		// Act
		assert_ok!(parachain_runtime_mock::PolkadotXcm::send_xcm(
			Here,
			MultiLocation::new(1, X1(Parachain(BASILISK_PARA_ID))),
			message
		));
	});

	Basilisk::execute_with(|| {
		// Assert
		assert!(basilisk_runtime::System::events().iter().any(|r| matches!(
			r.event,
			basilisk_runtime::RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
				error: cumulus_primitives_core::XcmError::NoPermission,
				..
			})
		)));
	});
}

#[test]
fn safe_call_filter_should_respect_runtime_call_filter() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::Balances::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			1_000 * UNITS,
		));
	});

	OtherParachain::execute_with(|| {
		// filtered by the runtime call filter
		let call = pallet_uniques::Call::<basilisk_runtime::Runtime>::create {
			collection: 1u128,
			admin: ALICE.into(),
		};
		let message = Xcm(vec![
			WithdrawAsset(
				(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(BASILISK_PARA_ID), GeneralIndex(0)),
					},
					900 * UNITS,
				)
					.into(),
			),
			BuyExecution {
				fees: (
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(BASILISK_PARA_ID), GeneralIndex(0)),
					},
					800 * UNITS,
				)
					.into(),
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
				beneficiary: Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		// Act
		assert_ok!(parachain_runtime_mock::PolkadotXcm::send_xcm(
			Here,
			MultiLocation::new(1, X1(Parachain(BASILISK_PARA_ID))),
			message
		));
	});

	Basilisk::execute_with(|| {
		// Assert
		assert!(basilisk_runtime::System::events().iter().any(|r| matches!(
			r.event,
			basilisk_runtime::RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
				error: cumulus_primitives_core::XcmError::NoPermission,
				..
			})
		)));
	});
}
