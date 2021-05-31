use super::*;
pub use crate::mock::{
	run_to_block, Currency, Event as TestEvent, ExtBuilder, LBPPallet, Origin, System, Test, ACA, ALICE, BOB, DOT, ETH,
	HDX,
};
use crate::mock::{ACA_DOT_POOL_ID, HDX_DOT_POOL_ID, INITIAL_BALANCE};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub use primitives::{asset::AssetPair, traits::AMMTransfer, MAX_IN_RATIO, MAX_OUT_RATIO};

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			LBPAssetInfo {
				id: ACA,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 90,
			},
			LBPAssetInfo {
				id: DOT,
				amount: 2_000_000_000,
				initial_weight: 80,
				final_weight: 10,
			},
			(10u64, 20u64),
			WeightCurveType::Linear,
			true,
		));

		assert_eq!(
			<PoolData<Test>>::get(ACA_DOT_POOL_ID),
			Pool {
				owner: ALICE,
				start: 10u64,
				end: 20u64,
				assets: (ACA, DOT),
				initial_weights: (20, 80),
				final_weights: (90, 10),
				last_weight_update: 0_u64,
				last_weights: (20, 80),
				weight_curve: WeightCurveType::Linear,
				pausable: true,
				paused: false,
			}
		);
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
fn weight_update_should_work() {
	new_test_ext().execute_with(|| {
		let asset_a = LBPAssetInfo {
			id: ACA,
			amount: 1,
			initial_weight: 20,
			final_weight: 80,
		};
		let asset_b = LBPAssetInfo {
			id: DOT,
			amount: 2,
			initial_weight: 80,
			final_weight: 20,
		};
		let duration = (10u64, 19u64);

		let mut linear_pool = Pool::new(ALICE, asset_a, asset_b, duration, WeightCurveType::Linear, false);

		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			asset_a,
			asset_b,
			duration,
			WeightCurveType::Linear,
			false
		));

		System::set_block_number(13);

		assert_ok!(LBPPallet::update_weights(&ACA_DOT_POOL_ID, &mut linear_pool));

		let mut linear_pool = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(linear_pool.last_weight_update, 13);
		assert_eq!(linear_pool.last_weights, (40u128, 60u128));

		// call update again in the same block, data should be the same
		assert_ok!(LBPPallet::update_weights(&ACA_DOT_POOL_ID, &mut linear_pool));

		let linear_pool = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(linear_pool.last_weight_update, 13);
		assert_eq!(linear_pool.last_weights, (40u128, 60u128));
	});
}

#[test]
fn validate_pool_data_should_work() {
	new_test_ext().execute_with(|| {
		let pool_data = Pool {
			owner: ALICE,
			start: 10u64,
			end: 20u64,
			assets: (ACA, DOT),
			initial_weights: (20, 80),
			final_weights: (90, 10),
			last_weight_update: 0u64,
			last_weights: (20, 80),
			weight_curve: WeightCurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		// null interval
		let pool_data = Pool {
			owner: ALICE,
			start: 0u64,
			end: 0u64,
			assets: (ACA, DOT),
			initial_weights: (20, 80),
			final_weights: (90, 10),
			last_weight_update: 0u64,
			last_weights: (20, 80),
			weight_curve: WeightCurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		let pool_data = Pool {
			owner: ALICE,
			start: 10u64,
			end: 2u64,
			assets: (ACA, DOT),
			initial_weights: (20, 80),
			final_weights: (90, 10),
			last_weight_update: 0u64,
			last_weights: (20, 80),
			weight_curve: WeightCurveType::Linear,
			pausable: true,
			paused: false,
		};
		assert_noop!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::InvalidBlockNumber
		);

		let pool_data = Pool {
			owner: ALICE,
			start: 10u64,
			end: 11u64 + u32::MAX as u64,
			assets: (ACA, DOT),
			initial_weights: (20, 80),
			final_weights: (90, 10),
			last_weight_update: 0u64,
			last_weights: (20, 80),
			weight_curve: WeightCurveType::Linear,
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
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			LBPAssetInfo {
				id: ACA,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 90,
			},
			LBPAssetInfo {
				id: DOT,
				amount: 2_000_000_000,
				initial_weight: 80,
				final_weight: 10,
			},
			(10u64, 20u64),
			WeightCurveType::Linear,
			true,
		));

		assert_eq!(Currency::free_balance(ACA, &ACA_DOT_POOL_ID), 1_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &ACA_DOT_POOL_ID), 2_000_000_000);
		assert_eq!(
			Currency::free_balance(ACA, &ALICE),
			INITIAL_BALANCE.saturating_sub(1_000_000_000)
		);
		assert_eq!(
			Currency::free_balance(DOT, &ALICE),
			INITIAL_BALANCE.saturating_sub(2_000_000_000)
		);

		let pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(pool_data.owner, ALICE);
		assert_eq!(pool_data.start, 10u64);
		assert_eq!(pool_data.end, 20u64);
		assert_eq!(pool_data.assets, (ACA, DOT));
		assert_eq!(pool_data.initial_weights, (20, 80));
		assert_eq!(pool_data.final_weights, (90, 10));
		assert_eq!(pool_data.weight_curve, WeightCurveType::Linear);
		assert_eq!(pool_data.pausable, true);
		// verify that `last_weight_update`, `last_weights` and `paused` fields are correctly initialized
		assert_eq!(pool_data.last_weight_update, 0);
		assert_eq!(pool_data.last_weights, (20, 80));
		assert_eq!(pool_data.paused, false);

		expect_events(vec![Event::PoolCreated(
			ALICE,
			ACA_DOT_POOL_ID,
			ACA,
			DOT,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn create_pool_from_basic_origin_should_not_work() {
	new_test_ext().execute_with(|| {
		// only CreatePoolOrigin is allowed to create new pools
		assert_noop!(
			LBPPallet::create_pool(
				Origin::signed(ALICE),
				ALICE,
				LBPAssetInfo {
					id: HDX,
					amount: 1_000_000_000,
					initial_weight: 20,
					final_weight: 90,
				},
				LBPAssetInfo {
					id: DOT,
					amount: 2_000_000_000,
					initial_weight: 80,
					final_weight: 10,
				},
				(10u64, 20u64),
				WeightCurveType::Linear,
				true,
			),
			BadOrigin
		);
	});
}

#[test]
fn create_same_pool_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			LBPAssetInfo {
				id: ACA,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 90,
			},
			LBPAssetInfo {
				id: DOT,
				amount: 2_000_000_000,
				initial_weight: 80,
				final_weight: 10,
			},
			(10u64, 20u64),
			WeightCurveType::Linear,
			true,
		));

		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				LBPAssetInfo {
					id: ACA,
					amount: 10_000_000_000,
					initial_weight: 30,
					final_weight: 70,
				},
				LBPAssetInfo {
					id: DOT,
					amount: 20_000_000_000,
					initial_weight: 70,
					final_weight: 30,
				},
				(100u64, 200u64),
				WeightCurveType::Linear,
				true,
			),
			Error::<Test>::PoolAlreadyExists
		);

		expect_events(vec![Event::PoolCreated(
			ALICE,
			ACA_DOT_POOL_ID,
			ACA,
			DOT,
			1_000_000_000,
			2_000_000_000,
		)
		.into()]);
	});
}

