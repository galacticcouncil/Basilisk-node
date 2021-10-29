use super::*;
use crate::mock::{
	BlockNumber, Event as TestEvent, ExtBuilder, LiquidityMining, Origin, System, Test, Tokens, ACA, BSX, HDX, KSM,
	TREASURY,
};

use primitives::{AssetId, Balance};

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

	for t in testing_values.iter() {
		assert_eq!(LiquidityMining::get_reward_per_period(t.0, t.1, t.2).unwrap(), t.3);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=478231890
fn get_new_accumulated_reward_per_share_should_work() {
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
		assert_eq!(LiquidityMining::get_new_accumulated_rps(t.0, t.1, t.2).unwrap(), t.3);
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
			LiquidityMining::get_user_reward(t.0, t.1, t.2, t.3, t.4).unwrap(),
			(t.5, t.6)
		);
	}
}

#[test]
/// https://docs.google.com/spreadsheets/d/1iSBWBM8XLalMkI4djhcFWRSxz-S4CHtjadoLzGxMD74/edit#gid=2010118745
fn update_global_pool_should_work() {
	//[(pool.updated_at, pool.total_shares, pool.accumulated_rps, pool.reward_currency,
	//pool_id, reward_left_to_distribute, period_now, reward_per_period, pool.accumulated_rps),...]
	let testing_values = vec![
		(
			BlockNumber::from(26_u64),
			Balance::from(25019447565169_u128),
			Balance::from(259_u128),
			HDX,
			11000_u64,
			Balance::from(48470226114_u128),
			BlockNumber::from(206_u64),
			Balance::from(651206_u128),
			Balance::from(259_u128),
		),
		(
			BlockNumber::from(188_u64),
			Balance::from(33769488247603_u128),
			Balance::from(1148_u128),
			BSX,
			11000_u64,
			Balance::from(30080406306_u128),
			BlockNumber::from(259_u64),
			Balance::from(1548635_u128),
			Balance::from(1148_u128),
		),
		(
			BlockNumber::from(195_u64),
			Balance::from(26098384286056_u128),
			Balance::from(523_u128),
			ACA,
			11000_u64,
			Balance::from(32055_u128),
			BlockNumber::from(326_u64),
			Balance::from(1712797_u128),
			Balance::from(523_u128),
		),
		(
			BlockNumber::from(181_u64),
			Balance::from(9894093650144_u128),
			Balance::from(317_u128),
			KSM,
			11000_u64,
			Balance::from(36806694280_u128),
			BlockNumber::from(1856_u64),
			Balance::from(1900156_u128),
			Balance::from(317_u128),
		),
		(
			BlockNumber::from(196_u64),
			Balance::from(26886423482043_u128),
			Balance::from(596_u128),
			ACA,
			14000_u64,
			Balance::from(30560755872_u128),
			BlockNumber::from(954_u64),
			Balance::from(78355_u128),
			Balance::from(596_u128),
		),
		(
			BlockNumber::from(68_u64),
			Balance::from(11380750657342_u128),
			Balance::from(4_u128),
			ACA,
			12000_u64,
			Balance::from(38398062768_u128),
			BlockNumber::from(161_u64),
			Balance::from(553033_u128),
			Balance::from(4_u128),
		),
		(
			BlockNumber::from(161_u64),
			Balance::from(24495534649923_u128),
			Balance::from(213_u128),
			KSM,
			14000_u64,
			Balance::from(11116735745_u128),
			BlockNumber::from(448_u64),
			Balance::from(1884698_u128),
			Balance::from(213_u128),
		),
		(
			BlockNumber::from(27_u64),
			Balance::from(22108454336644_u128),
			Balance::from(970_u128),
			KSM,
			13000_u64,
			Balance::from(8572779460_u128),
			BlockNumber::from(132_u64),
			Balance::from(1874081_u128),
			Balance::from(970_u128),
		),
		(
			BlockNumber::from(97_u64),
			Balance::from(33115801593208_u128),
			Balance::from(6_u128),
			HDX,
			10000_u64,
			Balance::from(18440792496_u128),
			BlockNumber::from(146_u64),
			Balance::from(741803_u128),
			Balance::from(6_u128),
		),
		(
			BlockNumber::from(154_u64),
			Balance::from(27279119649838_u128),
			Balance::from(713_u128),
			BSX,
			10000_u64,
			Balance::from(28318566664_u128),
			BlockNumber::from(202_u64),
			Balance::from(508869_u128),
			Balance::from(713_u128),
		),
		(
			BlockNumber::from(104_u64),
			Balance::from(20462312838954_u128),
			Balance::from(833_u128),
			BSX,
			12000_u64,
			Balance::from(3852003_u128),
			BlockNumber::from(131_u64),
			Balance::from(1081636_u128),
			Balance::from(833_u128),
		),
		(
			BlockNumber::from(90_u64),
			Balance::from(37650830596054_u128),
			Balance::from(586_u128),
			HDX,
			13000_u64,
			Balance::from(27990338179_u128),
			BlockNumber::from(110_u64),
			Balance::from(758482_u128),
			Balance::from(586_u128),
		),
		(
			BlockNumber::from(198_u64),
			Balance::from(31877785441215_u128),
			Balance::from(251_u128),
			ACA,
			10000_u64,
			Balance::from(3615346492_u128),
			BlockNumber::from(582_u64),
			Balance::from(69329_u128),
			Balance::from(251_u128),
		),
		(
			BlockNumber::from(29_u64),
			Balance::from(33478250_u128),
			Balance::from(77_u128),
			BSX,
			11000_u64,
			Balance::from(39174031245_u128),
			BlockNumber::from(100_u64),
			Balance::from(1845620_u128),
			Balance::from(80_u128),
		),
		(
			BlockNumber::from(91_u64),
			Balance::from(39392283517372_u128),
			Balance::from(2491_u128),
			ACA,
			11000_u64,
			Balance::from(6348629400_u128),
			BlockNumber::from(260_u64),
			Balance::from(1091233_u128),
			Balance::from(2491_u128),
		),
		(
			BlockNumber::from(67_u64),
			Balance::from(11290609546422_u128),
			Balance::from(295_u128),
			HDX,
			11000_u64,
			Balance::from(7492177402_u128),
			BlockNumber::from(229_u64),
			Balance::from(1227791_u128),
			Balance::from(295_u128),
		),
		(
			BlockNumber::from(168_u64),
			Balance::from(28351324279041_u128),
			Balance::from(450_u128),
			ACA,
			11000_u64,
			Balance::from(38796364068_u128),
			BlockNumber::from(361_u64),
			Balance::from(1015284_u128),
			Balance::from(450_u128),
		),
		(
			BlockNumber::from(3_u64),
			Balance::from(17631376575792_u128),
			Balance::from(82_u128),
			HDX,
			13000_u64,
			Balance::from(20473946880_u128),
			BlockNumber::from(52_u64),
			Balance::from(1836345_u128),
			Balance::from(82_u128),
		),
		(
			BlockNumber::from(49_u64),
			Balance::from(94059_u128),
			Balance::from(81_u128),
			HDX,
			14000_u64,
			Balance::from(11126653978_u128),
			BlockNumber::from(132_u64),
			Balance::from(1672829_u128),
			Balance::from(1557_u128),
		),
		(
			BlockNumber::from(38_u64),
			Balance::from(19307247584085_u128),
			Balance::from(266_u128),
			KSM,
			10000_u64,
			Balance::from(36115448964_u128),
			BlockNumber::from(400000_u64),
			Balance::from(886865_u128),
			Balance::from(266_u128),
		),
		(
			BlockNumber::from(158_u64),
			Balance::from(782023762784_u128),
			Balance::from(129_u128),
			BSX,
			11000_u64,
			Balance::from(21814882774_u128),
			BlockNumber::from(158_u64),
			Balance::from(789730_u128),
			Balance::from(129_u128),
		),
	];

	//[(pool.updated_at, pool.total_shares, pool.accumulated_rps, pool.reward_currency,
	//pool_id, reward_left_to_distribute, period_now, reward_per_period, pool.accumulated_rps),...]
	for t in testing_values.iter() {
		let mut p = GlobalPool {
			updated_at: t.0,
			total_shares: t.1,
			accumulated_rps: t.2,
			reward_currency: t.3,
		};

		let mut ext = new_test_ext();

		ext.execute_with(|| {
			let _ = Tokens::transfer(Origin::signed(TREASURY), t.4, t.3, t.5);
			assert_eq!(Tokens::free_balance(t.3, &t.4), t.5);

			LiquidityMining::update_global_pool(t.4, &mut p, t.6, t.7).unwrap();

			assert_eq!(p.accumulated_rps, t.8);

			//NOTE: don't check transer - this function doesn't transfer funds
		});
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
