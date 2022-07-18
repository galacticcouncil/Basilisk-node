use super::*;
use frame_support::weights::DispatchClass;
use sp_runtime::traits::Convert;
use sp_runtime::FixedPointNumber;

#[test]
fn multiplier_can_grow_from_zero() {
	let minimum_multiplier = MinimumMultiplier::get();
	let target = TargetBlockFullness::get() * BlockWeights::get().get(DispatchClass::Normal).max_total.unwrap();
	// if the min is too small, then this will not change, and we are doomed forever.
	// the weight is 1/100th bigger than target.
	run_with_system_weight(target * 101 / 100, || {
		let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
		assert!(next > minimum_multiplier, "{:?} !>= {:?}", next, minimum_multiplier);
	})
}

#[test]
#[ignore]
fn multiplier_growth_simulator() {
	// calculate the value of the fee multiplier after one hour of operation with fully loaded blocks
	let mut multiplier = Multiplier::saturating_from_integer(1);
	let block_weight = BlockWeights::get().get(DispatchClass::Normal).max_total.unwrap();
	for _block_num in 1..=HOURS {
		run_with_system_weight(block_weight, || {
			let next = SlowAdjustingFeeUpdate::<Runtime>::convert(multiplier);
			// ensure that it is growing as well.
			assert!(next > multiplier, "{:?} !>= {:?}", next, multiplier);
			multiplier = next;
		});
	}
	println!("multiplier = {:?}", multiplier);
}