#[test]
fn create_pool_with_same_assets_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				LBPAssetInfo {
					id: ACA,
					amount: 1_000_000_000,
					initial_weight: 20,
					final_weight: 90,
				},
				LBPAssetInfo {
					id: ACA,
					amount: 2_000_000_000,
					initial_weight: 80,
					final_weight: 10,
				},
				(20u64, 10u64),
				WeightCurveType::Linear,
				true,
			),
			Error::<Test>::CannotCreatePoolWithSameAssets
		);
	});
}

#[test]
fn create_pool_with_zero_liquidity_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				LBPAssetInfo {
					id: HDX,
					amount: 0,
					initial_weight: 20,
					final_weight: 90,
				},
				LBPAssetInfo {
					id: DOT,
					amount: 0,
					initial_weight: 80,
					final_weight: 10,
				},
				(10u64, 20u64),
				WeightCurveType::Linear,
				true,
			),
			Error::<Test>::CannotCreatePoolWithZeroLiquidity
		);

		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				LBPAssetInfo {
					id: HDX,
					amount: 0,
					initial_weight: 20,
					final_weight: 90,
				},
				LBPAssetInfo {
					id: DOT,
					amount: 2_000_000_000,
					initial_weight: 80,
					final_weight: 10,
				},
				(10u64, 20u64),
				WeightCurveType::Linear,
				true,
			),
			Error::<Test>::CannotCreatePoolWithZeroLiquidity
		);
	});
}

#[test]
fn create_pool_invalid_data_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				LBPAssetInfo {
					id: ACA,
					amount: 1_000_000_000,
					initial_weight: 20,
					final_weight: 90,
				},
				LBPAssetInfo {
					id: DOT,
					amount: 2_000_000_000,
					initial_weight: 80,
					final_weight: 10,
				},
				(20u64, 10u64), // reversed interval, the end precedes the beginning
				WeightCurveType::Linear,
				true,
			),
			Error::<Test>::InvalidBlockNumber
		);
	});
}

#[test]
fn update_pool_data_should_work() {
	predefined_test_ext().execute_with(|| {
		// update all parameters
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			Some(15),
			Some(18),
			Some(((ACA, 10), (DOT, 90))),
			Some(((ACA, 80), (DOT, 20))),
		));

		// verify changes
		let updated_pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data.start, 15);
		assert_eq!(updated_pool_data.end, 18);
		assert_eq!(updated_pool_data.initial_weights, (10, 90));
		assert_eq!(updated_pool_data.final_weights, (80, 20));

		// update only one parameter
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(30),
			None,
			None,
		));

		// verify changes
		let updated_pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data.start, 15);
		assert_eq!(updated_pool_data.end, 30);
		assert_eq!(updated_pool_data.initial_weights, (10, 90));
		assert_eq!(updated_pool_data.final_weights, (80, 20));

		// update only one parameter
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			None,
			Some(((ACA, 10), (DOT, 70))),
			None,
		));

		// verify changes
		let updated_pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data.start, 15);
		assert_eq!(updated_pool_data.end, 30);
		assert_eq!(updated_pool_data.initial_weights, (10, 70));
		assert_eq!(updated_pool_data.final_weights, (80, 20));

		// mix
		assert_ok!(LBPPallet::update_pool_data(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			None,
			Some(18),
			Some(((ACA, 10), (DOT, 90))),
			None,
		));

		// verify changes
		let updated_pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data.start, 15);
		assert_eq!(updated_pool_data.end, 18);
		assert_eq!(updated_pool_data.initial_weights, (10, 90));
		assert_eq!(updated_pool_data.final_weights, (80, 20));

		expect_events(vec![
			Event::PoolUpdated(ALICE, ACA_DOT_POOL_ID).into(),
			Event::PoolUpdated(ALICE, ACA_DOT_POOL_ID).into(),
			Event::PoolUpdated(ALICE, ACA_DOT_POOL_ID).into(),
		]);
	});
}

