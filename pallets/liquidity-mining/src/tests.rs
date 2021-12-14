use super::*;
use crate::mock::{
	asset_pair_to_map_key, run_to_block, BlockNumber, Event as TestEvent, ExtBuilder, LiquidityMining, Origin, Test,
	Tokens, ACA, ACA_FARM, ACC_1M, ALICE, AMM_POOLS, BOB, BSX, BSX_ACA_AMM, BSX_ACA_LM_POOL, BSX_ACA_SHARE_ID,
	BSX_DOT_AMM, BSX_DOT_LM_POOL, BSX_DOT_SHARE_ID, BSX_ETH_AMM, BSX_ETH_SHARE_ID, BSX_FARM, BSX_HDX_AMM,
	BSX_HDX_SHARE_ID, BSX_KSM_AMM, BSX_KSM_LM_POOL, BSX_KSM_SHARE_ID, BSX_TO1_AMM, BSX_TO1_SHARE_ID, BSX_TO2_AMM,
	BSX_TO2_SHARE_ID, DOT, ETH, GC, GC_FARM, HDX, INITIAL_BALANCE, KSM, KSM_FARM, TO1, TO2, TREASURY,
};

use frame_support::{assert_err, assert_noop, assert_ok};
use primitives::Balance;

use sp_arithmetic::traits::CheckedSub;
use sp_runtime::traits::BadOrigin;

use std::cmp::Ordering;

const ALICE_FARM: u32 = BSX_FARM;
const BOB_FARM: u32 = KSM_FARM;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| run_to_block(1));
	ext
}

const PREDEFINED_GLOBAL_POOLS: [GlobalPool<Test>; 3] = [
	GlobalPool {
		id: ALICE_FARM,
		updated_at: 0,
		reward_currency: BSX,
		yield_per_period: Permill::from_percent(20),
		planned_yielding_periods: 300_u64,
		blocks_per_period: 1_000_u64,
		owner: ALICE,
		incentivized_token: BSX,
		max_reward_per_period: 333_333_333,
		accumulated_rpz: 0,
		liq_pools_count: 0,
		paid_accumulated_rewards: 0,
		total_shares_z: 0,
		accumulated_rewards: 0,
	},
	GlobalPool {
		id: BOB_FARM,
		updated_at: 0,
		reward_currency: KSM,
		yield_per_period: Permill::from_percent(38),
		planned_yielding_periods: 5_000_u64,
		blocks_per_period: 10_000_u64,
		owner: BOB,
		incentivized_token: BSX,
		max_reward_per_period: 200_000,
		accumulated_rpz: 0,
		liq_pools_count: 0,
		paid_accumulated_rewards: 0,
		total_shares_z: 0,
		accumulated_rewards: 0,
	},
	GlobalPool {
		id: GC_FARM,
		updated_at: 0,
		reward_currency: BSX,
		yield_per_period: Permill::from_percent(50),
		planned_yielding_periods: 500_u64,
		blocks_per_period: 100_u64,
		owner: GC,
		incentivized_token: BSX,
		max_reward_per_period: 60_000_000,
		accumulated_rpz: 0,
		liq_pools_count: 2,
		paid_accumulated_rewards: 0,
		total_shares_z: 0,
		accumulated_rewards: 0,
	},
];

pub fn predefined_test_ext() -> sp_io::TestExternalities {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			100_000_000_000,
			BlockNumber::from(300_u32),
			BlockNumber::from(1_000_u32),
			BSX,
			BSX,
			ALICE,
			Permill::from_percent(20),
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			1_000_000_000,
			BlockNumber::from(5_000_u32),
			BlockNumber::from(10_000_u32),
			BSX,
			KSM,
			BOB,
			Permill::from_percent(38),
		));

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			30_000_000_000,
			BlockNumber::from(500_u32),
			BlockNumber::from(100_u32),
			BSX,
			BSX,
			GC,
			Permill::from_percent(50),
		));

		expect_events(vec![
			Event::FarmCreated(1, PREDEFINED_GLOBAL_POOLS[0].clone()).into(),
			frame_system::Event::NewAccount(187989685649991564771226578797).into(),
			orml_tokens::Event::Endowed(4_000, 187989685649991564771226578797, 1_000_000_000).into(),
			Event::FarmCreated(2, PREDEFINED_GLOBAL_POOLS[1].clone()).into(),
			frame_system::Event::NewAccount(267217848164255902364770529133).into(),
			orml_tokens::Event::Endowed(1_000, 267217848164255902364770529133, 30_000_000_000).into(),
			Event::FarmCreated(
				3,
				GlobalPool {
					liq_pools_count: 0,
					..PREDEFINED_GLOBAL_POOLS[2].clone()
				},
			)
			.into(),
		]);

		let amm_mock_data = vec![
			(
				AssetPair {
					asset_in: BSX,
					asset_out: ACA,
				},
				(BSX_ACA_AMM, BSX_ACA_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: KSM,
				},
				(BSX_KSM_AMM, BSX_KSM_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: DOT,
				},
				(BSX_DOT_AMM, BSX_DOT_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: ETH,
				},
				(BSX_ETH_AMM, BSX_ETH_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
				(BSX_HDX_AMM, BSX_HDX_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: TO1,
				},
				(BSX_TO1_AMM, BSX_TO1_SHARE_ID),
			),
			(
				AssetPair {
					asset_in: BSX,
					asset_out: TO2,
				},
				(BSX_TO2_AMM, BSX_TO2_SHARE_ID),
			),
		];

		AMM_POOLS.with(|h| {
			let mut hm = h.borrow_mut();
			for (k, v) in amm_mock_data {
				hm.insert(asset_pair_to_map_key(k), v);
			}
		});

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			AssetPair {
				asset_in: BSX,
				asset_out: TO1,
			},
			5,
			Some(LoyaltyCurve::default()),
		));

		expect_events(vec![Event::LiquidityPoolAdded(
			GC_FARM,
			BSX_TO1_AMM,
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 0,
				accumulated_rps: 0,
				accumulated_rpz: 0,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		)
		.into()]);

		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(GC),
			GC_FARM,
			AssetPair {
				asset_in: BSX,
				asset_out: TO2,
			},
			10,
			Some(LoyaltyCurve::default()),
		));

		expect_events(vec![Event::LiquidityPoolAdded(
			GC_FARM,
			BSX_TO2_AMM,
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 0,
				accumulated_rps: 0,
				accumulated_rpz: 0,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		)
		.into()]);
	});

	ext
}

pub fn predefined_test_ext_with_deposits() -> sp_io::TestExternalities {
	let mut ext = predefined_test_ext();

	ext.execute_with(|| {
		let farm_id = GC_FARM;
		let amm_1 = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		let amm_2 = AssetPair {
			asset_in: BSX,
			asset_out: TO2,
		};

		let pallet_acc = LiquidityMining::account_id();
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_amm_1_farm_acc = LiquidityMining::pool_account_id(4).unwrap();
		let liq_pool_amm_2_farm_acc = LiquidityMining::pool_account_id(5).unwrap();
		let amm_1_acc = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(amm_1)).unwrap().0);
		let amm_2_acc = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(amm_2)).unwrap().0);
		//DEPOSIT 1:
		run_to_block(1_800); //18-th period

		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 50, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_1,
			50
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 4, ALICE, 50, BSX_TO1_SHARE_ID, 0).into()
		]);

		// DEPOSIT 2 (deposit in same period):
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 52, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(Origin::signed(BOB), farm_id, amm_1, 80));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 4, BOB, 80, BSX_TO1_SHARE_ID, 1).into()
		]);

		// DEPOSIT 3 (same period, second liq pool yield farm):
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 8, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(Origin::signed(BOB), farm_id, amm_2, 25));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, BOB, 25, BSX_TO2_SHARE_ID, 0).into()
		]);

		// DEPOSIT 4 (new period):
		run_to_block(2051); //period 20
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 58, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			amm_2,
			800
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, BOB, 800, BSX_TO2_SHARE_ID, 1).into()
		]);

		// DEPOSIT 5 (same period, second liq pool yield farm):
		run_to_block(2_586); //period 20
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 3, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_2,
			87
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, ALICE, 87, BSX_TO2_SHARE_ID, 2).into()
		]);

		// DEPOSIT 6 (same period):
		run_to_block(2_596); //period 20
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 16, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_2,
			48
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, ALICE, 48, BSX_TO2_SHARE_ID, 3).into()
		]);

		// DEPOSIT 7 : (same period differen liq poll farm)
		run_to_block(2_596); //period 20
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 80, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_1,
			486
		));

		expect_events(vec![Event::SharesDeposited(
			GC_FARM,
			4,
			ALICE,
			486,
			BSX_TO1_SHARE_ID,
			2,
		)
		.into()]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 25,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 12,
				liq_pools_count: 2,
				total_shares_z: 703_990,
				accumulated_rewards: 231_650,
				paid_accumulated_rewards: 1_164_400,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 25,
				accumulated_rps: 60,
				accumulated_rpz: 12,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 227_700,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 25,
				accumulated_rps: 120,
				accumulated_rpz: 12,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 3, GC_FARM));
		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 4, GC_FARM));

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 616);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 960);

		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 1_164_400));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 212_400);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), 952_000);

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE), 3_000_000 - 536);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE), 3_000_000 - 135);

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB), 2_000_000 - 80);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB), 2_000_000 - 825);
	});

	ext
}

#[test]
fn get_period_number_should_work() {
	let num_1: BlockNumber = 1_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 1).unwrap(), 1);

	let num_1: BlockNumber = 1_000_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 1).unwrap(), 1_000);

	let num_1: BlockNumber = 23_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 15).unwrap(), 1);

	let num_1: BlockNumber = 843_712_398_u64;
	assert_eq!(
		LiquidityMining::get_period_number(num_1.into(), 13_412_341).unwrap(),
		62
	);

	let num_1: BlockNumber = 843_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 2_000).unwrap(), 0);

	let num_1: BlockNumber = 10_u64;
	assert_eq!(LiquidityMining::get_period_number(num_1.into(), 10).unwrap(), 1);
}

