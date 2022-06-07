// This file is part of Basilisk-node.

// Copyright (C) 2020-2021  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use test_ext::*;

#[test]
fn validate_create_farm_data_should_work() {
	assert_ok!(LiquidityMining::validate_create_farm_data(
		1_000_000,
		100,
		1,
		Permill::from_percent(50)
	));

	assert_ok!(LiquidityMining::validate_create_farm_data(
		9_999_000_000_000,
		2_000_000,
		500,
		Permill::from_percent(100)
	));

	assert_ok!(LiquidityMining::validate_create_farm_data(
		10_000_000,
		101,
		16_986_741,
		Permill::from_perthousand(1)
	));
}

#[test]
fn validate_create_farm_data_should_not_work() {
	assert_err!(
		LiquidityMining::validate_create_farm_data(999_999, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(9, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(0, 100, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidTotalRewards
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 99, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 0, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 87, 1, Permill::from_percent(50)),
		Error::<Test>::InvalidPlannedYieldingPeriods
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 0, Permill::from_percent(50)),
		Error::<Test>::InvalidBlocksPerPeriod
	);

	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 10, Permill::from_percent(0)),
		Error::<Test>::InvalidYieldPerPeriod
	);
}
#[test]
fn get_period_number_should_work() {
	let block_num: BlockNumber = 1_u64;
	let blocks_per_period = 1;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);

	let block_num: BlockNumber = 1_000_u64;
	let blocks_per_period = 1;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1_000
	);

	let block_num: BlockNumber = 23_u64;
	let blocks_per_period = 15;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);

	let block_num: BlockNumber = 843_712_398_u64;
	let blocks_per_period = 13_412_341;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		62
	);

	let block_num: BlockNumber = 843_u64;
	let blocks_per_period = 2_000;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		0
	);

	let block_num: BlockNumber = 10_u64;
	let blocks_per_period = 10;
	assert_eq!(
		LiquidityMining::get_period_number(block_num, blocks_per_period).unwrap(),
		1
	);
}

#[test]
fn get_period_number_should_not_work() {
	let block_num: BlockNumber = 10_u64;
	assert_err!(
		LiquidityMining::get_period_number(block_num, 0),
		Error::<Test>::Overflow
	);
}

#[test]
fn get_loyalty_multiplier_should_work() {
	let loyalty_curve_1 = LoyaltyCurve::default();
	let loyalty_curve_2 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from(1),
		scale_coef: 50,
	};
	let loyalty_curve_3 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from_inner(123_580_000_000_000_000), // 0.12358
		scale_coef: 23,
	};
	let loyalty_curve_4 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from_inner(0), // 0.12358
		scale_coef: 15,
	};

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

	//Special case: loyalty curve is None
	assert_eq!(
		LiquidityMining::get_loyalty_multiplier(10, None).unwrap(),
		FixedU128::one()
	);

	let precission_delta = FixedU128::from_inner(100_000_000); //0.000_000_000_1
	for (periods, expected_multiplier_1, expected_multiplier_2, expected_multiplier_3, expected_multiplier_4) in
		testing_values.iter()
	{
		//1th curve test
		assert!(is_approx_eq_fixedu128(
			LiquidityMining::get_loyalty_multiplier(*periods, Some(loyalty_curve_1.clone())).unwrap(),
			*expected_multiplier_1,
			precission_delta
		));

		//2nd curve test
		assert!(is_approx_eq_fixedu128(
			LiquidityMining::get_loyalty_multiplier(*periods, Some(loyalty_curve_2.clone())).unwrap(),
			*expected_multiplier_2,
			precission_delta
		));

		//3rd curve test
		assert!(is_approx_eq_fixedu128(
			LiquidityMining::get_loyalty_multiplier(*periods, Some(loyalty_curve_3.clone())).unwrap(),
			*expected_multiplier_3,
			precission_delta
		));

		//4th curve test
		assert!(is_approx_eq_fixedu128(
			LiquidityMining::get_loyalty_multiplier(*periods, Some(loyalty_curve_4.clone())).unwrap(),
			*expected_multiplier_4,
			precission_delta
		));
	}
}

