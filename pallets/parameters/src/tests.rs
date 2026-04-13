use crate::{mock::*, Pallet as Parameters, RelayParentOffsetOverride};

#[test]
fn relay_parent_offset_override_is_false_by_default() {
	ExtBuilder.build().execute_with(|| {
		assert!(!Parameters::<Test>::relay_parent_offset_override());
	});
}

#[test]
fn relay_parent_offset_override_can_be_read_from_storage() {
	ExtBuilder.build().execute_with(|| {
		RelayParentOffsetOverride::<Test>::put(true);
		assert!(Parameters::<Test>::relay_parent_offset_override());
	});
}
