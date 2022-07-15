#![cfg(test)]

use crate::kusama_test_net::*;

use basilisk_runtime::{DustRemovalWhitelist, Origin, XYK};
use hydradx_traits::AMM;
use primitives::{asset::AssetPair, Price};
use xcm_emulator::TestExt;

use frame_support::{assert_ok, traits::Contains};

#[test]
fn pair_account_should_be_added_into_whitelist_when_pool_is_created() {
	TestNet::reset();

	let asset_a = 1;
	let asset_b = 2;
	let asset_pair = AssetPair {
		asset_in: asset_a,
		asset_out: asset_b,
	};
	let pair_account = XYK::get_pair_id(asset_pair);

	Basilisk::execute_with(|| {
		//arrange & act
		assert_ok!(XYK::create_pool(
			Origin::signed(ALICE.into()),
			asset_a,
			asset_b,
			100 * UNITS,
			Price::from(2)
		));

		//assert
		assert!(DustRemovalWhitelist::contains(&pair_account));
	});
}

#[test]
fn pair_account_should_be_removed_from_whitelist_when_pool_was_destroyed() {
	TestNet::reset();

	let asset_a = 1;
	let asset_b = 2;
	let asset_pair = AssetPair {
		asset_in: asset_a,
		asset_out: asset_b,
	};
	let pair_account = XYK::get_pair_id(asset_pair);

	Basilisk::execute_with(|| {
		//arrange
		assert_ok!(XYK::create_pool(
			Origin::signed(ALICE.into()),
			asset_a,
			asset_b,
			100 * UNITS,
			Price::from(2)
		));
		assert!(DustRemovalWhitelist::contains(&pair_account));

		//act
		assert_ok!(XYK::remove_liquidity(
			Origin::signed(ALICE.into()),
			asset_a,
			asset_b,
			100 * UNITS
		));

		//assert
		assert!(!DustRemovalWhitelist::contains(&pair_account));
	});
}
