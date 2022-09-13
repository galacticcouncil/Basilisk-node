#![cfg(test)]
use crate::kusama_test_net::*;

use frame_support::{assert_noop, assert_ok};

use polkadot_xcm::{latest::prelude::*, VersionedMultiAssets};

use cumulus_primitives_core::ParaId;
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
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::parent())
		));
	});
	KusamaRelay::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::Origin::signed(ALICE.into()),
			Box::new(Parachain(BASILISK_PARA_ID).into().into()),
			Box::new(
				Junction::AccountId32 {
					id: BOB,
					network: NetworkId::Any
				}
				.into()
				.into()
			),
			Box::new((Here, 300 * UNITS).into()),
			0,
		));

		assert_eq!(
			kusama_runtime::Balances::free_balance(&ParaId::from(BASILISK_PARA_ID).into_account()),
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
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::parent())
		));

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			1,
			3 * UNITS,
			Box::new(
				MultiLocation::new(
					1,
					X1(Junction::AccountId32 {
						id: BOB,
						network: NetworkId::Any,
					})
				)
				.into()
			),
			4_600_000_000
		));
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(ALICE)),
			200 * UNITS - 3 * UNITS
		);
	});

	KusamaRelay::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
			2999893333340 // 3 * BSX - fee
		);
	});
}

#[test]
fn basilisk_should_receive_asset_when_sent_from_karura() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(KARURA_PARA_ID), GeneralIndex(0))))
		));
	});

	Karura::execute_with(|| {
		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			0,
			30 * UNITS,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 {
							id: BOB,
							network: NetworkId::Any,
						}
					)
				)
				.into()
			),
			399_600_000_000
		));
		assert_eq!(
			Balances::free_balance(&AccountId::from(ALICE)),
			200 * UNITS - 30 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1_029_999_989_814_815
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			10_185_185 // fees should go to treasury
		);
	});
}

#[test]
fn karura_should_fail_to_send_asset_to_basilisk_when_insufficient_amount_is_used() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(KARURA_PARA_ID), GeneralIndex(0))))
		));
	});

	Karura::execute_with(|| {
		let insufficient_amount = 55;
		assert_eq!(Balances::free_balance(&AccountId::from(ALICE)), 200000000000000);

		assert_noop!(
			XTokens::transfer(
				Origin::signed(ALICE.into()),
				0,
				insufficient_amount,
				Box::new(
					MultiLocation::new(
						1,
						X2(
							Junction::Parachain(BASILISK_PARA_ID),
							Junction::AccountId32 {
								id: BOB,
								network: NetworkId::Any,
							}
						)
					)
					.into()
				),
				399_600_000_000
			),
			orml_xtokens::Error::<basilisk_runtime::Runtime>::XcmExecutionFailed
		);

		assert_eq!(Balances::free_balance(&AccountId::from(ALICE)), 200000000000000);
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
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(KARURA_PARA_ID), GeneralIndex(0))))
		));

		// fee currency is not set before XCM transfer
		assert_eq!(
			basilisk_runtime::MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			None
		);
	});

	Karura::execute_with(|| {
		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			0,
			transfer_amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 {
							id: HITCHHIKER,
							network: NetworkId::Any,
						}
					)
				)
				.into()
			),
			399_600_000_000
		));
		assert_eq!(
			Balances::free_balance(&AccountId::from(ALICE)),
			200 * UNITS - transfer_amount
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

	Karura::execute_with(|| {
		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			0,
			30 * UNITS,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 {
							id: BOB,
							network: NetworkId::Any,
						}
					)
				)
				.into()
			),
			399_600_000_000
		));
		assert_eq!(
			Balances::free_balance(&AccountId::from(ALICE)),
			200 * UNITS - 30 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		expect_basilisk_events(vec![
			cumulus_pallet_xcmp_queue::Event::Fail(
				Some(hex!["4efbf4d7ba73f43d5bb4ebbec3189e132ccf2686aed37e97985af019e1cf62dc"].into()),
				XcmError::AssetNotFound,
			)
			.into(),
			pallet_relaychain_info::Event::CurrentBlockNumbers {
				parachain_block_number: 1,
				relaychain_block_number: 1,
			}
			.into(),
		]);
		let origin = MultiLocation::new(1, X1(Parachain(KARURA_PARA_ID)));
		let loc = MultiLocation::new(1, X2(Parachain(KARURA_PARA_ID), GeneralIndex(0)));
		let asset: MultiAsset = (loc, 30 * UNITS).into();
		let hash = determine_hash(&origin, vec![asset]);
		assert_eq!(basilisk_runtime::PolkadotXcm::asset_trap(hash), 1);
	});
}

#[test]
fn transfer_from_karura_and_back() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(KARURA_PARA_ID), GeneralIndex(0))))
		));
	});

	Karura::execute_with(|| {
		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			0,
			30 * UNITS,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 {
							id: BOB,
							network: NetworkId::Any,
						}
					)
				)
				.into()
			),
			399_600_000_000
		));
		assert_eq!(
			Balances::free_balance(&AccountId::from(ALICE)),
			200 * UNITS - 30 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			1_029_999_989_814_815
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			10_185_185 // fees should go to treasury
		);

		//transfer back
		assert_ok!(basilisk_runtime::MultiTransactionPayment::set_currency(
			basilisk_runtime::Origin::signed(BOB.into()),
			1,
		));

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(BOB.into()),
			0,
			30 * UNITS,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(KARURA_PARA_ID),
						Junction::AccountId32 {
							id: ALICE,
							network: NetworkId::Any,
						}
					)
				)
				.into()
			),
			399_600_000_000
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(BOB)),
			1000 * UNITS - 30 * UNITS
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			35_990_141 // fees should go to treasury
		);
	});

	Karura::execute_with(|| {
		assert_eq!(Balances::free_balance(&AccountId::from(ALICE)), 200 * UNITS);
	});
}

#[test]
fn karura_should_receive_asset_when_sent_from_basilisk() {
	TestNet::reset();

	Karura::execute_with(|| {
		assert_ok!(AssetRegistry::set_location(
			Origin::root(),
			1,
			AssetLocation(MultiLocation::new(1, X2(Parachain(BASILISK_PARA_ID), GeneralIndex(0))))
		));
	});

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			0,
			30 * UNITS,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(KARURA_PARA_ID),
						Junction::AccountId32 {
							id: BOB,
							network: NetworkId::Any,
						}
					)
				)
				.into()
			),
			399_600_000_000
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			200 * UNITS - 30 * UNITS
		);
	});

	Karura::execute_with(|| {
		assert_eq!(
			Tokens::free_balance(1, &AccountId::from(BOB)),
			1000 * UNITS + 30 * UNITS
		);
	});
}
