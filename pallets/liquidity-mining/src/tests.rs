use super::*;
use crate::mock::{Event as TestEvent, ExtBuilder, LiquidityMining, System, Test};

use primitives::{AssetId, Balance, BlockNumber};

use sp_arithmetic::traits::CheckedSub;

use sp_arithmetic::Perquintill;
use std::cmp::Ordering;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

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

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=432121354
fn get_loyalty_multiplier_should_work() {
	let c1 = LoyaltyCurve::default();
	let c2 = LoyaltyCurve {
		b: FixedU128::from(1),
		scale_coef: 50,
	};
	let c3 = LoyaltyCurve {
		b: FixedU128::from_inner(123_580_000_000_000_000), // 0.12358
		scale_coef: 23,
	};
	let c4 = LoyaltyCurve {
		b: FixedU128::from_inner(0), // 0.12358
		scale_coef: 15,
	};

	//vec[(periods, c1-multiplier, c2-multiplier, c3-multiplier, c4-multiplier),...]
	let testing_values = vec![
		(
			0,
			FixedU128::from_float(0.5_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.12358_f64),
			FixedU128::from_float(0_f64),
		),
		(
			1,
			FixedU128::from_float(0.504950495_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.1600975_f64),
			FixedU128::from_float(0.0625_f64),
		),
		(
			4,
			FixedU128::from_float(0.5192307692_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.25342_f64),
			FixedU128::from_float(0.2105263158_f64),
		),
		(
			130,
			FixedU128::from_float(0.7826086957_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.8682505882_f64),
			FixedU128::from_float(0.8965517241_f64),
		),
		(
			150,
			FixedU128::from_float(0.8_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.8834817341_f64),
			FixedU128::from_float(0.9090909091_f64),
		),
		(
			180,
			FixedU128::from_float(0.8214285714_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9007011823_f64),
			FixedU128::from_float(0.9230769231_f64),
		),
		(
			240,
			FixedU128::from_float(0.8529411765_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9233549049_f64),
			FixedU128::from_float(0.9411764706_f64),
		),
		(
			270,
			FixedU128::from_float(0.8648648649_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9312025256_f64),
			FixedU128::from_float(0.9473684211_f64),
		),
		(
			280,
			FixedU128::from_float(0.8684210526_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9334730693_f64),
			FixedU128::from_float(0.9491525424_f64),
		),
		(
			320,
			FixedU128::from_float(0.880952381_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.941231312_f64),
			FixedU128::from_float(0.9552238806_f64),
		),
		(
			380,
			FixedU128::from_float(0.8958333333_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9499809926_f64),
			FixedU128::from_float(0.9620253165_f64),
		),
		(
			390,
			FixedU128::from_float(0.8979591837_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9511921065_f64),
			FixedU128::from_float(0.962962963_f64),
		),
		(
			4000,
			FixedU128::from_float(0.987804878_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.994989396_f64),
			FixedU128::from_float(0.99626401_f64),
		),
		(
			4400,
			FixedU128::from_float(0.9888888889_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.9954425367_f64),
			FixedU128::from_float(0.9966024915_f64),
		),
		(
			4700,
			FixedU128::from_float(0.9895833333_f64),
			FixedU128::from_float(1_f64),
			FixedU128::from_float(0.995732022_f64),
			FixedU128::from_float(0.9968186638_f64),
		),
	];

	let precission_delta = FixedU128::from_inner(100_000_000); //0.000_000_000_1
	for t in testing_values.iter() {
		//1-th curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, &c1).unwrap();
		assert_eq!(is_approx_eq_fixedu128(m, t.1, precission_delta), true);

		//2-nd curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, &c2).unwrap();
		assert_eq!(is_approx_eq_fixedu128(m, t.2, precission_delta), true);

		//3-th ucrve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, &c3).unwrap();
		assert_eq!(is_approx_eq_fixedu128(m, t.3, precission_delta), true);

		//4-th curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, &c4).unwrap();
		assert_eq!(is_approx_eq_fixedu128(m, t.4, precission_delta), true);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=906912221
fn get_reward_per_period_should_work() {
	//vec[(yield_per_period, total_global_farm_shares (spec: Z), max_reward_per_period, reward_per_period),...]
	let testing_values = vec![
		(
			FixedU128::from_float(0.0008333333333),
			Balance::from(12578954_u128),
			Balance::from(156789_u128),
			Balance::from(10482_u128),
		),
		(
			FixedU128::from_float(0.08333333333),
			Balance::from(1246578_u128),
			Balance::from(4684789_u128),
			Balance::from(103881_u128),
		),
		(
			FixedU128::from_float(0.03666666667),
			Balance::from(3980_u128),
			Balance::from(488_u128),
			Balance::from(145_u128),
		),
		(
			FixedU128::from_float(0.1666666667),
			Balance::from(9897454_u128),
			Balance::from(1684653_u128),
			Balance::from(1649575_u128),
		),
		(
			FixedU128::from_float(0.00625),
			Balance::from(1687_u128),
			Balance::from(28_u128),
			Balance::from(10_u128),
		),
		(
			FixedU128::from_float(0.0125),
			Balance::from(3879_u128),
			Balance::from(7_u128),
			Balance::from(7_u128),
		),
		(
			FixedU128::from_float(0.1333333333),
			Balance::from(35189_u128),
			Balance::from(468787897_u128),
			Balance::from(4691_u128),
		),
		(
			FixedU128::from_float(0.003111392405),
			Balance::from(48954_u128),
			Balance::from(161_u128),
			Balance::from(152_u128),
		),
		(
			FixedU128::from_float(0.000375),
			Balance::from(54789782_u128),
			Balance::from(3_u128),
			Balance::from(3_u128),
		),
		(
			FixedU128::from_float(0.1385714286),
			Balance::from(17989865464312_u128),
			Balance::from(59898_u128),
			Balance::from(59898_u128),
		),
		(
			FixedU128::from_float(0.0375),
			Balance::from(2_u128),
			Balance::from(7987_u128),
			Balance::from(0_u128),
		),
		(
			FixedU128::from_float(0.07875),
			Balance::from(5_u128),
			Balance::from(498741_u128),
			Balance::from(0_u128),
		),
		(
			FixedU128::from_float(0.04),
			Balance::from(5468_u128),
			Balance::from(8798_u128),
			Balance::from(218_u128),
		),
		(
			FixedU128::from_float(0.0),
			Balance::from(68797_u128),
			Balance::from(789846_u128),
			Balance::from(0_u128),
		),
	];

    for t in testing_values.iter()  {
        assert_eq!(LiquidityMining::get_reward_per_period(t.0, t.1, t.2).unwrap(), t.3);
    }
}

//NOTE: look at approx pallet - https://github.com/brendanzab/approx
fn is_approx_eq_fixedu128(num_1: FixedU128, num_2: FixedU128, delta: FixedU128) -> bool {
	let diff = match num_1.cmp(&num_2) {
		Ordering::Less => num_2.checked_sub(&num_1).unwrap(),
		Ordering::Greater => num_1.checked_sub(&num_2).unwrap(),
		Ordering::Equal => return true,
	};

	if diff.cmp(&delta) == Ordering::Greater {
		println!("diff: {:?}; delta: {:?}; n1: {:?}; n2: {:?}", diff, delta, num_1, num_2);

		false
	} else {
		true
	}
}

/*
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
*/
