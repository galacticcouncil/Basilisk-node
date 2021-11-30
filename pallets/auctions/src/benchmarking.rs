#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::benchmarks;

benchmarks! {
	create {
	}: {
	} verify {
	}

	update {
	}: {
	} verify {
	}

	destroy {
	}: {
	} verify {
	}

	bid {
	}: {
	} verify {
	}
}

#[cfg(test)]
mod tests {
	use super::Pallet;
	use crate::mock::*;
	use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, super::ExtBuilder::default().build(), super::Test);
}
