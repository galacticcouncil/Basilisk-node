#![cfg(test)]
use crate::kusama_test_net::*;
use basilisk_runtime::{
	AccountId, AssetRegistry, AssetsAccountId, Balance, BlockNumber, Origin, Router, Stableswap, LBP, XYK,
};
use frame_support::{assert_noop, assert_ok};
use hydradx_traits::AMM;
use pallet_lbp::WeightCurveType;
use pallet_route_executor::Trade;
use primitives::{asset::AssetPair, AssetId};

use hydradx_traits::{router::PoolType, AccountIdFor, Registry};
use orml_traits::MultiCurrency;
use pallet_stableswap::types::AssetLiquidity;
use sp_runtime::{traits::One, Permill};
use xcm_emulator::TestExt;

const SALE_START: Option<BlockNumber> = Some(10);
const SALE_END: Option<BlockNumber> = Some(40);

const TRADER: [u8; 32] = BOB;

fn create_lbp_pool(accumulated_asset: u32, distributed_asset: u32) {
	assert_ok!(LBP::create_pool(
		Origin::root(),
		ALICE.into(),
		accumulated_asset,
		100 * UNITS,
		distributed_asset,
		200 * UNITS,
		20_000_000,
		80_000_000,
		WeightCurveType::Linear,
		(2, 1_000),
		CHARLIE.into(),
		0,
	));

	let account_id = get_lbp_pair_account_id(accumulated_asset, distributed_asset);

	assert_ok!(LBP::update_pool_data(
		Origin::signed(AccountId::from(ALICE)),
		account_id,
		None,
		SALE_START,
		SALE_END,
		None,
		None,
		None,
		None,
		None,
	));
}

fn get_lbp_pair_account_id(asset_a: AssetId, asset_b: AssetId) -> AccountId {
	let asset_pair = AssetPair {
		asset_in: asset_a,
		asset_out: asset_b,
	};
	LBP::get_pair_id(asset_pair)
}

fn start_lbp_campaign() {
	set_relaychain_block_number(SALE_START.unwrap() + 1);
}

fn create_xyk_pool(asset_a: u32, asset_b: u32) {
	assert_ok!(XYK::create_pool(
		Origin::signed(ALICE.into()),
		asset_a,
		100 * UNITS,
		asset_b,
		50 * UNITS,
	));
}

/// This function register `share_asset` and create stableswap pool from `assets`;
///
/// Retrun: share token id
fn create_stableswap_pool(assets: Vec<AssetId>, amplification: u16) -> AssetId {
	let share_asset_name = AssetsAccountId::<basilisk_runtime::Runtime>::name(&assets, Some(b"share_asset"));

	let share_asset = AssetRegistry::create_asset(&share_asset_name, Balance::one()).unwrap();

	assert_ok!(Stableswap::create_pool(
		Origin::root(),
		share_asset,
		assets.clone(),
		amplification,
		Permill::from_percent(0),
		Permill::from_percent(0)
	));

	let initial_amount = 100_000 * UNITS;

	let mut init_assets: Vec<AssetLiquidity<AssetId>> = Vec::new();
	assets.iter().for_each(|a_id| {
		init_assets.push(AssetLiquidity {
			asset_id: *a_id,
			amount: initial_amount,
		});
	});

	assert_ok!(Stableswap::add_liquidity(
		Origin::signed(STABLESWAP_LP.into()),
		share_asset,
		init_assets
	));

	share_asset
}

#[macro_export]
macro_rules! assert_trader_non_native_balance {
	($balance:expr,$asset_id:expr) => {{
		let trader_balance = basilisk_runtime::Tokens::free_balance($asset_id, &AccountId::from(TRADER));
		assert_eq!(
			trader_balance, $balance,
			"\r\nNon native asset({}) balance '{}' is not as expected '{}'",
			$asset_id, trader_balance, $balance
		);
	}};
}

#[macro_export]
macro_rules! assert_trader_bsx_balance {
	($balance:expr) => {{
		let trader_balance = basilisk_runtime::Balances::free_balance(&AccountId::from(TRADER));
		assert_eq!(
			trader_balance, $balance,
			"\r\nBSX asset balance '{}' is not as expected '{}'",
			trader_balance, $balance
		);
	}};
}

pub mod different_pools;
pub mod lbp;
pub mod stableswap;
pub mod xyk;
