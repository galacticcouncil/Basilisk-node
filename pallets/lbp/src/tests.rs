use super::*;
pub use crate::mock::{
	run_to_block, Currency, Event as TestEvent, ExtBuilder, LBPPallet, Origin, System, Test, ACA, ALICE, BOB, CHARLIE,
	DOT, ETH, HDX,
};
use crate::mock::{INITIAL_BALANCE, POOL_ADDRESS, POOL_DEPOSIT};
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
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: 0u64,
			last_weights: ((1, 20), (2, 80)),
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

// TODO: move me to the hydradx-math crate
#[test]
fn linear_weights_should_work() {
	let u32_cases = vec![
		(100u32, 200u32, 1_000u128, 2_000u128, 170u32, Ok(1_700), "Easy case"),
		(
			100u32,
			200u32,
			2_000u128,
			1_000u128,
			170u32,
			Ok(1_300),
			"Easy decreasing case",
		),
		(
			100u32,
			200u32,
			2_000u128,
			2_000u128,
			170u32,
			Ok(2_000),
			"Easy constant case",
		),
		(
			100u32,
			200u32,
			1_000u128,
			2_000u128,
			100u32,
			Ok(1_000),
			"Initial weight",
		),
		(
			100u32,
			200u32,
			2_000u128,
			1_000u128,
			100u32,
			Ok(2_000),
			"Initial decreasing weight",
		),
		(
			100u32,
			200u32,
			2_000u128,
			2_000u128,
			100u32,
			Ok(2_000),
			"Initial constant weight",
		),
		(100u32, 200u32, 1_000u128, 2_000u128, 200u32, Ok(2_000), "Final weight"),
		(
			100u32,
			200u32,
			2_000u128,
			1_000u128,
			200u32,
			Ok(1_000),
			"Final decreasing weight",
		),
		(
			100u32,
			200u32,
			2_000u128,
			2_000u128,
			200u32,
			Ok(2_000),
			"Final constant weight",
		),
		(
			200u32,
			100u32,
			1_000u128,
			2_000u128,
			170u32,
			Err(Overflow),
			"Invalid interval",
		),
		(
			100u32,
			100u32,
			1_000u128,
			2_000u128,
			100u32,
			Err(ZeroDuration),
			"Invalid interval",
		),
		(
			100u32,
			200u32,
			1_000u128,
			2_000u128,
			10u32,
			Err(Overflow),
			"Out of bound",
		),
		(
			100u32,
			200u32,
			1_000u128,
			2_000u128,
			210u32,
			Err(Overflow),
			"Out of bound",
		),
	];
	let u64_cases = vec![
		(100u64, 200u64, 1_000u128, 2_000u128, 170u64, Ok(1_700), "Easy case"),
		(
			100u64,
			u64::MAX,
			1_000u128,
			2_000u128,
			200u64,
			Err(Overflow),
			"Interval too long",
		),
	];

	for case in u32_cases {
		assert_eq!(
			crate::calculate_linear_weights(case.0, case.1, case.2, case.3, case.4),
			case.5,
			"{}",
			case.6
		);
	}
	for case in u64_cases {
		assert_eq!(
			crate::calculate_linear_weights(case.0, case.1, case.2, case.3, case.4),
			case.5,
			"{}",
			case.6
		);
	}
}

