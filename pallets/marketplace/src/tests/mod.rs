use crate::mock::*;
use crate::*;

use frame_support::{assert_noop, assert_ok, BoundedVec};

use std::convert::TryInto;

type Market = Pallet<Test>;

fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

mod unit;
mod make_offer;
