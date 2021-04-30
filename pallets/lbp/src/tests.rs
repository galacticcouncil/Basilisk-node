use super::*;
pub use crate::mock::{
	run_to_block, Currency, Event as TestEvent, ExtBuilder, LBPPallet, Origin, System, Test, ACA, ALICE, BOB, CHARLIE,
	DOT, ETH, HDX,
};
use crate::mock::{INITIAL_BALANCE, POOL_ADDRESS, POOL_DEPOSIT, POOL_SWAP_FEE};
use frame_support::{assert_err, assert_noop, assert_ok};

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let amount_a = 1_000_000_000;
		let amount_b = 2_000_000_000;
		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			final_weights: (40, 60),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};

		assert_ok!(LBPPallet::create_pool(
			Origin::signed(user),
			asset_a,
			amount_a,
			asset_b,
			amount_b,
			pool_data
		));
	});

	ext
}

fn last_events(n: usize) -> Vec<TestEvent> {
	frame_system::Pallet::<Test>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

fn expect_events(e: Vec<TestEvent>) {
	assert_eq!(last_events(e.len()), e);
}

#[test]
fn validate_pool_data_should_work() {
	new_test_ext().execute_with(|| {
		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			final_weights: (40, 60),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		let pool_data = Pool {
			start: 0u64,
			end: 0u64,
			final_weights: (40, 60),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		let pool_data = Pool {
			start: 10u64,
			end: 2u64,
			final_weights: (40, 60),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_err!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::BlockNumberInvalid
		);

		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			final_weights: (400, 60),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_err!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::MaxWeightExceeded
		);

		//TODO: add test !pausable && puased -> should return error
	});
}

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let amount_a = 1_000_000_000;
		let amount_b = 2_000_000_000;
		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			final_weights: (40, 60),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};

		assert_ok!(LBPPallet::create_pool(
			Origin::signed(user),
			asset_a,
			amount_a,
			asset_b,
			amount_b,
			pool_data
		));

		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		assert_eq!(Currency::free_balance(asset_a, &pool_id), amount_a);
		assert_eq!(Currency::free_balance(asset_b, &pool_id), amount_b);
		assert_eq!(
			Currency::free_balance(asset_a, &user),
			INITIAL_BALANCE.saturating_sub(amount_a)
		);
		assert_eq!(
			Currency::free_balance(asset_b, &user),
			INITIAL_BALANCE.saturating_sub(amount_b)
		);
		assert_eq!(Currency::reserved_balance(HDX, &user), POOL_DEPOSIT);
		assert_eq!(
			Currency::free_balance(HDX, &user),
			INITIAL_BALANCE.saturating_sub(POOL_DEPOSIT)
		);
		assert_eq!(LBPPallet::pool_deposit(&pool_id), POOL_DEPOSIT);

		assert_eq!(LBPPallet::get_pool_assets(&pool_id).unwrap(), vec![asset_a, asset_b]);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, amount_a, amount_b).into()
		]);
	});
}

#[test]
fn create_same_pool_should_not_work() {
	new_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let amount_a = 1_000_000_000;
		let amount_b = 2_000_000_000;
		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			final_weights: (40, 60),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};

		assert_ok!(LBPPallet::create_pool(
			Origin::signed(user),
			asset_a,
			amount_a,
			asset_b,
			amount_b,
			pool_data
		));
		assert_noop!(
			LBPPallet::create_pool(Origin::signed(user), asset_a, amount_a, asset_b, amount_b, pool_data),
			Error::<Test>::TokenPoolAlreadyExists
		);
		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, amount_a, amount_b).into()
		]);
	});
}

#[test]
fn create_pool_invalid_data_should_not_work() {
	new_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let amount_a = 1_000_000_000;
		let amount_b = 2_000_000_000;
		let pool_data = Pool {
			start: 10u64,
			end: 2u64,
			final_weights: (40, 60),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};

		assert_noop!(
			LBPPallet::create_pool(Origin::signed(user), asset_a, amount_a, asset_b, amount_b, pool_data),
			Error::<Test>::BlockNumberInvalid
		);
	});
}

#[test]
fn update_pool_data_should_work() {
	predefined_test_ext().execute_with(|| {
		let user = ALICE;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: ACA,
			asset_out: DOT,
		});
		let new_start = 15;
		let new_final_weights = (10, 90);

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(user),
			pool_id,
			Some(new_start),
			None,
			Some(new_final_weights)
		));

		let updated_pool_data = LBPPallet::pool_data(pool_id);
		assert_eq!(updated_pool_data.start, new_start);
		assert_eq!(updated_pool_data.end, 20);

		expect_events(vec![Event::UpdatePool(user, pool_id).into()]);
	});
}

