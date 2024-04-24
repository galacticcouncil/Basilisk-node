#![cfg(feature = "runtime-benchmarks")]

pub mod currencies;
pub mod duster;
mod helper;
pub mod marketplace;
pub mod multi_payment;
pub mod route_executor;
pub mod tokens;
pub mod vesting;

pub use helper::BenchmarkHelper;

use crate::AssetRegistry;

use crate::Currencies;

use frame_support::assert_ok;
use frame_system::RawOrigin;

use primitives::{AccountId, AssetId, Balance};
use sp_std::vec;
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
		None,
	)
	.map_err(|_| ())
}