#[test]
fn update_pool_data_without_changes_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::update_pool_data(Origin::signed(ALICE), ACA_DOT_POOL_ID, None, None, None, None,),
			Error::<Test>::NothingToUpdate
		);
	});
}

#[test]
fn update_pool_data_by_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(BOB),
				ACA_DOT_POOL_ID,
				Some(15),
				None,
				Some(((ACA, 10), (DOT, 90))),
				None,
			),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn update_pool_data_for_running_lbp_should_not_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(16);

		// update starting block and final weights
		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				Some(15),
				None,
				Some(((ACA, 10), (DOT, 90))),
				None,
			),
			Error::<Test>::SaleStarted
		);

		let updated_pool_data = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(updated_pool_data.start, 10);
		assert_eq!(updated_pool_data.end, 20);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
		]);
	});
}

#[test]
fn update_pool_data_with_wrong_asset_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::update_pool_data(
				Origin::signed(ALICE),
				ACA_DOT_POOL_ID,
				None,
				None,
				Some(((HDX, 10), (DOT, 90))),
				None,
			),
			Error::<Test>::InvalidAsset
		);
	});
}

#[test]
fn pause_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::pause_pool(Origin::signed(ALICE), ACA_DOT_POOL_ID));

		let paused_pool = LBPPallet::pool_data(ACA_DOT_POOL_ID);
		assert_eq!(
			paused_pool,
			Pool {
				owner: ALICE,
				start: 10u64,
				end: 20u64,
				assets: (ACA, DOT),
				initial_weights: (20, 80),
				final_weights: (90, 10),
				last_weight_update: 0u64,
				last_weights: (20, 80),
				weight_curve: WeightCurveType::Linear,
				pausable: true,
				paused: true,
			}
		);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::Paused(ALICE, ACA_DOT_POOL_ID).into(),
		]);
	});
}

#[test]
fn pause_non_existing_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let non_existing_id = 25486;
		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(ALICE), non_existing_id),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn pause_pool_by_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		//user is not pool owner
		let not_owner = BOB;
		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(not_owner), ACA_DOT_POOL_ID),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn pause_non_pausable_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		//pool is not pausable
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			BOB,
			LBPAssetInfo {
				id: ACA,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 40,
			},
			LBPAssetInfo {
				id: ETH,
				amount: 2_000_000_000,
				initial_weight: 80,
				final_weight: 60,
			},
			(200u64, 400u64),
			WeightCurveType::Linear,
			false,
		));

		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(BOB), 2_004_000),
			Error::<Test>::PoolIsNotPausable
		);
	});
}

#[test]
fn pause_paused_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			BOB,
			LBPAssetInfo {
				id: DOT,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 40,
			},
			LBPAssetInfo {
				id: ETH,
				amount: 2_000_000_000,
				initial_weight: 80,
				final_weight: 60,
			},
			(200u64, 400u64),
			WeightCurveType::Linear,
			true,
		));

		//pause the pool - pool is created as unpaused by default
		assert_ok!(LBPPallet::pause_pool(Origin::signed(BOB), 3_004_000));

		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(BOB), 3_004_000),
			Error::<Test>::CannotPausePausedPool
		);
	});
}

#[test]
fn pause_non_running_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		//pool is ended or ending in current block
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			LBPAssetInfo {
				id: DOT,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 40,
			},
			LBPAssetInfo {
				id: HDX,
				amount: 2_000_000_000,
				initial_weight: 80,
				final_weight: 60,
			},
			(200u64, 400u64),
			WeightCurveType::Linear,
			true,
		));

		run_to_block(400);
		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(ALICE), HDX_DOT_POOL_ID),
			Error::<Test>::CannotPauseEndedPool
		);

		run_to_block(500);
		assert_noop!(
			LBPPallet::pause_pool(Origin::signed(ALICE), HDX_DOT_POOL_ID),
			Error::<Test>::CannotPauseEndedPool
		);
	});
}

