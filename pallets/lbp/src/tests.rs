use super::*;
pub use crate::mock::{
	Currency, Event as TestEvent, ExtBuilder, LBPPallet, Origin, System, Test, ACA, ALICE, BOB, CHARLIE, DOT, ETH, HDX,
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
			initial_weights: (20, 80),
			final_weights: (90, 10),
			curve: CurveType::Linear,
			pausable: true,
		};

		let _ = LBPPallet::create_pool(
			Origin::signed(user),
			asset_a,
			amount_a,
			asset_b,
			amount_b,
			pool_data
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
fn validate_pool_data_should_work() {
	new_test_ext().execute_with(|| {
		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			initial_weights: (20, 80),
			final_weights: (90, 10),
			curve: CurveType::Linear,
			pausable: true,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		let pool_data = Pool {
			start: 0u64,
			end: 0u64,
			initial_weights: (20, 80),
			final_weights: (90, 10),
			curve: CurveType::Linear,
			pausable: true,
		};
		assert_ok!(LBPPallet::validate_pool_data(&pool_data));

		let pool_data = Pool {
			start: 10u64,
			end: 2u64,
			initial_weights: (20, 80),
			final_weights: (90, 10),
			curve: CurveType::Linear,
			pausable: true,
		};
		assert_err!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::BlockNumberInvalid
		);

		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			initial_weights: (20, 80),
			final_weights: (9_000_000, 10),
			curve: CurveType::Linear,
			pausable: true,
		};
		assert_err!(
			LBPPallet::validate_pool_data(&pool_data),
			Error::<Test>::MaxWeightExceeded
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
			initial_weights: (20, 80),
			final_weights: (90, 10),
			curve: CurveType::Linear,
			pausable: true,
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

		assert_eq!(
			LBPPallet::get_pool_assets(&pool_id).unwrap(),
			vec![asset_a, asset_b]
		);

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
			initial_weights: (20, 80),
			final_weights: (90, 10),
			curve: CurveType::Linear,
			pausable: true,
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
			initial_weights: (20, 80),
			final_weights: (90, 10),
			curve: CurveType::Linear,
			pausable: true,
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
			Some(new_final_weights),
			None,
		));

		let updated_pool_data = LBPPallet::pool_data(pool_id);
		assert_eq!(updated_pool_data.start, new_start);
		assert_eq!(updated_pool_data.end, 20);

		expect_events(vec![
			Event::CreatePool(user, ACA, DOT, 1_000_000_000, 2_000_000_000).into(),
			Event::UpdatePool(user, pool_id).into()
		]);
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
		let new_final_weights = (10, 90);

		assert_noop!(LBPPallet::update_pool_data(
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
			Event::AddLiquidity(pool_id, asset_a, asset_b, added_a, added_b).into()
		]);

		let (balance_a_before, balance_b_before) = LBPPallet::pool_balances(pool_id);

		assert_ok!(LBPPallet::add_liquidity(
			Origin::signed(user),
			pool_id,
			added_a,
			0,
		));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);

		assert_eq!(balance_a_after, balance_a_before.saturating_add(added_a));
		assert_eq!(balance_b_after, balance_b_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
			Event::AddLiquidity(pool_id, asset_a, asset_b, added_a, added_b).into(),
			Event::AddLiquidity(pool_id, asset_a, asset_b, added_a, 0).into()
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

		assert_noop!(LBPPallet::add_liquidity(
			Origin::signed(user),
			pool_id,
			0,
			0,
		),
			Error::<Test>::CannotAddZeroLiquidity
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
		]);
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

		assert_noop!(LBPPallet::add_liquidity(
			Origin::signed(user),
			pool_id,
			u128::MAX,
			0,
		),
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

		assert_noop!(LBPPallet::add_liquidity(
			Origin::signed(user),
			pool_id,
			1_000,
			1_000,
		),
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

		assert_noop!(LBPPallet::add_liquidity(
			Origin::signed(user),
			pool_id,
			1_000,
			1_000,
		),
			Error::<Test>::SaleStarted
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
		]);
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

		assert_ok!(LBPPallet::remove_liquidity(
			Origin::signed(user),
			pool_id,
			1_000,
			0,
		));

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

		assert_ok!(LBPPallet::remove_liquidity(
			Origin::signed(user),
			pool_id,
			removed_a,
			0,
		));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);

		assert_eq!(balance_a_after, balance_a_before.saturating_sub(removed_a));
		assert_eq!(balance_b_after, balance_b_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
			Event::RemoveLiquidity(pool_id, asset_a, asset_b, 1_000, 0).into(),
			Event::RemoveLiquidity(pool_id, asset_a, asset_b, removed_a, removed_b).into(),
			Event::RemoveLiquidity(pool_id, asset_a, asset_b, removed_a, 0).into()
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

		assert_noop!(LBPPallet::remove_liquidity(
			Origin::signed(user),
			pool_id,
			0,
			0,
		),
			Error::<Test>::CannotRemoveZeroLiquidity
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);
		assert_eq!(user_balance_b_after, user_balance_b_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
		]);
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

		assert_noop!(LBPPallet::remove_liquidity(
			Origin::signed(user),
			pool_id,
			u128::MAX,
			0,
		),
			Error::<Test>::LiquidityUnderflow
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
		]);
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

		assert_noop!(LBPPallet::remove_liquidity(
			Origin::signed(user),
			pool_id,
			1_000,
			0,
		),
			Error::<Test>::SaleNotEnded
		);

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, balance_a_before);
		assert_eq!(balance_b_after, balance_b_before);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before);

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
		]);
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

		assert_ok!(LBPPallet::destroy_pool(
			Origin::signed(user),
			pool_id,
		));

		let (balance_a_after, balance_b_after) = LBPPallet::pool_balances(pool_id);
		assert_eq!(balance_a_after, 0);
		assert_eq!(balance_b_after, 0);

		let user_balance_a_after = Currency::free_balance(asset_a, &user);
		assert_eq!(user_balance_a_after, user_balance_a_before.saturating_add(balance_a_before));

		let user_balance_b_after = Currency::free_balance(asset_b, &user);
		assert_eq!(user_balance_b_after, user_balance_b_before.saturating_add(balance_b_before));

		let user_balance_hdx_after = Currency::reserved_balance(HDX, &user);
		assert_eq!(user_balance_hdx_after, user_balance_hdx_before.saturating_sub(POOL_DEPOSIT));

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
			frame_system::Event::KilledAccount(pool_id).into(),
			Event::PoolDestroyed(pool_id, asset_a, asset_b, balance_a_before, balance_b_before).into(),
		]);
	});
}

#[test]
fn destroy_not_finalized_pool_should_not_work()
{
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

		assert_noop!(LBPPallet::destroy_pool(
			Origin::signed(user),
			pool_id,
		),
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

		expect_events(vec![
			Event::CreatePool(user, asset_a, asset_b, 1_000_000_000, 2_000_000_000).into(),
		]);
	});
}