#[test]
fn get_period_number_should_not_work() {
	let num_1: BlockNumber = 10_u64;
	assert_err!(
		LiquidityMining::get_period_number(num_1.into(), 0),
		Error::<Test>::Overflow
	);
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=432121354
fn get_loyalty_multiplier_should_work() {
	let c1 = LoyaltyCurve::default();
	let c2 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from(1),
		scale_coef: 50,
	};
	let c3 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from_inner(123_580_000_000_000_000), // 0.12358
		scale_coef: 23,
	};
	let c4 = LoyaltyCurve {
		initial_reward_percentage: FixedU128::from_inner(0), // 0.12358
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

	//Special case: loyalty curve is None
	assert_eq!(
		LiquidityMining::get_loyalty_multiplier(10, None).unwrap(),
		FixedU128::one()
	);

	let precission_delta = FixedU128::from_inner(100_000_000); //0.000_000_000_1
	for t in testing_values.iter() {
		//1-th curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, Some(c1.clone())).unwrap();
		assert_eq!(is_approx_eq_fixedu128(m, t.1, precission_delta), true);

		//2-nd curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, Some(c2.clone())).unwrap();
		assert_eq!(is_approx_eq_fixedu128(m, t.2, precission_delta), true);

		//3-th ucrve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, Some(c3.clone())).unwrap();
		assert_eq!(is_approx_eq_fixedu128(m, t.3, precission_delta), true);

		//4-th curve test
		let m = LiquidityMining::get_loyalty_multiplier(t.0, Some(c4.clone())).unwrap();
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

	for t in testing_values.iter() {
		assert_eq!(LiquidityMining::get_reward_per_period(t.0, t.1, t.2).unwrap(), t.3);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=478231890
fn get_accumulated_rps_should_work() {
	//vec[(AccPRSprevious, total_shares,reward,  newAccRPS),...]
	let testing_values = vec![
		(
			Balance::from(596850065_u128),
			Balance::from(107097_u128),
			Balance::from(58245794_u128),
			Balance::from(596850608_u128),
		),
		(
			Balance::from(610642940_u128),
			Balance::from(380089_u128),
			Balance::from(72666449_u128),
			Balance::from(610643131_u128),
		),
		(
			Balance::from(342873091_u128),
			Balance::from(328911_u128),
			Balance::from(32953786_u128),
			Balance::from(342873191_u128),
		),
		(
			Balance::from(678009825_u128),
			Balance::from(130956_u128),
			Balance::from(49126054_u128),
			Balance::from(678010200_u128),
		),
		(
			Balance::from(579839575_u128),
			Balance::from(349893_u128),
			Balance::from(48822879_u128),
			Balance::from(579839714_u128),
		),
		(
			Balance::from(53648392_u128),
			Balance::from(191826_u128),
			Balance::from(5513773_u128),
			Balance::from(53648420_u128),
		),
		(
			Balance::from(474641194_u128),
			Balance::from(224569_u128),
			Balance::from(88288774_u128),
			Balance::from(474641587_u128),
		),
		(
			Balance::from(323929643_u128),
			Balance::from(117672_u128),
			Balance::from(43395220_u128),
			Balance::from(323930011_u128),
		),
		(
			Balance::from(18684290_u128),
			Balance::from(293754_u128),
			Balance::from(84347520_u128),
			Balance::from(18684577_u128),
		),
		(
			Balance::from(633517462_u128),
			Balance::from(417543_u128),
			Balance::from(43648027_u128),
			Balance::from(633517566_u128),
		),
		(
			Balance::from(899481210_u128),
			Balance::from(217000_u128),
			Balance::from(46063156_u128),
			Balance::from(899481422_u128),
		),
		(
			Balance::from(732260582_u128),
			Balance::from(120313_u128),
			Balance::from(91003576_u128),
			Balance::from(732261338_u128),
		),
		(
			Balance::from(625857089_u128),
			Balance::from(349989_u128),
			Balance::from(71595913_u128),
			Balance::from(625857293_u128),
		),
		(
			Balance::from(567721341_u128),
			Balance::from(220776_u128),
			Balance::from(75561456_u128),
			Balance::from(567721683_u128),
		),
		(
			Balance::from(962034430_u128),
			Balance::from(196031_u128),
			Balance::from(40199198_u128),
			Balance::from(962034635_u128),
		),
		(
			Balance::from(548598381_u128),
			Balance::from(457172_u128),
			Balance::from(37345481_u128),
			Balance::from(548598462_u128),
		),
		(
			Balance::from(869164975_u128),
			Balance::from(172541_u128),
			Balance::from(4635196_u128),
			Balance::from(869165001_u128),
		),
		(
			Balance::from(776275145_u128),
			Balance::from(419601_u128),
			Balance::from(32861993_u128),
			Balance::from(776275223_u128),
		),
		(
			Balance::from(684419217_u128),
			Balance::from(396975_u128),
			Balance::from(24222103_u128),
			Balance::from(684419278_u128),
		),
		(
			Balance::from(967509392_u128),
			Balance::from(352488_u128),
			Balance::from(77778911_u128),
			Balance::from(967509612_u128),
		),
	];

	for t in testing_values.iter() {
		assert_eq!(LiquidityMining::get_accumulated_rps(t.0, t.1, t.2).unwrap(), t.3);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=1775700162
fn get_user_reward_should_work() {
	//[(user_accumulated_claimed_rewards, loyalty_multiplier, user_reward, unclaimable_rewards),...]
	let testing_values = vec![
		(
			Balance::from(79_u128),
			Balance::from(1733800371_u128),
			Balance::from(259_u128),
			Balance::from(2333894_u128),
			FixedU128::from_inner(456_446_123_846_332_000_u128),
			Balance::from(142447228701_u128),
			Balance::from(169634504185_u128),
		),
		(
			Balance::from(61_u128),
			Balance::from(3117_u128),
			Balance::from(1148_u128),
			Balance::from(34388_u128),
			FixedU128::from_inner(621_924_695_680_678_000_u128),
			Balance::from(2072804_u128),
			Balance::from(1280987_u128),
		),
		(
			Balance::from(0_u128),
			Balance::from(3232645500_u128),
			Balance::from(523_u128),
			Balance::from(1124892_u128),
			FixedU128::from_inner(000_001_000_000_000_000_u128),
			Balance::from(565781_u128),
			Balance::from(1690671905827_u128),
		),
		(
			Balance::from(159_u128),
			Balance::from(3501142339_u128),
			Balance::from(317_u128),
			Balance::from(3309752_u128),
			FixedU128::from_inner(384_109_209_525_475_000_u128),
			Balance::from(212478410818_u128),
			Balance::from(340698768992_u128),
		),
		(
			Balance::from(352_u128),
			Balance::from(156_u128),
			Balance::from(596_u128),
			Balance::from(2156_u128),
			FixedU128::from_inner(100_703_041_057_143_000_u128),
			Balance::from(1677_u128),
			Balance::from(34231_u128),
		),
		(
			Balance::from(0_u128),
			Balance::from(192208478782_u128),
			Balance::from(4_u128),
			Balance::from(534348_u128),
			FixedU128::from_inner(104_779_339_071_984_000_u128),
			Balance::from(80557375135_u128),
			Balance::from(688276005645_u128),
		),
		(
			Balance::from(138_u128),
			Balance::from(36579085_u128),
			Balance::from(213_u128),
			Balance::from(1870151_u128),
			FixedU128::from_inner(129_927_485_118_411_000_u128),
			Balance::from(354576988_u128),
			Balance::from(2386984236_u128),
		),
		(
			Balance::from(897_u128),
			Balance::from(1_u128),
			Balance::from(970_u128),
			Balance::from(1_u128),
			FixedU128::from_inner(502_367_859_476_566_000_u128),
			Balance::from(35_u128),
			Balance::from(37_u128),
		),
		(
			Balance::from(4_u128),
			Balance::from(38495028244_u128),
			Balance::from(6_u128),
			Balance::from(2568893_u128),
			FixedU128::from_inner(265_364_053_378_152_000_u128),
			Balance::from(20427824566_u128),
			Balance::from(56559663029_u128),
		),
		(
			Balance::from(10_u128),
			Balance::from(13343864050_u128),
			Balance::from(713_u128),
			Balance::from(1959317_u128),
			FixedU128::from_inner(279_442_586_539_696_000_u128),
			Balance::from(2621375291532_u128),
			Balance::from(6759359176301_u128),
		),
		(
			Balance::from(29_u128),
			Balance::from(18429339175_u128),
			Balance::from(833_u128),
			Balance::from(3306140_u128),
			FixedU128::from_inner(554_635_100_856_657_000_u128),
			Balance::from(8218129641066_u128),
			Balance::from(6599055749494_u128),
		),
		(
			Balance::from(224_u128),
			Balance::from(39102822603_u128),
			Balance::from(586_u128),
			Balance::from(1839083_u128),
			FixedU128::from_inner(654_427_828_000_143_000_u128),
			Balance::from(9263569206758_u128),
			Balance::from(4891650736445_u128),
		),
		(
			Balance::from(36_u128),
			Balance::from(55755691086_u128),
			Balance::from(251_u128),
			Balance::from(3521256_u128),
			FixedU128::from_inner(802_407_775_824_621_000_u128),
			Balance::from(9618838494628_u128),
			Balance::from(2368631567606_u128),
		),
		(
			Balance::from(36_u128),
			Balance::from(258339226986_u128),
			Balance::from(77_u128),
			Balance::from(2106922_u128),
			FixedU128::from_inner(743_748_274_128_360_000_u128),
			Balance::from(7877711415708_u128),
			Balance::from(2714194783796_u128),
		),
		(
			Balance::from(383_u128),
			Balance::from(34812134025_u128),
			Balance::from(2491_u128),
			Balance::from(1442758_u128),
			FixedU128::from_inner(130_076_146_093_442_000_u128),
			Balance::from(9545503668738_u128),
			Balance::from(63838473413204_u128),
		),
		(
			Balance::from(117_u128),
			Balance::from(44358629274_u128),
			Balance::from(295_u128),
			Balance::from(2076570_u128),
			FixedU128::from_inner(495_172_207_692_510_000_u128),
			Balance::from(3909796472461_u128),
			Balance::from(3986037461741_u128),
		),
		(
			Balance::from(172_u128),
			Balance::from(64667747645_u128),
			Balance::from(450_u128),
			Balance::from(33468_u128),
			FixedU128::from_inner(326_047_919_016_893_000_u128),
			Balance::from(5861570070642_u128),
			Balance::from(12116063741200_u128),
		),
		(
			Balance::from(37_u128),
			Balance::from(68875501378_u128),
			Balance::from(82_u128),
			Balance::from(230557_u128),
			FixedU128::from_inner(176_816_131_903_196_000_u128),
			Balance::from(548023257587_u128),
			Balance::from(2551374073866_u128),
		),
		(
			Balance::from(41_u128),
			Balance::from(100689735793_u128),
			Balance::from(81_u128),
			Balance::from(2268544_u128),
			FixedU128::from_inner(376_605_306_400_251_000_u128),
			Balance::from(1516809283443_u128),
			Balance::from(2510777879733_u128),
		),
		(
			Balance::from(252_u128),
			Balance::from(16283442689_u128),
			Balance::from(266_u128),
			Balance::from(3797763_u128),
			FixedU128::from_inner(189_489_655_763_324_000_u128),
			Balance::from(43193817533_u128),
			Balance::from(184770582350_u128),
		),
		(
			Balance::from(20_u128),
			Balance::from(205413646819_u128),
			Balance::from(129_u128),
			Balance::from(3184799_u128),
			FixedU128::from_inner(543_081_681_209_601_000_u128),
			Balance::from(12159643178907_u128),
			Balance::from(10230441139565_u128),
		),
		(
			Balance::from(23_u128),
			Balance::from(100000_u128),
			Balance::from(155_u128),
			Balance::from(1210762_u128),
			FixedU128::from_inner(404_726_206_620_574_000_u128),
			Balance::from(4131623_u128),
			Balance::from(7857615_u128),
		),
		(
			Balance::from(11_u128),
			Balance::from(84495025009_u128),
			Balance::from(166_u128),
			Balance::from(468012_u128),
			FixedU128::from_inner(735_133_167_032_114_000_u128),
			Balance::from(9627839308653_u128),
			Balance::from(3468889099730_u128),
		),
		(
			Balance::from(198_u128),
			Balance::from(79130076897_u128),
			Balance::from(571_u128),
			Balance::from(830256_u128),
			FixedU128::from_inner(689_497_061_649_446_000_u128),
			Balance::from(20350862574442_u128),
			Balance::from(9164655277883_u128),
		),
		(
			Balance::from(30_u128),
			Balance::from(68948735954_u128),
			Balance::from(72_u128),
			Balance::from(3278682_u128),
			FixedU128::from_inner(238_786_980_081_793_000_u128),
			Balance::from(691487259752_u128),
			Balance::from(2204356371634_u128),
		),
		(
			Balance::from(54_u128),
			Balance::from(280608075911_u128),
			Balance::from(158_u128),
			Balance::from(0_u128),
			FixedU128::from_inner(504_409_653_378_878_000_u128),
			Balance::from(14720307919780_u128),
			Balance::from(14462931974964_u128),
		),
		(
			Balance::from(193_u128),
			Balance::from(22787841433_u128),
			Balance::from(1696_u128),
			Balance::from(2962625_u128),
			FixedU128::from_inner(623_942_971_029_398_000_u128),
			Balance::from(21370122208415_u128),
			Balance::from(12880000502759_u128),
		),
	];

	for t in testing_values.iter() {
		assert_eq!(
			LiquidityMining::get_user_reward(t.0, t.1, t.3, t.2, t.4).unwrap(),
			(t.5, t.6)
		);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=2010118745
fn update_global_pool_should_work() {
	//[(pool.updated_at, pool.total_shares, pool.accumulated_rps, pool.reward_currency,
	//pool_id, reward_left_to_distribute, period_now, reward_per_period, pool.accumulated_reward, pool.accumulated_rps, pool.accumulated_reward),...]
	let testing_values = vec![
		(
			BlockNumber::from(26_u64),
			Balance::from(2501944769_u128),
			Balance::from(259_u128),
			HDX,
			BSX_FARM,
			Balance::from(0_u128),
			BlockNumber::from(206_u64),
			Balance::from(65192006_u128),
			Balance::from(55563662_u128),
			Balance::from(259_u128),
			Balance::from(55563662_u128),
		),
		(
			BlockNumber::from(188_u64),
			Balance::from(33769603_u128),
			Balance::from(1148_u128),
			BSX,
			BSX_FARM,
			Balance::from(30080406306_u128),
			BlockNumber::from(259_u64),
			Balance::from(1548635_u128),
			Balance::from(56710169_u128),
			Balance::from(1151_u128),
			Balance::from(166663254_u128),
		),
		(
			BlockNumber::from(195_u64),
			Balance::from(26098384286056_u128),
			Balance::from(523_u128),
			ACA,
			KSM_FARM,
			Balance::from(32055_u128),
			BlockNumber::from(326_u64),
			Balance::from(1712797_u128),
			Balance::from(61424428_u128),
			Balance::from(523_u128),
			Balance::from(61456483_u128),
		),
		(
			BlockNumber::from(181_u64),
			Balance::from(9894090144_u128),
			Balance::from(317_u128),
			KSM,
			ACA_FARM,
			Balance::from(36806694280_u128),
			BlockNumber::from(1856_u64),
			Balance::from(19009156_u128),
			Balance::from(52711084_u128),
			Balance::from(320_u128),
			Balance::from(31893047384_u128),
		),
		(
			BlockNumber::from(196_u64),
			Balance::from(26886423482043_u128),
			Balance::from(596_u128),
			ACA,
			KSM_FARM,
			Balance::from(30560755872_u128),
			BlockNumber::from(954_u64),
			Balance::from(78355_u128),
			Balance::from(34013971_u128),
			Balance::from(596_u128),
			Balance::from(93407061_u128),
		),
		(
			BlockNumber::from(68_u64),
			Balance::from(1138057342_u128),
			Balance::from(4_u128),
			ACA,
			KSM_FARM,
			Balance::from(38398062768_u128),
			BlockNumber::from(161_u64),
			Balance::from(55309798233_u128),
			Balance::from(71071995_u128),
			Balance::from(37_u128),
			Balance::from(38469134763_u128),
		),
		(
			BlockNumber::from(161_u64),
			Balance::from(24495534649923_u128),
			Balance::from(213_u128),
			KSM,
			BSX_FARM,
			Balance::from(11116735745_u128),
			BlockNumber::from(448_u64),
			Balance::from(326_u128),
			Balance::from(85963452_u128),
			Balance::from(213_u128),
			Balance::from(86057014_u128),
		),
		(
			BlockNumber::from(27_u64),
			Balance::from(22108444_u128),
			Balance::from(970_u128),
			KSM,
			KSM_FARM,
			Balance::from(8572779460_u128),
			BlockNumber::from(132_u64),
			Balance::from(1874081_u128),
			Balance::from(43974403_u128),
			Balance::from(978_u128),
			Balance::from(240752908_u128),
		),
		(
			BlockNumber::from(97_u64),
			Balance::from(1593208_u128),
			Balance::from(6_u128),
			HDX,
			BSX_FARM,
			Balance::from(18440792496_u128),
			BlockNumber::from(146_u64),
			Balance::from(741803_u128),
			Balance::from(14437690_u128),
			Balance::from(28_u128),
			Balance::from(50786037_u128),
		),
		(
			BlockNumber::from(154_u64),
			Balance::from(27279119649838_u128),
			Balance::from(713_u128),
			BSX,
			BSX_FARM,
			Balance::from(28318566664_u128),
			BlockNumber::from(202_u64),
			Balance::from(508869_u128),
			Balance::from(7533987_u128),
			Balance::from(713_u128),
			Balance::from(31959699_u128),
		),
		(
			BlockNumber::from(104_u64),
			Balance::from(20462312838954_u128),
			Balance::from(833_u128),
			BSX,
			ACA_FARM,
			Balance::from(3852003_u128),
			BlockNumber::from(131_u64),
			Balance::from(1081636_u128),
			Balance::from(75149021_u128),
			Balance::from(833_u128),
			Balance::from(79001024_u128),
		),
		(
			BlockNumber::from(90_u64),
			Balance::from(37650830596054_u128),
			Balance::from(586_u128),
			HDX,
			KSM_FARM,
			Balance::from(27990338179_u128),
			BlockNumber::from(110_u64),
			Balance::from(758482_u128),
			Balance::from(36765518_u128),
			Balance::from(586_u128),
			Balance::from(51935158_u128),
		),
		(
			BlockNumber::from(198_u64),
			Balance::from(318777215_u128),
			Balance::from(251_u128),
			ACA,
			ACA_FARM,
			Balance::from(3615346492_u128),
			BlockNumber::from(582_u64),
			Balance::from(69329_u128),
			Balance::from(12876432_u128),
			Balance::from(251_u128),
			Balance::from(39498768_u128),
		),
		(
			BlockNumber::from(29_u64),
			Balance::from(33478250_u128),
			Balance::from(77_u128),
			BSX,
			ACA_FARM,
			Balance::from(39174031245_u128),
			BlockNumber::from(100_u64),
			Balance::from(1845620_u128),
			Balance::from(26611087_u128),
			Balance::from(80_u128),
			Balance::from(157650107_u128),
		),
		(
			BlockNumber::from(91_u64),
			Balance::from(393922835172_u128),
			Balance::from(2491_u128),
			ACA,
			KSM_FARM,
			Balance::from(63486975129400_u128),
			BlockNumber::from(260_u64),
			Balance::from(109118678233_u128),
			Balance::from(85100506_u128),
			Balance::from(2537_u128),
			Balance::from(18441141721883_u128),
		),
		(
			BlockNumber::from(67_u64),
			Balance::from(1126422_u128),
			Balance::from(295_u128),
			HDX,
			ACA_FARM,
			Balance::from(7492177402_u128),
			BlockNumber::from(229_u64),
			Balance::from(1227791_u128),
			Balance::from(35844776_u128),
			Balance::from(471_u128),
			Balance::from(234746918_u128),
		),
		(
			BlockNumber::from(168_u64),
			Balance::from(28351324279041_u128),
			Balance::from(450_u128),
			ACA,
			KSM_FARM,
			Balance::from(38796364068_u128),
			BlockNumber::from(361_u64),
			Balance::from(1015284_u128),
			Balance::from(35695723_u128),
			Balance::from(450_u128),
			Balance::from(231645535_u128),
		),
		(
			BlockNumber::from(3_u64),
			Balance::from(17631376575792_u128),
			Balance::from(82_u128),
			HDX,
			BSX_FARM,
			Balance::from(20473946880_u128),
			BlockNumber::from(52_u64),
			Balance::from(1836345_u128),
			Balance::from(93293564_u128),
			Balance::from(82_u128),
			Balance::from(183274469_u128),
		),
		(
			BlockNumber::from(49_u64),
			Balance::from(94059_u128),
			Balance::from(81_u128),
			HDX,
			BSX_FARM,
			Balance::from(11126653978_u128),
			BlockNumber::from(132_u64),
			Balance::from(1672829_u128),
			Balance::from(75841904_u128),
			Balance::from(1557_u128),
			Balance::from(214686711_u128),
		),
		(
			BlockNumber::from(38_u64),
			Balance::from(14085_u128),
			Balance::from(266_u128),
			KSM,
			ACA_FARM,
			Balance::from(36115448964_u128),
			BlockNumber::from(400000_u64),
			Balance::from(886865_u128),
			Balance::from(52402278_u128),
			Balance::from(2564373_u128),
			Balance::from(36167851242_u128),
		),
		(
			BlockNumber::from(158_u64),
			Balance::from(762784_u128),
			Balance::from(129_u128),
			BSX,
			ACA_FARM,
			Balance::from(21814882774_u128),
			BlockNumber::from(158_u64),
			Balance::from(789730_u128),
			Balance::from(86085676_u128),
			Balance::from(129_u128),
			Balance::from(86085676_u128),
		),
	];

	//[(pool.updated_at, pool.total_shares, pool.accumulated_rps, pool.reward_currency,
	//pool_id, reward_left_to_distribute, period_now, reward_per_period, pool.accumulated_rps),...]
	for t in testing_values.iter() {
		let yield_per_period = Permill::from_percent(50);
		let planned_yielding_periods = 100;
		let blocks_per_period = 0;
		let owner = ALICE;
		let incentivized_token = BSX;
		let max_reward_per_period = Balance::from(10_000_u128);

		let mut p = GlobalPool::new(
			t.4,
			t.0,
			t.3,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		p.total_shares_z = t.1;
		p.accumulated_rewards = t.8;
		p.accumulated_rpz = t.2;
		p.paid_accumulated_rewards = 10;

		let mut ext = new_test_ext();

		ext.execute_with(|| {
			let farm_account_id = LiquidityMining::pool_account_id(t.4).unwrap();
			let _ = Tokens::transfer(Origin::signed(TREASURY), farm_account_id, t.3, t.5);
			assert_eq!(Tokens::free_balance(t.3, &farm_account_id), t.5);

			LiquidityMining::update_global_pool(&mut p, t.6, t.7).unwrap();

			let mut rhs_p = GlobalPool::new(
				t.4,
				t.6,
				t.3,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			rhs_p.total_shares_z = t.1;
			rhs_p.paid_accumulated_rewards = 10;
			rhs_p.accumulated_rpz = t.9;
			rhs_p.accumulated_rewards = t.10;

			assert_eq!(p, rhs_p);
		});
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=1562134162
fn claim_global_pool_should_work() {
	//(pool.updated_at, pool.total_shares, pool.accumulated_rps_start, pool.accumulated_rps, pool.reward_currency, pool.accumululated_rewards, ool.paid_accumularted_rewards, shares , reward, pool.accumulated_rps_start, pool.accumululated_rewards, pool.paid_accumularted_rewards)
	let testing_values = vec![
		(
			BlockNumber::from(26_u64),
			Balance::from(2501944769_u128),
			Balance::from(259_u128),
			Balance::from(299_u128),
			HDX,
			Balance::from(5556613662_u128),
			Balance::from(0_u128),
			Balance::from(55563662_u128),
			Balance::from(2222546480_u128),
			Balance::from(299_u128),
			Balance::from(3334067182_u128),
			Balance::from(2222546480_u128),
		),
		(
			BlockNumber::from(188_u64),
			Balance::from(33769603_u128),
			Balance::from(1148_u128),
			Balance::from(1151_u128),
			BSX,
			Balance::from(166663254_u128),
			Balance::from(30080406306_u128),
			Balance::from(5671016_u128),
			Balance::from(17013048_u128),
			Balance::from(1151_u128),
			Balance::from(149650206_u128),
			Balance::from(30097419354_u128),
		),
		(
			BlockNumber::from(195_u64),
			Balance::from(26098384286056_u128),
			Balance::from(523_u128),
			Balance::from(823_u128),
			ACA,
			Balance::from(61456483_u128),
			Balance::from(32055_u128),
			Balance::from(61428_u128),
			Balance::from(18428400_u128),
			Balance::from(823_u128),
			Balance::from(43028083_u128),
			Balance::from(18460455_u128),
		),
		(
			BlockNumber::from(181_u64),
			Balance::from(9894090144_u128),
			Balance::from(317_u128),
			Balance::from(320_u128),
			KSM,
			Balance::from(31893047384_u128),
			Balance::from(36806694280_u128),
			Balance::from(527114_u128),
			Balance::from(1581342_u128),
			Balance::from(320_u128),
			Balance::from(31891466042_u128),
			Balance::from(36808275622_u128),
		),
		(
			BlockNumber::from(196_u64),
			Balance::from(26886423482043_u128),
			Balance::from(596_u128),
			Balance::from(5684_u128),
			ACA,
			Balance::from(93407061_u128),
			Balance::from(30560755872_u128),
			Balance::from(3011_u128),
			Balance::from(15319968_u128),
			Balance::from(5684_u128),
			Balance::from(78087093_u128),
			Balance::from(30576075840_u128),
		),
		(
			BlockNumber::from(68_u64),
			Balance::from(1138057342_u128),
			Balance::from(4_u128),
			Balance::from(37_u128),
			ACA,
			Balance::from(38469134763_u128),
			Balance::from(38398062768_u128),
			Balance::from(71071995_u128),
			Balance::from(2345375835_u128),
			Balance::from(37_u128),
			Balance::from(36123758928_u128),
			Balance::from(40743438603_u128),
		),
		(
			BlockNumber::from(161_u64),
			Balance::from(24495534649923_u128),
			Balance::from(213_u128),
			Balance::from(678_u128),
			KSM,
			Balance::from(86057014_u128),
			Balance::from(11116735745_u128),
			Balance::from(85452_u128),
			Balance::from(39735180_u128),
			Balance::from(678_u128),
			Balance::from(46321834_u128),
			Balance::from(11156470925_u128),
		),
		(
			BlockNumber::from(27_u64),
			Balance::from(22108444_u128),
			Balance::from(970_u128),
			Balance::from(978_u128),
			KSM,
			Balance::from(240752908_u128),
			Balance::from(8572779460_u128),
			Balance::from(474403_u128),
			Balance::from(3795224_u128),
			Balance::from(978_u128),
			Balance::from(236957684_u128),
			Balance::from(8576574684_u128),
		),
		(
			BlockNumber::from(97_u64),
			Balance::from(1593208_u128),
			Balance::from(6_u128),
			Balance::from(28_u128),
			HDX,
			Balance::from(50786037_u128),
			Balance::from(18440792496_u128),
			Balance::from(147690_u128),
			Balance::from(3249180_u128),
			Balance::from(28_u128),
			Balance::from(47536857_u128),
			Balance::from(18444041676_u128),
		),
		(
			BlockNumber::from(154_u64),
			Balance::from(27279119649838_u128),
			Balance::from(713_u128),
			Balance::from(876_u128),
			BSX,
			Balance::from(319959699_u128),
			Balance::from(28318566664_u128),
			Balance::from(75987_u128),
			Balance::from(12385881_u128),
			Balance::from(876_u128),
			Balance::from(307573818_u128),
			Balance::from(28330952545_u128),
		),
		(
			BlockNumber::from(104_u64),
			Balance::from(20462312838954_u128),
			Balance::from(833_u128),
			Balance::from(8373_u128),
			BSX,
			Balance::from(790051024_u128),
			Balance::from(3852003_u128),
			Balance::from(7521_u128),
			Balance::from(56708340_u128),
			Balance::from(8373_u128),
			Balance::from(733342684_u128),
			Balance::from(60560343_u128),
		),
		(
			BlockNumber::from(90_u64),
			Balance::from(37650830596054_u128),
			Balance::from(586_u128),
			Balance::from(5886_u128),
			HDX,
			Balance::from(519356158_u128),
			Balance::from(27990338179_u128),
			Balance::from(318_u128),
			Balance::from(1685400_u128),
			Balance::from(5886_u128),
			Balance::from(517670758_u128),
			Balance::from(27992023579_u128),
		),
		(
			BlockNumber::from(198_u64),
			Balance::from(318777215_u128),
			Balance::from(251_u128),
			Balance::from(2591_u128),
			ACA,
			Balance::from(3949876895_u128),
			Balance::from(3615346492_u128),
			Balance::from(28732_u128),
			Balance::from(67232880_u128),
			Balance::from(2591_u128),
			Balance::from(3882644015_u128),
			Balance::from(3682579372_u128),
		),
		(
			BlockNumber::from(29_u64),
			Balance::from(33478250_u128),
			Balance::from(77_u128),
			Balance::from(80_u128),
			BSX,
			Balance::from(157650107_u128),
			Balance::from(39174031245_u128),
			Balance::from(26611087_u128),
			Balance::from(79833261_u128),
			Balance::from(80_u128),
			Balance::from(77816846_u128),
			Balance::from(39253864506_u128),
		),
		(
			BlockNumber::from(91_u64),
			Balance::from(393922835172_u128),
			Balance::from(2491_u128),
			Balance::from(2537_u128),
			ACA,
			Balance::from(18441141721883_u128),
			Balance::from(63486975129400_u128),
			Balance::from(85100506_u128),
			Balance::from(3914623276_u128),
			Balance::from(2537_u128),
			Balance::from(18437227098607_u128),
			Balance::from(63490889752676_u128),
		),
		(
			BlockNumber::from(67_u64),
			Balance::from(1126422_u128),
			Balance::from(295_u128),
			Balance::from(471_u128),
			HDX,
			Balance::from(234746918_u128),
			Balance::from(7492177402_u128),
			Balance::from(358776_u128),
			Balance::from(63144576_u128),
			Balance::from(471_u128),
			Balance::from(171602342_u128),
			Balance::from(7555321978_u128),
		),
		(
			BlockNumber::from(168_u64),
			Balance::from(28351324279041_u128),
			Balance::from(450_u128),
			Balance::from(952_u128),
			ACA,
			Balance::from(231645535_u128),
			Balance::from(38796364068_u128),
			Balance::from(356723_u128),
			Balance::from(179074946_u128),
			Balance::from(952_u128),
			Balance::from(52570589_u128),
			Balance::from(38975439014_u128),
		),
		(
			BlockNumber::from(3_u64),
			Balance::from(17631376575792_u128),
			Balance::from(82_u128),
			Balance::from(357_u128),
			HDX,
			Balance::from(1832794469_u128),
			Balance::from(20473946880_u128),
			Balance::from(932564_u128),
			Balance::from(256455100_u128),
			Balance::from(357_u128),
			Balance::from(1576339369_u128),
			Balance::from(20730401980_u128),
		),
		(
			BlockNumber::from(49_u64),
			Balance::from(94059_u128),
			Balance::from(81_u128),
			Balance::from(1557_u128),
			HDX,
			Balance::from(21495686711_u128),
			Balance::from(11126653978_u128),
			Balance::from(758404_u128),
			Balance::from(1119404304_u128),
			Balance::from(1557_u128),
			Balance::from(20376282407_u128),
			Balance::from(12246058282_u128),
		),
		(
			BlockNumber::from(38_u64),
			Balance::from(14085_u128),
			Balance::from(266_u128),
			Balance::from(2564373_u128),
			KSM,
			Balance::from(36167851242_u128),
			Balance::from(36115448964_u128),
			Balance::from(5278_u128),
			Balance::from(13533356746_u128),
			Balance::from(2564373_u128),
			Balance::from(22634494496_u128),
			Balance::from(49648805710_u128),
		),
		(
			BlockNumber::from(158_u64),
			Balance::from(762784_u128),
			Balance::from(129_u128),
			Balance::from(129_u128),
			BSX,
			Balance::from(86085676_u128),
			Balance::from(21814882774_u128),
			Balance::from(86085676_u128),
			Balance::from(0_u128),
			Balance::from(129_u128),
			Balance::from(86085676_u128),
			Balance::from(21814882774_u128),
		),
	];

	//(pool.updated_at, pool.total_shares, pool.accumulated_rps_start, pool.accumulated_rps, pool.reward_currency, pool.accumululated_rewards, pool.paid_accumularted_rewards, shares , reward, pool.accumulated_rps_start, pool.accumululated_rewards, pool.paid_accumularted_rewards)
	for t in testing_values.iter() {
		let g_pool_id = 1;
		let liq_pool_id = 2;
		let yield_per_period = Permill::from_percent(50);
		let planned_yielding_periods = 100;
		let blocks_per_period = 1;
		let owner = ALICE;
		let incentivized_token = BSX;
		let max_reward_per_period = Balance::from(10_000_u32);

		let mut g_pool = GlobalPool::new(
			g_pool_id,
			t.0,
			t.4,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		g_pool.total_shares_z = t.1;
		g_pool.accumulated_rpz = t.3;
		g_pool.accumulated_rewards = t.5;
		g_pool.paid_accumulated_rewards = t.6;

		let mut liq_pool = LiquidityPoolYieldFarm::new(liq_pool_id, t.0, None, 10, 1);
		liq_pool.accumulated_rpz = t.2;

		assert_eq!(
			LiquidityMining::claim_from_global_pool(&mut g_pool, &mut liq_pool, t.7).unwrap(),
			t.8
		);

		let mut rhs_g_pool = GlobalPool::new(
			g_pool_id,
			t.0,
			t.4,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		rhs_g_pool.total_shares_z = t.1;
		rhs_g_pool.accumulated_rpz = t.3;
		rhs_g_pool.accumulated_rewards = t.10;
		rhs_g_pool.paid_accumulated_rewards = t.11;

		assert_eq!(g_pool, rhs_g_pool);

		let mut rhs_liq_pool = LiquidityPoolYieldFarm::new(liq_pool_id, t.0, None, 10, 1);
		rhs_liq_pool.accumulated_rpz = t.9;

		assert_eq!(liq_pool, rhs_liq_pool);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=1639947555
fn update_pool_should_work() {
	//(globaPoolId, PoolId, pool.updated_at, period_now, pool.accRPS,pool.total_shares, globaPool.reward_currency, pool.accRPS-new, pool.updated_at-new, pool.account-balance, global_pool.account-balance)
	let testing_values = vec![
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			BlockNumber::from(26_u64),
			BlockNumber::from(206_u64),
			Balance::from(299_u128),
			Balance::from(0_u128),
			Balance::from(2222546480_u128),
			BSX,
			Balance::from(299_u128),
			BlockNumber::from(26_u64),
			Balance::from(0_u128),
			Balance::from(9000000000000_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(188_u64),
			BlockNumber::from(259_u64),
			Balance::from(1151_u128),
			Balance::from(33769603_u128),
			Balance::from(170130593048_u128),
			BSX,
			Balance::from(6188_u128),
			BlockNumber::from(259_u64),
			Balance::from(170130593048_u128),
			Balance::from(8829869406952_u128),
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			BlockNumber::from(195_u64),
			BlockNumber::from(326_u64),
			Balance::from(823_u128),
			Balance::from(2604286056_u128),
			Balance::from(8414312431200_u128),
			BSX,
			Balance::from(4053_u128),
			BlockNumber::from(326_u64),
			Balance::from(8414312431200_u128),
			Balance::from(585687568800_u128),
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			BlockNumber::from(181_u64),
			BlockNumber::from(1856_u64),
			Balance::from(320_u128),
			Balance::from(8940144_u128),
			Balance::from(190581342_u128),
			BSX,
			Balance::from(341_u128),
			BlockNumber::from(1856_u64),
			Balance::from(190581342_u128),
			Balance::from(8999809418658_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(196_u64),
			BlockNumber::from(954_u64),
			Balance::from(5684_u128),
			Balance::from(282043_u128),
			Balance::from(15319968_u128),
			BSX,
			Balance::from(5738_u128),
			BlockNumber::from(954_u64),
			Balance::from(15319968_u128),
			Balance::from(8999984680032_u128),
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			BlockNumber::from(68_u64),
			BlockNumber::from(161_u64),
			Balance::from(37_u128),
			Balance::from(1138057342_u128),
			Balance::from(2345375835_u128),
			BSX,
			Balance::from(39_u128),
			BlockNumber::from(161_u64),
			Balance::from(2345375835_u128),
			Balance::from(8997654624165_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(161_u64),
			BlockNumber::from(448_u64),
			Balance::from(678_u128),
			Balance::from(49923_u128),
			Balance::from(39735180_u128),
			BSX,
			Balance::from(1473_u128),
			BlockNumber::from(448_u64),
			Balance::from(39735180_u128),
			Balance::from(8999960264820_u128),
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			BlockNumber::from(27_u64),
			BlockNumber::from(132_u64),
			Balance::from(978_u128),
			Balance::from(2444_u128),
			Balance::from(3795224_u128),
			BSX,
			Balance::from(2530_u128),
			BlockNumber::from(132_u64),
			Balance::from(3795224_u128),
			Balance::from(8999996204776_u128),
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			BlockNumber::from(97_u64),
			BlockNumber::from(146_u64),
			Balance::from(28_u128),
			Balance::from(1593208_u128),
			Balance::from(3249180_u128),
			BSX,
			Balance::from(30_u128),
			BlockNumber::from(146_u64),
			Balance::from(3249180_u128),
			Balance::from(8999996750820_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(154_u64),
			BlockNumber::from(202_u64),
			Balance::from(876_u128),
			Balance::from(9838_u128),
			Balance::from(12385881_u128),
			BSX,
			Balance::from(2134_u128),
			BlockNumber::from(202_u64),
			Balance::from(12385881_u128),
			Balance::from(8999987614119_u128),
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			BlockNumber::from(104_u64),
			BlockNumber::from(131_u64),
			Balance::from(8373_u128),
			Balance::from(2046838954_u128),
			Balance::from(56708340909_u128),
			BSX,
			Balance::from(8400_u128),
			BlockNumber::from(131_u64),
			Balance::from(56708340909_u128),
			Balance::from(8943291659091_u128),
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			BlockNumber::from(90_u64),
			BlockNumber::from(110_u64),
			Balance::from(5886_u128),
			Balance::from(596054_u128),
			Balance::from(1685400_u128),
			BSX,
			Balance::from(5888_u128),
			BlockNumber::from(110_u64),
			Balance::from(1685400_u128),
			Balance::from(8999998314600_u128),
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			BlockNumber::from(198_u64),
			BlockNumber::from(582_u64),
			Balance::from(2591_u128),
			Balance::from(377215_u128),
			Balance::from(67232880_u128),
			BSX,
			Balance::from(2769_u128),
			BlockNumber::from(582_u64),
			Balance::from(67232880_u128),
			Balance::from(8999932767120_u128),
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			BlockNumber::from(29_u64),
			BlockNumber::from(100_u64),
			Balance::from(80_u128),
			Balance::from(8250_u128),
			Balance::from(79833261_u128),
			BSX,
			Balance::from(9756_u128),
			BlockNumber::from(100_u64),
			Balance::from(79833261_u128),
			Balance::from(8999920166739_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(91_u64),
			BlockNumber::from(260_u64),
			Balance::from(2537_u128),
			Balance::from(35172_u128),
			Balance::from(3914623276_u128),
			BSX,
			Balance::from(113836_u128),
			BlockNumber::from(260_u64),
			Balance::from(3914623276_u128),
			Balance::from(8996085376724_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(67_u64),
			BlockNumber::from(229_u64),
			Balance::from(471_u128),
			Balance::from(1126422_u128),
			Balance::from(63144576_u128),
			BSX,
			Balance::from(527_u128),
			BlockNumber::from(229_u64),
			Balance::from(63144576_u128),
			Balance::from(8999936855424_u128),
		),
		(
			BSX_FARM,
			BSX_DOT_LM_POOL,
			BlockNumber::from(168_u64),
			BlockNumber::from(361_u64),
			Balance::from(952_u128),
			Balance::from(28279041_u128),
			Balance::from(179074946_u128),
			BSX,
			Balance::from(958_u128),
			BlockNumber::from(361_u64),
			Balance::from(179074946_u128),
			Balance::from(8999820925054_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(3_u64),
			BlockNumber::from(52_u64),
			Balance::from(357_u128),
			Balance::from(2_u128),
			Balance::from(256455100_u128),
			BSX,
			Balance::from(128227907_u128),
			BlockNumber::from(52_u64),
			Balance::from(256455100_u128),
			Balance::from(8999743544900_u128),
		),
		(
			BSX_FARM,
			BSX_KSM_LM_POOL,
			BlockNumber::from(49_u64),
			BlockNumber::from(132_u64),
			Balance::from(1557_u128),
			Balance::from(94059_u128),
			Balance::from(1119404304_u128),
			BSX,
			Balance::from(13458_u128),
			BlockNumber::from(132_u64),
			Balance::from(1119404304_u128),
			Balance::from(8998880595696_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(38_u64),
			BlockNumber::from(38_u64),
			Balance::from(2564373_u128),
			Balance::from(14085_u128),
			Balance::from(13533356746_u128),
			BSX,
			Balance::from(2564373_u128),
			BlockNumber::from(38_u64),
			Balance::from(0_u128),
			Balance::from(9000000000000_u128),
		),
		(
			BSX_FARM,
			BSX_ACA_LM_POOL,
			BlockNumber::from(158_u64),
			BlockNumber::from(158_u64),
			Balance::from(129_u128),
			Balance::from(762784_u128),
			Balance::from(179074933_u128),
			BSX,
			Balance::from(129_u128),
			BlockNumber::from(158_u64),
			Balance::from(0_u128),
			Balance::from(9000000000000_u128),
		),
	];

	for t in testing_values.iter() {
		let owner = ALICE;
		let gid = t.0;
		let yield_per_period = Permill::from_percent(50);
		let blocks_per_period = BlockNumber::from(1_u32);
		let planned_yielding_periods = 100;
		let incentivized_token = BSX;
		let updated_at = BlockNumber::from(200_u64);
		let reward_currency = t.7;
		let max_reward_per_period = Balance::from(10_000_u32);

		let mut g_pool = GlobalPool::<Test>::new(
			gid,
			updated_at,
			reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		g_pool.total_shares_z = Balance::from(1_000_000_u128);
		g_pool.accumulated_rpz = Balance::from(200_u128);
		g_pool.accumulated_rewards = Balance::from(1_000_000_u128);
		g_pool.paid_accumulated_rewards = Balance::from(1_000_000_u128);

		let mut liq_pool = LiquidityPoolYieldFarm {
			id: t.1,
			updated_at: t.2,
			total_shares: Balance::from(200_u128),
			total_valued_shares: t.5,
			accumulated_rps: t.4,
			accumulated_rpz: Balance::from(200_u128),
			loyalty_curve: None,
			stake_in_global_pool: Balance::from(10_000_u32),
			multiplier: 10,
			nft_class: 1,
			canceled: false,
		};

		let mut ext = new_test_ext();

		let farm_account_id = LiquidityMining::pool_account_id(t.0).unwrap();
		let pool_account_id = LiquidityMining::pool_account_id(t.1).unwrap();

		ext.execute_with(|| {
			let _ = Tokens::transfer(
				Origin::signed(TREASURY),
				farm_account_id,
				g_pool.reward_currency,
				9_000_000_000_000,
			);
			assert_eq!(
				Tokens::free_balance(g_pool.reward_currency, &farm_account_id),
				9_000_000_000_000_u128
			);

			assert_eq!(Tokens::free_balance(t.7.try_into().unwrap(), &pool_account_id), 0);

			assert_ok!(LiquidityMining::update_pool(&mut liq_pool, t.6, t.3, t.0, t.7));

			let mut rhs_g_pool = GlobalPool::new(
				gid,
				updated_at,
				reward_currency,
				yield_per_period,
				planned_yielding_periods,
				blocks_per_period,
				owner,
				incentivized_token,
				max_reward_per_period,
			);

			rhs_g_pool.updated_at = BlockNumber::from(200_u64);
			rhs_g_pool.total_shares_z = Balance::from(1_000_000_u128);
			rhs_g_pool.accumulated_rpz = Balance::from(200_u128);
			rhs_g_pool.accumulated_rewards = Balance::from(1_000_000_u128);
			rhs_g_pool.paid_accumulated_rewards = Balance::from(1_000_000_u128);

			assert_eq!(g_pool, rhs_g_pool);

			assert_eq!(
				liq_pool,
				LiquidityPoolYieldFarm {
					id: t.1,
					updated_at: t.9,
					total_shares: Balance::from(200_u128),
					total_valued_shares: t.5,
					accumulated_rps: t.8,
					accumulated_rpz: Balance::from(200_u128),
					loyalty_curve: None,
					stake_in_global_pool: Balance::from(10_000_u32),
					multiplier: 10,
					nft_class: 1,
					canceled: false,
				}
			);

			assert_eq!(Tokens::free_balance(g_pool.reward_currency, &farm_account_id), t.11);
			assert_eq!(Tokens::free_balance(g_pool.reward_currency, &pool_account_id), t.10);
		});
	}
}

#[test]
fn next_id_should_work() {
	let mut ext = new_test_ext();

	ext.execute_with(|| {
		assert_eq!(LiquidityMining::get_next_id().unwrap(), 1);
		assert_eq!(LiquidityMining::pool_id(), 1);

		assert_eq!(LiquidityMining::get_next_id().unwrap(), 2);
		assert_eq!(LiquidityMining::pool_id(), 2);

		assert_eq!(LiquidityMining::get_next_id().unwrap(), 3);
		assert_eq!(LiquidityMining::pool_id(), 3);

		assert_eq!(LiquidityMining::get_next_id().unwrap(), 4);
		assert_eq!(LiquidityMining::pool_id(), 4);
	});
}

#[test]
fn pool_account_id_should_work() {
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::MAX];

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
	let ids: Vec<PoolId> = vec![1, 100, 543, u32::MAX];

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
	// total rawards
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

	//invalid min_farming_periods
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

	//invalid block per period
	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 0, Permill::from_percent(50)),
		Error::<Test>::InvalidBlocksPerPeriod
	);

	//invalid yield per period
	assert_err!(
		LiquidityMining::validate_create_farm_data(1_000_000, 100, 10, Permill::from_percent(0)),
		Error::<Test>::InvalidYieldPerPeriod
	);
}

#[test]
fn create_farm_should_work() {
	new_test_ext().execute_with(|| {
		let pool_id = 1;
		let total_rewards: Balance = 5_000_0000_000;
		let reward_currency = BSX;
		let planned_yielding_periods: BlockNumber = 1_000_000_000_u64;
		let blocks_per_period = 20_000;
		let incentivized_token = BSX;
		let owner = ALICE;
		let yield_per_period = Permill::from_percent(20);
		let max_reward_per_period: Balance = total_rewards.checked_div(planned_yielding_periods.into()).unwrap();

		let created_at_block = 15_896;

		run_to_block(created_at_block);

		let pool_account = LiquidityMining::pool_account_id(pool_id).unwrap();

		assert_eq!(Tokens::free_balance(reward_currency, &pool_account), 0);

		assert_ok!(LiquidityMining::create_farm(
			Origin::root(),
			total_rewards,
			planned_yielding_periods,
			blocks_per_period,
			incentivized_token,
			reward_currency,
			owner,
			yield_per_period
		));

		assert_eq!(Tokens::free_balance(reward_currency, &pool_account), total_rewards);
		assert_eq!(
			Tokens::free_balance(reward_currency, &ALICE),
			(INITIAL_BALANCE - total_rewards)
		);

		let updated_at = created_at_block / blocks_per_period;

		let global_pool = GlobalPool::new(
			pool_id,
			updated_at,
			reward_currency,
			yield_per_period,
			planned_yielding_periods,
			blocks_per_period,
			owner,
			incentivized_token,
			max_reward_per_period,
		);

		expect_events(vec![Event::FarmCreated(pool_id, global_pool.clone()).into()]);

		assert_eq!(LiquidityMining::global_pool(pool_id), Some(global_pool));
	});
}

#[test]
fn create_farm_from_basic_origin_should_not_work() {
	new_test_ext().execute_with(|| {
		let created_at_block = 15_896;

		run_to_block(created_at_block);

		assert_noop!(
			LiquidityMining::create_farm(
				Origin::signed(ALICE),
				1_000_000,
				1_000,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			BadOrigin
		);
	});
}

#[test]
fn create_farm_invalid_data_should_not_work() {
	new_test_ext().execute_with(|| {
		let created_at_block = 15_896;

		run_to_block(created_at_block);

		//total_rewards bellow min.
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				100,
				1_000,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			Error::<Test>::InvalidTotalRewards
		);

		//planned_yielding_periods bellow min.
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_000,
				10,
				300,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			Error::<Test>::InvalidPlannedYieldingPeriods
		);

		//blocks_per_period is 0.
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_000,
				1_000,
				0,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(20)
			),
			Error::<Test>::InvalidBlocksPerPeriod
		);

		//yield_per_period is 0.
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_000,
				1_000,
				1,
				BSX,
				BSX,
				ALICE,
				Permill::from_percent(0)
			),
			Error::<Test>::InvalidYieldPerPeriod
		);
	});
}

#[test]
fn create_farm_with_inssufficient_balance_should_not_work() {
	//owner accont have 10K bsx
	new_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::create_farm(
				Origin::root(),
				1_000_001,
				1_000,
				1,
				BSX,
				BSX,
				ACC_1M,
				Permill::from_percent(20)
			),
			Error::<Test>::InsufficientRewardCurrencyBalance
		);
	});
}

#[test]
fn add_liquidity_pool_should_work() {
	//(AssetPair, LiqudityPoo, ammPoolId, Origin, farmId, now)

	//Note: global_pool.updated_at isn't changed because pool is empty (no. liq. pool stake in
	//globalPool)
	let test_data = vec![
		(
			AssetPair {
				asset_in: BSX,
				asset_out: ACA,
			},
			LiquidityPoolYieldFarm {
				id: 6,
				updated_at: 17,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rps: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: 20_000,
				loyalty_curve: Some(LoyaltyCurve::default()),
				nft_class: 2,
				canceled: false,
			},
			BSX_ACA_AMM,
			ALICE,
			ALICE_FARM,
			17_850,
			GlobalPool {
				liq_pools_count: 1,
				..PREDEFINED_GLOBAL_POOLS[0].clone()
			},
		),
		(
			AssetPair {
				asset_in: BSX,
				asset_out: KSM,
			},
			LiquidityPoolYieldFarm {
				id: 7,
				updated_at: 17,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rps: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: 10_000,
				loyalty_curve: None,
				nft_class: 3,
				canceled: false,
			},
			BSX_KSM_AMM,
			ALICE,
			ALICE_FARM,
			17_850,
			GlobalPool {
				liq_pools_count: 2,
				..PREDEFINED_GLOBAL_POOLS[0].clone()
			},
		),
		(
			AssetPair {
				asset_in: BSX,
				asset_out: ETH,
			},
			LiquidityPoolYieldFarm {
				id: 8,
				updated_at: 20,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rps: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: 10_000,
				loyalty_curve: Some(LoyaltyCurve {
					initial_reward_percentage: FixedU128::from_inner(100_000_000_000_000_000),
					scale_coef: 50,
				}),
				nft_class: 4,
				canceled: false,
			},
			BSX_ETH_AMM,
			ALICE,
			ALICE_FARM,
			20_000,
			GlobalPool {
				liq_pools_count: 3,
				..PREDEFINED_GLOBAL_POOLS[0].clone()
			},
		),
		(
			AssetPair {
				asset_in: BSX,
				asset_out: ETH,
			},
			LiquidityPoolYieldFarm {
				id: 9,
				updated_at: 2,
				total_shares: 0,
				total_valued_shares: 0,
				accumulated_rps: 0,
				accumulated_rpz: 0,
				stake_in_global_pool: 0,
				multiplier: 50_000,
				loyalty_curve: Some(LoyaltyCurve {
					initial_reward_percentage: FixedU128::from_inner(1),
					scale_coef: 0,
				}),
				nft_class: 5,
				canceled: false,
			},
			BSX_ETH_AMM,
			BOB,
			BOB_FARM,
			20_000,
			GlobalPool {
				liq_pools_count: 1,
				..PREDEFINED_GLOBAL_POOLS[1].clone()
			},
		),
	];

	predefined_test_ext().execute_with(|| {
		for (assets, pool, amm_id, who, farm_id, now, g_pool) in test_data.clone() {
			run_to_block(now);
			assert_ok!(LiquidityMining::add_liquidity_pool(
				Origin::signed(who),
				farm_id,
				assets,
				pool.multiplier,
				pool.loyalty_curve.clone()
			));

			expect_events(vec![Event::LiquidityPoolAdded(farm_id, amm_id, pool.clone()).into()]);

			assert_eq!(LiquidityMining::global_pool(farm_id).unwrap(), g_pool);
		}

		for (_, pool, amm_id, _, farm_id, _, _) in test_data {
			assert_eq!(LiquidityMining::liquidity_pool(farm_id, amm_id).unwrap(), pool);
		}
	});
}

#[test]
fn add_liquidity_pool_non_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(BOB),
				ALICE_FARM,
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
				10_000,
				None
			),
			Error::<Test>::Forbidden
		);

		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(BOB),
				ALICE_FARM,
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
				10_000,
				Some(LoyaltyCurve::default())
			),
			Error::<Test>::Forbidden
		);
	});
}

#[test]
fn add_liquidity_pool_invalid_loyalty_curve_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let curves = vec![
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::one(),
				scale_coef: 0,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from_float(1.0),
				scale_coef: 1_000_000,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from_float(1.000_000_000_000_000_001),
				scale_coef: 25_996_000,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from(1_u128),
				scale_coef: 25_996_000,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from(5_u128),
				scale_coef: 25_996_000,
			}),
			Some(LoyaltyCurve {
				initial_reward_percentage: FixedU128::from(16_874_354_654_u128),
				scale_coef: 25_996_000,
			}),
		];

		for c in curves {
			assert_noop!(
				LiquidityMining::add_liquidity_pool(
					Origin::signed(ALICE),
					ALICE_FARM,
					AssetPair {
						asset_in: BSX,
						asset_out: HDX,
					},
					10_000,
					c
				),
				Error::<Test>::InvalidLoyaltyCurverParamB
			);
		}
	});
}

