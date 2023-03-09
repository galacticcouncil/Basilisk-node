#![cfg(test)]

use basilisk_runtime::{AssetsAccountId, Runtime};
use hydradx_traits::AccountIdFor;
use pretty_assertions::{assert_eq, assert_ne};
use primitives::AssetId;

#[test]
fn name_should_produce_same_result_for_unsorted_assets_when_identifier_is_none() {
	let sorted_assets: Vec<AssetId> = vec![1, 2, 3, 4, 5, 6];
	let unsorted_assets: Vec<AssetId> = vec![3, 2, 4, 1, 6, 5];

	assert_eq!(
		AssetsAccountId::<Runtime>::name(&sorted_assets, None),
		AssetsAccountId::<Runtime>::name(&unsorted_assets, None),
	);
}

#[test]
fn name_should_produce_same_result_for_unsorted_assets_when_identifier_is_provided() {
	let sorted_assets: Vec<AssetId> = vec![1, 2, 3, 4, 5, 6];
	let unsorted_assets: Vec<AssetId> = vec![3, 2, 4, 1, 6, 5];

	let identifier = Some("identfier".as_bytes());

	assert_eq!(
		AssetsAccountId::<Runtime>::name(&sorted_assets, identifier),
		AssetsAccountId::<Runtime>::name(&unsorted_assets, identifier)
	);
}

#[test]
fn from_assets_should_produce_same_result_for_unsorted_assets_when_identifier_is_none() {
	let sorted_assets: Vec<AssetId> = vec![1, 2, 3, 4, 5, 6];
	let unsorted_assets: Vec<AssetId> = vec![3, 2, 4, 1, 6, 5];

	assert_eq!(
		AssetsAccountId::<Runtime>::from_assets(&sorted_assets, None),
		AssetsAccountId::<Runtime>::from_assets(&unsorted_assets, None),
	);
}

#[test]
fn from_assets_should_produce_same_result_for_unsorted_assets_when_identifier_is_provided() {
	let sorted_assets: Vec<AssetId> = vec![1, 2, 3, 4, 5, 6];
	let unsorted_assets: Vec<AssetId> = vec![3, 2, 4, 1, 6, 5];

	let identifier = Some("identfier".as_bytes());

	assert_eq!(
		AssetsAccountId::<Runtime>::from_assets(&sorted_assets, identifier),
		AssetsAccountId::<Runtime>::from_assets(&unsorted_assets, identifier)
	);
}

#[test]
fn name_whith_and_without_indetifier_should_be_different() {
	let assets: Vec<AssetId> = vec![1, 2, 3, 4, 5, 6];

	let identifier = Some("identfier".as_bytes());

	assert_ne!(
		AssetsAccountId::<Runtime>::name(&assets, None),
		AssetsAccountId::<Runtime>::name(&assets, identifier)
	);
}

#[test]
fn from_assets_whith_and_without_indetifier_should_be_different() {
	let assets: Vec<AssetId> = vec![1, 2, 3, 4, 5, 6];

	let identifier = Some("identfier".as_bytes());

	assert_ne!(
		AssetsAccountId::<Runtime>::from_assets(&assets, None),
		AssetsAccountId::<Runtime>::from_assets(&assets, identifier)
	);
}
