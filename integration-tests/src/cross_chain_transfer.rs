#![cfg(test)]
use crate::kusama_test_net::Rococo;
use crate::kusama_test_net::*;

use frame_support::{assert_noop, assert_ok};

use polkadot_xcm::{v4::prelude::*, VersionedAssets, VersionedXcm};

use cumulus_primitives_core::ParaId;
use orml_traits::currency::MultiCurrency;
use polkadot_xcm::opaque::v3::{
	Junction,
	Junctions::{X1, X2},
	MultiLocation,
};
use sp_core::H256;
use sp_runtime::traits::{AccountIdConversion, BlakeTwo256, Hash};
use xcm_emulator::TestExt;

use sp_std::sync::Arc;

// Determine the hash for assets expected to be have been trapped.
fn determine_hash(origin: &MultiLocation, assets: Vec<Asset>) -> H256 {
	let versioned = VersionedAssets::from(Assets::from(assets));
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
	Rococo::execute_with(|| {
		assert_ok!(rococo_runtime::XcmPallet::reserve_transfer_assets(
			rococo_runtime::RuntimeOrigin::signed(ALICE.into()),
			Box::new(Parachain(BASILISK_PARA_ID).into_versioned()),
			Box::new(Junction::AccountId32 { id: BOB, network: None }.into_versioned()),
			Box::new((Here, 300 * UNITS).into()),
			0,
		));

		assert_eq!(
			rococo_runtime::Balances::free_balance(AccountIdConversion::<AccountId>::into_account_truncating(
				&ParaId::from(BASILISK_PARA_ID)
			)),
			310 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1_299_999_987_268_519
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			12_731_481
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

		let dest = MultiLocation::new(1, X1(Junction::AccountId32 { id: BOB, network: None })).into_versioned();

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			1,
			3 * UNITS,
			Box::new(dest),
			WeightLimit::Limited(Weight::from_parts(4_600_000_000, 10_000))
		));
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(ALICE)),
			ALICE_INITIAL_AUSD_BALANCE - 3 * UNITS
		);
	});

	Rococo::execute_with(|| {
		assert_eq!(
			rococo_runtime::Balances::free_balance(AccountId::from(BOB)),
			2999989698923 // 3 * BSX - fee
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
			basilisk_runtime::AssetLocation(MultiLocation::new(
				1,
				X2(Junction::Parachain(OTHER_PARA_ID), Junction::GeneralIndex(0))
			))
		));
	});

	OtherParachain::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
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
				.into_versioned()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
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
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(
				1,
				X2(Junction::Parachain(BASILISK_PARA_ID), Junction::GeneralIndex(0))
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
				.into_versioned()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
			ALICE_INITIAL_BSX_BALANCE - amount_to_send
		);
	});

	OtherParachain::execute_with(|| {
		let fee = 10185185;
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			BOB_INITIAL_AUSD_BALANCE_ON_OTHER_PARACHAIN + amount_to_send - fee
		);

		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::TreasuryAccount::get()),
			fee
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
			basilisk_runtime::AssetLocation(MultiLocation::new(
				1,
				X2(Junction::Parachain(OTHER_PARA_ID), Junction::GeneralIndex(0))
			))
		));
	});

	OtherParachain::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
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
				.into_versioned()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
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
				.into_versioned()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(BOB)),
			1000 * UNITS - amount_to_send
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			10_185_185 // fees should go to treasury
		);
	});

	OtherParachain::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(ALICE)),
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
			basilisk_runtime::AssetLocation(MultiLocation::new(
				1,
				X2(Junction::Parachain(OTHER_PARA_ID), Junction::GeneralIndex(0))
			))
		));
	});

	OtherParachain::execute_with(|| {
		let insufficient_amount = 55;
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN
		);

		assert_noop!(
			basilisk_runtime::XTokens::transfer(
				basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
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
					.into_versioned()
				),
				WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
			),
			orml_xtokens::Error::<basilisk_runtime::Runtime>::XcmExecutionFailed
		);

		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
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
			basilisk_runtime::AssetLocation(MultiLocation::new(
				1,
				X2(Junction::Parachain(OTHER_PARA_ID), Junction::GeneralIndex(0))
			))
		));

		// fee currency is not set before XCM transfer
		assert_eq!(
			basilisk_runtime::MultiTransactionPayment::get_currency(AccountId::from(HITCHHIKER)),
			None
		);
	});

	OtherParachain::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
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
				.into_versioned()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
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
			basilisk_runtime::MultiTransactionPayment::get_currency(AccountId::from(HITCHHIKER)),
			Some(1)
		);
	});
}

