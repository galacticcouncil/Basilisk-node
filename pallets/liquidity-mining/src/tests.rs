use super::*;
use crate::mock::{
	Event as TestEvent, ExtBuilder, LiquidityMining, Origin, System, Test, ALICE, BOB, BSX_ACA_POOL, BSX_ACA_SHARE_ID,
	BSX_DOT_POOL, BSX_DOT_SHARE_ID, BSX_ETH_POOL, BSX_ETH_SHARE_ID, CHARLIE, DAVE,
};
use frame_support::{assert_noop, assert_ok};
use primitives::{AssetId, Balance, BlockNumber};

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
		assert_ok!(LiquidityMining::create_pool(BSX_ACA_POOL, BSX_ACA_SHARE_ID));

		expect_events(vec![Event::PoolCreated(BSX_ACA_POOL).into()]);


		assert_eq!(
			LiquidityMining::pool(BSX_ACA_POOL).unwrap(),
			LmPool {
				accumulated_reward_per_share: FixedU128::default(),
				total_locked_shares: Balance::default(),
				share_id: BSX_ACA_SHARE_ID,
                updated_at: 81, //period
			}
		);
	});
}

#[test]
fn create_pool_should_not_work() {
	let mut ext = new_test_ext();
	ext.execute_with(|| {
		let orig_pool = LmPool {
			accumulated_reward_per_share: FixedU128::from(1_u128),
			total_locked_shares: 100,
			share_id: BSX_ACA_SHARE_ID,
            updated_at: 10 
		};

		Pools::<Test>::insert(BSX_ACA_POOL, &orig_pool);

		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap(), orig_pool);

		assert_noop!(
			LiquidityMining::create_pool(BSX_ACA_POOL, BSX_ACA_SHARE_ID),
			Error::<Test>::LmPoolExists
		);

		assert_eq!(LiquidityMining::pool(BSX_ACA_POOL).unwrap(), orig_pool);
	});
}
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
}*/

//#[test]
//fn deposit_shares_should_not_work() {}

#[test]
fn get_period_number_should_work() {
    let num_1: BlockNumber = 1_u32;
    assert_eq!(LiquidityMining::get_period_number(num_1.into(), 1).unwrap(), 1);
    
    let num_1: BlockNumber = 1_000_u32;
    assert_eq!(LiquidityMining::get_period_number(num_1.into(), 1).unwrap(), 1_000);

    let num_1: BlockNumber = 23_u32;
    assert_eq!(LiquidityMining::get_period_number(num_1.into(), 15).unwrap(), 1);

    let num_1: BlockNumber = 843_712_398_u32;
    assert_eq!(LiquidityMining::get_period_number(num_1.into(), 13_412_341).unwrap(), 62);
    
    let num_1: BlockNumber = 843_u32;
    assert_eq!(LiquidityMining::get_period_number(num_1.into(), 2_000).unwrap(), 0);
    
    let num_1: BlockNumber = 10_u32;
    assert_eq!(LiquidityMining::get_period_number(num_1.into(), 10).unwrap(), 1);
}

#[test]
fn get_period_number_should_not_work() {

    let num_1: BlockNumber = 10_u32;
    assert_eq!(LiquidityMining::get_period_number(num_1.into(), 0).unwrap_err(), Error::<Test>::Overflow);
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
