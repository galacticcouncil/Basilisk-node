#![cfg(feature = "runtime-benchmarks")]

pub mod currencies;
pub mod duster;
mod helper;
pub mod marketplace;
pub mod multi_payment;
pub mod route_executor;
pub mod tokens;
pub mod vesting;
pub mod xyk;

pub use helper::BenchmarkHelper;

pub use crate::{
	AssetLocation, AssetRegistry, Currencies, MultiTransactionPayment, Runtime, RuntimeCall, TreasuryAccount, XYK,
};

use frame_benchmarking::{account, BenchmarkError};
use frame_support::assert_ok;
use frame_system::RawOrigin;

use primitives::{AccountId, AssetId, Balance};
use sp_std::vec;
use sp_std::vec::Vec;

use frame_support::storage::with_transaction;
use hydradx_traits::{AssetKind, Create};
use orml_traits::MultiCurrencyExtended;
use polkadot_xcm::v5::Location;
use sp_runtime::{
	traits::{One, SaturatedConversion},
	FixedU128, TransactionOutcome,
};

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

pub fn register_external_asset(name: Vec<u8>) -> Result<AssetId, ()> {
	let n = name.try_into().map_err(|_| ())?;
	with_transaction(|| {
		TransactionOutcome::Commit(AssetRegistry::register_insufficient_asset(
			None,
			Some(n),
			AssetKind::External,
			Some(Balance::one()),
			None,
			None,
			None,
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

pub fn set_location(asset_id: AssetId, location: AssetLocation) -> Result<(), ()> {
	AssetRegistry::set_location(RawOrigin::Root.into(), asset_id, location).map_err(|_| ())
}

pub const DOT_ASSET_LOCATION: AssetLocation = AssetLocation(Location::parent());
fn setup_insufficient_asset_with_dot() -> Result<AssetId, BenchmarkError> {
	let dot = register_asset(b"DOT".to_vec(), 1u128).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
	set_location(dot, DOT_ASSET_LOCATION).map_err(|_| BenchmarkError::Stop("Failed to set location for weth"))?;
	pallet_transaction_multi_payment::Pallet::<Runtime>::add_currency(RawOrigin::Root.into(), dot, FixedU128::from(1))
		.map_err(|_| BenchmarkError::Stop("Failed to add supported currency"))?;
	let insufficient_asset =
		register_external_asset(b"FCA".to_vec()).map_err(|_| BenchmarkError::Stop("Failed to register asset"))?;
	create_xyk_pool(insufficient_asset, dot);

	Ok(insufficient_asset)
}

pub const INITIAL_BALANCE: Balance = 10_000_000 * ONE;
fn create_funded_account(name: &'static str, index: u32, assets: &[AssetId]) -> AccountId {
	let account: AccountId = account(name, index, 0);
	//Necessary to pay ED for insufficient assets.
	<Currencies as MultiCurrencyExtended<_>>::update_balance(
		0,
		&account,
		crate::benchmarking::route_executor::INITIAL_BALANCE as i128,
	)
	.unwrap();

	for asset in assets {
		assert_ok!(<Currencies as MultiCurrencyExtended<_>>::update_balance(
			*asset,
			&account,
			INITIAL_BALANCE.try_into().unwrap(),
		));
	}
	account
}

pub const ONE: Balance = 1_000_000_000_000;
pub fn create_xyk_pool(asset_a: u32, asset_b: u32) {
	let caller: AccountId = create_funded_account("caller", 0, &[asset_a, asset_b]);

	assert_ok!(Currencies::update_balance(
		RawOrigin::Root.into(),
		caller.clone(),
		0,
		10 * ONE as i128,
	));

	let amount = 100000 * ONE;
	assert_ok!(Currencies::update_balance(
		RawOrigin::Root.into(),
		caller.clone(),
		asset_a,
		amount as i128,
	));

	assert_ok!(Currencies::update_balance(
		RawOrigin::Root.into(),
		caller.clone(),
		asset_b,
		amount as i128,
	));

	assert_ok!(XYK::create_pool(
		RawOrigin::Signed(caller.clone()).into(),
		asset_a,
		amount,
		asset_b,
		amount,
	));

	assert_ok!(XYK::sell(
		RawOrigin::Signed(caller).into(),
		asset_a,
		asset_b,
		10 * ONE,
		0u128,
		false,
	));
}
