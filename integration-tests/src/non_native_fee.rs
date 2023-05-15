#![cfg(test)]

use crate::kusama_test_net::*;

use frame_support::{
	assert_ok,
	dispatch::{DispatchInfo, Weight},
	sp_runtime::traits::SignedExtension,
	traits::{OnFinalize, OnInitialize},
};

use pallet_transaction_multi_payment::Price;

use basilisk_runtime::{Balances, Currencies, MultiTransactionPayment, RuntimeOrigin, Tokens};

use hydradx_traits::{pools::SpotPriceProvider, AMM};
use orml_traits::currency::MultiCurrency;
use pallet_xyk::XYKSpotPrice;
use polkadot_primitives::v2::BlockNumber;
use primitives::asset::AssetPair;
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
fn non_native_fee_payment_works_with_xyk_spot_price() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let currency_0 = 0;
		let currency_1 = 1;

		let call = basilisk_runtime::RuntimeCall::MultiTransactionPayment(
			pallet_transaction_multi_payment::Call::set_currency { currency: currency_1 },
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

		let bob_balance = basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB));
		assert_eq!(bob_balance, 999_999_992_137_669);

		let pair_account = basilisk_runtime::XYK::get_pair_id(AssetPair {
			asset_in: currency_0,
			asset_out: currency_1,
		});

		assert_ok!(basilisk_runtime::Balances::set_balance(
			basilisk_runtime::RuntimeOrigin::root(),
			ALICE.into(),
			2_000_000_000_000 * UNITS,
			0,
		));

		assert_ok!(basilisk_runtime::Tokens::set_balance(
			basilisk_runtime::RuntimeOrigin::root(),
			ALICE.into(),
			1,
			2_000_000_000_000 * UNITS,
			0,
		));

		assert_ok!(basilisk_runtime::XYK::create_pool(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			currency_0, // 1000 BSX
			1_000 * UNITS,
			currency_1, // 500 KSM (500_000_033_400_002)
			500 * UNITS,
		));

		let spot_price = XYKSpotPrice::<basilisk_runtime::Runtime>::spot_price(currency_0, currency_1);
		assert_eq!(spot_price, Some(Price::from_float(0.5)));

		basilisk_run_to_block(2);

		assert_ok!(basilisk_runtime::XYK::buy(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			0,
			1,
			66 * UNITS,
			1_000 * UNITS,
			false,
		));

		basilisk_run_to_block(3);

		assert_eq!(
			basilisk_runtime::XYK::get_pool_assets(&pair_account),
			Some(vec![currency_0, currency_1])
		);

		// ------------ DAVE ------------
		let call = basilisk_runtime::RuntimeCall::MultiTransactionPayment(
			pallet_transaction_multi_payment::Call::set_currency { currency: currency_1 },
		);

		assert_ok!(
			pallet_transaction_payment::ChargeTransactionPayment::<basilisk_runtime::Runtime>::from(0).pre_dispatch(
				&AccountId::from(DAVE),
				&call,
				&info,
				len,
			)
		);

		let dave_balance = basilisk_runtime::Tokens::free_balance(1, &AccountId::from(DAVE));
		assert_eq!(dave_balance, 970_376_058_464_304);
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
