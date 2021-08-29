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

use super::Error;
use crate::mock::*;
use crate::types::{AssetDetails, AssetMetadata, AssetType};
use codec::Encode;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use polkadot_xcm::v0::{Junction::*, MultiLocation::*};
use primitives::AssetId;
use sp_std::convert::TryInto;

#[test]
fn register_asset_works() {
	new_test_ext().execute_with(|| {
		let too_long = [1u8; <Test as crate::Config>::StringLimit::get() as usize + 1];
		assert_noop!(
			AssetRegistryPallet::register(Origin::root(), too_long.to_vec(), AssetType::Token),
			Error::<Test>::TooLong
		);

		let name: Vec<u8> = b"HDX".to_vec();

		assert_ok!(AssetRegistryPallet::register(
			Origin::root(),
			name.clone(),
			AssetType::Token,
		));

		let bn = AssetRegistryPallet::to_bounded_name(name.clone()).unwrap();

		assert_eq!(AssetRegistryPallet::asset_ids(&bn).unwrap(), 1u32);
		assert_eq!(
			AssetRegistryPallet::assets(1u32).unwrap(),
			AssetDetails {
				name: bn,
				asset_type: AssetType::Token,
				locked: false
			}
		);

		assert_noop!(
			AssetRegistryPallet::register(Origin::root(), name.clone(), AssetType::Token),
			Error::<Test>::AssetAlreadyRegistered
		);
	});
}

#[test]
fn create_asset() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetRegistryPallet::get_or_create_asset(
			b"HDX".to_vec(),
			AssetType::Token
		));

		let dot_asset = AssetRegistryPallet::get_or_create_asset(b"DOT".to_vec(), AssetType::Token);
		assert_ok!(dot_asset);
		let dot_asset_id = dot_asset.ok().unwrap();

		assert_ok!(AssetRegistryPallet::get_or_create_asset(
			b"BTC".to_vec(),
			AssetType::Token
		));

		let current_asset_id = AssetRegistryPallet::next_asset_id();

		// Existing asset should return previously created one.
		assert_ok!(
			AssetRegistryPallet::get_or_create_asset(b"DOT".to_vec(), AssetType::Token),
			dot_asset_id
		);

		// Retrieving existing asset should not increased the next asset id counter.
		assert_eq!(AssetRegistryPallet::next_asset_id(), current_asset_id);

		let dot: BoundedVec<u8, <Test as crate::Config>::StringLimit> = b"DOT".to_vec().try_into().unwrap();
		let aaa: BoundedVec<u8, <Test as crate::Config>::StringLimit> = b"AAA".to_vec().try_into().unwrap();

		assert_eq!(AssetRegistryPallet::asset_ids(dot).unwrap(), 2u32);
		assert!(AssetRegistryPallet::asset_ids(aaa).is_none());
	});
}

#[test]
fn location_mapping_works() {
	new_test_ext().execute_with(|| {
		let bn = AssetRegistryPallet::to_bounded_name(b"HDX".to_vec()).unwrap();
		assert_ok!(AssetRegistryPallet::get_or_create_asset(
			b"HDX".to_vec(),
			AssetType::Token
		));
		let asset_id: AssetId = AssetRegistryPallet::get_or_create_asset(b"HDX".to_vec(), AssetType::Token).unwrap();

		crate::Assets::<Test>::insert(
			asset_id,
			AssetDetails::<AssetId, BoundedVec<u8, RegistryStringLimit>> {
				name: bn,
				asset_type: AssetType::Token,
				locked: false,
			},
		);

		let asset_location = AssetLocation(X3(Parent, Parachain(200), GeneralKey(asset_id.encode())));

		assert_ok!(AssetRegistryPallet::set_location(
			Origin::root(),
			asset_id,
			asset_location.clone()
		));

		assert_eq!(
			AssetRegistryPallet::location_to_asset(asset_location.clone()),
			Some(asset_id)
		);
		assert_eq!(AssetRegistryPallet::asset_to_location(asset_id), Some(asset_location));
	});
}

#[test]
fn genesis_config_works() {
	ExtBuilder::default()
		.with_native_asset_name(b"NATIVE".to_vec())
		.build()
		.execute_with(|| {
			let native: BoundedVec<u8, <Test as crate::Config>::StringLimit> = b"NATIVE".to_vec().try_into().unwrap();
			assert_eq!(AssetRegistryPallet::asset_ids(native).unwrap(), 0u32);
		});
	ExtBuilder::default()
		.with_assets(vec![b"ONE".to_vec()])
		.build()
		.execute_with(|| {
			let native: BoundedVec<u8, <Test as crate::Config>::StringLimit> = b"NATIVE".to_vec().try_into().unwrap();
			assert_eq!(AssetRegistryPallet::asset_ids(native), None);

			let bsx: BoundedVec<u8, <Test as crate::Config>::StringLimit> = b"BSX".to_vec().try_into().unwrap();
			assert_eq!(AssetRegistryPallet::asset_ids(bsx).unwrap(), 0u32);

			let one: BoundedVec<u8, <Test as crate::Config>::StringLimit> = b"ONE".to_vec().try_into().unwrap();
			assert_eq!(AssetRegistryPallet::asset_ids(one).unwrap(), 1u32);
		});
}

#[test]
fn set_metadata_works() {
	ExtBuilder::default()
		.with_assets(vec![b"DOT".to_vec()])
		.build()
		.execute_with(|| {
			let dot: BoundedVec<u8, <Test as crate::Config>::StringLimit> = b"DOT".to_vec().try_into().unwrap();
			let dot_id = AssetRegistryPallet::asset_ids(dot).unwrap();
			let b_symbol: BoundedVec<u8, <Test as crate::Config>::StringLimit> = b"xDOT".to_vec().try_into().unwrap();

			assert_ok!(AssetRegistryPallet::set_metadata(
				Origin::root(),
				dot_id,
				b"xDOT".to_vec(),
				12u8
			));

			assert_eq!(
				AssetRegistryPallet::asset_metadata(dot_id).unwrap(),
				AssetMetadata {
					decimals: 12u8,
					symbol: b_symbol.clone(),
				}
			);

			assert_ok!(AssetRegistryPallet::set_metadata(
				Origin::root(),
				dot_id,
				b"xDOT".to_vec(),
				30u8
			));

			assert_eq!(
				AssetRegistryPallet::asset_metadata(dot_id).unwrap(),
				AssetMetadata {
					decimals: 30u8,
					symbol: b_symbol
				}
			);

			assert_noop!(
				AssetRegistryPallet::set_metadata(Origin::root(), dot_id, b"JUST_TOO_LONG".to_vec(), 30u8),
				Error::<Test>::TooLong
			);

			assert_noop!(
				AssetRegistryPallet::set_metadata(Origin::root(), 100, b"NONE".to_vec(), 30u8),
				Error::<Test>::AssetNotFound
			);
		});
}