#[test]
fn unpause_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			LBPAssetInfo {
				id: DOT,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 40,
			},
			LBPAssetInfo {
				id: HDX,
				amount: 2_000_000_000,
				initial_weight: 80,
				final_weight: 60,
			},
			(200u64, 400u64),
			WeightCurveType::Linear,
			true,
		));

		//pool is created as unpaused by default
		assert_ok!(LBPPallet::pause_pool(Origin::signed(ALICE), HDX_DOT_POOL_ID));
		assert_ok!(LBPPallet::unpause_pool(Origin::signed(ALICE), HDX_DOT_POOL_ID,));

		let unpaused_pool = LBPPallet::pool_data(HDX_DOT_POOL_ID);
		assert_eq!(
			unpaused_pool,
			Pool {
				owner: ALICE,
				start: 200_u64,
				end: 400_u64,
				assets: (HDX, DOT),
				initial_weights: (80, 20),
				final_weights: (60, 40),
				last_weight_update: 0u64,
				last_weights: (80, 20),
				weight_curve: WeightCurveType::Linear,
				pausable: true,
				paused: false,
			}
		);

		expect_events(vec![
			Event::PoolCreated(ALICE, HDX_DOT_POOL_ID, DOT, HDX, 1_000_000_000, 2_000_000_000).into(),
			Event::Paused(ALICE, HDX_DOT_POOL_ID).into(),
			Event::Unpaused(ALICE, HDX_DOT_POOL_ID).into(),
		]);
	});
}

#[test]
fn unpause_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		//user is not pool owner
		let not_owner = BOB;
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(not_owner), ACA_DOT_POOL_ID),
			Error::<Test>::NotOwner
		);

		//pool is not found
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(ALICE), 24568),
			Error::<Test>::PoolNotFound
		);

		//predefined_test_ext pool is unpaused
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(ALICE), ACA_DOT_POOL_ID),
			Error::<Test>::PoolIsNotPaused
		);

		//pool is ended or ending in current block - pool is unpaused by default
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			LBPAssetInfo {
				id: DOT,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 40,
			},
			LBPAssetInfo {
				id: HDX,
				amount: 2_000_000_000,
				initial_weight: 80,
				final_weight: 60,
			},
			(200u64, 400u64),
			WeightCurveType::Linear,
			true,
		));

		// pause the pool before trying to unpause it
		assert_ok!(LBPPallet::pause_pool(Origin::signed(ALICE), HDX_DOT_POOL_ID,));

		run_to_block(400);
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(ALICE), HDX_DOT_POOL_ID),
			Error::<Test>::CannotUnpauseEndedPool
		);

		run_to_block(500);
		assert_noop!(
			LBPPallet::unpause_pool(Origin::signed(ALICE), HDX_DOT_POOL_ID),
			Error::<Test>::CannotUnpauseEndedPool
		);
	});
}

#[test]
fn add_liquidity_should_work() {
	predefined_test_ext().execute_with(|| {
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);
		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		let added_a = 10_000_000_000;
		let added_b = 20_000_000_000;

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			(ACA, added_a),
			(DOT, added_b),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);
		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(added_a));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_add(added_b));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(added_a));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(added_b));

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);
		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			(ACA, added_a),
			(DOT, 0),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);
		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(added_a));
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(added_a));
		assert_eq!(user_balance_b_after, user_balance_b_before);

		// change asset order
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);
		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			(DOT, added_b),
			(ACA, added_a),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);
		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(added_a));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_add(added_b));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(added_a));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(added_b));

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, added_a, added_b).into(),
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, added_a, 0).into(),
			Event::LiquidityAdded(ACA_DOT_POOL_ID, DOT, ACA, added_b, added_a).into(),
		]);
	});
}

#[test]
fn add_liquidity_by_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::add_liquidity(
				Origin::signed(BOB),
				ACA_DOT_POOL_ID,
				(ACA, 10_000_000_000),
				(DOT, 20_000_000_000),
			),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn add_zero_liquidity_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID, (ACA, 0), (DOT, 0),),
			Error::<Test>::CannotAddZeroLiquidity
		);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before);
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
		]);
	});
}

#[test]
fn add_liquidity_insufficient_balance_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID, (ACA, u128::MAX), (DOT, 0),),
			Error::<Test>::InsufficientAssetBalance
		);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before);
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before);
	});
}

#[test]
fn add_liquidity_after_sale_started_should_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(15);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(
			LBPPallet::add_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID, (ACA, 1_000), (DOT, 1_000),));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(1_000));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_add(1_000));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);

		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(1_000));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(1_000));

		// sale ended at the block number 20
		System::set_block_number(30);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(
			LBPPallet::add_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID, (ACA, 1_000), (DOT, 1_000),));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_add(1_000));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_add(1_000));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);

		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_sub(1_000));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_sub(1_000));

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, 1_000, 1_000).into(),
			Event::LiquidityAdded(ACA_DOT_POOL_ID, ACA, DOT, 1_000, 1_000).into(),
		]);
	});
}

#[test]
fn add_wrong_liquidity_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::add_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID, (ACA, 1_000), (HDX, 1_000),),
			Error::<Test>::InvalidAsset
		);
	});
}

