#![cfg(test)]
use crate::kusama_test_net::*;
use basilisk_runtime::CollatorRewards;
use basilisk_runtime::Runtime;
use frame_support::traits::Contains;
use pallet_session::SessionManager;
use polkadot_xcm::v3::prelude::*;
use polkadot_xcm::VersionedXcm;
use xcm_emulator::TestExt;

#[test]
fn new_session_should_rotate_collators_list() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let collator1 = basilisk::invulnerables()[0].0.clone(); //d435...
		let collator2 = basilisk::invulnerables()[1].0.clone(); //8eaf...
		let collator3 = basilisk::invulnerables()[2].0.clone(); //90b5...

		let collators = CollatorRewards::new_session(0).unwrap();
		assert_eq!(collators, vec![collator2.clone(), collator3.clone(), collator1.clone()]);

		let collators = CollatorRewards::new_session(1).unwrap();
		assert_eq!(collators, vec![collator2.clone(), collator3.clone(), collator1.clone()]);

		let collators = CollatorRewards::new_session(2).unwrap();
		assert_eq!(collators, vec![collator2.clone(), collator3.clone(), collator1.clone()]);

		let collators = CollatorRewards::new_session(3).unwrap();
		assert_eq!(collators, vec![collator3.clone(), collator1.clone(), collator2.clone()]);

		let collators = CollatorRewards::new_session(4).unwrap();
		assert_eq!(collators, vec![collator3.clone(), collator1.clone(), collator2.clone()]);

		let collators = CollatorRewards::new_session(5).unwrap();
		assert_eq!(collators, vec![collator3.clone(), collator1.clone(), collator2.clone()]);

		let collators = CollatorRewards::new_session(6).unwrap();
		assert_eq!(collators, vec![collator1.clone(), collator2.clone(), collator3.clone()]);

		let collators = CollatorRewards::new_session(7).unwrap();
		assert_eq!(collators, vec![collator1.clone(), collator2.clone(), collator3.clone()]);

		let collators = CollatorRewards::new_session(8).unwrap();
		assert_eq!(collators, vec![collator1.clone(), collator2.clone(), collator3.clone()]);

		let collators = CollatorRewards::new_session(9).unwrap();
		assert_eq!(collators, vec![collator2.clone(), collator3.clone(), collator1.clone()]);

		let collators = CollatorRewards::new_session(10).unwrap();
		assert_eq!(collators, vec![collator2.clone(), collator3.clone(), collator1.clone()]);
	});
}
