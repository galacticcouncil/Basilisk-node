#![cfg(test)]
use crate::kusama_test_net::*;
use polkadot_xcm::latest::prelude::*;

use frame_support::{assert_noop, assert_ok};
use cumulus_primitives_core::ParaId;
use orml_traits::currency::MultiCurrency;
use sp_runtime::traits::AccountIdConversion;
use xcm_emulator::TestExt;

#[test]
fn transfer_from_relay_chain() {
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
			Box::new(Parachain(2000).into().into()),
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
			kusama_runtime::Balances::free_balance(&ParaId::from(2000).into_account()),
			310 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			12780 * UNITS / 10
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			22 * UNITS
		);
	});
}

#[test]
fn transfer_to_relay_chain() {
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
fn transfer_from_hydra() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(0))))
		));
	});

	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			0,
			30 * UNITS,
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
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			200 * UNITS - 30 * UNITS
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			10080 * UNITS / 10
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &basilisk_runtime::Treasury::account_id()),
			22 * UNITS // fees should go to treasury
		);
	});
}

#[test]
fn transfer_native_pays_fee() {
	TestNet::reset();
	let xcm_fee = 200_000_000_000;

	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(2000), GeneralIndex(0))))
		));
	});

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			0,
			10 * BSX,
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
			),
			399_600_000_000
		));

		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			190 * BSX
		);
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&hydra_sovereign_account()),
			10 * BSX
		);
	});

	Hydra::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			10 * BSX - xcm_fee
		);

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(BOB.into()),
			1,
			5 * BSX,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(2000),
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
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			5 * BSX - xcm_fee
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&hydra_sovereign_account()),
			5 * BSX
		);
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&basilisk_runtime::Treasury::account_id()),
			xcm_fee
		);
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			195 * BSX - xcm_fee
		);
	})
}

#[test]
fn transfer_insufficient_amount_should_fail() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(0))))
		));
	});

	Hydra::execute_with(|| {
		assert_noop!(
			basilisk_runtime::XTokens::transfer(
				basilisk_runtime::Origin::signed(ALICE.into()),
				0,
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
			200000000000000
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
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralIndex(0))))
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
			0,
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
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			200 * UNITS - transfer_amount
		);
	});

	Basilisk::execute_with(|| {
		let fee_amount = 22 * UNITS;
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
