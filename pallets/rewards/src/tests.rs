use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_arithmetics::Percent;

#[test]
fn slash_loyalty_weight_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		// slash 50%
		assert_eq!(Rewards::slash_loyalty_weight(10, 20, Percent::from_percent(50)), 15);

		//slash 0%
		assert_eq!(
			Rewards::slash_loyalty_weight(125_864_754, 225_864_754, Percent::from_percent(0)),
			125_864_754
		);

		//slash 1%
		assert_eq!(
			Rewards::slash_loyalty_weight(125_864_754, 213_551_741, Percent::from_percent(1)),
			126_741_623
		);

		//slash 100%
		assert_eq!(
			Rewards::slash_loyalty_weight(125_864_754, 213_551_741, Percent::from_percent(100)),
			213_551_741
		);

		// slash  37%
		assert_eq!(
			Rewards::slash_loyalty_weight(458_796, 458_983, Percent::from_percent(37)),
			458_865
		);

		// slash 255% => same as 100% slash
		assert_eq!(Rewards::slash_loyalty_weight(100, 200, Percent::from_percent(255)), 200);
	});
}

#[test]
fn get_loyalty_weight_for_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		//Weight for same from to should be 0
		assert_eq!(Rewards::get_loyalty_weight_for(1, 1, 1).unwrap(), 0);

		assert_eq!(Rewards::get_loyalty_weight_for(1, 1, 1_000).unwrap(), 0);

		assert_eq!(Rewards::get_loyalty_weight_for(1, 10, 1).unwrap(), 9);

		assert_eq!(Rewards::get_loyalty_weight_for(1_234, 8_244, 2).unwrap(), 14_020);
	});
}

#[test]
fn get_loyalty_weight_for_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Rewards::get_loyalty_weight_for(u128::MAX, 1_000, 10_000),
			Error::<Test>::Overflow
		);
	});
}

#[test]
fn get_weighted_shares_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Rewards::get_weighted_shares(100, 10).unwrap(), 1_000);

		assert_eq!(
			Rewards::get_weighted_shares(468_138_468, 106_876_813).unwrap(),
			50_033_147_502_542_484
		);

		assert_eq!(Rewards::get_weighted_shares(468_138_468, 0).unwrap(), 0);
	});
}

#[test]
fn get_weighted_shares_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Rewards::get_weighted_shares(u128::MAX, 999_999_999_999_999_999),
			Error::<Test>::Overflow
		);
	});
}

#[test]
fn get_weighted_rewards_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		//claim 0 shares
		assert_eq!(Rewards::get_weighted_rewards(0, 5_000_000_258, 40_000).unwrap(), 0);

		assert_eq!(
			Rewards::get_weighted_rewards(1_000, 5_000_000_258, 40_000).unwrap(),
			125_000_006
		);

		assert_eq!(
			Rewards::get_weighted_rewards(1, 5_000_000_258, 40_000).unwrap(),
			125_000
		);

		assert_eq!(
			Rewards::get_weighted_rewards(875_284, 50_000_000_000, 853_877_984_524_165).unwrap(),
			51
		);

		//claim more shares than in pool
		//this should never happen
		assert_eq!(Rewards::get_weighted_rewards(2_000, 50_000, 1_000).unwrap(), 50_000);

		//claim all shares
		assert_eq!(
			Rewards::get_weighted_rewards(40_000, 5_000_000_258, 40_000).unwrap(),
			5_000_000_258
		);
	});
}

#[test]
fn get_weighted_rewards_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Rewards::get_weighted_rewards(1_000, 5_000_000_258, 0),
			Error::<Test>::Overflow
		);

		assert_noop!(
			Rewards::get_weighted_rewards(u128::MAX, 5_000_000_258, u128::MAX),
			Error::<Test>::Overflow
		);
	});
}