#[test]
fn weight_update_should_work() {
	new_test_ext().execute_with(|| {
		let mut linear_pool = Pool {
			start: 10u64,
			end: 19u64,
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 80), (2, 20)),
			last_weight_update: 2u64,
			last_weights: ((1, 2), (2, 2)),
			curve: CurveType::Linear,
			pausable: false,
			paused: false,
		};
		let mut constant_pool = Pool {
			start: 10u64,
			end: 19u64,
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 80), (2, 20)),
			last_weight_update: 2u64,
			last_weights: ((1, 2), (2, 2)),
			curve: CurveType::Constant,
			pausable: false,
			paused: false,
		};

		// TODO: add test: last_weights and last_weight_update values are initialized to meaningful values

		assert_ok!(LBPPallet::create_pool(
			Origin::signed(ALICE),
			HDX,
			1,
			DOT,
			1,
			linear_pool,
		));
		assert_ok!(LBPPallet::create_pool(
			Origin::signed(ALICE),
			HDX,
			1,
			ACA,
			1,
			constant_pool,
		));

		System::set_block_number(13);

		LBPPallet::update_weights(&mut linear_pool);
		LBPPallet::update_weights(&mut constant_pool);

		assert_eq!(linear_pool.last_weight_update, 13);
		assert_eq!(constant_pool.last_weight_update, 13);

		assert_eq!(linear_pool.last_weights, ((1, 40u128), (2, 60u128)));
		assert_eq!(constant_pool.last_weights, ((1, 20u128), (2, 80u128)));

		// call update again in the same block, data should be the same
		LBPPallet::update_weights(&mut linear_pool);
		LBPPallet::update_weights(&mut constant_pool);

		assert_eq!(linear_pool.last_weight_update, 13);
		assert_eq!(constant_pool.last_weight_update, 13);

		assert_eq!(linear_pool.last_weights, ((1, 40u128), (2, 60u128)));
		assert_eq!(constant_pool.last_weights, ((1, 20u128), (2, 80u128)));
	});
}

#[test]
fn validate_pool_data_should_work() {
	new_test_ext().execute_with(|| {
		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: 0u64,
			last_weights: ((1, 20), (2, 80)),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		let pool_data = Pool {
			start: 0u64,
			end: 0u64,
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: 0u64,
			last_weights: ((1, 20), (2, 80)),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		let pool_data = Pool {
			start: 10u64,
			end: 2u64,
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: 0u64,
			last_weights: ((1, 20), (2, 80)),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_noop!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::InvalidBlockNumber
		);

		let pool_data = Pool {
			start: 10u64,
			end: 11u64 + u32::MAX as u64,
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: 0u64,
			last_weights: ((1, 20), (2, 80)),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_noop!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::MaxSaleDurationExceeded
		);
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
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: 0u64,
			last_weights: ((1, 20), (2, 80)),
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
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: 0u64,
			last_weights: ((1, 20), (2, 80)),
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
			initial_weights: ((1, 20), (2, 80)),
			final_weights: ((1, 90), (2, 10)),
			last_weight_update: 0u64,
			last_weights: ((1, 20), (2, 80)),
			curve: CurveType::Linear,
			pausable: true,
			paused: false,
		};

		assert_noop!(
			LBPPallet::create_pool(Origin::signed(user), asset_a, amount_a, asset_b, amount_b, pool_data),
			Error::<Test>::InvalidBlockNumber
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
		let new_final_weights = ((1, 10), (2, 90));

		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(user),
			pool_id,
			Some(new_start),
			None,
			Some(new_final_weights),
			None,
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
				initial_weights: ((1, 20), (2, 80)),
				final_weights: ((1, 90), (2, 10)),
				last_weight_update: 0u64,
				last_weights: ((1, 20), (2, 80)),
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
				initial_weights: ((1, 20), (2, 80)),
				final_weights: ((1, 40), (2, 60)),
				last_weight_update: 0u64,
				last_weights: ((1, 20), (2, 80)),
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
				initial_weights: ((3, 20), (2, 80)),
				final_weights: ((3, 40), (3, 60)),
				last_weight_update: 0u64,
				last_weights: ((3, 20), (2, 80)),
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
				initial_weights: ((3, 20), (4, 80)),
				final_weights: ((3, 40), (4, 60)),
				last_weight_update: 0u64,
				last_weights: ((3, 20), (4, 80)),
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
				initial_weights: ((3, 20), (4, 80)),
				final_weights: ((3, 40), (4, 60)),
				last_weight_update: 0u64,
				last_weights: ((3, 20), (4, 80)),
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
				initial_weights: ((3, 20), (4, 80)),
				final_weights: ((3, 40), (4, 60)),
				last_weight_update: 0u64,
				last_weights: ((3, 20), (4, 80)),
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
				initial_weights: ((1, 20), (2, 80)),
				final_weights: ((1, 40), (2, 60)),
				last_weight_update: 0u64,
				last_weights: ((1, 20), (2, 80)),
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
				initial_weights: ((3, 20), (4, 80)),
				final_weights: ((3, 40), (4, 60)),
				last_weight_update: 0u64,
				last_weights: ((3, 20), (4, 80)),
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

#[test]
fn update_pool_data_for_running_lbp_should_not_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(16);

		let user = ALICE;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: ACA,
			asset_out: DOT,
		});
		let new_start = 15;
		let new_final_weights = ((1, 10), (2, 90));

		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(user),
				pool_id,
				Some(new_start),
				None,
				Some(new_final_weights),
				None,
			),
			Error::<Test>::SaleStarted
		);

		let updated_pool_data = LBPPallet::pool_data(pool_id);
		assert_eq!(updated_pool_data.start, 10);
		assert_eq!(updated_pool_data.end, 20);

		expect_events(vec![
			Event::CreatePool(user, ACA, DOT, 1_000_000_000, 2_000_000_000).into()
		]);
	});
}

