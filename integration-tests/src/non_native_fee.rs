#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{Balances, Currencies, MultiTransactionPayment, RuntimeOrigin, Tokens};

use frame_support::{
	assert_err, assert_ok,
	dispatch::DispatchInfo,
	sp_runtime::{
		traits::SignedExtension,
		transaction_validity::{InvalidTransaction, TransactionValidityError},
	},
	traits::{OnFinalize, OnInitialize},
	weights::Weight,
};
use hydradx_traits::AMM;
use orml_traits::currency::MultiCurrency;
use pallet_asset_registry::AssetType;
use pallet_transaction_multi_payment::Price;
use primitives::AssetId;
use xcm_emulator::TestExt;

pub fn basilisk_run_to_next_block() {
	let b = basilisk_runtime::System::block_number();

	basilisk_runtime::System::on_finalize(b);
	basilisk_runtime::EmaOracle::on_finalize(b);
	basilisk_runtime::MultiTransactionPayment::on_finalize(b);

	basilisk_runtime::System::on_initialize(b + 1);
	basilisk_runtime::EmaOracle::on_initialize(b + 1);
	basilisk_runtime::MultiTransactionPayment::on_initialize(b + 1);

	basilisk_runtime::System::set_block_number(b + 1);
}

#[test]
fn non_native_fee_payment_works_with_configured_price() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let call = basilisk_runtime::RuntimeCall::MultiTransactionPayment(
			pallet_transaction_multi_payment::Call::set_currency { currency: AUSD },
		);

		let info = DispatchInfo {
			weight: Weight::from_parts(106_957_000, 0),
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


//TODO: it will only work once we had OracleWhiteList properly set, as we don't have oracle data for the BSX/NET_TOKEN,
// so no new price is calculated in on_init of multi-payment-pallet
#[test]
fn non_native_fee_payment_works_with_oracle_price_based_on_onchain_route() {
	TestNet::reset();

	const NEW_TOKEN: AssetId = 42;

	let call =
		basilisk_runtime::RuntimeCall::MultiTransactionPayment(pallet_transaction_multi_payment::Call::set_currency {
			currency: NEW_TOKEN,
		});
	let info = DispatchInfo {
		weight: Weight::from_parts(106_957_000, 0),
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

		assert_ok!(basilisk_runtime::Balances::force_set_balance(
			basilisk_runtime::RuntimeOrigin::root(),
			ALICE.into(),
			2_000_000_000_000 * UNITS,
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

		assert_ok!(basilisk_runtime::XYK::buy(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			BSX,
			NEW_TOKEN,
			66 * UNITS,
			1_000 * UNITS,
			false,
		));

		/*let route = vec![Trade {
			pool: PoolType::XYK,
			asset_in: BSX,
			asset_out: NEW_TOKEN
		}];

		let route_asset_pair = hydradx_traits::router::AssetPair {
			asset_in: BSX,
			asset_out: NEW_TOKEN,
		};*/
		//assert!(basilisk_runtime::Router::get_route(route_asset_pair).is_some());

		//assert_ok!(basilisk_runtime::Router::set_route(basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),route_asset_pair, route ));

		basilisk_run_to_next_block();

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
		assert_eq!(MultiTransactionPayment::get_currency(AccountId::from(HITCHHIKER)), None);

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
			MultiTransactionPayment::get_currency(AccountId::from(HITCHHIKER)),
			Some(1)
		);

		// ------------ remove on delete ------------
		assert_ok!(Tokens::transfer_all(
			RuntimeOrigin::signed(HITCHHIKER.into()),
			BOB.into(),
			1,
			false,
		));

		assert_eq!(MultiTransactionPayment::get_currency(AccountId::from(HITCHHIKER)), None);
	});
}

#[test]
fn fee_currency_should_not_change_when_account_holds_native_currency_already() {
	TestNet::reset();
	Basilisk::execute_with(|| {
		assert_ok!(Balances::force_set_balance(
			RuntimeOrigin::root(),
			HITCHHIKER.into(),
			UNITS,
		));

		assert_ok!(Currencies::transfer(
			RuntimeOrigin::signed(ALICE.into()),
			HITCHHIKER.into(),
			1,
			50_000_000_000_000,
		));

		assert_eq!(Balances::free_balance(AccountId::from(HITCHHIKER)), UNITS);
		assert_eq!(MultiTransactionPayment::get_currency(AccountId::from(HITCHHIKER)), None);
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
			MultiTransactionPayment::get_currency(AccountId::from(HITCHHIKER)),
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

		assert_eq!(MultiTransactionPayment::get_currency(AccountId::from(HITCHHIKER)), None);
	});
}