#[test]
fn remove_liquidity_should_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(5);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::remove_liquidity(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			(ACA, 1_000),
			(DOT, 0),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_sub(1_000));
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_add(1_000));

		System::set_block_number(30);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		let removed_a = 10_000_000;
		let removed_b = 20_000_000;

		assert_ok!(LBPPallet::remove_liquidity(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			(ACA, removed_a),
			(DOT, removed_b),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_sub(removed_a));
		assert_eq!(pool_balance_b_after, pool_balance_b_before.saturating_sub(removed_b));

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_add(removed_a));
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_add(removed_b));

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::remove_liquidity(
			Origin::signed(ALICE),
			ACA_DOT_POOL_ID,
			(ACA, removed_a),
			(DOT, 0),
		));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before.saturating_sub(removed_a));
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::LiquidityRemoved(ACA_DOT_POOL_ID, ACA, DOT, 1_000, 0).into(),
			Event::LiquidityRemoved(ACA_DOT_POOL_ID, ACA, DOT, removed_a, removed_b).into(),
			Event::LiquidityRemoved(ACA_DOT_POOL_ID, ACA, DOT, removed_a, 0).into(),
		]);
	});
}

#[test]
fn remove_liquidity_by_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::remove_liquidity(
				Origin::signed(BOB),
				ACA_DOT_POOL_ID,
				(ACA, 10_000_000_000),
				(DOT, 20_000_000_000),
			),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn remove_zero_liquidity_should_not_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(30);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID, (ACA, 0), (DOT, 0),),
			Error::<Test>::CannotRemoveZeroLiquidity
		);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before);
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);

		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
		]);
	});
}

#[test]
fn remove_liquidity_insufficient_reserve_should_not_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(30);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID, (ACA, u128::MAX), (DOT, 0),),
			Error::<Test>::InsufficientAssetBalance
		);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before);
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
		]);
	});
}

#[test]
fn remove_liquidity_during_sale_should_not_work() {
	predefined_test_ext().execute_with(|| {
		// sale started at the block number 10
		System::set_block_number(15);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::remove_liquidity(Origin::signed(ALICE), ACA_DOT_POOL_ID, (ACA, 1_000), (DOT, 0),),
			Error::<Test>::SaleNotEnded
		);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, pool_balance_a_before);
		assert_eq!(pool_balance_b_after, pool_balance_b_before);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(user_balance_a_after, user_balance_a_before);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
		]);
	});
}

#[test]
fn destroy_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(21);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::destroy_pool(Origin::signed(ALICE), ACA_DOT_POOL_ID,));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, 0);
		assert_eq!(pool_balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(
			user_balance_a_after,
			user_balance_a_before.saturating_add(pool_balance_a_before)
		);

		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(
			user_balance_b_after,
			user_balance_b_before.saturating_add(pool_balance_b_before)
		);

		assert_eq!(<PoolData<Test>>::contains_key(ACA_DOT_POOL_ID), false);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			frame_system::Event::KilledAccount(ACA_DOT_POOL_ID).into(),
			Event::PoolDestroyed(ACA_DOT_POOL_ID, ACA, DOT, pool_balance_a_before, pool_balance_b_before).into(),
		]);
	});
}

#[test]
fn destroy_not_started_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::destroy_pool(Origin::signed(ALICE), ACA_DOT_POOL_ID,));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, 0);
		assert_eq!(pool_balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(
			user_balance_a_after,
			user_balance_a_before.saturating_add(pool_balance_a_before)
		);

		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(
			user_balance_b_after,
			user_balance_b_before.saturating_add(pool_balance_b_before)
		);

		assert_eq!(<PoolData<Test>>::contains_key(ACA_DOT_POOL_ID), false);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			frame_system::Event::KilledAccount(ACA_DOT_POOL_ID).into(),
			Event::PoolDestroyed(ACA_DOT_POOL_ID, ACA, DOT, pool_balance_a_before, pool_balance_b_before).into(),
		]);

		// sale duration is not specified
		assert_ok!(LBPPallet::create_pool(
				Origin::root(),
				ALICE,
				LBPAssetInfo {
					id: HDX,
					amount: 1_000_000_000,
					initial_weight: 20,
					final_weight: 90,
				},
				LBPAssetInfo {
					id: DOT,
					amount: 2_000_000_000,
					initial_weight: 80,
					final_weight: 10,
				},
				(0u64, 0u64),
				WeightCurveType::Linear,
				true,
			));
	});
}

#[test]
fn destroy_not_finalized_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(15);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_noop!(
			LBPPallet::destroy_pool(Origin::signed(ALICE), ACA_DOT_POOL_ID,),
			Error::<Test>::SaleNotEnded
		);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_before, pool_balance_a_after);
		assert_eq!(pool_balance_b_before, pool_balance_b_after);
		assert_eq!(user_balance_a_before, user_balance_a_after);
		assert_eq!(user_balance_b_before, user_balance_b_after);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
		]);
	});
}