#[test]
fn add_liquidity_should_work() {
	predefined_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let user_balance_b_before = Currency::free_balance(asset_b, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		let added_a = 10_000_000_000;
		let added_b = 20_000_000_000;

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(user),
			pool_id,
			added_a,
			added_b,
		));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before.saturating_add(added_a));
		assert_eq!(balance_b_after, balance_b_before.saturating_add(added_b));

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(added_a));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(added_b));

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
			Event::AddLiquidity(pool_id, asset_a, asset_b, added_a, added_b).into(),
		]);

		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_ok!(LBPPallet::add_liquidity(Origin::signed(user), pool_id, added_a, 0,));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);

		assert_eq!(balance_a_after, balance_a_before.saturating_add(added_a));
		assert_eq!(balance_b_after, balance_b_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
			Event::AddLiquidity(pool_id, asset_a, asset_b, added_a, added_b).into(),
			Event::AddLiquidity(pool_id, asset_a, asset_b, added_a, 0).into(),
		]);
	});
}

#[test]
fn add_zero_liquidity_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let user_balance_b_before = Currency::free_balance(asset_b, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(user), pool_id, 0, 0,),
			Error::<Test>::CannotAddZeroLiquidity
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		expect_events(vec![Event::CreatePool(
			user,
			asset_a,
			asset_b,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn add_liquidity_insufficient_balance_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(user), pool_id, u128::MAX, 0,),
			Error::<Test>::InsufficientAssetBalance
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);
	});
}

