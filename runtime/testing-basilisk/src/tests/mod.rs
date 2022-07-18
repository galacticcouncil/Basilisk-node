use super::*;
use crate::mock::System;
use crate::mock::*;

#[cfg(test)]
mod fees;
#[cfg(test)]
mod math;
#[cfg(test)]
mod vesting;

fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| set_block_number::<Test>(1));
	ext
}

fn run_with_system_weight<F>(w: Weight, mut assertions: F)
where
	F: FnMut(),
{
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap()
		.into();
	t.execute_with(|| {
		System::set_block_consumed_resources(w, 0);
		assertions()
	});
}
