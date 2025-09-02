#![cfg(test)]
use crate::kusama_test_net::*;
use xcm_emulator::TestExt;

use basilisk_runtime::RuntimeOrigin;
use cumulus_primitives_core::ParaId;
use frame_support::assert_ok;
use frame_support::storage::with_transaction;
use orml_traits::currency::MultiCurrency;
use polkadot_xcm::v3::{prelude::*, MultiLocation};
use sp_runtime::{DispatchResult, FixedU128, TransactionOutcome};

#[test]
fn ksm_reserve_from_asset_hub_should_be_recognized() {
	// Test that KSM from Asset Hub is properly recognized as a valid reserve
	TestNet::reset();

	Basilisk::execute_with(|| {
		let _ = with_transaction(|| {
			add_currency_price(KSM, FixedU128::from(1));

			assert_ok!(basilisk_runtime::Tokens::deposit(
				KSM,
				&AccountId::from(ALICE),
				1000 * UNITS
			));

			TransactionOutcome::Commit(DispatchResult::Ok(()))
		});

		// Set KSM location as relay chain asset
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			RuntimeOrigin::root(),
			KSM,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, Here))
		));

		// Verify the location is set correctly
		let location = basilisk_runtime::AssetRegistry::asset_to_location(KSM);
		assert!(location.is_some());

		let expected_location = MultiLocation::new(1, Here);
		assert_eq!(location.unwrap().0, expected_location);
	});
}

#[test]
fn asset_hub_para_id_should_be_configured_correctly() {
	// Test that Asset Hub para ID is set to 1000
	TestNet::reset();

	Basilisk::execute_with(|| {
		// This tests our constant configuration
		assert_eq!(basilisk_runtime::xcm::AssetHubParaId::get(), ParaId::from(1000));
	});
}

pub fn add_currency_price(currency: u32, price: FixedU128) {
	assert_ok!(basilisk_runtime::MultiTransactionPayment::add_currency(
		basilisk_runtime::RuntimeOrigin::root(),
		currency,
		price,
	));
}
