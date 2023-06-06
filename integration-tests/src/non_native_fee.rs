#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{Balances, Currencies, MultiTransactionPayment, RuntimeOrigin, Tokens};

use frame_support::{
	assert_err, assert_ok,
	dispatch::{DispatchInfo, Weight},
	sp_runtime::{
		traits::SignedExtension,
		transaction_validity::{InvalidTransaction, TransactionValidityError},
	},
	traits::{OnFinalize, OnInitialize},
};
use hydradx_traits::{pools::SpotPriceProvider, NativePriceOracle, AMM};
use orml_traits::currency::MultiCurrency;
use pallet_asset_registry::AssetType;
use pallet_transaction_multi_payment::Price;
use pallet_xyk::XYKSpotPrice;
use polkadot_primitives::v2::BlockNumber;
use primitives::{asset::AssetPair, AssetId};
use xcm_emulator::TestExt;

pub fn basilisk_run_to_block(to: BlockNumber) {
	while basilisk_runtime::System::block_number() < to {
		let b = basilisk_runtime::System::block_number();

		basilisk_runtime::System::on_finalize(b);
		basilisk_runtime::MultiTransactionPayment::on_finalize(b);

		basilisk_runtime::System::on_initialize(b + 1);
		basilisk_runtime::MultiTransactionPayment::on_initialize(b + 1);

		basilisk_runtime::System::set_block_number(b + 1);
	}
}

#[test]
fn non_native_fee_payment_works_with_configured_price() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let call = basilisk_runtime::RuntimeCall::MultiTransactionPayment(
			pallet_transaction_multi_payment::Call::set_currency { currency: AUSD },
		);

		let info = DispatchInfo {
			weight: Weight::from_ref_time(106_957_000),
			..Default::default()
		};
		let len: usize = 10;

		assert_ok!(
			pallet_transaction_payment::ChargeTransactionPayment::<basilisk_runtime::Runtime>::from(0).pre_dispatch(
				&AccountId::from(BOB),
				&call,
				&info,
				len,
			)
		);

		let bob_balance = basilisk_runtime::Tokens::free_balance(AUSD, &AccountId::from(BOB));
		assert_eq!(bob_balance, 999_999_992_137_669);
	});
}

#[test]
fn non_native_fee_payment_works_with_xyk_spot_price() {
	TestNet::reset();

	const NEW_TOKEN: AssetId = 42;

	let call =
		basilisk_runtime::RuntimeCall::MultiTransactionPayment(pallet_transaction_multi_payment::Call::set_currency {
			currency: NEW_TOKEN,
		});
	let info = DispatchInfo {
		weight: Weight::from_ref_time(106_957_000),
		..Default::default()
	};
	let len: usize = 10;

	Basilisk::execute_with(|| {
		// register the new token
		assert_ok!(basilisk_runtime::AssetRegistry::register(
			basilisk_runtime::RuntimeOrigin::root(),
			b"NEW_TOKEN".to_vec(),
			AssetType::Token,
			1_000,
			Some(NEW_TOKEN),
			None,
			None,
			None,
		));

		assert_ok!(basilisk_runtime::Balances::set_balance(
			basilisk_runtime::RuntimeOrigin::root(),
			ALICE.into(),
			2_000_000_000_000 * UNITS,
			0,
		));

		assert_ok!(basilisk_runtime::Tokens::set_balance(
			basilisk_runtime::RuntimeOrigin::root(),
			ALICE.into(),
			NEW_TOKEN,
			2_000_000_000_000 * UNITS,
			0,
		));

		assert_ok!(basilisk_runtime::Tokens::set_balance(
			basilisk_runtime::RuntimeOrigin::root(),
			DAVE.into(),
			NEW_TOKEN,
			1000 * UNITS,
			0,
		));

		assert_ok!(basilisk_runtime::MultiTransactionPayment::add_currency(
			basilisk_runtime::RuntimeOrigin::root(),
			NEW_TOKEN,
			Price::from(1_000_000_000_000), // set a ridiculously high price
		));

		// try and fail to pay with the new token
		assert_err!(
			pallet_transaction_payment::ChargeTransactionPayment::<basilisk_runtime::Runtime>::from(0).pre_dispatch(
				&AccountId::from(DAVE),
				&call,
				&info,
				len,
			),
			TransactionValidityError::from(InvalidTransaction::Payment)
		);

		assert_ok!(basilisk_runtime::XYK::create_pool(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			BSX,
			1_000 * UNITS,
			NEW_TOKEN,
			500 * UNITS,
		));

		assert!(basilisk_runtime::XYK::exists(AssetPair {
			asset_in: BSX,
			asset_out: NEW_TOKEN,
		}));

		let spot_price = XYKSpotPrice::<basilisk_runtime::Runtime>::spot_price(NEW_TOKEN, BSX);
		assert_eq!(spot_price, Some(Price::from_float(0.5)));

		basilisk_run_to_block(2);

		let pallet_price = basilisk_runtime::MultiTransactionPayment::price(NEW_TOKEN);
		assert_eq!(spot_price, pallet_price);

		assert_ok!(basilisk_runtime::XYK::buy(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			BSX,
			NEW_TOKEN,
			66 * UNITS,
			1_000 * UNITS,
			false,
		));

		basilisk_run_to_block(3);

		// pay with the new token
		assert_ok!(
			pallet_transaction_payment::ChargeTransactionPayment::<basilisk_runtime::Runtime>::from(0).pre_dispatch(
				&AccountId::from(DAVE),
				&call,
				&info,
				len,
			)
		);

		let dave_balance = basilisk_runtime::Tokens::free_balance(NEW_TOKEN, &AccountId::from(DAVE));
		assert_eq!(dave_balance, 990_264_297_166_679);
	});
}