#[test]
fn update_global_pool_should_work() {
	let testing_values = vec![
		(
			26_u64,
			2501944769_u128,
			259_u128,
			HDX,
			BSX_FARM,
			0_u128,
			206_u64,
			65192006_u128,
			55563662_u128,
			259_u128,
			55563662_u128,
		),
		(
			188_u64,
			33769603_u128,
			1148_u128,
			BSX,
			BSX_FARM,
			30080406306_u128,
			259_u64,
			1548635_u128,
			56710169_u128,
			1151_u128,
			166663254_u128,
		),
		(
			195_u64,
			26098384286056_u128,
			523_u128,
			ACA,
			KSM_FARM,
			32055_u128,
			326_u64,
			1712797_u128,
			61424428_u128,
			523_u128,
			61456483_u128,
		),
		(
			181_u64,
			9894090144_u128,
			317_u128,
			KSM,
			ACA_FARM,
			36806694280_u128,
			1856_u64,
			19009156_u128,
			52711084_u128,
			320_u128,
			31893047384_u128,
		),
		(
			196_u64,
			26886423482043_u128,
			596_u128,
			ACA,
			KSM_FARM,
			30560755872_u128,
			954_u64,
			78355_u128,
			34013971_u128,
			596_u128,
			93407061_u128,
		),
		(
			68_u64,
			1138057342_u128,
			4_u128,
			ACA,
			KSM_FARM,
			38398062768_u128,
			161_u64,
			55309798233_u128,
			71071995_u128,
			37_u128,
			38469134763_u128,
		),
		(
			161_u64,
			24495534649923_u128,
			213_u128,
			KSM,
			BSX_FARM,
			11116735745_u128,
			448_u64,
			326_u128,
			85963452_u128,
			213_u128,
			86057014_u128,
		),
		(
			27_u64,
			22108444_u128,
			970_u128,
			KSM,
			KSM_FARM,
			8572779460_u128,
			132_u64,
			1874081_u128,
			43974403_u128,
			978_u128,
			240752908_u128,
		),
		(
			97_u64,
			1593208_u128,
			6_u128,
			HDX,
			BSX_FARM,
			18440792496_u128,
			146_u64,
			741803_u128,
			14437690_u128,
			28_u128,
			50786037_u128,
		),
		(
			154_u64,
			27279119649838_u128,
			713_u128,
			BSX,
			BSX_FARM,
			28318566664_u128,
			202_u64,
			508869_u128,
			7533987_u128,
			713_u128,
			31959699_u128,
		),
		(
			104_u64,
			20462312838954_u128,
			833_u128,
			BSX,
			ACA_FARM,
			3852003_u128,
			131_u64,
			1081636_u128,
			75149021_u128,
			833_u128,
			79001024_u128,
		),
		(
			90_u64,
			37650830596054_u128,
			586_u128,
			HDX,
			KSM_FARM,
			27990338179_u128,
			110_u64,
			758482_u128,
			36765518_u128,
			586_u128,
			51935158_u128,
		),
		(
			198_u64,
			318777215_u128,
			251_u128,
			ACA,
			ACA_FARM,
			3615346492_u128,
			582_u64,
			69329_u128,
			12876432_u128,
			251_u128,
			39498768_u128,
		),
		(
			29_u64,
			33478250_u128,
			77_u128,
			BSX,
			ACA_FARM,
			39174031245_u128,
			100_u64,
			1845620_u128,
			26611087_u128,
			80_u128,
			157650107_u128,
		),
		(
			91_u64,
			393922835172_u128,
			2491_u128,
			ACA,
			KSM_FARM,
			63486975129400_u128,
			260_u64,
			109118678233_u128,
			85100506_u128,
			2537_u128,
			18441141721883_u128,
		),
		(
			67_u64,
			1126422_u128,
			295_u128,
			HDX,
			ACA_FARM,
			7492177402_u128,
			229_u64,
			1227791_u128,
			35844776_u128,
			471_u128,
			234746918_u128,
		),
		(
			168_u64,
			28351324279041_u128,
			450_u128,
			ACA,
			KSM_FARM,
			38796364068_u128,
			361_u64,
			1015284_u128,
			35695723_u128,
			450_u128,
			231645535_u128,
		),
		(
			3_u64,
			17631376575792_u128,
			82_u128,
			HDX,
			BSX_FARM,
			20473946880_u128,
			52_u64,
			1836345_u128,
			93293564_u128,
			82_u128,
			183274469_u128,
		),
		(
			49_u64,
			94059_u128,
			81_u128,
			HDX,
			BSX_FARM,
			11126653978_u128,
			132_u64,
			1672829_u128,
			75841904_u128,
			1557_u128,
			214686711_u128,
		),
		(
			38_u64,
			14085_u128,
			266_u128,
			KSM,
			ACA_FARM,
			36115448964_u128,
			400000_u64,
			886865_u128,
			52402278_u128,
			2564373_u128,
			36167851242_u128,
		),
		(
			158_u64,
			762784_u128,
			129_u128,
			BSX,
			ACA_FARM,
			21814882774_u128,
			158_u64,
			789730_u128,
			86085676_u128,
			129_u128,
			86085676_u128,
		),
	];

	for (
		updated_at,
		total_shares_z,
		accumulated_rpz,
		reward_currency,
		id,
		rewards_left_to_distribute,
		now_period,
		reward_per_period,
		accumulated_rewards,
		expected_accumulated_rpz,
		expected_accumulated_rewards,
	) in testing_values.iter()
	{
		let yield_per_period = Permill::from_percent(50);
		let planned_yielding_periods = 100;
		let blocks_per_period = 0;
		let owner = ALICE;
		let incentivized_token = BSX;
		let max_reward_per_period = 10_000_u128;

		let mut global_pool = GlobalPool::new(
			*id,
			*updated_at,
			*reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		global_pool.total_shares_z = *total_shares_z;
		global_pool.accumulated_rewards = *accumulated_rewards;
		global_pool.accumulated_rpz = *accumulated_rpz;
		global_pool.paid_accumulated_rewards = 10;

		let mut ext = new_test_ext();

		ext.execute_with(|| {
			let farm_account_id = LiquidityMining::pool_account_id(*id).unwrap();
			let _ = Tokens::transfer(
				Origin::signed(TREASURY),
				farm_account_id,
				*reward_currency,
				*rewards_left_to_distribute,
			);
			assert_eq!(
				Tokens::free_balance(*reward_currency, &farm_account_id),
				*rewards_left_to_distribute
			);

			LiquidityMining::update_global_pool(&mut global_pool, *now_period, *reward_per_period).unwrap();

			let mut expected_global_pool = GlobalPool::new(
				*id,
				*now_period,
				*reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			expected_global_pool.total_shares_z = *total_shares_z;
			expected_global_pool.paid_accumulated_rewards = 10;
			expected_global_pool.accumulated_rpz = *expected_accumulated_rpz;
			expected_global_pool.accumulated_rewards = *expected_accumulated_rewards;

			assert_eq!(global_pool, expected_global_pool);
		});
	}
}

#[test]
fn claim_from_global_pool_should_work() {
	let testing_values = vec![
		(
			26_u64,
			2501944769_u128,
			259_u128,
			299_u128,
			HDX,
			5556613662_u128,
			0_u128,
			55563662_u128,
			2222546480_u128,
			299_u128,
			3334067182_u128,
			2222546480_u128,
		),
		(
			188_u64,
			33769603_u128,
			1148_u128,
			1151_u128,
			BSX,
			166663254_u128,
			30080406306_u128,
			5671016_u128,
			17013048_u128,
			1151_u128,
			149650206_u128,
			30097419354_u128,
		),
		(
			195_u64,
			26098384286056_u128,
			523_u128,
			823_u128,
			ACA,
			61456483_u128,
			32055_u128,
			61428_u128,
			18428400_u128,
			823_u128,
			43028083_u128,
			18460455_u128,
		),
		(
			181_u64,
			9894090144_u128,
			317_u128,
			320_u128,
			KSM,
			31893047384_u128,
			36806694280_u128,
			527114_u128,
			1581342_u128,
			320_u128,
			31891466042_u128,
			36808275622_u128,
		),
		(
			196_u64,
			26886423482043_u128,
			596_u128,
			5684_u128,
			ACA,
			93407061_u128,
			30560755872_u128,
			3011_u128,
			15319968_u128,
			5684_u128,
			78087093_u128,
			30576075840_u128,
		),
		(
			68_u64,
			1138057342_u128,
			4_u128,
			37_u128,
			ACA,
			38469134763_u128,
			38398062768_u128,
			71071995_u128,
			2345375835_u128,
			37_u128,
			36123758928_u128,
			40743438603_u128,
		),
		(
			161_u64,
			24495534649923_u128,
			213_u128,
			678_u128,
			KSM,
			86057014_u128,
			11116735745_u128,
			85452_u128,
			39735180_u128,
			678_u128,
			46321834_u128,
			11156470925_u128,
		),
		(
			27_u64,
			22108444_u128,
			970_u128,
			978_u128,
			KSM,
			240752908_u128,
			8572779460_u128,
			474403_u128,
			3795224_u128,
			978_u128,
			236957684_u128,
			8576574684_u128,
		),
		(
			97_u64,
			1593208_u128,
			6_u128,
			28_u128,
			HDX,
			50786037_u128,
			18440792496_u128,
			147690_u128,
			3249180_u128,
			28_u128,
			47536857_u128,
			18444041676_u128,
		),
		(
			154_u64,
			27279119649838_u128,
			713_u128,
			876_u128,
			BSX,
			319959699_u128,
			28318566664_u128,
			75987_u128,
			12385881_u128,
			876_u128,
			307573818_u128,
			28330952545_u128,
		),
		(
			104_u64,
			20462312838954_u128,
			833_u128,
			8373_u128,
			BSX,
			790051024_u128,
			3852003_u128,
			7521_u128,
			56708340_u128,
			8373_u128,
			733342684_u128,
			60560343_u128,
		),
		(
			90_u64,
			37650830596054_u128,
			586_u128,
			5886_u128,
			HDX,
			519356158_u128,
			27990338179_u128,
			318_u128,
			1685400_u128,
			5886_u128,
			517670758_u128,
			27992023579_u128,
		),
		(
			198_u64,
			318777215_u128,
			251_u128,
			2591_u128,
			ACA,
			3949876895_u128,
			3615346492_u128,
			28732_u128,
			67232880_u128,
			2591_u128,
			3882644015_u128,
			3682579372_u128,
		),
		(
			29_u64,
			33478250_u128,
			77_u128,
			80_u128,
			BSX,
			157650107_u128,
			39174031245_u128,
			26611087_u128,
			79833261_u128,
			80_u128,
			77816846_u128,
			39253864506_u128,
		),
		(
			91_u64,
			393922835172_u128,
			2491_u128,
			2537_u128,
			ACA,
			18441141721883_u128,
			63486975129400_u128,
			85100506_u128,
			3914623276_u128,
			2537_u128,
			18437227098607_u128,
			63490889752676_u128,
		),
		(
			67_u64,
			1126422_u128,
			295_u128,
			471_u128,
			HDX,
			234746918_u128,
			7492177402_u128,
			358776_u128,
			63144576_u128,
			471_u128,
			171602342_u128,
			7555321978_u128,
		),
		(
			168_u64,
			28351324279041_u128,
			450_u128,
			952_u128,
			ACA,
			231645535_u128,
			38796364068_u128,
			356723_u128,
			179074946_u128,
			952_u128,
			52570589_u128,
			38975439014_u128,
		),
		(
			3_u64,
			17631376575792_u128,
			82_u128,
			357_u128,
			HDX,
			1832794469_u128,
			20473946880_u128,
			932564_u128,
			256455100_u128,
			357_u128,
			1576339369_u128,
			20730401980_u128,
		),
		(
			49_u64,
			94059_u128,
			81_u128,
			1557_u128,
			HDX,
			21495686711_u128,
			11126653978_u128,
			758404_u128,
			1119404304_u128,
			1557_u128,
			20376282407_u128,
			12246058282_u128,
		),
		(
			38_u64,
			14085_u128,
			266_u128,
			2564373_u128,
			KSM,
			36167851242_u128,
			36115448964_u128,
			5278_u128,
			13533356746_u128,
			2564373_u128,
			22634494496_u128,
			49648805710_u128,
		),
		(
			158_u64,
			762784_u128,
			129_u128,
			129_u128,
			BSX,
			86085676_u128,
			21814882774_u128,
			86085676_u128,
			0_u128,
			129_u128,
			86085676_u128,
			21814882774_u128,
		),
	];

	for (
		updated_at,
		total_shares_z,
		liq_pool_accumulated_rpz,
		global_pool_accumulated_rpz,
		reward_currency,
		accumulated_rewards,
		paid_accumulated_rewards,
		liq_pool_stake_in_global_pool,
		expected_rewards_from_global_pool,
		expected_liq_pool_accumulated_rpz,
		expected_global_pool_accumulated_rewards,
		expected_global_pool_paid_accumulated_rewards,
	) in testing_values.iter()
	{
		let global_pool_id = 1;
		let liq_pool_id = 2;
		let yield_per_period = Permill::from_percent(50);
		let planned_yielding_periods = 100;
		let blocks_per_period = 1;
		let owner = ALICE;
		let incentivized_token = BSX;
		let max_reward_per_period = Balance::from(10_000_u32);

		let mut global_pool = GlobalPool::new(
			global_pool_id,
			*updated_at,
			*reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		global_pool.total_shares_z = *total_shares_z;
		global_pool.accumulated_rpz = *global_pool_accumulated_rpz;
		global_pool.accumulated_rewards = *accumulated_rewards;
		global_pool.paid_accumulated_rewards = *paid_accumulated_rewards;

		let mut liq_pool = LiquidityPoolYieldFarm::new(liq_pool_id, *updated_at, None, FixedU128::from(10_u128));
		liq_pool.accumulated_rpz = *liq_pool_accumulated_rpz;

		assert_eq!(
			LiquidityMining::claim_from_global_pool(&mut global_pool, &mut liq_pool, *liq_pool_stake_in_global_pool)
				.unwrap(),
			*expected_rewards_from_global_pool
		);

		let mut expected_global_pool = GlobalPool::new(
			global_pool_id,
			*updated_at,
			*reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		expected_global_pool.total_shares_z = *total_shares_z;
		expected_global_pool.accumulated_rpz = *global_pool_accumulated_rpz;
		expected_global_pool.accumulated_rewards = *expected_global_pool_accumulated_rewards;
		expected_global_pool.paid_accumulated_rewards = *expected_global_pool_paid_accumulated_rewards;

		assert_eq!(global_pool, expected_global_pool);

		let mut expected_liq_pool =
			LiquidityPoolYieldFarm::new(liq_pool_id, *updated_at, None, FixedU128::from(10_u128));
		expected_liq_pool.accumulated_rpz = *expected_liq_pool_accumulated_rpz;

		assert_eq!(liq_pool, expected_liq_pool);
	}
}

#[test]
fn update_pool_should_work() {
	let testing_values = vec![
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			26_u64,
			206_u64,
			299_u128,
			0_u128,
			2222546480_u128,
			BSX,
			299_u128,
			26_u64,
			0_u128,
			9000000000000_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			188_u64,
			259_u64,
			1151_u128,
			33769603_u128,
			170130593048_u128,
			BSX,
			6188_u128,
			259_u64,
			170130593048_u128,
			8829869406952_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			195_u64,
			326_u64,
			823_u128,
			2604286056_u128,
			8414312431200_u128,
			BSX,
			4053_u128,
			326_u64,
			8414312431200_u128,
			585687568800_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			181_u64,
			1856_u64,
			320_u128,
			8940144_u128,
			190581342_u128,
			BSX,
			341_u128,
			1856_u64,
			190581342_u128,
			8999809418658_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			196_u64,
			954_u64,
			5684_u128,
			282043_u128,
			15319968_u128,
			BSX,
			5738_u128,
			954_u64,
			15319968_u128,
			8999984680032_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			68_u64,
			161_u64,
			37_u128,
			1138057342_u128,
			2345375835_u128,
			BSX,
			39_u128,
			161_u64,
			2345375835_u128,
			8997654624165_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			161_u64,
			448_u64,
			678_u128,
			49923_u128,
			39735180_u128,
			BSX,
			1473_u128,
			448_u64,
			39735180_u128,
			8999960264820_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			27_u64,
			132_u64,
			978_u128,
			2444_u128,
			3795224_u128,
			BSX,
			2530_u128,
			132_u64,
			3795224_u128,
			8999996204776_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			97_u64,
			146_u64,
			28_u128,
			1593208_u128,
			3249180_u128,
			BSX,
			30_u128,
			146_u64,
			3249180_u128,
			8999996750820_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			154_u64,
			202_u64,
			876_u128,
			9838_u128,
			12385881_u128,
			BSX,
			2134_u128,
			202_u64,
			12385881_u128,
			8999987614119_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			104_u64,
			131_u64,
			8373_u128,
			2046838954_u128,
			56708340909_u128,
			BSX,
			8400_u128,
			131_u64,
			56708340909_u128,
			8943291659091_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			90_u64,
			110_u64,
			5886_u128,
			596054_u128,
			1685400_u128,
			BSX,
			5888_u128,
			110_u64,
			1685400_u128,
			8999998314600_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			198_u64,
			582_u64,
			2591_u128,
			377215_u128,
			67232880_u128,
			BSX,
			2769_u128,
			582_u64,
			67232880_u128,
			8999932767120_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			29_u64,
			100_u64,
			80_u128,
			8250_u128,
			79833261_u128,
			BSX,
			9756_u128,
			100_u64,
			79833261_u128,
			8999920166739_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			91_u64,
			260_u64,
			2537_u128,
			35172_u128,
			3914623276_u128,
			BSX,
			113836_u128,
			260_u64,
			3914623276_u128,
			8996085376724_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			67_u64,
			229_u64,
			471_u128,
			1126422_u128,
			63144576_u128,
			BSX,
			527_u128,
			229_u64,
			63144576_u128,
			8999936855424_u128,
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			168_u64,
			361_u64,
			952_u128,
			28279041_u128,
			179074946_u128,
			BSX,
			958_u128,
			361_u64,
			179074946_u128,
			8999820925054_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			3_u64,
			52_u64,
			357_u128,
			2_u128,
			256455100_u128,
			BSX,
			128227907_u128,
			52_u64,
			256455100_u128,
			8999743544900_u128,
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			49_u64,
			132_u64,
			1557_u128,
			94059_u128,
			1119404304_u128,
			BSX,
			13458_u128,
			132_u64,
			1119404304_u128,
			8998880595696_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			38_u64,
			38_u64,
			2564373_u128,
			14085_u128,
			13533356746_u128,
			BSX,
			2564373_u128,
			38_u64,
			0_u128,
			9000000000000_u128,
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			158_u64,
			158_u64,
			129_u128,
			762784_u128,
			179074933_u128,
			BSX,
			129_u128,
			158_u64,
			0_u128,
			9000000000000_u128,
		),
	];

	for (
		global_pool_id,
		liq_pool_id,
		liq_pool_updated_at,
		now_period,
		liq_pool_accumulated_rpvs,
		liq_pool_total_valued_shares,
		liq_pool_rewards,
		reward_currency,
		expected_liq_pool_accumulated_rpvs,
		expected_updated_at,
		expected_liq_pool_reward_currency_balance,
		expected_global_pool_reward_currency_balance,
	) in testing_values.iter()
	{
		let owner = ALICE;
		let yield_per_period = Permill::from_percent(50);
		let blocks_per_period = BlockNumber::from(1_u32);
		let planned_yielding_periods = 100;
		let incentivized_token = BSX;
		let updated_at = 200_u64;
		let max_reward_per_period = Balance::from(10_000_u32);

		let mut global_pool = GlobalPool::<Test>::new(
			*global_pool_id,
			updated_at,
			*reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		global_pool.total_shares_z = 1_000_000_u128;
		global_pool.accumulated_rpz = 200_u128;
		global_pool.accumulated_rewards = 1_000_000_u128;
		global_pool.paid_accumulated_rewards = 1_000_000_u128;

		let mut liq_pool = LiquidityPoolYieldFarm {
			id: *liq_pool_id,
			updated_at: *liq_pool_updated_at,
			total_shares: 200_u128,
			total_valued_shares: *liq_pool_total_valued_shares,
			accumulated_rpvs: *liq_pool_accumulated_rpvs,
			accumulated_rpz: 200_u128,
			loyalty_curve: None,
			stake_in_global_pool: Balance::from(10_000_u32),
			multiplier: FixedU128::from(10_u128),
			canceled: false,
		};

		let mut ext = new_test_ext();

		let farm_account_id = LiquidityMining::pool_account_id(*global_pool_id).unwrap();
		let pool_account_id = LiquidityMining::pool_account_id(*liq_pool_id).unwrap();

		ext.execute_with(|| {
			let _ = Tokens::transfer(
				Origin::signed(TREASURY),
				farm_account_id,
				global_pool.reward_currency,
				9_000_000_000_000,
			);
			assert_eq!(
				Tokens::free_balance(global_pool.reward_currency, &farm_account_id),
				9_000_000_000_000_u128
			);

			assert_eq!(Tokens::free_balance(*reward_currency, &pool_account_id), 0);

			assert_ok!(LiquidityMining::update_liq_pool(
				&mut liq_pool,
				*liq_pool_rewards,
				*now_period,
				*global_pool_id,
				*reward_currency
			));

			let mut rhs_global_pool = GlobalPool::new(
				*global_pool_id,
				updated_at,
				*reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			rhs_global_pool.updated_at = 200_u64;
			rhs_global_pool.total_shares_z = 1_000_000_u128;
			rhs_global_pool.accumulated_rpz = 200_u128;
			rhs_global_pool.accumulated_rewards = 1_000_000_u128;
			rhs_global_pool.paid_accumulated_rewards = 1_000_000_u128;

			assert_eq!(global_pool, rhs_global_pool);

			assert_eq!(
				liq_pool,
				LiquidityPoolYieldFarm {
					id: *liq_pool_id,
					updated_at: *expected_updated_at,
					total_shares: 200_u128,
					total_valued_shares: *liq_pool_total_valued_shares,
					accumulated_rpvs: *expected_liq_pool_accumulated_rpvs,
					accumulated_rpz: 200_u128,
					loyalty_curve: None,
					stake_in_global_pool: Balance::from(10_000_u32),
					multiplier: FixedU128::from(10_u128),
					canceled: false,
				}
			);

			assert_eq!(
				Tokens::free_balance(global_pool.reward_currency, &farm_account_id),
				*expected_global_pool_reward_currency_balance
			);
			assert_eq!(
				Tokens::free_balance(global_pool.reward_currency, &pool_account_id),
				*expected_liq_pool_reward_currency_balance
			);
		});
	}
}

#[test]
fn get_next_pool_id_should_work() {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 1);
		assert_eq!(LiquidityMining::pool_id(), 1);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 2);
		assert_eq!(LiquidityMining::pool_id(), 2);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 3);
		assert_eq!(LiquidityMining::pool_id(), 3);

		assert_eq!(LiquidityMining::get_next_pool_id().unwrap(), 4);
		assert_eq!(LiquidityMining::pool_id(), 4);
	});
}