#[test]
fn add_liquidity_pool_invalid_weight_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					asset_in: BSX,
					asset_out: HDX,
				},
				0,
				Some(LoyaltyCurve::default())
			),
			Error::<Test>::InvalidMultiplier
		);
	});
}

#[test]
fn add_liquidity_pool_non_existing_amm_should_not_work() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					//AMM for this assetPair does not exist
					asset_in: BSX,
					asset_out: 999_999_999,
				},
				10_000,
				Some(LoyaltyCurve::default())
			),
			Error::<Test>::AmmPoolDoesNotExist
		);
	});
}

#[test]
fn add_liquidity_pool_add_duplicate_amm_should_not_work() {
	predefined_test_ext().execute_with(|| {
		run_to_block(20_000);
		assert_ok!(LiquidityMining::add_liquidity_pool(
			Origin::signed(ALICE),
			ALICE_FARM,
			AssetPair {
				//AMM for this assetPair does not exist
				asset_in: BSX,
				asset_out: ACA,
			},
			10_000,
			Some(LoyaltyCurve::default())
		));

		let existing_pool = LiquidityPoolYieldFarm {
			id: 6,
			updated_at: 20,
			total_shares: 0,
			total_valued_shares: 0,
			accumulated_rps: 0,
			accumulated_rpz: 0,
			loyalty_curve: Some(LoyaltyCurve::default()),
			stake_in_global_pool: 0,
			multiplier: 10_000,
			nft_class: 2,
			canceled: false,
		};
		assert_eq!(
			LiquidityMining::liquidity_pool(ALICE_FARM, BSX_ACA_AMM).unwrap(),
			existing_pool
		);

		expect_events(vec![
			Event::LiquidityPoolAdded(ALICE_FARM, BSX_ACA_AMM, existing_pool).into()
		]);

		//try to add duplicate pool
		//in the same block(period)
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					//AMM for this assetPair does not exist
					asset_in: BSX,
					asset_out: ACA,
				},
				9_000,
				Some(LoyaltyCurve::default()),
			),
			Error::<Test>::LiquidityPoolAlreadyExists
		);

		run_to_block(30_000);
		//in later block(period)
		assert_noop!(
			LiquidityMining::add_liquidity_pool(
				Origin::signed(ALICE),
				ALICE_FARM,
				AssetPair {
					//AMM for this assetPair does not exist
					asset_in: BSX,
					asset_out: ACA,
				},
				9_000,
				Some(LoyaltyCurve::default()),
			),
			Error::<Test>::LiquidityPoolAlreadyExists
		);
	});
}

