#![cfg(test)]
use crate::kusama_test_net::*;
use frame_support::traits::Contains;
use polkadot_xcm::v3::prelude::*;
use polkadot_xcm::VersionedXcm;
use xcm_emulator::TestExt;

#[test]
fn calling_pallet_uniques_extrinsic_should_be_filtered_by_call_filter() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let call = basilisk_runtime::RuntimeCall::Uniques(pallet_uniques::Call::create {
			collection: 1u128,
			admin: AccountId::from(ALICE),
		});

		assert!(!basilisk_runtime::BaseFilter::contains(&call));
	});
}

#[test]
fn calling_pallet_xcm_extrinsic_should_be_filtered_by_call_filter() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		// the values here don't need to make sense, all we need is a valid Call
		let call = basilisk_runtime::RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
			dest: Box::new(MultiLocation::parent().into_versioned()),
			message: Box::new(VersionedXcm::from(Xcm(vec![]))),
		});

		assert!(!basilisk_runtime::BaseFilter::contains(&call));
	});
}

#[test]
fn calling_orml_xcm_extrinsic_should_be_filtered_by_call_filter() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		// the values here don't need to make sense, all we need is a valid Call
		let call = basilisk_runtime::RuntimeCall::OrmlXcm(orml_xcm::Call::send_as_sovereign {
			dest: Box::new(MultiLocation::parent().into_versioned()),
			message: Box::new(VersionedXcm::from(Xcm(vec![]))),
		});

		assert!(!basilisk_runtime::BaseFilter::contains(&call));
	});
}
