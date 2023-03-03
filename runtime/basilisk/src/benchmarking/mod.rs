#![cfg(feature = "runtime-benchmarks")]

pub mod currencies;
pub mod duster;
pub mod marketplace;
pub mod multi_payment;
pub mod route_executor;
pub mod tokens;
pub mod vesting;

use crate::AssetRegistry;
use crate::XYK;

use crate::Currencies;

use frame_support::assert_ok;
use frame_system::RawOrigin;

use common_runtime::AccountId;
use primitives::{AssetId, Balance};
use sp_std::vec::Vec;

use orml_traits::MultiCurrencyExtended;
use sp_runtime::traits::SaturatedConversion;

pub const BSX: Balance = primitives::constants::currency::UNITS;

pub fn register_asset(name: Vec<u8>, deposit: Balance) -> Result<AssetId, ()> {
	AssetRegistry::register_asset(
		AssetRegistry::to_bounded_name(name).map_err(|_| ())?,
		pallet_asset_registry::AssetType::<AssetId>::Token,
		deposit,
		None,
	)
	.map_err(|_| ())
}

pub fn update_balance(currency_id: AssetId, who: &AccountId, balance: Balance) {
	assert_ok!(<Currencies as MultiCurrencyExtended<_>>::update_balance(
		currency_id,
		who,
		balance.saturated_into()
	));
}

pub fn update_asset(asset_id: AssetId, name: Vec<u8>, deposit: Balance) -> Result<(), ()> {
	AssetRegistry::update(
		RawOrigin::Root.into(),
		asset_id,
		name,
		pallet_asset_registry::AssetType::<AssetId>::Token,
		Some(deposit),
	)
	.map_err(|_| ())
}

pub fn create_pool(who: AccountId, asset_a: AssetId, amount_a: Balance, asset_b: AssetId, amount_b: Balance) {
	assert_ok!(XYK::create_pool(
		RawOrigin::Signed(who).into(),
		asset_a,
		amount_a,
		asset_b,
		amount_b,
	));
}
