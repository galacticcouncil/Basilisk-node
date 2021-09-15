#![cfg(test)]

use crate::builder::*;
use crate::kusama_test_net::*;

use frame_support::assert_ok;
use polkadot_xcm::v0::{
	Junction::{self, Parachain, Parent},
	MultiAsset::*,
	MultiLocation::*,
	NetworkId,
};

use cumulus_primitives_core::ParaId;
use orml_traits::MultiCurrency;
use sp_runtime::traits::AccountIdConversion;
use xcm_emulator::TestExt;

#[test]
fn transfer_from_relay_chain() {
	Basilisk::execute_with(|| {
		assert_ok!(AssetRegistry::set_location(
			Origin::root(),
			1,
			AssetLocation(X1(Parent,))
		));
	});
	Kusama::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::Origin::signed(ALICE.into()),
			X1(Parachain(2000)),
			X1(Junction::AccountId32 {
				id: BOB,
				network: NetworkId::Any
			}),
			vec![ConcreteFungible {
				id: Null,
				amount: 3 * BSX
			}],
			1_600_000_000
		));

		assert_eq!(
			kusama_runtime::Balances::free_balance(&ParaId::from(2000).into_account()),
			13 * BSX
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(Tokens::free_balance(1, &AccountId::from(BOB)), 3 * BSX);
	});
}

#[test]
fn transfer_to_relay_chain() {
	Basilisk::execute_with(|| {
		assert_ok!(AssetRegistry::set_location(
			Origin::root(),
			1,
			AssetLocation(X1(Parent,))
		));

		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			1,
			3 * BSX,
			X2(
				Parent,
				Junction::AccountId32 {
					id: BOB,
					network: NetworkId::Any,
				}
			),
			399_600_000_000
		));
		assert_eq!(Tokens::free_balance(1, &AccountId::from(ALICE)), 200 * BSX - 3 * BSX);
	});

	Kusama::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
			2989344000666 // 3 - fee
		);
	});
}
