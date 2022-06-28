#![cfg(test)]
use crate::kusama_test_net::*;

use frame_support::{assert_noop, assert_ok};

use polkadot_xcm::latest::prelude::*;

use cumulus_primitives_core::ParaId;
use orml_traits::currency::MultiCurrency;
use sp_runtime::traits::AccountIdConversion;
use xcm_emulator::TestExt;

#[test]
fn transfer_from_relay_chain() {
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			KSM_ID,
			basilisk_runtime::AssetLocation(MultiLocation::parent())
		));
	});

	let transfer_amount = 3 * KSM;
	KusamaRelay::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::Origin::signed(ALICE.into()),
			Box::new(Parachain(2000).into().into()),
			Box::new(
				Junction::AccountId32 {
					id: BOB,
					network: NetworkId::Any
				}
				.into()
				.into()
			),
			Box::new((Here, transfer_amount).into()),
			0,
		));

		assert_eq!(
			kusama_runtime::Balances::free_balance(&ParaId::from(2000).into_account()),
			10 * KSM + transfer_amount
		);
	});

	Basilisk::execute_with(|| {
		let fees = 2 * KSM / 10;
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(KSM_ID, &AccountId::from(BOB)),
			1000 * KSM + transfer_amount - fees
		);
		// fees should go to treasury
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			fees
		);
	});
}

#[test]
fn transfer_to_relay_chain() {
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			KSM_ID,
			basilisk_runtime::AssetLocation(MultiLocation::parent())
		));

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			KSM_ID,
			3 * KSM,
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
			basilisk_runtime::Tokens::free_balance(KSM_ID, &AccountId::from(ALICE)),
			200 * KSM - 3 * KSM
		);
	});

	KusamaRelay::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
			2999893333340 // 3 * KSM - fee
		);
	});
}

#[test]
fn transfer_from_hydra() {
	TestNet::reset();

	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			SNEK_ID,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(SNEK_ID.into()))))
		));
	});

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			SNEK_ID,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(SNEK_ID.into()))))
		));
	});

	let transfer_amount = 3 * SNEK;
	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			SNEK_ID,
			transfer_amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(2000),
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
			basilisk_runtime::Tokens::free_balance(SNEK_ID, &AccountId::from(ALICE)),
			200 * SNEK - transfer_amount
		);
	});

	let fees = 2 * SNEK / 10;
	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(SNEK_ID, &AccountId::from(BOB)),
			1000 * SNEK + transfer_amount - fees
		);
		// fees should go to treasury
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(SNEK_ID, &basilisk_runtime::Treasury::account_id()),
			fees
		);
	});
}

#[test]
fn transfer_insufficient_amount_should_fail() {
	TestNet::reset();

	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			SNEK_ID,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(SNEK_ID.into()))))
		));
	});

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			SNEK_ID,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(SNEK_ID.into()))))
		));
	});

	Hydra::execute_with(|| {
		assert_noop!(
			basilisk_runtime::XTokens::transfer(
				basilisk_runtime::Origin::signed(ALICE.into()),
				SNEK_ID,
				1_000_000 - 1,
				Box::new(
					MultiLocation::new(
						1,
						X2(
							Junction::Parachain(2000),
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
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			200 * SNEK
		);
	});

	Basilisk::execute_with(|| {
		// Xcm should fail therefore nothing should be deposit into beneficiary account
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(SNEK_ID, &AccountId::from(BOB)),
			1000 * SNEK
		);
	});
}

#[test]
fn fee_currency_set_on_xcm_transfer() {
	TestNet::reset();

	const HITCHHIKER: [u8; 32] = [42u8; 32];

	let transfer_amount = 100 * SNEK;

	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			SNEK_ID,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(SNEK_ID.into()))))
		));
	});

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			SNEK_ID,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(SNEK_ID.into()))))
		));

		// fee currency is not set before XCM transfer
		assert_eq!(
			basilisk_runtime::MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			None
		);
	});

	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			SNEK_ID,
			transfer_amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(2000),
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
			basilisk_runtime::Tokens::free_balance(SNEK_ID, &AccountId::from(ALICE)),
			200 * SNEK - transfer_amount
		);
	});

	Basilisk::execute_with(|| {
		let fee_amount = 2 * SNEK / 10;
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(SNEK_ID, &AccountId::from(HITCHHIKER)),
			transfer_amount - fee_amount
		);
		// fees should go to treasury
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(SNEK_ID, &basilisk_runtime::Treasury::account_id()),
			fee_amount
		);
		// fee currency is set after XCM transfer
		assert_eq!(
			basilisk_runtime::MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			Some(SNEK_ID)
		);
	});
}

#[test]
fn transfer_of_native_should_fail() {
	env_logger::init();
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_noop!(
			basilisk_runtime::PolkadotXcm::limited_reserve_transfer_assets(
				basilisk_runtime::Origin::signed(ALICE.into()),
				Box::new(MultiLocation::new(1, X1(Junction::Parachain(3000))).into()), // dest
				Box::new(
					MultiLocation::new(
						0,
						X1(Junction::AccountId32 {
							id: BOB,
							network: NetworkId::Any,
						})
					)
					.into()
				), // beneficiary
				Box::new((basilisk_runtime::xcm::NativeLocation::get(), 3 * BSX).into()), // assets
				0,                                                                     // fee_item
				WeightLimit::Limited(399_600_000_000),
			),
			pallet_xcm::Error::<basilisk_runtime::Runtime>::Filtered
		);

		assert_noop!(
			basilisk_runtime::XTokens::transfer(
				basilisk_runtime::Origin::signed(ALICE.into()),
				basilisk_runtime::CORE_ASSET_ID, // currency
				3 * BSX,                         // amount
				Box::new(
					MultiLocation::new(
						1,
						X2(
							Junction::Parachain(3000),
							Junction::AccountId32 {
								id: BOB,
								network: NetworkId::Any,
							}
						)
					)
					.into()
				), // dest
				399_600_000_000                  // weight
			),
			orml_xtokens::Error::<basilisk_runtime::Runtime>::NotCrossChainTransferableCurrency
		);
	});
}
