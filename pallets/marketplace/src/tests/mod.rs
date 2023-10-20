use crate::mock::*;
use crate::*;

use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::Dispatchable;

mod accept_offer;
mod add_royalty;
mod buy;
mod make_offer;
mod set_price;
mod withdraw_offer;

type Market = Pallet<Test>;
type Origin = RuntimeOrigin;