#[test]
fn destroy_farm_should_work() {
	predefined_test_ext().execute_with(|| {
		//remove all rewards from reward account
		let farm_account = LiquidityMining::pool_account_id(BOB_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_POOLS[1].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[1].reward_currency, &farm_account),
			0
		);

		assert_ok!(LiquidityMining::destroy_farm(Origin::signed(BOB), BOB_FARM));

		expect_events(vec![Event::FarmDestroyed(BOB_FARM, BOB).into()]);

		assert_eq!(LiquidityMining::global_pool(BOB_FARM).is_none(), true);
	});
}

#[test]
fn destroy_farm_not_owner_should_not_work() {
	predefined_test_ext().execute_with(|| {
		//remove all rewards from reward account
		let farm_account = LiquidityMining::pool_account_id(BOB_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_POOLS[1].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[1].reward_currency, &farm_account),
			0
		);

		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(ALICE), BOB_FARM),
			Error::<Test>::Forbidden
		);

		assert_eq!(
			LiquidityMining::global_pool(BOB_FARM).unwrap(),
			PREDEFINED_GLOBAL_POOLS[1]
		);
	});
}

#[test]
fn destroy_farm_farm_not_exists_should_not_work() {
	predefined_test_ext().execute_with(|| {
		const NON_EXISTING_FARM: u32 = 999_999_999;
		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(ALICE), NON_EXISTING_FARM),
			Error::<Test>::FarmNotFound
		);
	});
}

