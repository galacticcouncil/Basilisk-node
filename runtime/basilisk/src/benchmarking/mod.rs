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

use crate::{AssetLocation, Currencies};

use frame_support::assert_ok;
use frame_system::RawOrigin;

use primitives::{AccountId, AssetId, Balance};
use sp_std::vec;
use sp_std::vec::Vec;

use frame_support::storage::with_transaction;
use hydradx_traits::{registry::Create, AssetKind};
use orml_traits::MultiCurrencyExtended;
use sp_runtime::traits::SaturatedConversion;
use sp_runtime::TransactionOutcome;

pub const BSX: Balance = primitives::constants::currency::UNITS;

pub fn register_asset(name: Vec<u8>, deposit: Balance, location: Option<AssetLocation>) -> Result<AssetId, ()> {
	with_transaction(|| {
		TransactionOutcome::Commit(AssetRegistry::register_sufficient_asset(
			None,
			Some(&name),
			AssetKind::Token,
			Some(deposit),
			None,
			None,
			location,
			None,
		))
	})
	.map_err(|_| ())
}

pub fn update_balance(currency_id: AssetId, who: &AccountId, balance: Balance) {
	assert_ok!(<Currencies as MultiCurrencyExtended<_>>::update_balance(
		currency_id,
		who,
		balance.saturated_into()
	));
}

pub fn update_asset(asset_id: AssetId, name: Option<Vec<u8>>, deposit: Balance) -> Result<(), ()> {
	with_transaction(|| {
		TransactionOutcome::Commit(AssetRegistry::update(
			RawOrigin::Root.into(),
			asset_id,
			name,
			None,
			Some(deposit),
			None,
			None,
			None,
			None,
			None,
		))
	})
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