#[test]
fn assets_should_be_trapped_when_assets_are_unknown() {
	TestNet::reset();

	OtherParachain::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
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
				.into_versioned()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - 30 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		assert_xcm_message_processing_failed();
		let origin = MultiLocation::new(1, X1(Junction::Parachain(OTHER_PARA_ID)));
		let asset: Asset = Asset {
			id: cumulus_primitives_core::AssetId(Location::new(
				1,
				cumulus_primitives_core::Junctions::X2(Arc::new(
					vec![
						cumulus_primitives_core::Junction::Parachain(OTHER_PARA_ID),
						cumulus_primitives_core::Junction::GeneralIndex(0),
					]
					.try_into()
					.unwrap(),
				)),
			)),
			fun: Fungible(30 * UNITS),
		};
		let hash = determine_hash(&origin, vec![asset.clone()]);
		assert_eq!(basilisk_runtime::PolkadotXcm::asset_trap(hash), 1);

		expect_basilisk_event(basilisk_runtime::RuntimeEvent::PolkadotXcm(
			pallet_xcm::Event::AssetsTrapped {
				hash,
				origin: origin.try_into().unwrap(),
				assets: vec![asset].into(),
			},
		))
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
			basilisk_runtime::AssetLocation(MultiLocation::new(
				1,
				X2(Junction::Parachain(OTHER_PARA_ID), Junction::GeneralIndex(0))
			))
		));
	});

	let bob_loc = Location::new(
		0,
		cumulus_primitives_core::Junctions::X1(Arc::new(
			vec![cumulus_primitives_core::Junction::AccountId32 { id: BOB, network: None }]
				.try_into()
				.unwrap(),
		)),
	);

	claim_asset(asset.clone(), bob_loc);

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1000 * UNITS + 29_999_992_361_112u128
		);

		let origin = MultiLocation::new(1, X1(Junction::Parachain(OTHER_PARA_ID)));
		let hash = determine_hash(&origin, vec![asset]);
		assert_eq!(basilisk_runtime::PolkadotXcm::asset_trap(hash), 0);
	});
}

fn trap_asset() -> Asset {
	OtherParachain::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN
		);
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
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
				.into_versioned()
			),
			WeightLimit::Limited(Weight::from_parts(399_600_000_000, 0))
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
			200 * UNITS - 30 * UNITS
		);
	});

	let asset: Asset = Asset {
		id: cumulus_primitives_core::AssetId(Location::new(
			1,
			cumulus_primitives_core::Junctions::X2(Arc::new(
				vec![
					cumulus_primitives_core::Junction::Parachain(OTHER_PARA_ID),
					cumulus_primitives_core::Junction::GeneralIndex(0),
				]
				.try_into()
				.unwrap(),
			)),
		)),
		fun: Fungible(30 * UNITS),
	};

	Basilisk::execute_with(|| {
		assert_xcm_message_processing_failed();
		let origin = MultiLocation::new(1, X1(Junction::Parachain(OTHER_PARA_ID)));
		let hash = determine_hash(&origin, vec![asset.clone()]);
		assert_eq!(basilisk_runtime::PolkadotXcm::asset_trap(hash), 1);
	});

	asset
}

fn claim_asset(asset: Asset, recipient: Location) {
	OtherParachain::execute_with(|| {
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
		assert_ok!(basilisk_runtime::PolkadotXcm::send(
			basilisk_runtime::RuntimeOrigin::root(),
			Box::new(MultiLocation::new(1, X1(Junction::Parachain(BASILISK_PARA_ID))).into_versioned()),
			Box::new(VersionedXcm::from(xcm_msg))
		));
	});
}

#[test]
fn polkadot_xcm_execute_extrinsic_should_not_be_allowed() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let xcm_msg = Xcm(vec![
			WithdrawAsset((Here, 410000000000u128).into()),
			BuyExecution {
				fees: (Here, 400000000000u128).into(),
				weight_limit: Unlimited,
			},
			ClearError,
		]);

		assert_noop!(
			basilisk_runtime::PolkadotXcm::execute(
				basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
				Box::new(VersionedXcm::from(xcm_msg)),
				Weight::from_parts(400_000_000_000, 0)
			),
			sp_runtime::DispatchErrorWithPostInfo {
				post_info: frame_support::dispatch::PostDispatchInfo {
					actual_weight: Some(Weight::from_parts(10613000, 0)),
					pays_fee: frame_support::dispatch::Pays::Yes,
				},
				error: pallet_xcm::Error::<basilisk_runtime::Runtime>::Filtered.into()
			}
		);
	});
}