#[test]
fn destroy_farm_with_pools_should_not_work() {
	//in this case all rewards was distributed but liq. pool still exists in farm
	predefined_test_ext().execute_with(|| {
		//remove all rewards from reward account
		let farm_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let _ = Tokens::transfer_all(
			Origin::signed(farm_account),
			TREASURY,
			PREDEFINED_GLOBAL_POOLS[2].reward_currency,
			false,
		);
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[2].reward_currency, &farm_account),
			0
		);

		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(GC), GC_FARM),
			Error::<Test>::FarmIsNotEmpty
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			PREDEFINED_GLOBAL_POOLS[2]
		);
	});
}

#[test]
fn destroy_farm_with_undistributed_rewards_and_no_pools_should_not_work() {
	predefined_test_ext().execute_with(|| {
		let farm_account = LiquidityMining::pool_account_id(BOB_FARM).unwrap();
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[1].reward_currency, &farm_account).is_zero(),
			false
		);

		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(BOB), BOB_FARM),
			Error::<Test>::RewardBalanceIsNotZero
		);

		assert_eq!(
			LiquidityMining::global_pool(BOB_FARM).unwrap(),
			PREDEFINED_GLOBAL_POOLS[1]
		);
	});
}

#[test]
fn destroy_farm_healthy_should_not_work() {
	//farm with undistributed rewards and liq. pools
	predefined_test_ext().execute_with(|| {
		let farm_account = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		assert_eq!(
			Tokens::free_balance(PREDEFINED_GLOBAL_POOLS[2].reward_currency, &farm_account).is_zero(),
			false
		);

		assert_noop!(
			LiquidityMining::destroy_farm(Origin::signed(GC), GC_FARM),
			Error::<Test>::FarmIsNotEmpty
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			PREDEFINED_GLOBAL_POOLS[2]
		);
	});
}

