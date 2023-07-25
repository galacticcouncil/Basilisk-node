#![cfg(test)]

use crate::kusama_test_net::*;

use frame_support::assert_ok;
use frame_support::weights::Weight;
use orml_traits::currency::MultiCurrency;
use pallet_asset_registry::AssetType;
use polkadot_xcm::prelude::*;
use xcm_emulator::TestExt;

pub const EVE: [u8; 32] = [8u8; 32];

/// Returns the message hash in the `XcmpMessageSent` event at the `n`th last event (1-indexed, so if the second to last
/// event has the hash, pass `2`);
fn get_message_hash_from_event(n: usize) -> Option<[u8; 32]> {
	use cumulus_pallet_xcmp_queue::Event;
	use parachain_runtime_mock::RuntimeEvent;
	let RuntimeEvent::XcmpQueue(Event::XcmpMessageSent { message_hash }) = &last_parachain_events(n)[0] else {
		panic!("expecting to find message sent event");
	};
	*message_hash
}

// NOTE: Tests disabled until toggling the `runtime-benchmarks` feature no longer fails these tests.
// Github issue: https://github.com/galacticcouncil/Basilisk-node/issues/637
#[ignore]
#[test]
fn xcm_rate_limiter_should_limit_aca_when_limit_is_exceeded() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			AUSD,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))))
		));

		// set an xcm rate limit
		assert_ok!(basilisk_runtime::AssetRegistry::update(
			basilisk_runtime::RuntimeOrigin::root(),
			AUSD,
			b"AUSD".to_vec(),
			AssetType::Token,
			None,
			Some(50 * UNITS),
		));

		assert_eq!(basilisk_runtime::Tokens::free_balance(AUSD, &AccountId::from(EVE)), 0);
	});

	let amount = 100 * UNITS;
	let mut message_hash = None;
	OtherParachain::execute_with(|| {
		assert!(parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)) >= amount);
		// Act
		assert_ok!(parachain_runtime_mock::XTokens::transfer(
			parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
			0,
			amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 { id: EVE, network: None }
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_ref_time(399_600_000_000))
		));

		// Assert
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - amount
		);

		message_hash = get_message_hash_from_event(2);
	});

	Basilisk::execute_with(|| {
		expect_basilisk_events(vec![
			cumulus_pallet_xcmp_queue::Event::XcmDeferred {
				sender: OTHER_PARA_ID.into(),
				sent_at: 3,
				deferred_to: basilisk_runtime::DeferDuration::get() + 4,
				message_hash,
			}
			.into(),
			pallet_relaychain_info::Event::CurrentBlockNumbers {
				parachain_block_number: 1,
				relaychain_block_number: 5,
			}
			.into(),
		]);
		assert_eq!(basilisk_runtime::Tokens::free_balance(AUSD, &AccountId::from(EVE)), 0);
	});
}

#[test]
#[ignore]
fn xcm_rate_limiter_should_not_limit_aca_when_limit_is_not_exceeded() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			AUSD,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))))
		));

		// set an xcm rate limit
		assert_ok!(basilisk_runtime::AssetRegistry::update(
			basilisk_runtime::RuntimeOrigin::root(),
			AUSD,
			b"AUSD".to_vec(),
			AssetType::Token,
			None,
			Some(101 * UNITS),
		));
	});

	let amount = 100 * UNITS;
	OtherParachain::execute_with(|| {
		// Act
		assert_ok!(parachain_runtime_mock::XTokens::transfer(
			parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
			0,
			amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 { id: EVE, network: None }
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_ref_time(399_600_000_000))
		));

		// Assert
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - amount
		);
	});

	Basilisk::execute_with(|| {
		let fee = basilisk_runtime::Tokens::free_balance(AUSD, &basilisk_runtime::Treasury::account_id());
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(AUSD, &AccountId::from(EVE)),
			amount - fee
		);
	});
}

#[test]
#[ignore]
fn deferred_messages_should_be_executable_by_root() {
	// Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			AUSD,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))))
		));

		// set an xcm rate limit
		assert_ok!(basilisk_runtime::AssetRegistry::update(
			basilisk_runtime::RuntimeOrigin::root(),
			AUSD,
			b"AUSD".to_vec(),
			AssetType::Token,
			None,
			Some(50 * UNITS),
		));

		assert_eq!(basilisk_runtime::Tokens::free_balance(AUSD, &AccountId::from(EVE)), 0);
	});

	let amount = 100 * UNITS;
	let mut message_hash = None;
	let max_weight = Weight::from_ref_time(399_600_000_000);

	OtherParachain::execute_with(|| {
		assert!(parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)) >= amount);
		// Act
		assert_ok!(parachain_runtime_mock::XTokens::transfer(
			parachain_runtime_mock::RuntimeOrigin::signed(ALICE.into()),
			0,
			amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(BASILISK_PARA_ID),
						Junction::AccountId32 { id: EVE, network: None }
					)
				)
				.into()
			),
			WeightLimit::Limited(Weight::from_ref_time(399_600_000_000))
		));

		// Assert
		assert_eq!(
			parachain_runtime_mock::Balances::free_balance(&AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - amount
		);

		message_hash = get_message_hash_from_event(2);
	});

	Basilisk::execute_with(|| {
		expect_basilisk_events(vec![
			cumulus_pallet_xcmp_queue::Event::XcmDeferred {
				sender: OTHER_PARA_ID.into(),
				sent_at: 3,
				deferred_to: basilisk_runtime::DeferDuration::get() + 4,
				message_hash,
			}
			.into(),
			pallet_relaychain_info::Event::CurrentBlockNumbers {
				parachain_block_number: 1,
				relaychain_block_number: 5,
			}
			.into(),
		]);
		assert_eq!(basilisk_runtime::Tokens::free_balance(AUSD, &AccountId::from(EVE)), 0);

		set_relaychain_block_number(basilisk_runtime::DeferDuration::get() + 4);

		assert_eq!(basilisk_runtime::Tokens::free_balance(AUSD, &AccountId::from(EVE)), 0);
		assert_ok!(basilisk_runtime::XcmpQueue::service_deferred(
			basilisk_runtime::RuntimeOrigin::root(),
			max_weight,
			OTHER_PARA_ID.into(),
		));

		let fee = basilisk_runtime::Tokens::free_balance(AUSD, &basilisk_runtime::Treasury::account_id());
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(AUSD, &AccountId::from(EVE)),
			amount - fee
		);
	});
}