#[test]
fn pool_account_id_should_work() {
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::max_value()];

	for id in ids {
		assert_ok!(LiquidityMining::pool_account_id(id));
	}
}

#[test]
fn pool_account_id_should_not_work() {
	let ids: Vec<PoolId> = vec![0];

	for id in ids {
		assert_err!(LiquidityMining::pool_account_id(id), Error::<Test>::InvalidPoolId);
	}
}

#[test]
fn validate_pool_id_should_work() {
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::max_value()];

	for id in ids {
		assert_ok!(LiquidityMining::validate_pool_id(id));
	}
}

#[test]
fn validate_pool_id_should_not_work() {
	assert_eq!(
		LiquidityMining::validate_pool_id(0).unwrap_err(),
		Error::<Test>::InvalidPoolId
	);
}

#[test]
fn do_claim_rewards_should_work() {
	predefined_test_ext().execute_with(|| {
		let empty_liq_pool: LiquidityPoolYieldFarm<Test> = LiquidityPoolYieldFarm {
			id: 1,
			updated_at: 0,
			total_shares: 0,
			total_valued_shares: 0,
			accumulated_rpvs: 0,
			accumulated_rpz: 0,
			loyalty_curve: Some(LoyaltyCurve::default()),
			stake_in_global_pool: 0,
			multiplier: FixedU128::from(100),
			canceled: false,
		};

		#[allow(clippy::type_complexity)]
		let test_data: [(
			Deposit<Test>,
			LiquidityPoolYieldFarm<Test>,
			PeriodOf<Test>,
			(Balance, Balance),
		); 3] = [
			(
				Deposit {
					shares: 100,
					valued_shares: 500,
					accumulated_claimed_rewards: 0,
					accumulated_rpvs: 56,
					entered_at: 12,
					updated_at: 45,
				},
				LiquidityPoolYieldFarm {
					loyalty_curve: Some(LoyaltyCurve::default()),
					accumulated_rpvs: 7_789,
					..empty_liq_pool
				},
				45,
				(0, 0),
			),
			(
				Deposit {
					shares: 12_315_314,
					valued_shares: 1_454_565_765_765,
					accumulated_claimed_rewards: 65_454,
					accumulated_rpvs: 9_809,
					entered_at: 3,
					updated_at: 3,
				},
				LiquidityPoolYieldFarm {
					loyalty_curve: Some(LoyaltyCurve {
						initial_reward_percentage: FixedU128::from_float(0.674_651_900_4_f64),
						scale_coef: 360,
					}),
					accumulated_rpvs: 10_743,
					..empty_liq_pool
				},
				50,
				(967_600_574_016_191, 390_963_851_142_865),
			),
			(
				Deposit {
					shares: 97_634,
					valued_shares: 7_483_075,
					accumulated_claimed_rewards: 1_657_649,
					accumulated_rpvs: 10_989,
					entered_at: 39,
					updated_at: 329,
				},
				LiquidityPoolYieldFarm {
					loyalty_curve: None, //no loyalty factor
					accumulated_rpvs: 11_000,
					..empty_liq_pool
				},
				1002,
				(80_656_176, 0),
			),
		];

		let liq_pool_account = LiquidityMining::pool_account_id(1).unwrap();
		assert_ok!(Tokens::set_balance(
			Origin::root(),
			liq_pool_account,
			BSX,
			10_000_000_000_000_000_000_000,
			0
		));

		for (mut deposit, liq_pool, now_period, expected_result) in test_data {
			let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
			let lib_pool_bsx_balance = Tokens::free_balance(BSX, &liq_pool_account);

			assert_eq!(
				LiquidityMining::do_claim_rewards(ALICE, &mut deposit, &liq_pool, now_period, BSX).unwrap(),
				expected_result
			);

			let expected_alice_balance = alice_bsx_balance + expected_result.0;
			let expected_pool_balance = lib_pool_bsx_balance - expected_result.0;

			assert_eq!(Tokens::free_balance(BSX, &ALICE), expected_alice_balance);
			assert_eq!(Tokens::free_balance(BSX, &liq_pool_account), expected_pool_balance);
		}
	});
}

