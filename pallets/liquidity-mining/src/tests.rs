use super::*;
use crate::mock::{
	Event as TestEvent, ExtBuilder, LiquidityMining, Origin, PoolId, System, Test, ALICE, BOB, BSX_ACA_POOL,
	BSX_ACA_SHARE_ID, BSX_DOT_POOL, BSX_DOT_SHARE_ID, BSX_ETH_POOL, BSX_ETH_SHARE_ID, CHARLIE, DAVE, DECIMALS
};
use frame_support::{assert_noop, assert_ok};
use primitives::{AssetId, Balance, BlockNumber};
use sp_runtime::traits::BadOrigin;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn create_pool_should_work() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL), None);

		mock::run_to_block(81_324);
		//default loyalty cureve params
		assert_ok!(LiquidityMining::create_pool(
			Origin::root(),
			BSX_ACA_POOL,
			BSX_ACA_SHARE_ID,
			None
		));

		expect_events(vec![Event::PoolCreated(BSX_ACA_POOL).into()]);
		assert_eq!(
			LiquidityMining::pool(BSX_ACA_POOL).unwrap(),
			get_default_pool(BSX_ACA_SHARE_ID, 81_u64, None),
		);

		mock::run_to_block(92_324);
		assert_eq!(LiquidityMining::pool(BSX_ETH_POOL), None);
		assert_ok!(LiquidityMining::create_pool(
			Origin::root(),
			BSX_ETH_POOL,
			BSX_ETH_SHARE_ID,
			Some(LoyaltyCurve {
				b: FixedU128::from_inner(235_000_000_000_000_000),
				scale_coef: 2,
			})
		));

		expect_events(vec![Event::PoolCreated(BSX_ETH_POOL).into()]);
		assert_eq!(
			LiquidityMining::pool(BSX_ETH_POOL).unwrap(),
			get_default_pool(
				BSX_ETH_SHARE_ID,
				92_u64,
				Some(LoyaltyCurve {
					b: FixedU128::from_inner(235_000_000_000_000_000),
					scale_coef: 2,
				})
			),
		);
	});
}

#[test]
fn create_pool_should_not_work() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		mock::run_to_block(1000);

		let mut orig_pool = get_default_pool(BSX_ACA_SHARE_ID, 1, None);
		orig_pool.accumulated_reward_per_share = FixedU128::from(2);
		orig_pool.total_locked_shares = Balance::from(100_000_u128);
		orig_pool.unpaid_rewards = Balance::from(10_000_u128);
		orig_pool.paid_rewards = Balance::from(5_000_u128);

		Pools::<Test>::insert(BSX_ACA_POOL, &orig_pool);
		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap(), orig_pool);
		assert_noop!(
			LiquidityMining::create_pool(Origin::root(), BSX_ACA_POOL, BSX_ACA_SHARE_ID, None),
			Error::<Test>::LmPoolExists
		);
		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap(), orig_pool);

		assert_eq!(LiquidityMining::pool(BSX_ETH_POOL), None);
		assert_noop!(
			LiquidityMining::create_pool(Origin::signed(ALICE), BSX_ETH_POOL, BSX_ETH_SHARE_ID, None),
			BadOrigin
		);
		assert_eq!(LiquidityMining::pool(BSX_ETH_POOL), None);
	});
}

