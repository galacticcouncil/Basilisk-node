use crate::{mock::*, IsTestnet, Pallet as Parameters};

#[test]
fn is_testnet_is_false_by_default() {
	ExtBuilder.build().execute_with(|| {
		assert!(!Parameters::<Test>::is_testnet());
	});
}

#[test]
fn is_testnet_can_be_read_from_storage() {
	ExtBuilder.build().execute_with(|| {
		IsTestnet::<Test>::put(true);
		assert!(Parameters::<Test>::is_testnet());
	});
}
