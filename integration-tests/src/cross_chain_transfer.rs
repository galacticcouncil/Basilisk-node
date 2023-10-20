#![cfg(test)]
use crate::kusama_test_net::Kusama;
use crate::kusama_test_net::*;

use frame_support::{assert_noop, assert_ok};

use polkadot_xcm::{latest::prelude::*, VersionedMultiAssets, VersionedXcm};

use cumulus_primitives_core::ParaId;
use frame_support::weights::Weight;
use hex_literal::hex;
use orml_traits::currency::MultiCurrency;
use sp_core::H256;
use sp_runtime::traits::{AccountIdConversion, BlakeTwo256, Hash};
use xcm_emulator::TestExt;

// Determine the hash for assets expected to be have been trapped.
fn determine_hash<M>(origin: &MultiLocation, assets: M) -> H256
where
	M: Into<MultiAssets>,
{
	let versioned = VersionedMultiAssets::from(assets.into());
	BlakeTwo256::hash_of(&(origin, &versioned))
}

#[test]
fn basilisk_should_receive_asset_when_transferred_from_relaychain() {
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::parent())
		));
	});
	Kusama::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::RuntimeOrigin::signed(ALICE.into()),
			Box::new(Parachain(BASILISK_PARA_ID).into_versioned()),
			Box::new(Junction::AccountId32 { id: BOB, network: None }.into()),
			Box::new((Here, 300 * UNITS).into()),
			0,
		));

		assert_eq!(
			kusama_runtime::Balances::free_balance(&ParaId::from(BASILISK_PARA_ID).into_account_truncating()),
			310 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1_299_999_989_814_815
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			10_185_185
		);
	});
}

#[test]
fn relaychain_should_receive_asset_when_transferred_from_basilisk() {
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::parent())
		));

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			1,
			3 * UNITS,
			Box::new(MultiLocation::new(1, X1(Junction::AccountId32 { id: BOB, network: None })).into()),
			WeightLimit::Limited(Weight::from_parts(4_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(ALICE)),
			ALICE_INITIAL_AUSD_BALANCE - 3 * UNITS
		);
	});

	Kusama::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
			2999909712564 // 3 * BSX - fee
		);
	});
}

#[test]
fn basilisk_should_receive_asset_when_sent_from_other_parachain() {
	TestNet::reset();

	let amount_to_send = 30 * UNITS;

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))))
		));
	});

	OtherParachain::execute_with(|| {
		assert_ok!(parachain_runtime_mock::XTokens::transfer(
			parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
			0,
			amount_to_send,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 { id: BOB, network: None }
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - amount_to_send
		);
	});

	let fee = 10_185_185;
	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1000 * UNITS + amount_to_send - fee
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			fee // fees should go to treasury
		);
	});
}

#[test]
fn other_parachain_should_receive_asset_when_sent_from_basilisk() {
	TestNet::reset();

	let amount_to_send = 30 * UNITS;

	OtherParachain::execute_with(|| {
		assert_ok!(parachain_runtime_mock::AssetRegistry::set_location(
			parachain_runtime_mock::RuntimeOrigin::root(),
			1,
			parachain_runtime_mock::AssetLocation(MultiLocation::new(
				1,
				X2(Parachain(BASILISK_PARA_ID), GeneralIndex(0))
			))
		));
	});

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::MultiTransactionPayment::set_currency(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			1,
		));

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			0,
			amount_to_send,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(OTHER_PARA_ID),
						Junction::AccountId32 { id: BOB, network: None }
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_BSX_BALANCE - amount_to_send
		);
	});

	OtherParachain::execute_with(|| {
		let fee = 10175000000;
		assert_eq!(
			parachain_runtime_mock::Tokens::free_balance(1, &AccountId::from(BOB)),
			BOB_INITIAL_AUSD_BALANCE_ON_OTHER_PARACHAIN + amount_to_send - fee
		);

		assert_eq!(
			parachain_runtime_mock::Tokens::free_balance(1, &parachain_runtime_mock::ParachainTreasuryAccount::get()),
			10175000000
		);
	});
}