#[test]
fn update_pool_should_work() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		mock::run_to_block(1_000);
		let orig_pool = get_default_pool(BSX_ACA_SHARE_ID, 1, None);
		let pool = &mut orig_pool.clone();

		//nothing should change in the same period
		assert_ok!(LiquidityMining::update_pool(BSX_ACA_POOL, pool));

		assert_eq!(orig_pool, *pool);

		mock::run_to_block(2_000);
		//+1_000 shasres locked
		pool.total_locked_shares = 1_000;
		assert_ok!(LiquidityMining::update_pool(BSX_ACA_POOL, pool));
		assert_eq!(
			*pool,
			LmPool {
				accumulated_reward_per_share: FixedU128::from(1),
				total_locked_shares: 1_000_u128,
				share_id: BSX_ACA_SHARE_ID,
				updated_at: 2_u64,
				unpaid_rewards: 1_000_u128,
				paid_rewards: 0_u128,
				canceled: false,
				loyalty_curve: LoyaltyCurve::default(),
			}
		);
       
        //nothing happend for few periods
		mock::run_to_block(5_000);  
		pool.total_locked_shares += 1_000_000; //lock additional shares
		assert_ok!(LiquidityMining::update_pool(BSX_ACA_POOL, pool));
		/*assert_eq!(
			*pool,
			LmPool {
				accumulated_reward_per_share: 1_u128,
				total_locked_shares: 1_000_u128,
				share_id: BSX_ACA_SHARE_ID,
				updated_at: 2_u64,
				unpaid_rewards: 1_000_u128,
				paid_rewards: 0_u128,
				canceled: false,
				loyalty_curve: LoyaltyCurve::default(),
			}
		);*/
        
	});
}
/*
#[test]
fn cancel_pool_should_work() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		mock::run_to_block(2000);

		let mut pool = get_default_pool(BSX_ACA_SHARE_ID, 2, None);
		pool.total_locked_shares = 1_000;   //all 1_000 shares was locked in 1th period
		pool.accumulated_reward_per_share = 1_000;
		Pools::<Test>::insert(BSX_ACA_POOL, get_default_pool(BSX_ACA_SHARE_ID, 2, None));

		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap().canceled, false);

		//TODO: initialize pool, pool should be updated

		mock::run_to_block(6000);
		assert_ok!(LiquidityMining::cancel_pool(Origin::root(), BSX_ACA_POOL));

		expect_events(vec![Event::PoolCanceled(BSX_ACA_POOL).into()]);

		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap().accumulated_reward_per_share, 4_000);
		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap().unpaid_rewards, 4_000);
		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap().canceled, true);

	});

}
*/
/*
#[test]
fn deposit_shares_should_work() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		let bsx_aca_pool = LmPool {
			accumulated_reward_per_share: FixedU128::from(0_u128),
			total_locked_shares: Balance::default(),
			share_id: BSX_ACA_SHARE_ID,
			updated_at: 0
		};

		let bsx_eth_pool = LmPool {
			accumulated_reward_per_share: FixedU128::from(0_u128),
			total_locked_shares: Balance::from(0_u128),
			share_id: BSX_ETH_SHARE_ID,
			updated_at: 0,
		};

		let bsx_dot_pool = LmPool {
			accumulated_reward_per_share: FixedU128::from(1_u128),
			total_locked_shares: 100,
			share_id: BSX_DOT_SHARE_ID,
			updated_at: 0,
		};

		assert_ok!(LiquidityMining::create_pool(BSX_ACA_POOL, BSX_ACA_SHARE_ID));
		assert_ok!(LiquidityMining::create_pool(BSX_ETH_POOL, BSX_ETH_SHARE_ID));

		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap(), bsx_aca_pool);
		assert_eq!(LiquidityMining::pool(BSX_ETH_POOL).unwrap(), bsx_eth_pool);

		Pools::<Test>::insert(BSX_DOT_POOL, &bsx_dot_pool);
		assert_eq!(LiquidityMining::pool(BSX_DOT_POOL).unwrap(), bsx_dot_pool);

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			BSX_ACA_POOL,
			1_000_000
		));

		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap(), LmPool {
			accumulated_reward_per_share: FixedU128::from(0_u128),
			total_locked_shares: 1_000_000,
			share_id: BSX_ACA_SHARE_ID,
			updated_at: 0
		});

		assert_eq!(LiquidityMining::liq_provider(BSX_ACA_POOL, ALICE).unwrap(), LpInfo {
			acc_reward_per_share: FixedU128::from(0),
			locked_shares: 1_000_000
		});
	});
}

//#[test]
//fn deposit_shares_should_not_work() {}
*/

#[test]
fn get_period_number_should_work() {
	let num_1: BlockNumber = 1_u32;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 1).unwrap(), 1);

	let num_1: BlockNumber = 1_000_u32;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 1).unwrap(), 1_000);

	let num_1: BlockNumber = 23_u32;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 15).unwrap(), 1);

	let num_1: BlockNumber = 843_712_398_u32;
	assert_eq!(
		LiquidityMining::get_period_number(num_1.into(), 13_412_341).unwrap(),
		62
	);

	let num_1: BlockNumber = 843_u32;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 2_000).unwrap(), 0);

	let num_1: BlockNumber = 10_u32;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 10).unwrap(), 1);
}

#[test]
fn get_period_number_should_not_work() {
	let num_1: BlockNumber = 10_u32;
	assert_eq!(
		LiquidityMining::get_period_number(num_1.into(), 0).unwrap_err(),
		Error::<Test>::Overflow
	);
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

fn get_default_pool(
	share_id: AssetId,
	period: u64,
	loyalty_curve: Option<LoyaltyCurve>,
) -> LmPool<Balance, AssetId, u64> {
	let loyalty_curve = match loyalty_curve {
		Some(v) => v,
		None => LoyaltyCurve::default(),
	};

	LmPool {
		accumulated_reward_per_share: FixedU128::from(0_u128),
		total_locked_shares: Balance::default(),
		updated_at: period,
		paid_rewards: Balance::default(),
		unpaid_rewards: Balance::default(),
		canceled: false,
		share_id,
		loyalty_curve,
	}
}