const HITCHHIKER: [u8; 32] = [42u8; 32];

#[test]
fn fee_currency_on_account_lifecycle() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_eq!(
			MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			None
		);

		// ------------ set on create ------------
		assert_ok!(Currencies::transfer(
			RuntimeOrigin::signed(BOB.into()),
			HITCHHIKER.into(),
			1,
			50_000_000_000_000,
		));

		assert_eq!(
			Tokens::free_balance(1, &AccountId::from(HITCHHIKER)),
			50_000_000_000_000
		);
		assert_eq!(
			MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			Some(1)
		);

		// ------------ remove on delete ------------
		assert_ok!(Tokens::transfer_all(
			RuntimeOrigin::signed(HITCHHIKER.into()),
			BOB.into(),
			1,
			false,
		));

		assert_eq!(
			MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			None
		);
	});
}

#[test]
fn fee_currency_should_not_change_when_account_holds_native_currency_already() {
	TestNet::reset();
	Basilisk::execute_with(|| {
		assert_ok!(Balances::set_balance(
			RuntimeOrigin::root(),
			HITCHHIKER.into(),
			UNITS,
			0,
		));

		assert_ok!(Currencies::transfer(
			RuntimeOrigin::signed(ALICE.into()),
			HITCHHIKER.into(),
			1,
			50_000_000_000_000,
		));

		assert_eq!(Balances::free_balance(&AccountId::from(HITCHHIKER)), UNITS);
		assert_eq!(
			MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			None
		);
	});
}

#[test]
fn fee_currency_should_not_change_when_account_holds_other_token_already() {
	TestNet::reset();
	Basilisk::execute_with(|| {
		assert_ok!(Currencies::transfer(
			RuntimeOrigin::signed(ALICE.into()),
			HITCHHIKER.into(),
			1,
			50_000_000_000_000,
		));

		assert_ok!(Currencies::transfer(
			RuntimeOrigin::signed(ALICE.into()),
			HITCHHIKER.into(),
			2,
			50_000_000_000,
		));

		assert_eq!(
			MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			Some(1)
		);
	});
}

#[test]
fn fee_currency_should_reset_to_default_when_account_spends_tokens() {
	TestNet::reset();
	Basilisk::execute_with(|| {
		assert_ok!(Currencies::transfer(
			RuntimeOrigin::signed(ALICE.into()),
			HITCHHIKER.into(),
			1,
			50_000_000_000_000,
		));

		assert_ok!(Currencies::transfer(
			RuntimeOrigin::signed(ALICE.into()),
			HITCHHIKER.into(),
			2,
			50_000_000_000,
		));
		assert_ok!(Tokens::transfer_all(
			RuntimeOrigin::signed(HITCHHIKER.into()),
			ALICE.into(),
			1,
			false,
		));

		assert_eq!(
			MultiTransactionPayment::get_currency(&AccountId::from(HITCHHIKER)),
			None
		);
	});
}