#[test]
fn add_shares_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Rewards::pools(DOT_POOL), (PoolState::default(), PoolState::default()));
		assert_eq!(Rewards::pools(BSX_POOL), (PoolState::default(), PoolState::default()));

		const PERIOD: u128 = 1;
		assert_ok!(Rewards::add_shares(&ALICE, DOT_POOL, 3_000, PERIOD));

		assert_eq!(
			Rewards::pools(DOT_POOL),
			(
				PoolState::default(),
				PoolState {
					total_shares: 3_000,
					total_weighted_shares: 6_000,
					rewards: 0,
					period: 0
				}
			)
		);

		assert_eq!(
			Rewards::liquidity_providers(DOT_POOL, ALICE).unwrap(),
			LpInfo {
				shares: 3_000,
				loyalty_from: 2,
				claim_from: 2
			}
		);

		assert_ok!(Rewards::add_shares(&BOB, DOT_POOL, 8_000_000_000_000, PERIOD));

		assert_eq!(
			Rewards::pools(DOT_POOL),
			(
				PoolState::default(),
				PoolState {
					total_shares: 8_000_000_003_000,
					total_weighted_shares: 16_000_000_006_000,
					rewards: 0,
					period: 0
				}
			)
		);

		assert_eq!(
			Rewards::liquidity_providers(DOT_POOL, BOB).unwrap(),
			LpInfo {
				shares: 8_000_000_000_000,
				loyalty_from: 2,
				claim_from: 2
			}
		);

		assert_ok!(Rewards::add_shares(&CHARLIE, DOT_POOL, 1, PERIOD));

		assert_eq!(
			Rewards::pools(DOT_POOL),
			(
				PoolState::default(),
				PoolState {
					total_shares: 8_000_000_003_001,
					total_weighted_shares: 16_000_000_006_002,
					rewards: 0,
					period: 0
				}
			)
		);

		assert_eq!(
			Rewards::liquidity_providers(DOT_POOL, CHARLIE).unwrap(),
			LpInfo {
				shares: 1,
				loyalty_from: 2,
				claim_from: 2
			}
		);

		assert_ok!(Rewards::add_shares(&ALICE, BSX_POOL, 1_000, PERIOD));

		assert_eq!(
			Rewards::pools(BSX_POOL),
			(
				PoolState::default(),
				PoolState {
					total_shares: 1_000,
					total_weighted_shares: 2_000,
					rewards: 0,
					period: 0
				}
			)
		);

		assert_eq!(
			Rewards::liquidity_providers(DOT_POOL, ALICE).unwrap(),
			LpInfo {
				shares: 3_000,
				loyalty_from: 2,
				claim_from: 2
			}
		);

		assert_eq!(
			Rewards::liquidity_providers(BSX_POOL, ALICE).unwrap(),
			LpInfo {
				shares: 1_000,
				loyalty_from: 2,
				claim_from: 2
			}
		);
	});
}

#[test]
fn add_shares_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Rewards::pools(DOT_POOL), (PoolState::default(), PoolState::default()));

		const PERIOD: u128 = 1;
		assert_ok!(Rewards::add_shares(&ALICE, DOT_POOL, 3_000, PERIOD));

		assert_eq!(
			Rewards::pools(DOT_POOL),
			(
				PoolState::default(),
				PoolState {
					total_shares: 3_000,
					total_weighted_shares: 6_000,
					rewards: 0,
					period: 0
				}
			)
		);

		assert_eq!(
			Rewards::liquidity_providers(DOT_POOL, ALICE).unwrap(),
			LpInfo {
				shares: 3_000,
				loyalty_from: 2,
				claim_from: 2
			}
		);

		//add shares to the same pools should fail
		assert_noop!(
			Rewards::add_shares(&ALICE, DOT_POOL, 3_000, PERIOD),
			Error::<Test>::DuplicateShares
		);
	});
}

#[test]
fn claim_rewards_should_work() {
	ExtBuilder::default().build().execute_with(|| {
        Snapshots::<Test>::insert(DOT_POOL, vec![
            PoolState {
                total_weighted_shares: 6_000,
                total_shares: 3_000,
                rewards: 4_000_000,
                period: 5
            }
        ]); 
        //Snapshots::<Test>::insert(BSX_POOL, vec![]); 
        //Snapshots::<Test>::insert(HDX_POOL, vec![]); 

		LiquidityProviders::<Test>::insert(
			DOT_POOL,
			ALICE,
			LpInfo {
				shares: 1_000,
				loyalty_from: 2,
				claim_from: 2,
			},
		);
	});
}
