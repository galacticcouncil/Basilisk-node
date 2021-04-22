use super::*;
pub use crate::mock::{
	Currency, Event as TestEvent, ExtBuilder, LBPPallet, Origin, System, Test, ACA, ALICE, BOB, CHARLIE, DOT, ETH, HDX,
};
use crate::mock::{INITIAL_BALANCE, POOL_ADDRESS, POOL_DEPOSIT, POOL_SWAP_FEE};
use frame_support::{assert_noop, assert_ok};

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
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
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {

		let asset_a = DOT;
		let asset_b = ACA;
		let amount_a = 1_000_000_000;
		let amount_b = 2_000_000_000;

		let pool_data = Pool {
			start: 10u64,
			end: 20u64,
			end_weights: (40, 60),
			curve: CurveType::Linear,
			pausable: true
		};

		assert_ok!(LBPPallet::create_pool(
			Origin::signed(ALICE),
			asset_a,
			amount_a,
			asset_b,
			amount_b,
			pool_data
		));

		let pool_account = LBPPallet::get_pair_id(AssetPair {
			asset_in: asset_a,
			asset_out: asset_b,
		});

		assert_eq!(Currency::free_balance(asset_a, &pool_account), amount_a);
		assert_eq!(Currency::free_balance(asset_b, &pool_account), amount_b);
		assert_eq!(Currency::free_balance(asset_a, &ALICE), INITIAL_BALANCE.saturating_sub(amount_a));
		assert_eq!(Currency::free_balance(asset_b, &ALICE), INITIAL_BALANCE.saturating_sub(amount_b));
		assert_eq!(Currency::reserved_balance(HDX, &ALICE), POOL_DEPOSIT);
		assert_eq!(Currency::free_balance(HDX, &ALICE), INITIAL_BALANCE.saturating_sub(POOL_DEPOSIT));

		expect_events(vec![Event::CreatePool(ALICE, asset_a, asset_b, amount_a, amount_b).into()]);
	});
}