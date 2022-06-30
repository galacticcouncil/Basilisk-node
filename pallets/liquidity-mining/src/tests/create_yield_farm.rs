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
use pretty_assertions::assert_eq;
use test_ext::*;

const BSX_ACA_ASSET_PAIR: AssetPair = AssetPair {
	asset_in: BSX,
	asset_out: ACA,
};

#[test]
fn create_yield_farm_should_work() {
	//Arrange
	let yield_farm = YieldFarmData {
		id: 12,
		updated_at: 17,
		total_shares: 0,
		total_valued_shares: 0,
		accumulated_rpvs: 0,
		accumulated_rpz: 0,
		multiplier: FixedU128::from(20_000_u128),
		loyalty_curve: Some(LoyaltyCurve::default()),
		state: FarmState::Active,
		entries_count: 0,
		_phantom: PhantomData,
	};

	let global_farm = GlobalFarmData {
		yield_farms_count: (1, 1),
		..PREDEFINED_GLOBAL_FARMS[0].clone()
	};

	predefined_test_ext().execute_with(|| {
		set_block_number(17_850);

		//Act
		assert_ok!(LiquidityMining::create_yield_farm(
			Origin::signed(ALICE),
			ALICE_FARM,
			BSX_ACA_ASSET_PAIR,
			yield_farm.multiplier,
			yield_farm.loyalty_curve.clone()
		));

		//Assert
		expect_events(vec![mock::Event::LiquidityMining(Event::YieldFarmCreated {
			farm_id: ALICE_FARM,
			yield_farm_id: yield_farm.id,
			multiplier: yield_farm.multiplier,
			nft_class: LIQ_MINING_NFT_CLASS,
			loyalty_curve: yield_farm.loyalty_curve.clone(),
			asset_pair: BSX_ACA_ASSET_PAIR,
		})]);

		assert_eq!(WarehouseLM::global_farm(ALICE_FARM).unwrap(), global_farm);
		assert_eq!(
			WarehouseLM::yield_farm((BSX_ACA_AMM, ALICE_FARM, yield_farm.id)).unwrap(),
			yield_farm
		);
	})
}

#[test]
fn create_yield_farm_should_fail_when_called_by_not_signed_owner() {
	predefined_test_ext().execute_with(|| {
		assert_noop!(
			LiquidityMining::create_yield_farm(
				Origin::none(),
				ALICE_FARM,
				BSX_ACA_ASSET_PAIR,
				FixedU128::from(20_000_u128),
				Some(LoyaltyCurve::default())
			),
			BadOrigin
		);
	});
}

#[test]
fn create_yield_farm_should_fail_when_asset_pair_has_not_known_asset() {
	let not_known_asset = 9999;
	let bsx_with_invalid_assets = AssetPair {
		asset_in: BSX,
		asset_out: not_known_asset,
	};

	predefined_test_ext_with_deposits().execute_with(|| {
		assert_noop!(
			LiquidityMining::create_yield_farm(
				Origin::signed(ALICE),
				ALICE_FARM,
				bsx_with_invalid_assets,
				FixedU128::from(20_000_u128),
				Some(LoyaltyCurve::default())
			),
			Error::<Test>::AmmPoolDoesNotExist
		);
	});
}