#[test]
fn transfer_from_other_parachain_and_back() {
	TestNet::reset();

	let amount_to_send = 30 * UNITS;

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))))
		));
	});

	OtherParachain::execute_with(|| {
		assert_ok!(parachain_runtime_mock::XTokens::transfer(
			parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
			0,
			amount_to_send,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 { id: BOB, network: None }
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - amount_to_send
		);
	});

	let fee = 10_185_185;
	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1000 * UNITS + amount_to_send - fee
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			fee // fees should go to treasury
		);

		//transfer back
		assert_ok!(basilisk_runtime::MultiTransactionPayment::set_currency(
			basilisk_runtime::RuntimeOrigin::signed(BOB.into()),
			1,
		));

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(BOB.into()),
			0,
			amount_to_send,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(OTHER_PARA_ID),
						Junction::AccountId32 {
							id: ALICE,
							network: None,
						}
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(BOB)),
			1000 * UNITS - amount_to_send
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			10_185_185 // fees should go to treasury
		);
	});

	OtherParachain::execute_with(|| {
		assert_eq!(
			parachain_runtime_mock::Tokens::free_balance(1, &AccountId::from(ALICE)),
			ALICE_INITIAL_AUSD_BALANCE_ON_OTHER_PARACHAIN
		);
	});
}

#[test]
fn other_parachain_should_fail_to_send_asset_to_basilisk_when_insufficient_amount_is_used() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))))
		));
	});

	OtherParachain::execute_with(|| {
		let insufficient_amount = 55;
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN
		);

		assert_noop!(
			parachain_runtime_mock::XTokens::transfer(
				parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
				0,
				insufficient_amount,
				Box::new(
					MultiLocation::new(
						1,
						X2(
							Junction::Parachain(BASILISK_PARA_ID),
							Junction::AccountId32 { id: BOB, network: None }
						)
					)
					.into()
				),
				WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
			),
			orml_xtokens::Error::<basilisk_runtime::Runtime>::XcmExecutionFailed
		);

		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN
		);
	});

	Basilisk::execute_with(|| {
		// Xcm should fail therefore nothing should be deposit into beneficiary account
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1000 * UNITS
		);
	});
}

#[test]
fn fee_currency_set_on_xcm_transfer() {
	TestNet::reset();

	const HITCHHIKER: [u8; 32] = [42u8; 32];

	let transfer_amount = 100 * UNITS;

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))))
		));

		// fee currency is not set before XCM transfer
		assert_eq!(
			basilisk_runtime::MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			None
		);
	});

	OtherParachain::execute_with(|| {
		assert_ok!(parachain_runtime_mock::XTokens::transfer(
			parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
			0,
			transfer_amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 {
							id: HITCHHIKER,
							network: None,
						}
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - transfer_amount
		);
	});

	Basilisk::execute_with(|| {
		let fee_amount = 10_185_185;
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(HITCHHIKER)),
			transfer_amount - fee_amount
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			fee_amount // fees should go to treasury
		);
		// fee currency is set after XCM transfer
		assert_eq!(
			basilisk_runtime::MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			Some(1)
		);
	});
}

#[test]
fn assets_should_be_trapped_when_assets_are_unknown() {
	TestNet::reset();

	OtherParachain::execute_with(|| {
		assert_ok!(parachain_runtime_mock::XTokens::transfer(
			parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
			0,
			30 * UNITS,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 { id: BOB, network: None }
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - 30 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		expect_basilisk_events(vec![
			cumulus_pallet_xcmp_queue::Event::Fail {
				message_hash: hex!["30291d1dfb68ae6f66d4c841facb78f44e7611ab2a25c84f4fb7347f448d2944"],
				message_id: hex!["30291d1dfb68ae6f66d4c841facb78f44e7611ab2a25c84f4fb7347f448d2944"],
				error: XcmError::AssetNotFound,
				weight: Weight::from_parts(300_000_000, 0),
			}
			.into(),
			pallet_relaychain_info::Event::CurrentBlockNumbers {
				parachain_block_number: 1,
				relaychain_block_number: 4,
			}
			.into(),
		]);
		let origin = MultiLocation::new(1, X1(Parachain(OTHER_PARA_ID)));
		let loc = MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0)));
		let asset: MultiAsset = (loc, 30 * UNITS).into();
		let hash = determine_hash(&origin, vec![asset]);
		assert_eq!(basilisk_runtime::PolkadotXcm::asset_trap(hash), 1);
	});
}