#[test]
fn add_liquidity_after_sale_started_should_not_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(15);

		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let user_balance_b_before = Currency::free_balance(asset_b, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(user), pool_id, 1_000, 1_000,),
			Error::<Test>::SaleStarted
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		// sale ended at the block number 20
		System::set_block_number(30);

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let user_balance_b_before = Currency::free_balance(asset_b, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(user), pool_id, 1_000, 1_000,),
			Error::<Test>::SaleStarted
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		expect_events(vec![Event::CreatePool(
			user,
			asset_a,
			asset_b,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn remove_liquidity_should_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(5);

		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_ok!(LBPPallet::remove_liquidity(Origin::signed(user), pool_id, 1_000, 0,));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before.saturating_sub(1_000));
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_add(1_000));

		System::set_block_number(30);

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let user_balance_b_before = Currency::free_balance(asset_b, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		let removed_a = 10_000_000;
		let removed_b = 20_000_000;

		assert_ok!(LBPPallet::remove_liquidity(
			Origin::signed(user),
			pool_id,
			removed_a,
			removed_b,
		));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before.saturating_sub(removed_a));
		assert_eq!(balance_b_after, balance_b_before.saturating_sub(removed_b));

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_add(removed_a));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_add(removed_b));

		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_ok!(LBPPallet::remove_liquidity(Origin::signed(user), pool_id, removed_a, 0,));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);

		assert_eq!(balance_a_after, balance_a_before.saturating_sub(removed_a));
		assert_eq!(balance_b_after, balance_b_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
			Event::RemoveLiquidity(pool_id, asset_a, asset_b, 1_000, 0).into(),
			Event::RemoveLiquidity(pool_id, asset_a, asset_b, removed_a, removed_b).into(),
			Event::RemoveLiquidity(pool_id, asset_a, asset_b, removed_a, 0).into(),
		]);
	});
}

#[test]
fn remove_zero_liquidity_should_not_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(30);

		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let user_balance_b_before = Currency::free_balance(asset_b, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(user), pool_id, 0, 0,),
			Error::<Test>::CannotRemoveZeroLiquidity
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		expect_events(vec![Event::CreatePool(
			user,
			asset_a,
			asset_b,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn remove_liquidity_insufficient_reserve_should_not_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(30);

		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(user), pool_id, u128::MAX, 0,),
			Error::<Test>::LiquidityUnderflow
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);

		expect_events(vec![Event::CreatePool(
			user,
			asset_a,
			asset_b,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn remove_liquidity_during_sale_should_not_work() {
	predefined_test_ext().execute_with(|| {
		// sale started at the block number 10
		System::set_block_number(15);

		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(user), pool_id, 1_000, 0,),
			Error::<Test>::SaleNotEnded
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);

		expect_events(vec![Event::CreatePool(
			user,
			asset_a,
			asset_b,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn destroy_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(21);

		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let user_balance_b_before = Currency::free_balance(asset_b, &user);
		let user_balance_hdx_before = Currency::reserved_balance(HDX, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_ok!(LBPPallet::destroy_pool(Origin::signed(user), pool_id,));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, 0);
		assert_eq!(balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		assert_eq!(
			user_balance_a_after,
			user_balance_a_before.saturating_add(balance_a_before)
		);

		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(
			user_balance_b_after,
			user_balance_b_before.saturating_add(balance_b_before)
		);

		let user_balance_hdx_after = Currency::reserved_balance(HDX, &user);
		assert_eq!(
			user_balance_hdx_after,
			user_balance_hdx_before.saturating_sub(POOL_DEPOSIT)
		);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
			frame_system::Event::KilledAccount(pool_id).into(),
			Event::PoolDestroyed(pool_id, asset_a, asset_b, balance_a_before, balance_b_before).into(),
		]);
	});
}

#[test]
fn destroy_not_finalized_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let user = ALICE;
		let asset_a = ACA;
		let asset_b = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		let user_balance_a_before = Currency::free_balance(asset_a, &user);
		let user_balance_b_before = Currency::free_balance(asset_b, &user);
		let user_balance_hdx_before = Currency::reserved_balance(HDX, &user);
		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_noop!(
			LBPPallet::destroy_pool(Origin::signed(user), pool_id,),
			Error::<Test>::SaleNotEnded
		);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		let user_balance_hdx_after = Currency::reserved_balance(HDX, &user);
		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);

		assert_eq!(balance_a_before, balance_a_after);
		assert_eq!(balance_b_before, balance_b_after);
		assert_eq!(user_balance_a_before, user_balance_a_after);
		assert_eq!(user_balance_b_before, user_balance_b_after);
		assert_eq!(user_balance_hdx_before, user_balance_hdx_after);

		expect_events(vec![Event::CreatePool(
			user,
			asset_a,
			asset_b,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}