#[test]
fn get_next_nft_id_should_work() {
	new_test_ext().execute_with(|| {
		//(pool_id, result)
		let test_data = vec![
			(1, 4_294_967_297),
			(6_886, 8_589_941_478),
			(87_321, 12_884_989_209),
			(56, 17_179_869_240),
			(789, 21_474_837_269),
			(248, 25_769_804_024),
			(1_000_000_200, 31_064_771_272),
			(u32::max_value(), 38_654_705_663),
		];

		for (pool_id, expected_nft_id) in test_data {
			assert_eq!(LiquidityMining::get_next_nft_id(pool_id).unwrap(), expected_nft_id);
		}

		//This is last allowed sequencer number - 1, test with max pool id
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0])
				.checked_sub(1_u128)
				.unwrap();

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_334
		);

		assert_eq!(
			LiquidityMining::get_next_nft_id(u32::max_value()).unwrap(),
			u128::max_value()
		);

		//This is last allowed sequencer number - 1, test with min. pool id
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0])
				.checked_sub(1_u128)
				.unwrap();

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_334
		);

		assert_eq!(
			LiquidityMining::get_next_nft_id(1).unwrap(),
			340_282_366_920_938_463_463_374_607_427_473_244_161
		);
	});
}

#[test]
fn get_next_nft_id_should_not_work() {
	new_test_ext().execute_with(|| {
		//This is last allowed sequencer number, next should throw error
		let last_nft_sequencer_num =
			u128::from_le_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0]);

		<NftInstanceSequencer<Test>>::set(last_nft_sequencer_num);
		assert_eq!(
			<NftInstanceSequencer<Test>>::get(),
			79_228_162_514_264_337_593_543_950_335
		);

		assert_noop!(
			LiquidityMining::get_next_nft_id(u32::max_value()),
			Error::<Test>::NftIdOverflow
		);

		assert_noop!(LiquidityMining::get_next_nft_id(1), Error::<Test>::NftIdOverflow);
	});
}