#[test]
fn destroy_finalized_pool_should_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(21);

		let user_balance_a_before = Currency::free_balance(ACA, &ALICE);
		let user_balance_b_before = Currency::free_balance(DOT, &ALICE);

		let pool_balance_a_before = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_before = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_ok!(LBPPallet::destroy_pool(Origin::signed(ALICE), ACA_DOT_POOL_ID,));

		let pool_balance_a_after = Currency::free_balance(ACA, &ACA_DOT_POOL_ID);
		let pool_balance_b_after = Currency::free_balance(DOT, &ACA_DOT_POOL_ID);

		assert_eq!(pool_balance_a_after, 0);
		assert_eq!(pool_balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(ACA, &ALICE);
		assert_eq!(
			user_balance_a_after,
			user_balance_a_before.saturating_add(pool_balance_a_before)
		);

		let user_balance_b_after = Currency::free_balance(DOT, &ALICE);
		assert_eq!(
			user_balance_b_after,
			user_balance_b_before.saturating_add(pool_balance_b_before)
		);

		assert_eq!(<PoolData<Test>>::contains_key(ACA_DOT_POOL_ID), false);

		expect_events(vec![
			Event::PoolCreated(ALICE, ACA_DOT_POOL_ID, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			frame_system::Event::KilledAccount(ACA_DOT_POOL_ID).into(),
			Event::PoolDestroyed(ACA_DOT_POOL_ID, ACA, DOT, pool_balance_a_before, pool_balance_b_before).into(),
		]);
	});
}

#[test]
fn destroy_pool_by_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::destroy_pool(Origin::signed(BOB), ACA_DOT_POOL_ID),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn execute_trade_should_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 5_000_000_u128;
		let amount_out = 10_000_000_u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_ok!(LBPPallet::execute_trade(&t));

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_998_995_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_010_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_005_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_990_000_000);
	});
}

// This test ensure storage was not modified on error
#[test]
fn execute_trade_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 5_000_000_u128;
		let amount_out = 10_000_000_000_000_000u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_noop!(LBPPallet::execute_trade(&t), orml_tokens::Error::<Test>::BalanceTooLow);

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);
	});
}

#[test]
fn execute_sell_should_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 8_000_000_u128;
		let amount_out = 20_000_000_u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_ok!(LBPPallet::execute_sell(&t));

		expect_events(vec![Event::SellExecuted(
			ALICE, asset_in, asset_out, amount_in, amount_out,
		)
		.into()]);

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_998_992_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_020_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_008_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_980_000_000);

		expect_events(vec![Event::SellExecuted(
			ALICE, asset_in, asset_out, 8_000_000, 20_000_000,
		)
		.into()]);
	});
}

// This test ensure storage was not modified on error
#[test]
fn execute_sell_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 8_000_000_000_u128;
		let amount_out = 200_000_000_000_000_u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_noop!(LBPPallet::execute_sell(&t), orml_tokens::Error::<Test>::BalanceTooLow);

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);
	});
}

#[test]
fn execute_buy_should_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 8_000_000_u128;
		let amount_out = 20_000_000_u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_ok!(LBPPallet::execute_buy(&t));

		expect_events(vec![Event::BuyExecuted(
			ALICE, asset_out, asset_in, amount_in, amount_out,
		)
		.into()]);

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_998_992_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_020_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_008_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_980_000_000);

		expect_events(vec![Event::BuyExecuted(
			ALICE, asset_out, asset_in, 8_000_000, 20_000_000,
		)
		.into()]);
	});
}

// This test ensure storage was not modified on error
#[test]
fn execute_buy_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		let amount_in = 8_000_000_000_u128;
		let amount_out = 200_000_000_000_000_u128;
		let t = AMMTransfer {
			origin: ALICE,
			assets: AssetPair { asset_in, asset_out },
			amount: amount_in,
			amount_out,
			discount: false,
			discount_amount: 0_u128,
		};

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);

		assert_noop!(LBPPallet::execute_buy(&t), orml_tokens::Error::<Test>::BalanceTooLow);

		assert_eq!(Currency::free_balance(asset_in, &ALICE), 999_999_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &ALICE), 999_998_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000);
	});
}

#[test]
fn sell_zero_amount_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::sell(Origin::signed(BOB), ACA, DOT, 0_u128, 200_000_u128),
			Error::<Test>::ZeroAmount
		);
	});
}

#[test]
fn buy_zero_amount_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::buy(Origin::signed(BOB), ACA, DOT, 0_u128, 200_000_u128),
			Error::<Test>::ZeroAmount
		);
	});
}

#[test]
fn sell_to_non_existing_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::sell(Origin::signed(BOB), ACA, ETH, 800_000_u128, 200_000_u128),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn buy_from_non_existing_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LBPPallet::buy(Origin::signed(BOB), ACA, ETH, 800_000_u128, 200_000_u128),
			Error::<Test>::PoolNotFound
		);
	});
}

#[test]
fn exceed_max_in_ratio_should_not_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block(11); //start sale
		assert_noop!(
			LBPPallet::sell(
				Origin::signed(BOB),
				ACA,
				DOT,
				1_000_000_000 / MAX_IN_RATIO + 1,
				200_000_u128
			),
			Error::<Test>::MaxInRatioExceeded
		);

		//1/2 should not work
		assert_noop!(
			LBPPallet::sell(Origin::signed(BOB), ACA, DOT, 1_000_000_000 / 2, 200_000_u128),
			Error::<Test>::MaxInRatioExceeded
		);

		//max ratio should work
		assert_ok!(LBPPallet::sell(
			Origin::signed(BOB),
			ACA,
			DOT,
			1_000_000_000 / MAX_IN_RATIO,
			2_000_u128
		));
	});
}

