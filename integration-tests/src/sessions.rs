#![cfg(test)]
use crate::kusama_test_net::*;
use basilisk_runtime::CollatorRewards;
use pallet_session::SessionManager;
use pretty_assertions::assert_eq;
use xcm_emulator::TestExt;
#[test]
fn new_session_should_rotate_collators_list() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let collator1 = basilisk::invulnerables()[0].0.clone(); //d435...
		let collator2 = basilisk::invulnerables()[1].0.clone(); //8eaf...
		let collator3 = basilisk::invulnerables()[2].0.clone(); //90b5...
		let collator4 = basilisk::invulnerables()[3].0.clone(); //6ebe...
		let collator5 = basilisk::invulnerables()[4].0.clone(); //ec5e...
		let collator6 = basilisk::invulnerables()[5].0.clone(); //9c78...
		let collator7 = basilisk::invulnerables()[6].0.clone(); //a678...
		let collator8 = basilisk::invulnerables()[7].0.clone(); //2433...
		let collator9 = basilisk::invulnerables()[8].0.clone(); //ee28...
		let collator10 = basilisk::invulnerables()[9].0.clone(); //da53...

		let collators = CollatorRewards::new_session(0).unwrap();
		assert_eq!(
			collators,
			vec![
				collator8.clone(),
				collator4.clone(),
				collator2.clone(),
				collator3.clone(),
				collator6.clone(),
				collator7.clone(),
				collator1.clone(),
				collator10.clone(),
				collator5.clone(),
				collator9.clone()
			]
		);

		let collators = CollatorRewards::new_session(1).unwrap();
		assert_eq!(
			collators,
			vec![
				collator4.clone(),
				collator2.clone(),
				collator3.clone(),
				collator6.clone(),
				collator7.clone(),
				collator1.clone(),
				collator10.clone(),
				collator5.clone(),
				collator9.clone(),
				collator8.clone(),
			]
		);

		let collators = CollatorRewards::new_session(2).unwrap();
		assert_eq!(
			collators,
			vec![
				collator2.clone(),
				collator3.clone(),
				collator6.clone(),
				collator7.clone(),
				collator1.clone(),
				collator10.clone(),
				collator5.clone(),
				collator9.clone(),
				collator8.clone(),
				collator4.clone(),
			]
		);

		let collators = CollatorRewards::new_session(3).unwrap();
		assert_eq!(
			collators,
			vec![
				collator3.clone(),
				collator6.clone(),
				collator7.clone(),
				collator1.clone(),
				collator10.clone(),
				collator5.clone(),
				collator9.clone(),
				collator8.clone(),
				collator4.clone(),
				collator2.clone(),
			]
		);
	});
}