#[test]
fn claim_trapped_asset_should_work() {
	TestNet::reset();

	// traps asset when asset is not registered yet
	let asset = trap_asset();

	// register the asset
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))))
		));
	});

	claim_asset(asset.clone(), BOB);

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1000 * UNITS + 29_999_992_361_112u128
		);

		let origin = MultiLocation::new(1, X1(Parachain(OTHER_PARA_ID)));
		let hash = determine_hash(&origin, vec![asset]);
		assert_eq!(basilisk_runtime::PolkadotXcm::asset_trap(hash), 0);
	});
}

fn trap_asset() -> MultiAsset {
	OtherParachain::execute_with(|| {
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN
		);
		assert_ok!(parachain_runtime_mock::XTokens::transfer(
			parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
			0,
			30 * UNITS,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 { id: BOB, network: None }
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			200 * UNITS - 30 * UNITS
		);
	});

	let loc = MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0)));
	let asset: MultiAsset = (loc, 30 * UNITS).into();

	Basilisk::execute_with(|| {
		expect_basilisk_events(vec![
			cumulus_pallet_xcmp_queue::Event::Fail {
				message_hash: hex!["30291d1dfb68ae6f66d4c841facb78f44e7611ab2a25c84f4fb7347f448d2944"],
				message_id: hex!["30291d1dfb68ae6f66d4c841facb78f44e7611ab2a25c84f4fb7347f448d2944"],
				error: XcmError::AssetNotFound,
				weight: Weight::from_parts(300_000_000, 0),
			}
			.into(),
			pallet_relaychain_info::Event::CurrentBlockNumbers {
				parachain_block_number: 1,
				relaychain_block_number: 4,
			}
			.into(),
		]);
		let origin = MultiLocation::new(1, X1(Parachain(OTHER_PARA_ID)));
		let loc = MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0)));
		let asset: MultiAsset = (loc, 30 * UNITS).into();
		let hash = determine_hash(&origin, vec![asset]);
		assert_eq!(basilisk_runtime::PolkadotXcm::asset_trap(hash), 1);
	});

	asset
}

fn claim_asset(asset: MultiAsset, recipient: [u8; 32]) {
	OtherParachain::execute_with(|| {
		let recipient = MultiLocation::new(
			0,
			X1(Junction::AccountId32 {
				network: None,
				id: recipient,
			}),
		);
		let xcm_msg = Xcm(vec![
			ClaimAsset {
				assets: vec![asset.clone()].into(),
				ticket: Here.into(),
			},
			BuyExecution {
				fees: asset,
				weight_limit: Unlimited,
			},
			DepositAsset {
				assets: All.into(),
				beneficiary: recipient,
			},
		]);
		assert_ok!(parachain_runtime_mock::PolkadotXcm::send(
			parachain_runtime_mock::RuntimeOrigin::root(),
			Box::new(MultiLocation::new(1, X1(Parachain(BASILISK_PARA_ID))).into()),
			Box::new(VersionedXcm::from(xcm_msg))
		));
	});
}

#[test]
fn polkadot_xcm_execute_extrinsic_should_not_be_allowed() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let message = VersionedXcm::V3(Xcm(vec![
			WithdrawAsset((Here, 410000000000u128).into()),
			BuyExecution {
				fees: (Here, 400000000000u128).into(),
				weight_limit: Unlimited,
			},
		]));

		assert_noop!(
			basilisk_runtime::PolkadotXcm::execute(
				basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
				Box::new(message),
				Weight::from_parts(400_000_000_000, 0)
			),
			pallet_xcm::Error::<basilisk_runtime::Runtime>::Filtered
		);
	});
}