#[test]
fn exceed_max_out_ratio_should_not_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block(11); //start sale

		//max_ratio_out + 1 should not work
		assert_noop!(
			LBPPallet::buy(
				Origin::signed(BOB),
				ACA,
				DOT,
				1_000_000_000 / MAX_OUT_RATIO + 1,
				200_000_u128
			),
			Error::<Test>::MaxOutRatioExceeded
		);

		//1/2 should not work
		assert_noop!(
			LBPPallet::buy(Origin::signed(BOB), ACA, DOT, 1_000_000_000 / 2, 200_000_u128),
			Error::<Test>::MaxOutRatioExceeded
		);

		//max ratio should work
		assert_ok!(LBPPallet::buy(
			Origin::signed(BOB),
			ACA,
			DOT,
			1_000_000_000 / MAX_OUT_RATIO,
			2_000_000_000_u128
		));
	});
}

#[test]
fn trade_in_non_running_pool_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let amount = 800_000_u128;
		let limit = 200_000_u128;

		//sale not started
		run_to_block(9);
		assert_noop!(
			LBPPallet::sell(Origin::signed(who), asset_in, asset_out, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);
		assert_noop!(
			LBPPallet::buy(Origin::signed(who), asset_in, asset_out, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);

		//sale ended
		run_to_block(21);
		assert_noop!(
			LBPPallet::sell(Origin::signed(who), asset_in, asset_out, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);
		assert_noop!(
			LBPPallet::buy(Origin::signed(who), asset_in, asset_out, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);

		//unpaused pool - pool is created as unpaused by default
		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			ALICE,
			LBPAssetInfo {
				id: HDX,
				amount: 1_000_000_000,
				initial_weight: 20,
				final_weight: 90,
			},
			LBPAssetInfo {
				id: ETH,
				amount: 10_000,
				initial_weight: 80,
				final_weight: 10,
			},
			(30u64, 40u64),
			WeightCurveType::Linear,
			true,
		));

		assert_ok!(LBPPallet::pause_pool(Origin::signed(ALICE), 4_000));
		//pool started but is paused
		run_to_block(30);
		assert_noop!(
			LBPPallet::sell(Origin::signed(BOB), HDX, ETH, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);
		assert_noop!(
			LBPPallet::buy(Origin::signed(BOB), HDX, ETH, amount, limit),
			Error::<Test>::SaleIsNotRunning
		);
	});
}

//AssetBalanceLimitExceeded
#[test]
fn exceed_trader_limit_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let amount = 800_000_u128;
		let sell_limit = 800_000_u128;
		let buy_limit = 1_000_u128;

		//start sale
		run_to_block(11);
		assert_noop!(
			LBPPallet::sell(Origin::signed(who), asset_in, asset_out, amount, sell_limit),
			Error::<Test>::AssetBalanceLimitExceeded
		);

		assert_noop!(
			LBPPallet::buy(Origin::signed(who), asset_in, asset_out, amount, buy_limit),
			Error::<Test>::AssetBalanceLimitExceeded
		);
	});
}

#[test]
fn sell_with_insufficient_balance_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = ETH;
		let amount = 800_000_u128;

		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			who,
			LBPAssetInfo {
				id: asset_in,
				amount: INITIAL_BALANCE,
				initial_weight: 20,
				final_weight: 90,
			},
			LBPAssetInfo {
				id: asset_out,
				amount: 10_000,
				initial_weight: 80,
				final_weight: 10,
			},
			(30u64, 40u64),
			WeightCurveType::Linear,
			true,
		));

		//start sale
		run_to_block(31);
		assert_noop!(
			LBPPallet::sell(Origin::signed(who), asset_in, asset_out, amount, 2_u128),
			Error::<Test>::InsufficientAssetBalance
		);
	});
}

#[test]
fn buy_with_insufficient_balance_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = ETH;
		let amount = 800_000_u128;

		assert_ok!(LBPPallet::create_pool(
			Origin::root(),
			who,
			LBPAssetInfo {
				id: asset_in,
				amount: INITIAL_BALANCE,
				initial_weight: 20,
				final_weight: 90,
			},
			LBPAssetInfo {
				id: asset_out,
				amount: 10_000,
				initial_weight: 80,
				final_weight: 10,
			},
			(30u64, 40u64),
			WeightCurveType::Linear,
			true,
		));

		//start sale
		run_to_block(31);
		assert_noop!(
			LBPPallet::buy(
				Origin::signed(who),
				asset_out,
				asset_in,
				amount,
				2_000_000_000_000_000_000_000_u128
			),
			Error::<Test>::InsufficientAssetBalance
		);
	});
}