#[test]
fn deposit_shares_should_work() {
	//NOTE: farm incentivize BSX token
	predefined_test_ext().execute_with(|| {
		let farm_id = GC_FARM;
		let amm_1 = AssetPair {
			asset_in: BSX,
			asset_out: TO1,
		};

		let amm_2 = AssetPair {
			asset_in: BSX,
			asset_out: TO2,
		};

		let pallet_acc = LiquidityMining::account_id();
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();
		let liq_pool_amm_1_farm_acc = LiquidityMining::pool_account_id(4).unwrap();
		let liq_pool_amm_2_farm_acc = LiquidityMining::pool_account_id(5).unwrap();
		let amm_1_acc = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(amm_1)).unwrap().0);
		let amm_2_acc = AMM_POOLS.with(|v| v.borrow().get(&asset_pair_to_map_key(amm_2)).unwrap().0);
		//DEPOSIT 1:
		run_to_block(1_800); //18-th period

		let alice_bsx_to1_shares = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 50, 0).unwrap();
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 0);

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_1,
			50
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 4, ALICE, 50, BSX_TO1_SHARE_ID, 0).into()
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 0,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 0,
				liq_pools_count: 2,
				paid_accumulated_rewards: 0,
				total_shares_z: 12_500,
				accumulated_rewards: 0
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 0,
				accumulated_rps: 0,
				accumulated_rpz: 0,
				total_shares: 50,
				total_valued_shares: 2_500,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 12_500,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 1, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(0, 0).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rps: 0,
				accumulated_claimed_rewards: 0,
				entered_period: 18,
			},
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_bsx_to1_shares - 50
		);
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 50);

		// DEPOSIT 2 (deposit in same period):
		let bob_bsx_to1_shares = Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB);
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 52, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(Origin::signed(BOB), farm_id, amm_1, 80));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 4, BOB, 80, BSX_TO1_SHARE_ID, 1).into()
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 18,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 9,
				liq_pools_count: 2,
				paid_accumulated_rewards: 112_500,
				total_shares_z: 33_300,
				accumulated_rewards: 0,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 18,
				accumulated_rps: 45,
				accumulated_rpz: 9,
				total_shares: 130,
				total_valued_shares: 6_660,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 33_300,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 2, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(0, 1).unwrap(),
			Deposit {
				shares: 80,
				valued_shares: 4_160,
				accumulated_rps: 45,
				accumulated_claimed_rewards: 0,
				entered_period: 18,
			},
		);

		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB), bob_bsx_to1_shares - 80);
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 130);

		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 112_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);

		// DEPOSIT 3 (same period, second liq pool yield farm):
		let bob_bsx_to2_shares = Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB);
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 8, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(Origin::signed(BOB), farm_id, amm_2, 25));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, BOB, 25, BSX_TO2_SHARE_ID, 0).into()
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 18,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 9,
				liq_pools_count: 2,
				paid_accumulated_rewards: 112_500,
				total_shares_z: 35_300,
				accumulated_rewards: 0,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 0,
				accumulated_rps: 0,
				accumulated_rpz: 0,
				total_shares: 25,
				total_valued_shares: 200,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 2_000,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 1, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(1, 0).unwrap(),
			Deposit {
				shares: 25,
				valued_shares: 200,
				accumulated_rps: 0,
				accumulated_claimed_rewards: 0,
				entered_period: 18,
			},
		);

		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB), bob_bsx_to2_shares - 25);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 25);

		//no pools update no transfers
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 112_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), 0);

		// DEPOSIT 4 (new period):
		run_to_block(2051); //period 20
		let bob_bsx_to2_shares = Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB);
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 58, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(BOB),
			farm_id,
			amm_2,
			800
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, BOB, 800, BSX_TO2_SHARE_ID, 1).into()
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 20,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 10,
				liq_pools_count: 2,
				paid_accumulated_rewards: 132_500,
				total_shares_z: 499_300,
				accumulated_rewards: 15_300,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 20,
				accumulated_rps: 100,
				accumulated_rpz: 10,
				total_shares: 825,
				total_valued_shares: 46_600,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 466_000,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 2, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(1, 1).unwrap(),
			Deposit {
				shares: 800,
				valued_shares: 46_400,
				accumulated_rps: 100,
				accumulated_claimed_rewards: 0,
				entered_period: 20,
			},
		);

		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB), bob_bsx_to2_shares - 800);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 825);

		//no pools update no transfers
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 132_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), 20_000);

		// DEPOSIT 5 (same period, second liq pool yield farm):
		run_to_block(2_586); //period 20
		let alice_bsx_to2_shares = Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE);
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 3, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_2,
			87
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, ALICE, 87, BSX_TO2_SHARE_ID, 2).into()
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 25,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 12,
				liq_pools_count: 2,
				total_shares_z: 501_910,
				accumulated_rewards: 331_550,
				paid_accumulated_rewards: 1_064_500,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 25,
				accumulated_rps: 120,
				accumulated_rpz: 12,
				total_shares: 912,
				total_valued_shares: 46_861,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 468_610,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 3, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(1, 2).unwrap(),
			Deposit {
				shares: 87,
				valued_shares: 261,
				accumulated_rps: 120,
				accumulated_claimed_rewards: 0,
				entered_period: 25,
			},
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE),
			alice_bsx_to2_shares - 87
		);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 912);

		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 1_064_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), (20_000 + 932_000)); //NOTE: 20k from prew deposit

		// DEPOSIT 6 (same period):
		run_to_block(2_596); //period 20
		let alice_bsx_to2_shares = Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE);
		Tokens::set_balance(Origin::root(), amm_2_acc, BSX, 16, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_2,
			48
		));

		expect_events(vec![
			Event::SharesDeposited(GC_FARM, 5, ALICE, 48, BSX_TO2_SHARE_ID, 3).into()
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 25,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 12,
				liq_pools_count: 2,
				total_shares_z: 509_590,
				accumulated_rewards: 331_550,
				paid_accumulated_rewards: 1_064_500,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 25,
				accumulated_rps: 120,
				accumulated_rpz: 12,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 4, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(1, 3).unwrap(),
			Deposit {
				shares: 48,
				valued_shares: 768,
				accumulated_rps: 120,
				accumulated_claimed_rewards: 0,
				entered_period: 25,
			},
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE),
			alice_bsx_to2_shares - 48
		);
		assert_eq!(Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc), 960);

		//no pools update no transfers
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 1_064_500));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 112_500);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), (20_000 + 932_000)); //NOTE: 20k from prew deposit

		// DEPOSIT 7 : (same period differen liq poll farm)
		run_to_block(2_596); //period 20
		let alice_bsx_to1_shares = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		Tokens::set_balance(Origin::root(), amm_1_acc, BSX, 80, 0).unwrap();

		assert_ok!(LiquidityMining::deposit_shares(
			Origin::signed(ALICE),
			farm_id,
			amm_1,
			486
		));

		expect_events(vec![Event::SharesDeposited(
			GC_FARM,
			4,
			ALICE,
			486,
			BSX_TO1_SHARE_ID,
			2,
		)
		.into()]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 25,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 12,
				liq_pools_count: 2,
				total_shares_z: 703_990,
				accumulated_rewards: 231_650,
				paid_accumulated_rewards: 1_164_400,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 25,
				accumulated_rps: 60,
				accumulated_rpz: 12,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 227_700,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 3, GC_FARM));

		assert_eq!(
			LiquidityMining::deposit(0, 2).unwrap(),
			Deposit {
				shares: 486,
				valued_shares: 38_880,
				accumulated_rps: 60,
				accumulated_claimed_rewards: 0,
				entered_period: 25,
			},
		);

		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_bsx_to1_shares - 486
		);
		assert_eq!(Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc), 616);

		//no pools update no transfers
		assert_eq!(Tokens::free_balance(BSX, &g_pool_acc), (30_000_000_000 - 1_164_400));
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_1_farm_acc), 212_400);
		assert_eq!(Tokens::free_balance(BSX, &liq_pool_amm_2_farm_acc), 952_000);
	});
}
/*
#[test]
fn deposit_shares_should_not_work() {
	todo!()
}
*/

