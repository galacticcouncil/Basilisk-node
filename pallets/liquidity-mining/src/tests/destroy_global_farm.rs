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
fn destroy_global_farm_should_work() {
	predefined_test_ext().execute_with(|| {
		assert_ok!(LiquidityMining::destroy_global_farm(Origin::signed(BOB), BOB_FARM));

		assert!(WarehouseLM::global_farm(BOB_FARM).is_none());
		assert_last_event!(crate::Event::GlobalFarmDestroyed {
			global_farm_id: BOB_FARM,
			who: BOB,
			reward_currency: KSM,
			undistributed_rewards: BOB_GLOBAL_FARM_TOTAL_REWARDS,
		}
		.into());
	});
}

#[test]
fn destroy_global_farm_should_fail_when_origin_is_not_signed() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::destroy_global_farm(mock::Origin::none(), BOB_FARM),
			BadOrigin
		);

		assert_eq!(WarehouseLM::global_farm(BOB_FARM).unwrap(), PREDEFINED_GLOBAL_FARMS[1]);
	});
}

#[test]
fn destroy_global_farm_should_fail_with_propagated_error_when_farm_does_not_exist() {
	const NON_EXISTING_FARM: u32 = 999_999_999;

	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::destroy_global_farm(Origin::signed(BOB), NON_EXISTING_FARM),
			warehouse_liquidity_mining::Error::<Test, Instance1>::GlobalFarmNotFound
		);
	});
}