#[test]
fn pause_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		let user = ALICE;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: ACA,
			asset_out: DOT,
		});

		assert_ok!(LBPPallet::pause_pool(Origin::signed(user), pool_id,));

		let paused_pool = LBPPallet::pool_data(pool_id);
		assert_eq!(
			paused_pool,
			Pool {
				start: 10u64,
				end: 20u64,
				final_weights: (40, 60),
				curve: CurveType::Linear,
				pausable: true,
				paused: true
			}
		);

		expect_events(vec![Event::Paused(user).into()]);
	});
}

#[test]
fn pause_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let owner = ALICE;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: ACA,
			asset_out: DOT,
		});

		//user is not pool owner
		let not_owner = BOB;
		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(not_owner), pool_id),
			Error::<Test>::NotOwner
		);

		//pool is not found
		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(owner), 24568),
			Error::<Test>::TokenPoolNotFound
		);

		//pool is not puasable
		assert_ok!(LBPPallet::create_pool(
			Origin::signed(BOB),
			ACA,
			1_000_000_000,
			ETH,
			2_000_000_000,
			Pool {
				start: 200_u64,
				end: 400_u64,
				final_weights: (40, 60),
				curve: CurveType::Linear,
				pausable: false,
				paused: false,
			}
		));
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: ACA,
			asset_out: ETH,
		});

		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(BOB), pool_id),
			Error::<Test>::PoolIsNotPausable
		);

		//pool is already paused
		assert_ok!(LBPPallet::create_pool(
			Origin::signed(BOB),
			DOT,
			1_000_000_000,
			ETH,
			2_000_000_000,
			Pool {
				start: 200_u64,
				end: 400_u64,
				final_weights: (40, 60),
				curve: CurveType::Linear,
				pausable: true,
				paused: true,
			}
		));
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: DOT,
			asset_out: ETH,
		});

		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(BOB), pool_id),
			Error::<Test>::CannotPausePausedPool
		);

		//pooled ended or ending in current block
		assert_ok!(LBPPallet::create_pool(
			Origin::signed(ALICE),
			DOT,
			1_000_000_000,
			HDX,
			2_000_000_000,
			Pool {
				start: 200_u64,
				end: 400_u64,
				final_weights: (40, 60),
				curve: CurveType::Linear,
				pausable: true,
				paused: false,
			}
		));
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: DOT,
			asset_out: HDX,
		});

		run_to_block(400);
		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(ALICE), pool_id),
			Error::<Test>::CannotPauseEndedPool
		);

		run_to_block(500);
		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(ALICE), pool_id),
			Error::<Test>::CannotPauseEndedPool
		);
	});
}

#[test]
fn unpause_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		let owner = ALICE;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: DOT,
			asset_out: HDX,
		});

		assert_ok!(LBPPallet::create_pool(
			Origin::signed(owner),
			DOT,
			1_000_000_000,
			HDX,
			2_000_000_000,
			Pool {
				start: 200u64,
				end: 400u64,
				final_weights: (40, 60),
				curve: CurveType::Linear,
				pausable: true,
				paused: true,
			}
		));

		assert_ok!(LBPPallet::unpause_pool(Origin::signed(owner), pool_id,));

		let unpaused_pool = LBPPallet::pool_data(pool_id);
		assert_eq!(
			unpaused_pool,
			Pool {
				start: 200_u64,
				end: 400_u64,
				final_weights: (40, 60),
				curve: CurveType::Linear,
				pausable: true,
				paused: false
			}
		);

		expect_events(vec![Event::Unpaused(owner).into()]);
	});
}

#[test]
fn unpause_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let owner = ALICE;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: ACA,
			asset_out: DOT,
		});

		//user is not pool owner
		let not_owner = BOB;
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(not_owner), pool_id),
			Error::<Test>::NotOwner
		);

		//pool is not found
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(owner), 24568),
			Error::<Test>::TokenPoolNotFound
		);

		//pool is not puased
		assert_ok!(LBPPallet::create_pool(
			Origin::signed(BOB),
			ACA,
			1_000_000_000,
			ETH,
			2_000_000_000,
			Pool {
				start: 200_u64,
				end: 400_u64,
				final_weights: (40, 60),
				curve: CurveType::Linear,
				pausable: false,
				paused: false,
			}
		));
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: ACA,
			asset_out: ETH,
		});

		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(BOB), pool_id),
			Error::<Test>::PoolIsNotPaused
		);

		//pooled ended or ending in current block
		assert_ok!(LBPPallet::create_pool(
			Origin::signed(ALICE),
			DOT,
			1_000_000_000,
			HDX,
			2_000_000_000,
			Pool {
				start: 200_u64,
				end: 400_u64,
				final_weights: (40, 60),
				curve: CurveType::Linear,
				pausable: true,
				paused: true,
			}
		));
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: DOT,
			asset_out: HDX,
		});

		run_to_block(400);
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(ALICE), pool_id),
			Error::<Test>::CannotUnpauseEndedPool
		);

		run_to_block(500);
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(ALICE), pool_id),
			Error::<Test>::CannotUnpauseEndedPool
		);
	});
}