#[test]
fn get_pool_id_from_nft_id_should_work() {
	new_test_ext().execute_with(|| {
		//(nft_id, liq. pool id)
		let test_data = vec![
			(4_294_967_297, 1),
			(8_589_941_478, 6_886),
			(12_884_989_209, 87_321),
			(17_179_869_240, 56),
			(21_474_837_269, 789),
			(25_769_804_024, 248),
			(31_064_771_272, 1_000_000_200),
			(38_654_705_663, u32::max_value()),
			(u128::max_value(), u32::max_value()),
			(340_282_366_920_938_463_463_374_607_427_473_244_161, 1),
			(340_282_366_920_938_463_463_374_607_427_473_244_161, 1),
		];

		for (nft_id, expected_pool_id) in test_data {
			assert_eq!(
				LiquidityMining::get_pool_id_from_nft_id(nft_id).unwrap(),
				expected_pool_id
			);
		}
	});
}

#[test]
fn get_pool_id_from_nft_id_should_not_work() {
	new_test_ext().execute_with(|| {
		let test_data = vec![0, 132_342_314, 4_294_967_296];

		for nft_id in test_data {
			assert_noop!(
				LiquidityMining::get_pool_id_from_nft_id(nft_id),
				Error::<Test>::InvalidNftId
			);
		}
	});
}