#[test]
fn claim_rewards_should_work() {
	predefined_test_ext_with_deposits().execute_with(|| {
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);
		let bsx_to1_lm_account = LiquidityMining::pool_account_id(4).unwrap();
		let bsx_to2_lm_account = LiquidityMining::pool_account_id(5).unwrap();
		let liq_pool_bsx_to1_rewarad_balance = Tokens::free_balance(BSX, &bsx_to1_lm_account);

		//claim A1.1  (dep A1 1-th time)
		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0));

		expect_events(vec![Event::RewardClaimed(ALICE, GC_FARM, 4, 79_906, BSX).into()]);

		assert_eq!(
			LiquidityMining::deposit(0, 0).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rps: 0,
				accumulated_claimed_rewards: 79_906,
				entered_period: 18,
			}
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + 79_906);
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_to1_lm_account),
			liq_pool_bsx_to1_rewarad_balance - 79_906
		);

		// claim B3.1
		run_to_block(3_056);
		let liq_pool_bsx_to2_rewarad_balance = Tokens::free_balance(BSX, &bsx_to2_lm_account);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 1, 2));

		expect_events(vec![Event::RewardClaimed(ALICE, GC_FARM, 5, 2_734, BSX).into()]);

		assert_eq!(
			LiquidityMining::deposit(1, 2).unwrap(),
			Deposit {
				shares: 87,
				valued_shares: 261,
				accumulated_rps: 120,
				accumulated_claimed_rewards: 2_734,
				entered_period: 25,
			}
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 30,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 14,
				liq_pools_count: 2,
				total_shares_z: 703_990,
				accumulated_rewards: 1_039_045,
				paid_accumulated_rewards: 2_116_980,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 30,
				accumulated_rps: 140,
				accumulated_rpz: 14,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + 2_734);
		//NOTE: + claim from global pool - paid reward to user
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_to2_lm_account),
			liq_pool_bsx_to2_rewarad_balance + 952_580 - 2_734
		);

		//run for log time(longer than planned_yielding_periods) without interaction or claim.
		//planned_yielding_periods = 500; 100 blocks per period
		//claim A1.2
		run_to_block(125_879);
		let liq_pool_bsx_to1_rewarad_balance = Tokens::free_balance(BSX, &bsx_to1_lm_account);
		let alice_bsx_balance = Tokens::free_balance(BSX, &ALICE);

		assert_ok!(LiquidityMining::claim_rewards(Origin::signed(ALICE), 0, 0));

		expect_events(vec![Event::RewardClaimed(ALICE, GC_FARM, 4, 7_477_183, BSX).into()]);

		assert_eq!(
			LiquidityMining::deposit(0, 0).unwrap(),
			Deposit {
				shares: 50,
				valued_shares: 2_500,
				accumulated_rps: 0,
				accumulated_claimed_rewards: 7_557_089,
				entered_period: 18,
			}
		);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 1_258,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 628,
				liq_pools_count: 2,
				total_shares_z: 703_990,
				accumulated_rewards: 293_025_705,
				paid_accumulated_rewards: 142_380_180,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 1_258,
				accumulated_rps: 3_140,
				accumulated_rpz: 628,
				total_shares: 616,
				total_valued_shares: 45_540,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 227_700,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 30,
				accumulated_rps: 140,
				accumulated_rpz: 14,
				total_shares: 960,
				total_valued_shares: 47_629,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 476_290,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);

		assert_eq!(Tokens::free_balance(BSX, &ALICE), alice_bsx_balance + 7_477_183);
		//NOTE: + claim from global pool - paid reward to user
		assert_eq!(
			Tokens::free_balance(BSX, &bsx_to1_lm_account),
			liq_pool_bsx_to1_rewarad_balance + 140_263_200 - 7_477_183
		);
	});
}