#[test]
fn buy_should_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		assert_eq!(Currency::free_balance(asset_in, &who), 1_000_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &who), 1_000_000_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000_u128);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000_u128);

		//start sale
		run_to_block(11);
		assert_ok!(LBPPallet::buy(
			Origin::signed(who),
			asset_out,
			asset_in,
			10_000_000_u128,
			2_000_000_000_u128
		));

		let pool = <PoolData<Test>>::get(pool_id);

		assert_eq!(
			Pool {
				owner: ALICE,
				start: 10u64,
				end: 20u64,
				assets: (asset_in, asset_out),
				initial_weights: (20, 80),
				final_weights: (90, 10),
				last_weight_update: 11u64,
				last_weights: (27, 73),
				weight_curve: WeightCurveType::Linear,
				pausable: true,
				paused: false,
			},
			pool
		);

		assert_eq!(Currency::free_balance(asset_in, &who), 999_999_986_327_783_u128);
		assert_eq!(Currency::free_balance(asset_out, &who), 1_000_000_010_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_013_672_217_u128);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_990_000_000_u128);

		expect_events(vec![Event::BuyExecuted(
			who, asset_out, asset_in, 13_672_217, 10_000_000,
		)
		.into()]);
	});
}

#[test]
fn sell_should_work() {
	predefined_test_ext().execute_with(|| {
		let who = BOB;
		let asset_in = ACA;
		let asset_out = DOT;
		let pool_id = LBPPallet::get_pair_id(AssetPair { asset_in, asset_out });

		assert_eq!(Currency::free_balance(asset_in, &who), 1_000_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_out, &who), 1_000_000_000_000_000);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_000_000_000_u128);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 2_000_000_000_u128);

		//start sale
		run_to_block(11);
		assert_ok!(LBPPallet::sell(
			Origin::signed(who),
			asset_in,
			asset_out,
			10_000_000_u128,
			2_000_u128
		));

		let pool = <PoolData<Test>>::get(pool_id);

		assert_eq!(
			Pool {
				owner: ALICE,
				start: 10u64,
				end: 20u64,
				assets: (asset_in, asset_out),
				initial_weights: (20, 80),
				final_weights: (90, 10),
				last_weight_update: 11u64,
				last_weights: (27, 73),
				weight_curve: WeightCurveType::Linear,
				pausable: true,
				paused: false,
			},
			pool
		);

		assert_eq!(Currency::free_balance(asset_in, &who), INITIAL_BALANCE - 10_000_000);
		assert_eq!(Currency::free_balance(asset_out, &who), 1_000_000_007_332_274);

		assert_eq!(Currency::free_balance(asset_in, &pool_id), 1_010_000_000_u128);
		assert_eq!(Currency::free_balance(asset_out, &pool_id), 1_992_667_726_u128);

		expect_events(vec![Event::SellExecuted(
			who, asset_in, asset_out, 10_000_000, 7_332_274,
		)
		.into()]);
	});
}

#[test]
fn amm_trait_should_work() {
	predefined_test_ext().execute_with(|| {
		let asset_pair = AssetPair {
			asset_in: ACA,
			asset_out: DOT,
		};
		let reversed_asset_pair = AssetPair {
			asset_in: DOT,
			asset_out: ACA,
		};
		let non_existing_asset_pair = AssetPair {
			asset_in: DOT,
			asset_out: HDX,
		};

		assert_eq!(LBPPallet::exists(asset_pair), true);
		assert_eq!(LBPPallet::exists(reversed_asset_pair), true);
		assert_eq!(LBPPallet::exists(non_existing_asset_pair), false);

		assert_eq!(LBPPallet::get_pair_id(asset_pair), ACA_DOT_POOL_ID);
		assert_eq!(LBPPallet::get_pair_id(reversed_asset_pair), ACA_DOT_POOL_ID);

		assert_eq!(LBPPallet::get_pool_assets(&ACA_DOT_POOL_ID), Some(vec![ACA, DOT]));
		assert_eq!(LBPPallet::get_pool_assets(&HDX_DOT_POOL_ID), None);

		// TODO: test all methods from the AMM trait
	});
}

#[test]
fn get_spot_price_should_work() {
	predefined_test_ext().execute_with(|| {
		System::set_block_number(10);

		let price = hydra_dx_math::lbp::calculate_spot_price(
			1_000_000_000_u128,
			2_000_000_000_u128,
			20_u128,
			80_u128,
			1_000_000_u128,
		)
		.unwrap_or_else(|_| BalanceOf::<Test>::zero());

		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, DOT, 1_000_000_u128), price);

		// swap assets
		let price = hydra_dx_math::lbp::calculate_spot_price(
			2_000_000_000_u128,
			1_000_000_000_u128,
			80_u128,
			20_u128,
			1_000_000_u128,
		)
		.unwrap_or_else(|_| BalanceOf::<Test>::zero());

		assert_eq!(LBPPallet::get_spot_price_unchecked(DOT, ACA, 1_000_000_u128), price);

		// change weights
		System::set_block_number(20);

		let price = hydra_dx_math::lbp::calculate_spot_price(
			1_000_000_000_u128,
			2_000_000_000_u128,
			90_u128,
			10_u128,
			1_000_000_u128,
		)
		.unwrap_or_else(|_| BalanceOf::<Test>::zero());

		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, DOT, 1_000_000), price);

		// pool does not exist
		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, HDX, 1_000_000), 0);

		// overflow
		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, DOT, u128::MAX), 0);

		// sale ended
		System::set_block_number(21);
		assert_eq!(LBPPallet::get_spot_price_unchecked(ACA, DOT, 1_000_000), 0);
	});
}

// TODO: test calculate_spot_price function
// TODO: test update_weights_and_validate_trade function
