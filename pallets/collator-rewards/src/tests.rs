use super::*;

use crate::mock::{
	CollatorRewards, Tokens, ALICE, BOB, CHARLIE, COLLATOR_REWARD, DAVE, GC_COLL_1,
	GC_COLL_2, GC_COLL_3, NATIVE_TOKEN, new_test_ext,
};

use sp_runtime::testing::UintAuthorityId;

#[test]
fn reward_collator_on_new_session_should_work() {
	new_test_ext().execute_with(|| {
		//this are blaklisted collators and should not be rewarded
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &GC_COLL_1), 0);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &GC_COLL_2), 0);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &GC_COLL_3), 0);

		//this collators which should be rewarded
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &ALICE), 0);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &CHARLIE), 0);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &BOB), 0);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &DAVE), 0);

		let collator_set = vec![
			(&ALICE, UintAuthorityId(ALICE).to_public_key()),
			(&BOB, UintAuthorityId(BOB).to_public_key()),
			(&CHARLIE, UintAuthorityId(CHARLIE).to_public_key()),
			(&DAVE, UintAuthorityId(DAVE).to_public_key()),
		];

		CollatorRewards::on_new_session(true, collator_set.into_iter(), vec![].into_iter());

		//this are blaklisted collators and should not be rewarded
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &GC_COLL_1), 0);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &GC_COLL_2), 0);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &GC_COLL_3), 0);

		//this collators which should be rewarded
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &ALICE), COLLATOR_REWARD);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &CHARLIE), COLLATOR_REWARD);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &BOB), COLLATOR_REWARD);
		assert_eq!(Tokens::free_balance(NATIVE_TOKEN, &DAVE), COLLATOR_REWARD);
	});
}