/*
#[test]
fn claim_rewards_should_not_work() {
	todo!()
}
*/

#[test]
fn withdraw_shares_should_work() {
	let amm_1 = AssetPair {
		asset_in: BSX,
		asset_out: TO1,
	};

	let amm_2 = AssetPair {
		asset_in: BSX,
		asset_out: TO2,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		const REWARD_CURRENCY: u32 = BSX;

		let pallet_acc = LiquidityMining::account_id();
		let liq_pool_amm_1_acc = LiquidityMining::pool_account_id(4).unwrap();
		let liq_pool_amm_2_acc = LiquidityMining::pool_account_id(5).unwrap();
		let g_pool_acc = LiquidityMining::pool_account_id(GC_FARM).unwrap();

		// withdraw 1A
		let alice_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let pallet_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 0, 0));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, 4, 79_906, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, 50).into(),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 25,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 12,
				liq_pools_count: 2,
				total_shares_z: 691_490,
				accumulated_rewards: 231_650,
				paid_accumulated_rewards: 1_164_400,
			}
		);

		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 25,
				accumulated_rps: 60,
				accumulated_rpz: 12,
				total_shares: 566,
				total_valued_shares: 43_040,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 215_200,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_rew_curr_balance + 79_906
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_amm_1_shares_balance + 50
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_amm_1_shares_balance - 50
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance
		);

		//liq pool farms balance checks
		//NOTE ... - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance - (79_906 + 70_094)
		);
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 70_094
		);

		assert_eq!(LiquidityMining::deposit(0, 0), None);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 2, GC_FARM));

		run_to_block(12_800);
		
        // withdraw 3B
		let alice_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let pallet_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 1, 2));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, 5, 100_324, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO2_SHARE_ID, 87).into(),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 688_880,
				accumulated_rewards: 11_552_595,
				paid_accumulated_rewards: 25_455_190,
			}
		);
            
        // this pool should not change
		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 25,
				accumulated_rps: 60,
				accumulated_rpz: 12,
				total_shares: 566,
				total_valued_shares: 43_040,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 215_200,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);
		
        assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 128,
				accumulated_rps: 630,
				accumulated_rpz: 63,
				total_shares: 873,
				total_valued_shares: 47_368,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 473_680,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);

		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_rew_curr_balance + 100_324
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE),
			alice_amm_2_shares_balance + 87
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_amm_1_shares_balance
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance - 87
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance
		);
		//NOTE ... pool reward - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			(liq_pool_amm_2_rew_curr_balance + 24_290_790 - (100_324 + 32_786))
		);

		//global pool balance checks
        //note ... + unclaimabe - pool reward
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 32_786 - 24_290_790
		);

		assert_eq!(LiquidityMining::deposit(1, 2), None);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 3, GC_FARM));
        
        // withdraw 3A
		let alice_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let pallet_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);


		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 0, 2));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, 4, 7_472_429, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO1_SHARE_ID, 486).into(),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 494_480,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);
            
		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 128,
				accumulated_rps: 315,
				accumulated_rpz: 63,
				total_shares: 80,
				total_valued_shares: 4_160,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 20_800,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);
		
		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_rew_curr_balance +  7_472_429
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &ALICE),
			alice_amm_1_shares_balance + 486
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			pallet_amm_1_shares_balance - 486
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance + 10_975_200 - (7_472_429 + 2_441_971)
		);
		//NOTE ... pool reward - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance
		);

		//global pool balance checks
        //note ... + unclaimabe - pool reward
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance +  2_441_971- 10_975_200
		);

		assert_eq!(LiquidityMining::deposit(0, 2), None);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 1, GC_FARM));

        // withdraw 2A
		let bob_amm_1_shares_balance = Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB);
		let bob_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);


		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(BOB), 0, 1));

		expect_events(vec![
			Event::RewardClaimed(BOB, GC_FARM, 4, 855_771, BSX).into(),
			Event::SharesWithdrawn(BOB, BSX_TO1_SHARE_ID, 80).into(),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 473_680,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);
            
		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 128,
				accumulated_rps: 315,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);
		
		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_rew_curr_balance + 855_771 
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &BOB),
			bob_amm_1_shares_balance + 80 
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			0 
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance - (855_771 + 267_429)
		);
		//NOTE ... pool reward - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance
		);

		//global pool balance checks
        //note ... + unclaimabe - pool reward
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance +  267_429
		);

		assert_eq!(LiquidityMining::deposit(0, 1), None);

		assert_eq!(LiquidityMining::nft_class(0).unwrap(), (amm_1, 0, GC_FARM));

        // withdraw 1B
		let bob_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB);
		let bob_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(BOB), 1, 0));

		expect_events(vec![
			Event::RewardClaimed(BOB, GC_FARM, 5, 95_999, BSX).into(),
			Event::SharesWithdrawn(BOB, BSX_TO2_SHARE_ID, 25).into(),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 471_680,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);
            
		assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO1_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 4,
				updated_at: 128,
				accumulated_rps: 315,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: 5,
				nft_class: 0,
				canceled: false,
			},
		);
		
        assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 128,
				accumulated_rps: 630,
				accumulated_rpz: 63,
				total_shares: 848,
				total_valued_shares: 47_168,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 471_680,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);
		
		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_rew_curr_balance +  95_999
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB),
			bob_amm_2_shares_balance + 25 
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			0 
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance - 25
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance
		);
		//NOTE ... - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance - (95_999 + 30_001)
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 30_001 
		);

		assert_eq!(LiquidityMining::deposit(1, 0), None);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 2, GC_FARM));
        
        // withdraw 4B
		let alice_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE);
		let alice_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &ALICE);
		let pallet_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);
		let liq_pool_amm_2_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(ALICE), 1, 3));

		expect_events(vec![
			Event::RewardClaimed(ALICE, GC_FARM, 5, 295_207, BSX).into(),
			Event::SharesWithdrawn(ALICE, BSX_TO2_SHARE_ID, 48).into(),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 464_000,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);
		
        assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 128,
				accumulated_rps: 630,
				accumulated_rpz: 63,
				total_shares: 800,
				total_valued_shares: 46_400,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 464_000,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);
		
		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &ALICE),
			alice_rew_curr_balance + 29_5207 
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &ALICE),
			alice_amm_2_shares_balance + 48
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			0 
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
			pallet_amm_2_shares_balance - 48
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance
		);
		//NOTE ... - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
			liq_pool_amm_2_rew_curr_balance - (29_5207 + 96_473)
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance +  96_473
		);

		assert_eq!(LiquidityMining::deposit(1, 3), None);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 1, GC_FARM));
        

        // withdraw 2B
		let bob_amm_2_shares_balance = Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB);
		let bob_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &BOB);
		let g_pool_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc);
		let liq_pool_amm_1_rew_curr_balance = Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc);

		assert_ok!(LiquidityMining::withdraw_shares(Origin::signed(BOB), 1, 1));

		expect_events(vec![
			Event::RewardClaimed(BOB, GC_FARM, 5, 18_680_461, BSX).into(),
            frame_system::Event::KilledAccount(29533360621462889584138678125).into(),
            Event::SharesWithdrawn(BOB, BSX_TO2_SHARE_ID, 800).into(),
		]);

		assert_eq!(
			LiquidityMining::global_pool(GC_FARM).unwrap(),
			GlobalPool {
				id: GC_FARM,
				updated_at: 128,
				reward_currency: BSX,
				yield_per_period: Permill::from_percent(50),
				planned_yielding_periods: 500_u64,
				blocks_per_period: 100_u64,
				owner: GC,
				incentivized_token: BSX,
				max_reward_per_period: 60_000_000,
				accumulated_rpz: 63,
				liq_pools_count: 2,
				total_shares_z: 0,
				accumulated_rewards: 577_395,
				paid_accumulated_rewards: 36_430_390,
			}
		);
		
        assert_eq!(
			LiquidityMining::liquidity_pool(GC_FARM, BSX_TO2_AMM).unwrap(),
			LiquidityPoolYieldFarm {
				id: 5,
				updated_at: 128,
				accumulated_rps: 630,
				accumulated_rpz: 63,
				total_shares: 0,
				total_valued_shares: 0,
				loyalty_curve: Some(LoyaltyCurve::default()),
				stake_in_global_pool: 0,
				multiplier: 10,
				nft_class: 1,
				canceled: false,
			},
		);
		
		//user balances checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &BOB),
			bob_rew_curr_balance + 18_680_461 
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &BOB),
			bob_amm_2_shares_balance +800
		);

		//pallet balances checks
		assert_eq!(
			Tokens::free_balance(BSX_TO1_SHARE_ID, &pallet_acc),
			0 
		);
		assert_eq!(
			Tokens::free_balance(BSX_TO2_SHARE_ID, &pallet_acc),
            0
		);

		//liq pool farms balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_1_acc),
			liq_pool_amm_1_rew_curr_balance
		);
		//NOTE ... - (reward + unclaimabe)
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &liq_pool_amm_2_acc),
            0
		);

		//global pool balance checks
		assert_eq!(
			Tokens::free_balance(REWARD_CURRENCY, &g_pool_acc),
			g_pool_rew_curr_balance + 5_911_539 
		);

		assert_eq!(LiquidityMining::deposit(1, 1), None);

		assert_eq!(LiquidityMining::nft_class(1).unwrap(), (amm_2, 0, GC_FARM));
	});
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
